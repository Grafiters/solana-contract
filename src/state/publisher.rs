use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::{constant::PUBLISHER_PDA_TYPE, error::INVALID_PUBLISHER_LEN_ERROR};

pub const PUBLISHER_IS_INITIALIZED: u8 = 0b00000001;
pub const PUBLISHER_IS_OPEN_FOR_OFFERING_CREATION: u8 = 0b00000010;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PublisherAccount {
    pub discriminator: u8,
    pub state: u8,
    pub creator: Pubkey,
    pub publisher_seed: Pubkey,
    pub update_authority: Pubkey,
    // when doing transfer of authority, the the transfer authority should be set to the new authority
    // then the new authority can accept the transfer of authority
    pub transfer_authority: Pubkey,
    pub approval_authority: Pubkey,
}

impl PublisherAccount {
    pub fn is_open_for_offering_creation(&self) -> bool {
        (self.state & PUBLISHER_IS_OPEN_FOR_OFFERING_CREATION)
            == PUBLISHER_IS_OPEN_FOR_OFFERING_CREATION
    }
}

impl Sealed for PublisherAccount {}

impl IsInitialized for PublisherAccount {
    fn is_initialized(&self) -> bool {
        self.discriminator == PUBLISHER_PDA_TYPE
            && (self.state & PUBLISHER_IS_INITIALIZED) == PUBLISHER_IS_INITIALIZED
    }
}

impl Pack for PublisherAccount {
    const LEN: usize = 0
      + 1 // discriminator
      + 1 // state
      + 32 // creator
      + 32 // publisher_seed
      + 32 // update_authority
      + 32 // transfer_authority
      + 32 // approval_authority
      ;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let (discriminator, rest) = src.split_at(1);
        let (state, rest) = rest.split_at(1);
        let (creator, rest) = rest.split_at(32);
        let (publisher_seed, rest) = rest.split_at(32);
        let (update_authority, rest) = rest.split_at(32);
        let (transfer_authority, rest) = rest.split_at(32);
        let (approval_authority, rest) = rest.split_at(32);

        if false
            || discriminator.len() != 1
            || state.len() != 1
            || creator.len() != 32
            || publisher_seed.len() != 32
            || update_authority.len() != 32
            || transfer_authority.len() != 32
            || approval_authority.len() != 32
            || rest.len() != 0
        {
            return Err(INVALID_PUBLISHER_LEN_ERROR);
        }

        Ok(PublisherAccount {
            discriminator: discriminator[0],
            state: state[0],
            creator: Pubkey::new_from_array((*creator).try_into().unwrap()),
            publisher_seed: Pubkey::new_from_array((*publisher_seed).try_into().unwrap()),
            update_authority: Pubkey::new_from_array((*update_authority).try_into().unwrap()),
            transfer_authority: Pubkey::new_from_array((*transfer_authority).try_into().unwrap()),
            approval_authority: Pubkey::new_from_array((*approval_authority).try_into().unwrap()),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let (discriminator, rest) = dst.split_at_mut(1);
        let (state, rest) = rest.split_at_mut(1);
        let (creator, rest) = rest.split_at_mut(32);
        let (publisher_seed, rest) = rest.split_at_mut(32);
        let (update_authority, rest) = rest.split_at_mut(32);
        let (transfer_authority, rest) = rest.split_at_mut(32);
        let (approval_authority, rest) = rest.split_at_mut(32);

        if false
            || discriminator.len() != 1
            || state.len() != 1
            || creator.len() != 32
            || publisher_seed.len() != 32
            || update_authority.len() != 32
            || transfer_authority.len() != 32
            || approval_authority.len() != 32
            || rest.len() != 0
        {
            return;
        }

        discriminator[0] = PUBLISHER_PDA_TYPE;
        state[0] = self.state | PUBLISHER_IS_INITIALIZED;
        creator.copy_from_slice(self.creator.as_ref());
        publisher_seed.copy_from_slice(self.publisher_seed.as_ref());
        update_authority.copy_from_slice(self.update_authority.as_ref());
        transfer_authority.copy_from_slice(self.transfer_authority.as_ref());
        approval_authority.copy_from_slice(self.approval_authority.as_ref());
    }
}