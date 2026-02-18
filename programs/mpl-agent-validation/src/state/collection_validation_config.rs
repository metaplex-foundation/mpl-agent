use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::MplAgentValidationError, instruction::accounts::RegisterValidationV1Accounts};

use super::Key;

/// PDA account structure using zero-copy patterns.
///
/// # Layout
/// - key: 1 byte (account discriminator)
/// - bump: 1 byte (PDA bump seed)
/// - _padding: 6 bytes (alignment to 8 bytes)
///
/// Total: 8 bytes (8-byte aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct CollectionValidationConfigV1 {
    /// Account discriminator.
    #[idl_type(Key)]
    pub key: u8,
    /// PDA bump seed.
    pub bump: u8,
    /// Padding for 8-byte alignment.
    #[padding]
    pub _padding: [u8; 6],
    /// The address of the collection.
    pub collection: Pubkey,
}

// Compile-time assertion to ensure struct is 8-byte aligned.
const _: () = assert!(core::mem::size_of::<CollectionValidationConfigV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<CollectionValidationConfigV1>() == 40);

impl CollectionValidationConfigV1 {
    /// PDA seed prefix for this account type.
    pub const PREFIX: &'static [u8] = b"collection_validation_config";

    pub fn check_pda_derivation(
        address: &AccountInfo,
        collection: &Pubkey,
    ) -> Result<u8, ProgramError> {
        solana_program::msg!("Checking PDA derivation for collection");
        assert_derivation(
            &crate::ID,
            address,
            &[Self::PREFIX, collection.as_ref()],
            MplAgentValidationError::InvalidAccountData,
        )
    }

    pub fn create_account(accounts: &RegisterValidationV1Accounts, bump: u8) -> ProgramResult {
        solana_program::msg!("Creating collection validation config account");
        create_or_allocate_account_raw(
            crate::ID,
            accounts.collection_validation_config,
            accounts.system_program,
            accounts.payer,
            core::mem::size_of::<CollectionValidationConfigV1>(),
            &[Self::PREFIX, accounts.collection.key.as_ref(), &[bump]],
        )
    }

    /// Initialize the account with the given bump seed.
    #[inline]
    pub fn initialize(&mut self, bump: u8, collection: &Pubkey) {
        solana_program::msg!("Initializing collection validation config account");
        self.key = Key::CollectionValidationConfigV1 as u8;
        self.bump = bump;
        self._padding = [0u8; 6];
        self.collection = *collection;
    }
}
