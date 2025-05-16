use solana_program::{account_info::{next_account_info, AccountInfo}, clock::Clock, log, program::invoke_signed, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar};

use constant::{
    APPROVAL_OFFERING_TAG, CLAIM_PUBLISHER_AUTHORITY_TAG, CLOSED_PDA_TYPE, CLOSE_OFFERING_TAG,
    CUSTODIAL_PDA_SEED, DISTRIBUTE_TAG, FINALIZE_OFFERING_CREATION_TAG,
    FINALIZE_OFFERING_MODE_PAYER_AND_PROMOTER_ARE_THE_SAME,
    FINALIZE_OFFERING_MODE_PAYMENT_AND_SYSTEM_ARE_THE_SAME, INITIATE_OFFERING_TAG,
    INITIATE_PUBLISHER_TAG, INITIATE_PURCHASE_TAG, INITITATE_PUBLISHER_TOKEN_TAG,
    INIT_OFFERING_MODE_FUNDED_AND_SYSTEM_ARE_THE_SAME,
    INIT_OFFERING_MODE_OFFERED_AND_SYSTEM_ARE_THE_SAME,
    INIT_OFFERING_MODE_PAYER_AND_PROMOTER_ARE_THE_SAME,
    INIT_PUBLISHER_MODE_PAYER_AND_AUTHORITY_ARE_THE_SAME,
    INIT_PUBLISHER_TOKEN_MODE_PAYER_AND_AUTHORITY_ARE_THE_SAME,
    INIT_PUBLISHER_TOKEN_MODE_TOKEN_PROGRAM_AND_SYSTEM_ARE_THE_SAME,
    INIT_PURCHASE_MODE_FUNDED_AND_SYSTEM_ARE_THE_SAME,
    INIT_PURCHASE_MODE_PAYER_AND_SUBJECT_ARE_THE_SAME, MODE_PAYER_AND_SUBJECT_ARE_THE_SAME,
    MODE_TRANSFER_AND_APPROVAL_ARE_THE_SAME, MODE_TRANSFER_AND_AUTHORITY_ARE_THE_SAME,
    OFFERING_APPROVAL_EVENT, OFFERING_CLOSED_EVENT, OFFERING_CREATION_FINALIZED_EVENT,
    OFFERING_INITIATED_EVENT, OFFERING_MODE_FUNDED_AND_SYSTEM_ARE_THE_SAME,
    OFFERING_MODE_PAYER_AND_PROMOTER_ARE_THE_SAME, OFFERING_PDA_SEED, OFFERING_PDA_TYPE,
    PUBLISHER_AUTHORITY_CLAIMED_EVENT, PUBLISHER_INITIATED_EVENT, PUBLISHER_PDA_SEED,
    PUBLISHER_PDA_TYPE, PUBLISHER_TOKEN_INITIATED_EVENT, PUBLISHER_TOKEN_PDA_SEED,
    PUBLISHER_TOKEN_PDA_TYPE, PUBLISHER_TOKEN_UPDATED_EVENT, PUBLISHER_TOKEN_WITHDRAWAL_EVENT,
    PUBLISHER_UPDATED_EVENT, PURCHASE_ADDED_EVENT, PURCHASE_DISTRIBUTION_DISTRIBUTED_EVENT,
    PURCHASE_INITIATED_EVENT, PURCHASE_MODE_FUNDED_AND_SYSTEM_ARE_THE_SAME,
    PURCHASE_MODE_PAYER_AND_SUBJECT_ARE_THE_SAME, PURCHASE_PDA_SEED, PURCHASE_PDA_TYPE,
    PURCHASE_REFUNDED_EVENT, REFUND_PURCHASE_TAG, UPDATE_OFFERING_TARGET_TAG, UPDATE_PUBLISHER_TAG,
    UPDATE_PUBLISHER_TOKEN_TAG, WITHDRAW_PUBLISHER_TOKEN_TAG,
};

use error::{
    ACCOUNT_IS_CLOSED_ERROR, INVALID_ATA_ERROR, INVALID_DISTRIBUTION_QUANTITY_CALC_ERROR,
    INVALID_INSTRUCTION_ERROR, INVALID_NUM_OF_VESTINGS_ERROR, INVALID_OFFERING_EXIT_TIME_ERROR,
    INVALID_PDA_ERROR, INVALID_PURCHASE_QUANTITY_ERROR, INVALID_PURCHASE_TOTAL_PRICE_ERROR,
    INVALID_PURCHASE_UNDER_MIN_QUANTITY_ERROR, INVALID_TAG_ERROR, INVALID_VESTING_PERCENTAGE_ERROR,
    INVALID_VESTING_TIME_ERROR, OFFERING_CLOSE_TIME_IS_INVALID_ERROR,
    OFFERING_DISTRIBUTED_QUANTITY_OVERFLOW_ERROR, OFFERING_FUNDING_AMOUNT_OVERFLOW_ERROR,
    OFFERING_FUNDS_IS_ALREADY_WITHDRAWN_ERROR, OFFERING_IS_NOT_APPROVED_BY_PUBLISHER_ERROR,
    OFFERING_IS_NOT_READY_FOR_DISTRIUBTION_ERROR, OFFERING_IS_NOT_READY_FOR_PURCHASE_ERROR,
    OFFERING_IS_ON_VESTING_ERROR, OFFERING_IS_STILL_PURCHASEABLE_ERROR,
    OFFERING_MARKET_POOL_PERCENTAGE_IS_INVALID_ERROR, OFFERING_MAX_FUNDING_EXCEEDED_ERROR,
    OFFERING_MIN_FUNDING_IS_INVALID_ERROR, OFFERING_MIN_FUNDING_NOT_MET_ERROR,
    OFFERING_OFFERED_MINT_IS_INVALID_ERROR, OFFERING_OFFER_IS_NOT_PROVIDED_ERROR,
    OFFERING_OPEN_TIME_IS_INVALID_ERROR, OFFERING_PUBLISH_TIME_IS_INVALID_ERROR,
    OFFERING_RELEASE_TIME_OVERFLOW_ERROR, OFFERING_SOFTCAP_IS_NOT_REACHED_ERROR,
    OFFERING_UNHANDLED_STATE_ERROR, PUBLISHER_IS_CLOSED_FOR_OFFERING_CREATION_ERROR,
    PUBLISHER_TOKEN_BALANCE_IS_INSUFFICIENT_ERROR, PUBLISHER_TOKEN_IS_DISABLED_ERROR,
    PUBLISHER_TOKEN_IS_INVALID, PUBLISHER_TOKEN_IS_NOT_ALLOWED_AS_FUNDING_ERROR,
    PUBLISHER_TOKEN_IS_NOT_ALLOWED_AS_OFFERS_ERROR,
    PUBLISHER_TOKEN_IS_NOT_ALLOWED_AS_OFFER_CREATION_PAYMENT_ERROR,
    PURCHASE_DISTRIBUTION_IS_DISTRIBUTED_ERROR, PURCHASE_DISTRIBUTION_QUANTITY_OVERFLOW_ERROR,
    PURCHASE_FUNDING_AMOUNT_OVERFLOW_ERROR, PURCHASE_OFFER_QUANTITY_OVERFLOW_ERROR,
    TRANSFER_INSUFFICIENT_FUNDS_ERROR, UNAUTHORIZED_ERROR,
};

