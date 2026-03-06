use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::{
    entrypoint::ProgramResult, program::invoke, pubkey::Pubkey, rent::Rent, system_instruction,
    sysvar::Sysvar,
};

use crate::{instruction::accounts::RegisterExecutorV1Accounts, state::Key};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankAccount)]
pub struct ExecutorProfileV1 {
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
const _: () = assert!(core::mem::size_of::<ExecutorProfileV1>() % 8 == 0);
const _: () = assert!(core::mem::size_of::<ExecutorProfileV1>() == 40);

impl ExecutorProfileV1 {
    // Create the account.
    pub fn create_account(accounts: &RegisterExecutorV1Accounts) -> ProgramResult {
        let rent = &Rent::get()?;
        let required_lamports = rent
            .minimum_balance(core::mem::size_of::<ExecutorProfileV1>())
            .max(1)
            .saturating_sub(accounts.executor_profile.lamports());

        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(
                    accounts.payer.key,
                    accounts.executor_profile.key,
                    required_lamports,
                ),
                &[
                    accounts.payer.clone(),
                    accounts.executor_profile.clone(),
                    accounts.system_program.clone(),
                ],
            )?;
        }

        invoke(
            &system_instruction::allocate(
                accounts.executor_profile.key,
                core::mem::size_of::<ExecutorProfileV1>()
                    .try_into()
                    .unwrap(),
            ),
            &[
                accounts.executor_profile.clone(),
                accounts.system_program.clone(),
            ],
        )?;

        invoke(
            &system_instruction::assign(accounts.executor_profile.key, &crate::ID),
            &[
                accounts.executor_profile.clone(),
                accounts.system_program.clone(),
            ],
        )?;

        Ok(())
    }

    /// Initialize the account with the given bump seed.
    #[inline]
    pub fn initialize(&mut self, authority: &Pubkey) {
        solana_program::msg!("Initializing agent executor account");
        self.key = Key::ExecutorProfileV1 as u8;
        self._padding = [0u8; 7];
        self.authority = *authority;
    }
}
