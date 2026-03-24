use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use podded::pod::OptionalPubkey;
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::MplAgentIdentityError, instruction::accounts::RegisterIdentityV1Accounts};

use super::Key;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct AgentIdentityV2 {
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
    /// The address of the agent token.
    pub agent_token: OptionalPubkey,
    // Reserved for future use.
    #[padding]
    pub _reserved: [u8; 32],
}

// Compile-time assertion to ensure struct is 8-byte aligned.
const _: () = assert!(core::mem::size_of::<AgentIdentityV2>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<AgentIdentityV2>() == 104);

impl AgentIdentityV2 {
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
            core::mem::size_of::<AgentIdentityV2>(),
            &[Self::PREFIX, accounts.asset.key.as_ref(), &[bump]],
        )
    }

    /// Initialize the account with the given bump seed.
    #[inline]
    pub fn initialize(&mut self, bump: u8, asset: &Pubkey) {
        solana_program::msg!("Initializing agent identity account");
        self.key = Key::AgentIdentityV2 as u8;
        self.bump = bump;
        self._padding = [0u8; 6];
        self.asset = *asset;
        self.agent_token = OptionalPubkey::default();
    }
}
