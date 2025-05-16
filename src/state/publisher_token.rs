use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::{constant::PUBLISHER_TOKEN_PDA_TYPE, error::INVALID_PUBLISHER_TOKEN_LEN_ERROR};

pub const PUBLISHER_TOKEN_IS_INITIALIZED: u8 = 0b00000001;
pub const ALLOW_TOKEN_AS_OFFER_CREATION_PAYMENT: u8 = 0b00000010;
pub const ALLOW_TOKEN_AS_OFFERS: u8 = 0b00000100;
pub const ALLOW_TOKEN_AS_FUNDING: u8 = 0b00001000;
pub const PUBLISHER_TOKEN_IS_DISABLED: u8 = 0b00010000;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PublisherTokenAccount {
    pub discriminator: u8,
    pub state: u8,
    pub publisher: Pubkey,
    pub mint: Pubkey,
    pub offer_creation_price: u64,
}

impl PublisherTokenAccount {}

impl Sealed for PublisherTokenAccount {}

impl IsInitialized for PublisherTokenAccount {
    fn is_initialized(&self) -> bool {
        self.discriminator == PUBLISHER_TOKEN_PDA_TYPE
            && (self.state & PUBLISHER_TOKEN_IS_INITIALIZED) == PUBLISHER_TOKEN_IS_INITIALIZED
    }
}

impl Pack for PublisherTokenAccount {
    const LEN: usize = 0
        + 1 // Discriminator
        + 1 // State
        + 32 // Publisher
        + 32 // Mint
        + 8 // Offering Creation Price
        ;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let (discriminator, rest) = src.split_at(1);
        let (state, rest) = rest.split_at(1);
        let (publisher, rest) = rest.split_at(32);
        let (mint, rest) = rest.split_at(32);
        let (price, rest) = rest.split_at(8);

        if false
            || discriminator.len() != 1
            || state.len() != 1
            || publisher.len() != 32
            || mint.len() != 32
            || price.len() != 8
            || rest.len() != 0
        {
            return Err(INVALID_PUBLISHER_TOKEN_LEN_ERROR);
        }

        Ok(PublisherTokenAccount {
            discriminator: discriminator[0],
            state: state[0],
            publisher: Pubkey::new_from_array((*publisher).try_into().unwrap()),
            mint: Pubkey::new_from_array((*mint).try_into().unwrap()),
            offer_creation_price: u64::from_le_bytes(price.try_into().unwrap()),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let (discriminator_dst, dst) = dst.split_at_mut(1);
        let (state_dst, dst) = dst.split_at_mut(1);
        let (publisher_dst, dst) = dst.split_at_mut(32);
        let (mint_dst, dst) = dst.split_at_mut(32);
        let (price_dst, dst) = dst.split_at_mut(8);

        if false
            || discriminator_dst.len() != 1
            || state_dst.len() != 1
            || publisher_dst.len() != 32
            || mint_dst.len() != 32
            || price_dst.len() != 8
            || dst.len() != 0
        {
            return;
        }

        discriminator_dst[0] = PUBLISHER_TOKEN_PDA_TYPE;
        state_dst[0] = self.state | PUBLISHER_TOKEN_IS_INITIALIZED;
        publisher_dst.copy_from_slice(self.publisher.as_ref());
        mint_dst.copy_from_slice(self.mint.as_ref());
        price_dst.copy_from_slice(&self.offer_creation_price.to_le_bytes());
    }
}