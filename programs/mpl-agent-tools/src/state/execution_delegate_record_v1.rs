use bytemuck::{Pod, Zeroable};
use mpl_utils::{assert_derivation, create_or_allocate_account_raw};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};

use crate::{
    error::MplAgentToolsError,
    instruction::accounts::{DelegateExecutionV1Accounts, RegisterExecutorV1Accounts},
    state::Key,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct ExecutionDelegateRecordV1 {
    /// Account discriminator.
    #[idl_type(Key)]
    pub key: u8,
    /// PDA bump seed.
    pub bump: u8,
    /// Padding for 8-byte alignment.
    #[padding]
    pub _padding: [u8; 6],
    /// The address of the executor profile.
    pub executor_profile: Pubkey,
    /// The address of the agent asset.
    pub agent_asset: Pubkey,
}

// Compile-time assertion to ensure struct is 8-byte aligned.
const _: () = assert!(core::mem::size_of::<ExecutionDelegateRecordV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<ExecutionDelegateRecordV1>() == 72);

impl ExecutionDelegateRecordV1 {
    const PREFIX: &[u8] = b"execution_delegate_record";

    // Check the PDA derivation.
    pub fn check_pda_derivation(
        address: &AccountInfo,
        executor_profile: &Pubkey,
        agent_asset: &Pubkey,
    ) -> Result<u8, ProgramError> {
        assert_derivation(
            &crate::ID,
            address,
            &[
                Self::PREFIX,
                executor_profile.as_ref(),
                agent_asset.as_ref(),
            ],
            MplAgentToolsError::InvalidExecutionDelegateRecordDerivation,
        )
    }

    // Create the account.
    pub fn create_account(accounts: &DelegateExecutionV1Accounts, bump: u8) -> ProgramResult {
        create_or_allocate_account_raw(
            crate::ID,
            accounts.execution_delegate_record,
            accounts.system_program,
            accounts.payer,
            core::mem::size_of::<ExecutionDelegateRecordV1>(),
            &[
                Self::PREFIX,
                accounts.executor_profile.key.as_ref(),
                accounts.agent_asset.key.as_ref(),
                &[bump],
            ],
        )
    }

    /// Initialize the account with the given bump seed.
    #[inline]
    pub fn initialize(&mut self, bump: u8, executor_profile: &Pubkey, agent_asset: &Pubkey) {
        solana_program::msg!("Initializing execution delegate record account");
        self.key = Key::ExecutionDelegateRecordV1 as u8;
        self.bump = bump;
        self._padding = [0u8; 6];
        self.executor_profile = *executor_profile;
        self.agent_asset = *agent_asset;
    }
}
