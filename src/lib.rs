use solana_program::{account_info::{next_account_info, AccountInfo}, clock::Clock, log, program::invoke_signed, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar};

use constant::{
    CLOSED_PDA_TYPE, INIT_PUBLISHER_MODE_PAYER_AND_AUTHORITY_ARE_THE_SAME,
    PUBLISHER_INITIATED_EVENT, PUBLISHER_PDA_SEED,
    PUBLISHER_PDA_TYPE,
};

use error::
    INVALID_PDA_ERROR
;

use state::
    publisher::{
        PublisherAccount, PUBLISHER_IS_INITIALIZED,
    }
;

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