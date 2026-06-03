use bytemuck::{Pod, Zeroable};
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{BubblegumV2, PermanentFreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair},
};
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};
use solana_system_interface::program as system_program;

use crate::{
    error::MplAgentToolsError,
    instruction::accounts::CreateReceiptsCollectionV1Accounts,
    state::{
        check_receipts_authority_pda, check_receipts_collection_pda, RECEIPTS_COLLECTION_PREFIX,
    },
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct CreateReceiptsCollectionV1Args {
    #[skip]
    pub discriminator: u8,
    #[padding]
    pub _padding: [u8; 7],
}
const _: () = assert!(core::mem::size_of::<CreateReceiptsCollectionV1Args>() == 8);

/// Permissionless, idempotent bootstrap: create the canonical receipts
/// collection at `["receipts_collection"]` PDA with the program's
/// `["receipts_authority"]` PDA as `update_authority`. Anyone may call
/// — but because the authority is a program-derived PDA (not the
/// caller), a hostile first caller cannot capture control. A second
/// call fails at MPL Core's CreateCollectionV2 because the account is
/// already initialized.
pub fn create_receipts_collection_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &CreateReceiptsCollectionV1Args,
) -> ProgramResult {
    let ctx = CreateReceiptsCollectionV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.mpl_core_program.key != mpl_core::ID {
        return Err(MplAgentToolsError::InvalidMplCoreProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentToolsError::InvalidSystemProgram.into());
    }

    // Verify the collection account is the canonical PDA and the
    // authority account is the canonical signer PDA. Bumps captured for
    // invoke_signed below.
    let collection_bump = check_receipts_collection_pda(ctx.accounts.collection)?;
    let _ = check_receipts_authority_pda(ctx.accounts.authority)?;

    // Pre-flight: bail fast if the collection is already initialized.
    if ctx.accounts.collection.data_len() != 0
        || *ctx.accounts.collection.owner != system_program::id()
    {
        return Err(MplAgentToolsError::ReceiptsCollectionAlreadyInitialized.into());
    }

    let collection_signer_seeds: &[&[u8]] = &[RECEIPTS_COLLECTION_PREFIX, &[collection_bump]];

    CreateCollectionV2CpiBuilder::new(ctx.accounts.mpl_core_program)
        .collection(ctx.accounts.collection)
        .update_authority(Some(ctx.accounts.authority))
        .payer(ctx.accounts.payer)
        .system_program(ctx.accounts.system_program)
        .name("Agent Work Receipts".to_string())
        .uri("".to_string())
        .plugins(vec![
            PluginAuthorityPair {
                plugin: Plugin::BubblegumV2(BubblegumV2 {}),
                authority: None,
            },
            // Soulbound: every cNFT minted into this collection inherits
            // `permanent_lvl_frozen=true`. Authority = UpdateAuthority
            // (the receipts authority PDA), so only this program could
            // ever thaw — and it never exposes a thaw path.
            PluginAuthorityPair {
                plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
                authority: Some(PluginAuthority::UpdateAuthority),
            },
        ])
        .invoke_signed(&[collection_signer_seeds])?;

    Ok(())
}

pub fn cast_create_receipts_collection_args(
    data: &[u8],
) -> Result<&CreateReceiptsCollectionV1Args, ProgramError> {
    if data.len() < core::mem::size_of::<CreateReceiptsCollectionV1Args>() {
        return Err(MplAgentToolsError::InvalidInstructionData.into());
    }
    Ok(bytemuck::from_bytes(
        &data[..core::mem::size_of::<CreateReceiptsCollectionV1Args>()],
    ))
}
