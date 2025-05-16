use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::constant::OFFERING_PDA_TYPE;

pub const OFFERING_IS_INITIALIZED: u8 = 0b00000001;
pub const OFFERING_HAS_SOLD_OUT: u8 = 0b00000100;
pub const OFFERING_IS_APPROVED_BY_PUBLISHER: u8 = 0b00001000;
pub const OFFERING_IS_REJECTED_BY_PUBLISHER: u8 = 0b00010000;
pub const OFFERING_OFFER_IS_PROVIDED: u8 = 0b00100000;
pub const OFFERING_FUNDS_IS_WITHDRAWN: u8 = 0b01000000;
pub const OFFERING_FUNDS_IS_DEPOSITED_TO_MARKET_POOL: u8 = 0b10000000;

#[repr(C)]
pub struct OfferingAccount {
    pub discriminator: u8,
    pub state: u8,
    pub publisher: Pubkey,
    pub creator: Pubkey,
    pub offering_seed: Pubkey,
    pub promoter: Pubkey,
    pub funded_mint: Pubkey,
    pub offered_mint: Pubkey,
    pub softcap: u64,
    pub hardcap: u64,
    pub funded_amount: u64,
    pub min_funding: u64,
    pub max_funding: u64,
    pub offer_quantity: u64,
    pub distributed_quantity: u64,
    pub market_pool_percentage: u8,
    pub publish_time: i64,
    pub open_time: i64,
    pub close_time: i64,
    pub exit_time: i64,
    pub num_of_purchasers: u64,
    pub metadata_cid: Vec<u8>,
    pub vestings: Vec<OfferingVesting>,
}

#[repr(C)]
pub struct OfferingVesting {
    pub release_time: i64,
    /**
     * this percentage must be between 0 and 100
     */
    pub percentage: u8,
}

impl OfferingAccount {
    pub fn is_initialized(&self) -> bool {
        self.discriminator == OFFERING_PDA_TYPE
            && (self.state & OFFERING_IS_INITIALIZED) == OFFERING_IS_INITIALIZED
    }

    pub fn is_published(&self, time: i64) -> bool {
        (self.state & OFFERING_IS_INITIALIZED) == OFFERING_IS_INITIALIZED
            && time > self.publish_time
    }

    pub fn is_all_vesting_distributed(&self) -> bool {
        (self.state & OFFERING_IS_INITIALIZED) == OFFERING_IS_INITIALIZED
            && self.offer_quantity == self.distributed_quantity
    }

    pub fn is_purchasable(&self, time: i64) -> bool {
        (self.state & OFFERING_IS_INITIALIZED) == OFFERING_IS_INITIALIZED
            && time > self.open_time
            && time < self.close_time
            && self.hardcap > self.funded_amount
    }

    pub fn is_first_vesting_released(&self, time: i64) -> bool {
        (self.state & OFFERING_IS_INITIALIZED) == OFFERING_IS_INITIALIZED
            && self.vestings.len() > 0
            && time > self.vestings[0].release_time
    }

    pub fn has_sold_out(&self) -> bool {
        (self.state & OFFERING_HAS_SOLD_OUT) == OFFERING_HAS_SOLD_OUT
    }

    pub fn fundable_amount(&self) -> u64 {
        self.hardcap - self.funded_amount
    }
}

const FIXED_OFFERING_ACCOUNT_DATA_LEN: usize = 0
    + 1 // discriminator
    + 1 // state
    + 32 // publisher
    + 32 // creator
    + 32 // offering_seed
    + 32 // promoter
    + 32 // funded_mint: token yang diminta oleh promoter
    + 32 // offered_mint: token yang ditawarkan oleh promoter
    + 8 // softcap: batasan bawah penggalagan dana
    + 8 // hardcap: batasan atas penggalangan dana
    + 8 // funded_amount: jumlah dana yang sudah terkumpul
    + 8 // min_funding: jumlah minimum yang harus terkumpul
    + 8 // max_funding: jumlah maksimum yang harus terkumpul
    + 8 // offer_quantity: jumlah penawaran token yang akan dibagikan
    + 8 // distributed_quantity: jumlah token yang sudah dibagikan
    + 1 // market_pool_percentage
    + 8 // publish_time
    + 8 // open_time
    + 8 // close_time
    + 8 // exit_time
    + 8 // num_of_purchasers
    + 8 // metadata_cid_len
    + 1 // num_of_vestings
    ;

