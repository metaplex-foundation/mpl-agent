use bytemuck::{Pod, Zeroable};
use mpl_bubblegum::{instructions::CreateTreeConfigV2CpiBuilder, ID as BUBBLEGUM_ID};
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{BubblegumV2, PermanentFreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair},
};
use mpl_utils::{assert_derivation, assert_signer};
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction, system_program,
    sysvar::Sysvar,
};

use crate::{
    error::MplAgentReputationError,
    instruction::accounts::{InitializeReviewsConfigV1Accounts, RegisterReviewsTreeV1Accounts},
    state::{check_reviews_tree_pda, ReviewsConfigV1, REVIEWS_TREE_PREFIX},
};

const REVIEWS_COLLECTION_PREFIX: &[u8] = b"reviews_collection";
const MPL_ACCOUNT_COMPRESSION_ID: Pubkey =
    solana_program::pubkey!("mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW");

// ---------- InitializeReviewsConfigV1 ------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct InitializeReviewsConfigV1Args {
    #[skip]
    pub discriminator: u8,
    #[padding]
    pub _padding: [u8; 7],
}
const _: () = assert!(core::mem::size_of::<InitializeReviewsConfigV1Args>() == 8);

/// Bootstrap the program: create the singleton config PDA, create the
/// canonical reviews collection, and capture the receipts collection
/// reference. Receipts collection is supplied by the caller (admin
/// already initialized agent-tools' program config and knows its address).
pub fn initialize_program_config_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: &InitializeReviewsConfigV1Args,
) -> ProgramResult {
    let ctx = InitializeReviewsConfigV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.admin)?;
    if *ctx.accounts.mpl_core_program.key != mpl_core::ID {
        return Err(MplAgentReputationError::InvalidMplCoreProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentReputationError::InvalidSystemProgram.into());
    }

    let config_bump = ReviewsConfigV1::check_pda_derivation(ctx.accounts.program_config)?;
    if ctx.accounts.program_config.data_len() != 0
        || *ctx.accounts.program_config.owner != system_program::id()
    {
        return Err(MplAgentReputationError::ProgramConfigAlreadyInitialized.into());
    }

    let collection_bump: u8 = assert_derivation(
        &crate::ID,
        ctx.accounts.reviews_collection,
        &[REVIEWS_COLLECTION_PREFIX],
        MplAgentReputationError::InvalidAccountData,
    )?;

    ReviewsConfigV1::create_account(
        ctx.accounts.program_config,
        ctx.accounts.system_program,
        ctx.accounts.admin,
        config_bump,
    )?;
    {
        let mut data = ctx.accounts.program_config.try_borrow_mut_data()?;
        let cfg: &mut ReviewsConfigV1 =
            bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<ReviewsConfigV1>()]);
        cfg.initialize(
            config_bump,
            ctx.accounts.admin.key,
            ctx.accounts.reviews_collection.key,
            ctx.accounts.receipts_collection.key,
        );
    }

    let collection_seeds: &[&[u8]] = &[REVIEWS_COLLECTION_PREFIX, &[collection_bump]];

    CreateCollectionV2CpiBuilder::new(ctx.accounts.mpl_core_program)
        .collection(ctx.accounts.reviews_collection)
        .update_authority(Some(ctx.accounts.program_config))
        .payer(ctx.accounts.admin)
        .system_program(ctx.accounts.system_program)
        .name("Agent Reviews".to_string())
        .uri("".to_string())
        .plugins(vec![
            PluginAuthorityPair {
                plugin: Plugin::BubblegumV2(BubblegumV2 {}),
                authority: None,
            },
            PluginAuthorityPair {
                plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
                authority: Some(PluginAuthority::UpdateAuthority),
            },
        ])
        .invoke_signed(&[collection_seeds])?;

    Ok(())
}

pub fn cast_initialize_program_config_args(
    data: &[u8],
) -> Result<&InitializeReviewsConfigV1Args, ProgramError> {
    if data.len() < core::mem::size_of::<InitializeReviewsConfigV1Args>() {
        return Err(MplAgentReputationError::InvalidInstructionData.into());
    }
    Ok(bytemuck::from_bytes(
        &data[..core::mem::size_of::<InitializeReviewsConfigV1Args>()],
    ))
}

// ---------- RegisterReviewsTreeV1 ----------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterReviewsTreeV1Args {
    #[skip]
    pub discriminator: u8,
    #[padding]
    pub _pad: [u8; 3],
    pub max_depth: u32,
    pub max_buffer_size: u32,
    pub canopy_depth: u32,
}
const _: () = assert!(core::mem::size_of::<RegisterReviewsTreeV1Args>() == 16);

