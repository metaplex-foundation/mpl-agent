use bytemuck::{from_bytes, Pod, Zeroable};
use mpl_agent_identity::{accounts::AgentIdentityV2, types::Key as MplAgentIdentityKey};
// Note: We use AgentIdentityV2 only for find_pda (same seeds as V1).
// We read bump from raw bytes to avoid borsh deserialization issues with V1-sized accounts.
use mpl_core::{accounts::BaseAssetV1, types::Key as MplCoreKey};
use shank::ShankType;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use solana_system_interface::program as system_program;

use crate::{
    error::MplAgentToolsError,
    instruction::accounts::DelegateExecutionV1Accounts,
    state::{ExecutionDelegateRecordV1, ExecutiveProfileV1, Key},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct DelegateExecutionV1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<DelegateExecutionV1Args>() == 8);

pub fn delegate_execution_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &DelegateExecutionV1Args,
) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = DelegateExecutionV1Accounts::context(accounts)?;

    /****************************************************/
    /****************** Account Guards ******************/
    /****************************************************/
    // Assert that the executive profile is initialized and a valid executive profile.
    if ctx.accounts.executive_profile.owner != &crate::ID
        || ctx.accounts.executive_profile.data_len() == 0
        || ctx.accounts.executive_profile.try_borrow_data()?[0] != Key::ExecutiveProfileV1 as u8
    {
        return Err(MplAgentToolsError::ExecutiveProfileMustBeUninitialized.into());
    }

    let executive_profile_data = ctx.accounts.executive_profile.try_borrow_data()?;
    let executive_profile: &ExecutiveProfileV1 =
        from_bytes(&executive_profile_data[..core::mem::size_of::<ExecutiveProfileV1>()]);

    // Assert that the agent asset is initialized a valid agent asset.
    if ctx.accounts.agent_asset.owner != &mpl_core::ID
        || ctx.accounts.agent_asset.data_len() == 0
        || ctx.accounts.agent_asset.try_borrow_data()?[0] != MplCoreKey::AssetV1 as u8
    {
        return Err(MplAgentToolsError::InvalidCoreAsset.into());
    }

    // Also assert that the owner is the one signing.
    let asset = BaseAssetV1::try_from(ctx.accounts.agent_asset)?;
    if asset.owner != *ctx.accounts.authority.unwrap_or(ctx.accounts.payer).key {
        return Err(MplAgentToolsError::AssetOwnerMustBeTheOneToDelegateExecution.into());
    }

    // Assert that the agent identity is correct and initialized.
    // Accept both V1 and V2 discriminators since registration now creates V2 accounts.
    // Require at least 2 bytes so the subsequent bump read at offset 1 cannot panic
    // on a truncated account.
    {
        let agent_identity_data = ctx.accounts.agent_identity.try_borrow_data()?;
        if ctx.accounts.agent_identity.owner != &mpl_agent_identity::ID
            || agent_identity_data.len() < 2
            || (agent_identity_data[0] != MplAgentIdentityKey::AgentIdentityV1 as u8
                && agent_identity_data[0] != MplAgentIdentityKey::AgentIdentityV2 as u8)
        {
            return Err(MplAgentToolsError::AgentIdentityNotRegistered.into());
        }
    }

    // PDA seeds are the same for V1 and V2; bump is at byte offset 1 in both.
    let (agent_identity_pda, agent_identity_bump) =
        AgentIdentityV2::find_pda(ctx.accounts.agent_asset.key);

    if ctx.accounts.agent_identity.key != &agent_identity_pda
        || ctx.accounts.agent_identity.try_borrow_data()?[1] != agent_identity_bump
    {
        return Err(MplAgentToolsError::InvalidAgentIdentity.into());
    }

    // Check the delegation account is not initialized and is the correct derivation.
    let bump = ExecutionDelegateRecordV1::check_pda_derivation(
        ctx.accounts.execution_delegate_record,
        ctx.accounts.executive_profile.key,
        ctx.accounts.agent_asset.key,
    )?;

    if ctx.accounts.execution_delegate_record.owner != &system_program::ID
        || ctx.accounts.execution_delegate_record.data_len() > 0
    {
        return Err(MplAgentToolsError::ExecutionDelegateRecordMustBeUninitialized.into());
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
    ExecutionDelegateRecordV1::create_account(&ctx.accounts, bump)?;

    // Initialize the account using zero-copy.
    // Borrow the account data mutably and cast to our struct.
    let mut data = ctx
        .accounts
        .execution_delegate_record
        .try_borrow_mut_data()?;
    let execution_delegate_record: &mut ExecutionDelegateRecordV1 =
        bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<ExecutionDelegateRecordV1>()]);

    execution_delegate_record.initialize(
        bump,
        ctx.accounts.executive_profile.key,
        ctx.accounts.agent_asset.key,
        &executive_profile.authority,
    );

    Ok(())
}
