use bytemuck::{Pod, Zeroable};
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use solana_system_interface::program as system_program;

use crate::{
    error::MplAgentToolsError, instruction::accounts::RegisterExecutiveV1Accounts,
    state::ExecutiveProfileV1,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterExecutiveV1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<RegisterExecutiveV1Args>() == 8);

pub fn register_executive_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &RegisterExecutiveV1Args,
) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = RegisterExecutiveV1Accounts::context(accounts)?;

    /****************************************************/
    /****************** Account Guards ******************/
    /****************************************************/
    // Payer must sign.
    assert_signer(ctx.accounts.payer)?;

    // The authority is the party being registered as the executive. Since the
    // PDA is seeded on its pubkey and the profile records it verbatim, it must
    // actually consent by signing — otherwise anyone could squat on any
    // pubkey's profile slot.
    if let Some(authority) = ctx.accounts.authority {
        assert_signer(authority)?;
    }

    // Assert that the executive profile is not already initialized.
    if ctx.accounts.executive_profile.owner != &system_program::ID
        || ctx.accounts.executive_profile.data_len() != 0
    {
        return Err(MplAgentToolsError::ExecutiveProfileMustBeUninitialized.into());
    }

    let bump = ExecutiveProfileV1::check_pda_derivation(
        ctx.accounts.executive_profile,
        ctx.accounts.authority.unwrap_or(ctx.accounts.payer),
    )?;

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
    ExecutiveProfileV1::create_account(&ctx.accounts, bump)?;

    // Initialize the account using zero-copy.
    // Borrow the account data mutably and cast to our struct.
    let mut data = ctx.accounts.executive_profile.try_borrow_mut_data()?;
    let executive_profile: &mut ExecutiveProfileV1 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<ExecutiveProfileV1>()]);

    executive_profile.initialize(ctx.accounts.authority.unwrap_or(ctx.accounts.payer).key);

    Ok(())
}