pub fn calc_offering_account_len(offering: &OfferingAccount) -> usize {
    let len = FIXED_OFFERING_ACCOUNT_DATA_LEN
        + offering.metadata_cid.len() // metadata_cid
        + (offering.vestings.len() * (8 + 1)) // vestings
        ;

    return len;
}

pub fn check_offering_account_data(src: &[u8]) -> bool {
    if src.len() < FIXED_OFFERING_ACCOUNT_DATA_LEN {
        return false;
    }

    if src[0] != OFFERING_PDA_TYPE {
        return false;
    }

    if (src[1] & OFFERING_IS_INITIALIZED) != OFFERING_IS_INITIALIZED {
        return false;
    }

    return true;
}

pub fn unpack_offering_account(src: &[u8]) -> Result<OfferingAccount, ProgramError> {
    let (discriminator, rest) = src.split_at(1);
    let (state, rest) = rest.split_at(1);
    let (publisher, rest) = rest.split_at(32);
    let (creator, rest) = rest.split_at(32);
    let (offering_seed, rest) = rest.split_at(32);
    let (promoter, rest) = rest.split_at(32);
    let (funded_mint, rest) = rest.split_at(32);
    let (offered_mint, rest) = rest.split_at(32);
    let (softcap, rest) = rest.split_at(8);
    let (hardcap, rest) = rest.split_at(8);
    let (funded_amount, rest) = rest.split_at(8);
    let (min_funding, rest) = rest.split_at(8);
    let (max_funding, rest) = rest.split_at(8);
    let (offer_quantity, rest) = rest.split_at(8);
    let (distributed_quantity, rest) = rest.split_at(8);
    let (market_pool_percentage, rest) = rest.split_at(1);
    let (publish_time, rest) = rest.split_at(8);
    let (open_time, rest) = rest.split_at(8);
    let (close_time, rest) = rest.split_at(8);
    let (exit_time, rest) = rest.split_at(8);
    let (num_of_purchasers, rest) = rest.split_at(8);
    let (metatadata_cid_len_data, rest) = rest.split_at(8);
    let (num_of_vestings, rest) = rest.split_at(1);

    let metadata_cid_len = u64::from_le_bytes(metatadata_cid_len_data.try_into().unwrap()) as usize;
    let (metadata_cid, rest) = rest.split_at(metadata_cid_len);

    let mut vesting_src = rest;

    let vestings = (0..num_of_vestings[0])
        .map(|_| {
            let (release_time, rest) = vesting_src.split_at(8);
            let (percentage, rest) = rest.split_at(1);
            vesting_src = rest;
            OfferingVesting {
                release_time: i64::from_le_bytes(release_time.try_into().unwrap()),
                percentage: percentage[0],
            }
        })
        .collect();

    return Ok(OfferingAccount {
        discriminator: discriminator[0],
        state: state[0],
        publisher: Pubkey::new_from_array((*publisher).try_into().unwrap()),
        creator: Pubkey::new_from_array((*creator).try_into().unwrap()),
        offering_seed: Pubkey::new_from_array((*offering_seed).try_into().unwrap()),
        promoter: Pubkey::new_from_array((*promoter).try_into().unwrap()),
        funded_mint: Pubkey::new_from_array((*funded_mint).try_into().unwrap()),
        offered_mint: Pubkey::new_from_array((*offered_mint).try_into().unwrap()),
        softcap: u64::from_le_bytes(softcap.try_into().unwrap()),
        hardcap: u64::from_le_bytes(hardcap.try_into().unwrap()),
        funded_amount: u64::from_le_bytes(funded_amount.try_into().unwrap()),
        min_funding: u64::from_le_bytes(min_funding.try_into().unwrap()),
        max_funding: u64::from_le_bytes(max_funding.try_into().unwrap()),
        offer_quantity: u64::from_le_bytes(offer_quantity.try_into().unwrap()),
        distributed_quantity: u64::from_le_bytes(distributed_quantity.try_into().unwrap()),
        market_pool_percentage: market_pool_percentage[0],
        publish_time: i64::from_le_bytes(publish_time.try_into().unwrap()),
        open_time: i64::from_le_bytes(open_time.try_into().unwrap()),
        close_time: i64::from_le_bytes(close_time.try_into().unwrap()),
        exit_time: i64::from_le_bytes(exit_time.try_into().unwrap()),
        num_of_purchasers: u64::from_le_bytes(num_of_purchasers.try_into().unwrap()),
        metadata_cid: metadata_cid.to_vec(),
        vestings,
    });
}

