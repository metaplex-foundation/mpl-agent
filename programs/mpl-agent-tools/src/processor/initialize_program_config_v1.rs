use bytemuck::{Pod, Zeroable};
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{BubblegumV2, PermanentFreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair},
};
use mpl_utils::{assert_derivation, assert_signer};
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_system_interface::program as system_program;

use crate::{
    error::MplAgentToolsError, instruction::accounts::InitializeToolsConfigV1Accounts,
    state::ToolsConfigV1,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct InitializeToolsConfigV1Args {
    #[skip]
    pub discriminator: u8,
    #[padding]
    pub _padding: [u8; 7],
}
const _: () = assert!(core::mem::size_of::<InitializeToolsConfigV1Args>() == 8);

/// Seeds for the canonical receipts collection PDA.
const RECEIPTS_COLLECTION_PREFIX: &[u8] = b"receipts_collection";

/// One-shot bootstrap: create the program config PDA, create the canonical
/// receipts collection (owned by the program config PDA), wire its
/// `BubblegumV2` + permanent-freeze plugins, and record `admin` as the
/// authority allowed to register receipts trees.
pub fn initialize_program_config_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &InitializeToolsConfigV1Args,
) -> ProgramResult {
    let ctx = InitializeToolsConfigV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.admin)?;

    if *ctx.accounts.mpl_core_program.key != mpl_core::ID {
        return Err(MplAgentToolsError::InvalidMplCoreProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentToolsError::InvalidSystemProgram.into());
    }

    // Derive the config PDA and verify the account matches.
    let config_bump = ToolsConfigV1::check_pda_derivation(ctx.accounts.program_config)?;
    if ctx.accounts.program_config.data_len() != 0
        || *ctx.accounts.program_config.owner != system_program::id()
    {
        return Err(MplAgentToolsError::ProgramConfigAlreadyInitialized.into());
    }

    // Derive the collection PDA and capture its bump for invoke_signed.
    let collection_bump: u8 = assert_derivation(
        &crate::ID,
        ctx.accounts.collection,
        &[RECEIPTS_COLLECTION_PREFIX],
        MplAgentToolsError::InvalidAccountData,
    )?;

    // Allocate + own the program config account.
    ToolsConfigV1::create_account(
        ctx.accounts.program_config,
        ctx.accounts.system_program,
        ctx.accounts.admin,
        config_bump,
    )?;

    {
        let mut data = ctx.accounts.program_config.try_borrow_mut_data()?;
        let cfg: &mut ToolsConfigV1 =
            bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<ToolsConfigV1>()]);
        cfg.initialize(
            config_bump,
            ctx.accounts.admin.key,
            ctx.accounts.collection.key,
        );
    }

    // Create the canonical receipts collection at the collection PDA, with
    // the program config PDA as its update authority — this is what lets
    // the program later sign as collection authority for cNFT mints.
    let collection_signer_seeds: &[&[u8]] = &[RECEIPTS_COLLECTION_PREFIX, &[collection_bump]];

    CreateCollectionV2CpiBuilder::new(ctx.accounts.mpl_core_program)
        .collection(ctx.accounts.collection)
        .update_authority(Some(ctx.accounts.program_config))
        .payer(ctx.accounts.admin)
        .system_program(ctx.accounts.system_program)
        .name("Agent Work Receipts".to_string())
        .uri("".to_string())
        .plugins(vec![
            PluginAuthorityPair {
                plugin: Plugin::BubblegumV2(BubblegumV2 {}),
                authority: None,
            },
            // Receipts inherit `permanent_lvl_frozen=true` so each receipt
            // is soulbound in the client's wallet. Authority = update
            // authority (the program config PDA), so only this program
            // could ever thaw — and it never exposes a thaw path.
            PluginAuthorityPair {
                plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
                authority: Some(PluginAuthority::UpdateAuthority),
            },
        ])
        .invoke_signed(&[collection_signer_seeds])?;

    Ok(())
}

/// Re-export the receipts collection prefix so other instructions can use it.
pub const fn receipts_collection_prefix() -> &'static [u8] {
    RECEIPTS_COLLECTION_PREFIX
}

/// Hand-rolled deserializer (zero-copy via bytemuck) for the args struct,
/// matching the convention used elsewhere in the program.
pub fn cast_args<'a>(data: &'a [u8]) -> Result<&'a InitializeToolsConfigV1Args, ProgramError> {
    if data.len() < core::mem::size_of::<InitializeToolsConfigV1Args>() {
        return Err(MplAgentToolsError::InvalidInstructionData.into());
    }
    Ok(bytemuck::from_bytes(
        &data[..core::mem::size_of::<InitializeToolsConfigV1Args>()],
    ))
}
