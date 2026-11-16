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
    error::MplAgentReputationError,
    instruction::accounts::CreateReviewsCollectionV1Accounts,
    state::{check_reviews_authority_pda, check_reviews_collection_pda, REVIEWS_COLLECTION_PREFIX},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct CreateReviewsCollectionV1Args {
    #[skip]
    pub discriminator: u8,
    #[padding]
    pub _padding: [u8; 7],
}
const _: () = assert!(core::mem::size_of::<CreateReviewsCollectionV1Args>() == 8);

/// Permissionless, idempotent bootstrap: create the canonical reviews
/// collection at `["reviews_collection"]` PDA with the program's
/// `["reviews_authority"]` PDA as `update_authority`. Anyone may call
/// — but because the authority is a program-derived PDA (not the
/// caller), a hostile first caller cannot capture control. A second
/// call fails at MPL Core's CreateCollectionV2 because the account is
/// already initialized.
pub fn create_reviews_collection_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &CreateReviewsCollectionV1Args,
) -> ProgramResult {
    let ctx = CreateReviewsCollectionV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.mpl_core_program.key != mpl_core::ID {
        return Err(MplAgentReputationError::InvalidMplCoreProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentReputationError::InvalidSystemProgram.into());
    }

    let collection_bump = check_reviews_collection_pda(ctx.accounts.collection)?;
    let _ = check_reviews_authority_pda(ctx.accounts.authority)?;

    if ctx.accounts.collection.data_len() != 0
        || *ctx.accounts.collection.owner != system_program::id()
    {
        return Err(MplAgentReputationError::ReviewsCollectionAlreadyInitialized.into());
    }

    let collection_signer_seeds: &[&[u8]] = &[REVIEWS_COLLECTION_PREFIX, &[collection_bump]];

    CreateCollectionV2CpiBuilder::new(ctx.accounts.mpl_core_program)
        .collection(ctx.accounts.collection)
        .update_authority(Some(ctx.accounts.authority))
        .payer(ctx.accounts.payer)
        .system_program(ctx.accounts.system_program)
        .name("Agent Feedback".to_string())
        .uri("".to_string())
        .plugins(vec![
            PluginAuthorityPair {
                plugin: Plugin::BubblegumV2(BubblegumV2 {}),
                authority: None,
            },
            // Soulbound: every cNFT minted into this collection inherits
            // `permanent_lvl_frozen=true`. Authority = UpdateAuthority
            // (the reviews authority PDA), so only this program could
            // ever thaw — and it never exposes a thaw path.
            PluginAuthorityPair {
                plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
                authority: Some(PluginAuthority::UpdateAuthority),
            },
        ])
        .invoke_signed(&[collection_signer_seeds])?;

    Ok(())
}

pub fn cast_create_reviews_collection_args(
    data: &[u8],
) -> Result<&CreateReviewsCollectionV1Args, ProgramError> {
    if data.len() < core::mem::size_of::<CreateReviewsCollectionV1Args>() {
        return Err(MplAgentReputationError::InvalidInstructionData.into());
    }
    Ok(bytemuck::from_bytes(
        &data[..core::mem::size_of::<CreateReviewsCollectionV1Args>()],
    ))
}