pub fn pack_offering_account(
    dst: &mut [u8],
    offering: &OfferingAccount,
) -> Result<(), ProgramError> {
    let (discriminator, rest) = dst.split_at_mut(1);
    let (state, rest) = rest.split_at_mut(1);
    let (publisher, rest) = rest.split_at_mut(32);
    let (creator, rest) = rest.split_at_mut(32);
    let (offering_seed, rest) = rest.split_at_mut(32);
    let (promoter, rest) = rest.split_at_mut(32);
    let (funded_mint, rest) = rest.split_at_mut(32);
    let (offered_mint, rest) = rest.split_at_mut(32);
    let (softcap, rest) = rest.split_at_mut(8);
    let (hardcap, rest) = rest.split_at_mut(8);
    let (funded_amount, rest) = rest.split_at_mut(8);
    let (min_funding, rest) = rest.split_at_mut(8);
    let (max_funding, rest) = rest.split_at_mut(8);
    let (offer_quantity, rest) = rest.split_at_mut(8);
    let (distributed_quantity, rest) = rest.split_at_mut(8);
    let (market_pool_percentage, rest) = rest.split_at_mut(1);
    let (publish_time, rest) = rest.split_at_mut(8);
    let (open_time, rest) = rest.split_at_mut(8);
    let (close_time, rest) = rest.split_at_mut(8);
    let (exit_time, rest) = rest.split_at_mut(8);
    let (num_of_purchasers, rest) = rest.split_at_mut(8);
    let (metadata_len, rest) = rest.split_at_mut(8);
    let (num_of_vestings, rest) = rest.split_at_mut(1);

    discriminator[0] = OFFERING_PDA_TYPE;
    state[0] = offering.state | OFFERING_IS_INITIALIZED;
    publisher.copy_from_slice(offering.publisher.as_ref());
    creator.copy_from_slice(offering.creator.as_ref());
    offering_seed.copy_from_slice(offering.offering_seed.as_ref());
    promoter.copy_from_slice(offering.promoter.as_ref());
    funded_mint.copy_from_slice(offering.funded_mint.as_ref());
    offered_mint.copy_from_slice(offering.offered_mint.as_ref());
    softcap.copy_from_slice(&offering.softcap.to_le_bytes());
    hardcap.copy_from_slice(&offering.hardcap.to_le_bytes());
    funded_amount.copy_from_slice(&offering.funded_amount.to_le_bytes());
    min_funding.copy_from_slice(&offering.min_funding.to_le_bytes());
    max_funding.copy_from_slice(&offering.max_funding.to_le_bytes());
    offer_quantity.copy_from_slice(&offering.offer_quantity.to_le_bytes());
    distributed_quantity.copy_from_slice(&offering.distributed_quantity.to_le_bytes());
    market_pool_percentage[0] = offering.market_pool_percentage;
    publish_time.copy_from_slice(&offering.publish_time.to_le_bytes());
    open_time.copy_from_slice(&offering.open_time.to_le_bytes());
    close_time.copy_from_slice(&offering.close_time.to_le_bytes());
    exit_time.copy_from_slice(&offering.exit_time.to_le_bytes());
    num_of_purchasers.copy_from_slice(&offering.num_of_purchasers.to_le_bytes());
    metadata_len.copy_from_slice(&offering.metadata_cid.len().to_le_bytes());
    num_of_vestings[0] = offering.vestings.len() as u8;

    let (metadata_cid, rest) = rest.split_at_mut(offering.metadata_cid.len());
    metadata_cid.copy_from_slice(&offering.metadata_cid);

    let mut vesting_dst = rest;

    for vesting in offering.vestings.iter() {
        let (release_time, rest) = vesting_dst.split_at_mut(8);
        let (percentage, rest) = rest.split_at_mut(1);

        release_time.copy_from_slice(&vesting.release_time.to_le_bytes());
        percentage[0] = vesting.percentage;

        vesting_dst = rest;
    }

    return Ok(());
}