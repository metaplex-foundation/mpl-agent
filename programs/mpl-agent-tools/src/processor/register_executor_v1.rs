use bytemuck::{Pod, Zeroable};
use shank::ShankType;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

use crate::{
    error::MplAgentToolsError, instruction::accounts::RegisterExecutorV1Accounts,
    state::ExecutorProfileV1,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterExecutorV1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<RegisterExecutorV1Args>() == 8);

pub fn register_executor_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &RegisterExecutorV1Args,
) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = RegisterExecutorV1Accounts::context(accounts)?;

    /****************************************************/
    /****************** Account Guards ******************/
    /****************************************************/
    // Assert that the executor profile is not already initialized.
    if ctx.accounts.executor_profile.owner != &system_program::ID
        || ctx.accounts.executor_profile.data_len() != 0
    {
        return Err(MplAgentToolsError::ExecutorProfileMustBeUninitialized.into());
    }

    // Validate system program.
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentToolsError::InvalidSystemProgram.into());
    }

    /****************************************************/
    /***************** Argument Guards ******************/
    /****************************************************/

    // Add any argument validation here.

    /****************************************************/
    /********************* Actions **********************/
    /****************************************************/
    // Create the agent executor account.
    ExecutorProfileV1::create_account(&ctx.accounts)?;

    // Initialize the account using zero-copy.
    // Borrow the account data mutably and cast to our struct.
    let mut data = ctx.accounts.executor_profile.try_borrow_mut_data()?;
    let executor_profile: &mut ExecutorProfileV1 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<ExecutorProfileV1>()]);

    executor_profile.initialize(ctx.accounts.authority.unwrap_or(ctx.accounts.payer).key);

    Ok(())
}