use state::{
    publisher::{
        PublisherAccount, PUBLISHER_IS_INITIALIZED, PUBLISHER_IS_OPEN_FOR_OFFERING_CREATION,
    },
    publisher_token::{
        PublisherTokenAccount, ALLOW_TOKEN_AS_FUNDING, ALLOW_TOKEN_AS_OFFERS,
        ALLOW_TOKEN_AS_OFFER_CREATION_PAYMENT, PUBLISHER_TOKEN_IS_DISABLED,
        PUBLISHER_TOKEN_IS_INITIALIZED,
    },
};

pub mod constant;
pub mod error;
pub mod state;

solana_program::declare_id!("4Kh1PYfvoLJ3zAus4UhLcZRPLvBQRPjq1PnQzTEvKVuD");

// fn process_instruction(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     instruction_data: &[u8]
// ) -> Result<(), ProgramError> {
//     let (tag, rest) = instruction_data.split_at(1);
//      if tag.len() != 1 {
//         log::sol_log_data(&[tag]);
//         return Err(INVALID_TAG_ERROR);
//     }
// }

fn initiate_publisher(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    mode: u8,
    publisher_seed: &[u8],
) -> Result<(), ProgramError> {
    let sys_clock = Clock::get()?;
    let sys_rent = Rent::get()?;

    let account_info_iter = &mut accounts.iter();

    let payer_info = next_account_info(account_info_iter)?;
    let authority_info = if mode & INIT_PUBLISHER_MODE_PAYER_AND_AUTHORITY_ARE_THE_SAME == 0 {
        next_account_info(account_info_iter)?
    } else {
        payer_info
    };
    let publisher_pda_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if payer_info.is_signer == false {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let publisher_pda_bump = verify_pda(
        program_id,
        publisher_pda_info,
        &[PUBLISHER_PDA_SEED, payer_info.key.as_ref(), publisher_seed],
    )?;

    let publisher_pda_seeds: &[&[u8]] = &[
        PUBLISHER_PDA_SEED,
        payer_info.key.as_ref(),
        publisher_seed,
        &[publisher_pda_bump],
    ];

    let publisher_pda_rent_exempt = sys_rent.minimum_balance(PublisherAccount::LEN);

    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            publisher_pda_info.key,
            publisher_pda_rent_exempt,
            PublisherAccount::LEN as u64,
            program_id,
        ),
        &[
            payer_info.clone(),
            publisher_pda_info.clone(),
            system_program_info.clone(),
        ],
        &[publisher_pda_seeds],
    )?;

    let publisher_seed = Pubkey::new_from_array(publisher_seed.try_into().unwrap());

    let publisher = PublisherAccount {
        discriminator: PUBLISHER_PDA_TYPE,
        state: PUBLISHER_IS_INITIALIZED,
        creator: *payer_info.key,
        publisher_seed,
        update_authority: *authority_info.key,
        transfer_authority: *authority_info.key,
        approval_authority: *authority_info.key,
    };

    PublisherAccount::pack(publisher, &mut publisher_pda_info.data.borrow_mut())?;

    log::sol_log_data(&[
        PUBLISHER_INITIATED_EVENT,
        &sys_clock.unix_timestamp.to_le_bytes(),
        &publisher_pda_info.key.to_bytes(),
        &publisher_pda_info.data.borrow(),
        &payer_info.key.to_bytes(),
        &authority_info.key.to_bytes(),
    ]);

    Ok(())
}

fn verify_pda(
    program_id: &Pubkey,
    account: &AccountInfo,
    raw_seeds: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (pda, bump) = Pubkey::find_program_address(raw_seeds, program_id);
    if pda.ne(account.key) {
        log::sol_log_data(raw_seeds);
        return Err(INVALID_PDA_ERROR);
    }
    Ok(bump)
}

fn is_closed_account(account: &AccountInfo) -> bool {
    let account_data = account.data.borrow();
    if account_data.len() == 0 {
        return false;
    }
    account_data[0] == CLOSED_PDA_TYPE
}