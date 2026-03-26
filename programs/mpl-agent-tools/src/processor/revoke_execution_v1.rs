use bytemuck::{from_bytes, Pod, Zeroable};
use mpl_core::{accounts::BaseAssetV1, types::Key as MplCoreKey};
use mpl_utils::close_account_raw;
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey, system_program,
};

use crate::{
    error::MplAgentToolsError,
    instruction::accounts::RevokeExecutionV1Accounts,
    state::{ExecutionDelegateRecordV1, Key},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RevokeExecutionV1Args {
    /// Instruction discriminator (not included in IDL).
    #[skip]
    pub discriminator: u8,
    /// Padding for alignment.
    #[padding]
    pub _padding: [u8; 7],
}

// Compile-time assertion to ensure struct is properly sized.
const _: () = assert!(core::mem::size_of::<RevokeExecutionV1Args>() == 8);

pub fn revoke_execution_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &RevokeExecutionV1Args,
) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = RevokeExecutionV1Accounts::context(accounts)?;

    /****************************************************/
    /****************** Account Guards ******************/
    /****************************************************/

    // Assert that the execution delegate record is initialized and owned by our program.
    if ctx.accounts.execution_delegate_record.owner != &crate::ID
        || ctx.accounts.execution_delegate_record.data_len() == 0
        || ctx.accounts.execution_delegate_record.try_borrow_data()?[0]
            != Key::ExecutionDelegateRecordV1 as u8
    {
        return Err(MplAgentToolsError::ExecutionDelegateRecordMustBeInitialized.into());
    }

    // Read the delegate record via bytemuck and extract needed values.
    // We must drop the borrow before calling close_account which needs mut access.
    let (record_executive_profile, record_authority, record_agent_asset): (Pubkey, Pubkey, Pubkey) = {
        let record_data = ctx.accounts.execution_delegate_record.try_borrow_data()?;
        let record: &ExecutionDelegateRecordV1 =
            from_bytes(&record_data[..core::mem::size_of::<ExecutionDelegateRecordV1>()]);
        (
            record.executive_profile,
            record.authority,
            record.agent_asset,
        )
    };

    // Verify the record's agent_asset matches the passed agent_asset account.
    if record_agent_asset != *ctx.accounts.agent_asset.key {
        return Err(MplAgentToolsError::InvalidExecutionDelegateRecordDerivation.into());
    }

    // Check PDA derivation.
    ExecutionDelegateRecordV1::check_pda_derivation(
        ctx.accounts.execution_delegate_record,
        &record_executive_profile,
        ctx.accounts.agent_asset.key,
    )?;

    // Assert that the agent asset is a valid MPL Core asset.
    if ctx.accounts.agent_asset.owner != &mpl_core::ID
        || ctx.accounts.agent_asset.data_len() == 0
        || ctx.accounts.agent_asset.try_borrow_data()?[0] != MplCoreKey::AssetV1 as u8
    {
        return Err(MplAgentToolsError::InvalidCoreAsset.into());
    }

    // Read the asset to get the owner.
    let asset = BaseAssetV1::try_from(ctx.accounts.agent_asset)?;

    // Determine the signer.
    let signer = ctx.accounts.authority.unwrap_or(ctx.accounts.payer);

    // Authorization check: signer must be the asset owner OR the executive authority.
    let is_asset_owner = asset.owner == *signer.key;
    let is_executive_authority = record_authority == *signer.key;

    if !is_asset_owner && !is_executive_authority {
        return Err(MplAgentToolsError::UnauthorizedRevoke.into());
    }

    // Validate system program.
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentToolsError::InvalidSystemProgram.into());
    }

    /****************************************************/
    /********************* Actions **********************/
    /****************************************************/

    // Close the execution delegate record and refund rent to destination.
    close_account_raw(
        ctx.accounts.destination,
        ctx.accounts.execution_delegate_record,
    )?;

    Ok(())
}
