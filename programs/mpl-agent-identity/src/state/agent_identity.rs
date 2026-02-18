use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::MplAgentIdentityError, instruction::accounts::RegisterIdentityV1Accounts};

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
pub struct AgentIdentityV1 {
    /// Account discriminator.
    #[idl_type(Key)]
    pub key: u8,
    /// PDA bump seed.
    pub bump: u8,
    /// Padding for 8-byte alignment.
    #[padding]
    pub _padding: [u8; 6],
    /// The address of the asset.
    pub asset: Pubkey,
}

// Compile-time assertion to ensure struct is 8-byte aligned.
const _: () = assert!(core::mem::size_of::<AgentIdentityV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<AgentIdentityV1>() == 40);

impl AgentIdentityV1 {
    /// PDA seed prefix for this account type.
    pub const PREFIX: &'static [u8] = b"agent_identity";

    pub fn check_pda_derivation(address: &AccountInfo, asset: &Pubkey) -> Result<u8, ProgramError> {
        solana_program::msg!("Checking PDA derivation for asset");
        assert_derivation(
            &crate::ID,
            address,
            &[Self::PREFIX, asset.as_ref()],
            MplAgentIdentityError::InvalidAccountData,
        )
    }

    pub fn create_account(accounts: &RegisterIdentityV1Accounts, bump: u8) -> ProgramResult {
        solana_program::msg!("Creating agent identity account");
        create_or_allocate_account_raw(
            crate::ID,
            accounts.agent_identity,
            accounts.system_program,
            accounts.payer,
            core::mem::size_of::<AgentIdentityV1>(),
            &[Self::PREFIX, accounts.asset.key.as_ref(), &[bump]],
        )
    }

    /// Initialize the account with the given bump seed.
    #[inline]
    pub fn initialize(&mut self, bump: u8, asset: &Pubkey) {
        solana_program::msg!("Initializing agent identity account");
        self.key = Key::AgentIdentityV1 as u8;
        self.bump = bump;
        self._padding = [0u8; 6];
        self.asset = *asset;
    }
}