pub fn register_reviews_tree_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: &RegisterReviewsTreeV1Args,
) -> ProgramResult {
    let ctx = RegisterReviewsTreeV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.admin)?;
    if *ctx.accounts.bubblegum_program.key != BUBBLEGUM_ID {
        return Err(MplAgentReputationError::InvalidBubblegumProgram.into());
    }
    if *ctx.accounts.compression_program.key != MPL_ACCOUNT_COMPRESSION_ID {
        return Err(MplAgentReputationError::InvalidCompressionProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentReputationError::InvalidSystemProgram.into());
    }

    let config_bump = ReviewsConfigV1::check_pda_derivation(ctx.accounts.program_config)?;
    if ctx.accounts.program_config.owner != &crate::ID
        || ctx.accounts.program_config.data_len() < core::mem::size_of::<ReviewsConfigV1>()
    {
        return Err(MplAgentReputationError::ProgramConfigNotInitialized.into());
    }

    let next_tree_index: u64 = {
        let cfg_data = ctx.accounts.program_config.try_borrow_data()?;
        let cfg: &ReviewsConfigV1 =
            bytemuck::from_bytes(&cfg_data[..core::mem::size_of::<ReviewsConfigV1>()]);
        if cfg.admin != *ctx.accounts.admin.key {
            return Err(MplAgentReputationError::UnauthorizedAdmin.into());
        }
        cfg.next_tree_index
    };

    let tree_bump = check_reviews_tree_pda(ctx.accounts.merkle_tree, next_tree_index)?;
    if ctx.accounts.merkle_tree.data_len() != 0
        || *ctx.accounts.merkle_tree.owner != system_program::id()
    {
        return Err(MplAgentReputationError::InvalidAccountData.into());
    }

    let size = merkle_tree_account_size(
        args.max_depth as usize,
        args.max_buffer_size as usize,
        args.canopy_depth as usize,
    )?;
    let rent_lamports = Rent::get()?.minimum_balance(size);
    let index_bytes = next_tree_index.to_le_bytes();
    let tree_seeds: &[&[u8]] = &[REVIEWS_TREE_PREFIX, &index_bytes, &[tree_bump]];

    invoke_signed(
        &system_instruction::create_account(
            ctx.accounts.admin.key,
            ctx.accounts.merkle_tree.key,
            rent_lamports,
            size as u64,
            &MPL_ACCOUNT_COMPRESSION_ID,
        ),
        &[
            ctx.accounts.admin.clone(),
            ctx.accounts.merkle_tree.clone(),
            ctx.accounts.system_program.clone(),
        ],
        &[tree_seeds],
    )?;

    let config_seeds: &[&[u8]] = &[ReviewsConfigV1::PREFIX, &[config_bump]];

    CreateTreeConfigV2CpiBuilder::new(ctx.accounts.bubblegum_program)
        .tree_config(ctx.accounts.tree_config)
        .merkle_tree(ctx.accounts.merkle_tree)
        .payer(ctx.accounts.admin)
        .tree_creator(Some(ctx.accounts.program_config))
        .log_wrapper(ctx.accounts.log_wrapper)
        .compression_program(ctx.accounts.compression_program)
        .system_program(ctx.accounts.system_program)
        .max_depth(args.max_depth)
        .max_buffer_size(args.max_buffer_size)
        .public(false)
        .invoke_signed(&[config_seeds])?;

    {
        let mut cfg_data = ctx.accounts.program_config.try_borrow_mut_data()?;
        let cfg: &mut ReviewsConfigV1 =
            bytemuck::from_bytes_mut(&mut cfg_data[..core::mem::size_of::<ReviewsConfigV1>()]);
        cfg.next_tree_index = next_tree_index
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    msg!(
        "Registered reviews tree #{} at {}",
        next_tree_index,
        ctx.accounts.merkle_tree.key
    );

    Ok(())
}

pub fn cast_register_reviews_tree_args(
    data: &[u8],
) -> Result<&RegisterReviewsTreeV1Args, ProgramError> {
    if data.len() < core::mem::size_of::<RegisterReviewsTreeV1Args>() {
        return Err(MplAgentReputationError::InvalidInstructionData.into());
    }
    Ok(bytemuck::from_bytes(
        &data[..core::mem::size_of::<RegisterReviewsTreeV1Args>()],
    ))
}

// Same formula as in agent-tools; kept inline to avoid cross-crate state.
fn merkle_tree_account_size(
    max_depth: usize,
    max_buffer_size: usize,
    canopy_depth: usize,
) -> Result<usize, ProgramError> {
    const HEADER_SIZE: usize = 88;
    const NODE_SIZE: usize = 32;
    let change_log_size = NODE_SIZE + max_depth * NODE_SIZE + 4 + 4;
    let change_logs_total = change_log_size
        .checked_mul(max_buffer_size)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let scalars = 24;
    // Path<MAX_DEPTH>: [Node; MAX_DEPTH] + index + _padding (no leaf field).
    let path_size = max_depth * NODE_SIZE + 4 + 4;
    let canopy_size = if canopy_depth == 0 {
        0
    } else {
        let nodes = (1usize << (canopy_depth + 1))
            .checked_sub(2)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        nodes
            .checked_mul(NODE_SIZE)
            .ok_or(ProgramError::ArithmeticOverflow)?
    };
    Ok(HEADER_SIZE + scalars + change_logs_total + path_size + canopy_size)
}
