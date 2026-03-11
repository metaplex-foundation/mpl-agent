use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::MplAgentToolsError, instruction::accounts::RegisterExecutiveV1Accounts, state::Key,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct ExecutiveProfileV1 {
    /// Account discriminator.
    #[idl_type(Key)]
    pub key: u8,
    /// Padding for 8-byte alignment.
    #[padding]
    pub _padding: [u8; 7],
    /// The address of the authority signer for the executor..
    pub authority: Pubkey,
}

// Compile-time assertion to ensure struct is 8-byte aligned.
const _: () = assert!(core::mem::size_of::<ExecutiveProfileV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<ExecutiveProfileV1>() == 40);

impl ExecutiveProfileV1 {
    const PREFIX: &[u8] = b"executive_profile";

    // Check the PDA derivation.
    pub fn check_pda_derivation(
        address: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<u8, ProgramError> {
        assert_derivation(
            &crate::ID,
            address,
            &[Self::PREFIX, authority.key.as_ref()],
            MplAgentToolsError::InvalidExecutiveProfileDerivation,
        )
    }
    // Create the account.
    pub fn create_account(accounts: &RegisterExecutiveV1Accounts, bump: u8) -> ProgramResult {
        create_or_allocate_account_raw(
            crate::ID,
            accounts.executive_profile,
            accounts.system_program,
            accounts.payer,
            core::mem::size_of::<ExecutiveProfileV1>(),
            &[
                Self::PREFIX,
                accounts.authority.unwrap_or(accounts.payer).key.as_ref(),
                &[bump],
            ],
        )
    }

    /// Initialize the account with the given bump seed.
    #[inline]
    pub fn initialize(&mut self, authority: &Pubkey) {
        solana_program::msg!("Initializing agent executive account");
        self.key = Key::ExecutiveProfileV1 as u8;
        self._padding = [0u8; 7];
        self.authority = *authority;
    }
}
