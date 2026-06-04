use bytemuck::{Pod, Zeroable};
use mpl_bubblegum::{instructions::CreateTreeConfigV2CpiBuilder, ID as BUBBLEGUM_ID};
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, rent::Rent, sysvar::Sysvar,
};
use solana_system_interface::{instruction as system_instruction, program as system_program};

use crate::{
    error::MplAgentReputationError,
    instruction::accounts::RegisterReviewsTreeV1Accounts,
    state::{
        check_reviews_authority_pda, check_reviews_tree_pda, REVIEWS_AUTHORITY_PREFIX,
        REVIEWS_TREE_PREFIX,
    },
};

/// MPL Account Compression program id (owner of the merkle tree account).
const MPL_ACCOUNT_COMPRESSION_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey!("mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW");

/// Permissionless tree registration: the caller picks an unused
/// `tree_index` and pays the rent. The tree is created at PDA
/// `["reviews_tree", index_le]` and Bubblegum is configured with
/// `tree_creator = ["reviews_authority"]` PDA, so the program signs
/// every future mint.
///
/// First-come-first-served: if two callers race for the same index the
/// loser's CreateAccount fails because the account is already
/// initialised. The loser just retries with a higher index.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterReviewsTreeV1Args {
    #[skip]
    pub discriminator: u8,
    #[padding]
    pub _pad: [u8; 7],
    pub tree_index: u64,
    pub max_depth: u32,
    pub max_buffer_size: u32,
    pub canopy_depth: u32,
    #[padding]
    pub _pad2: [u8; 4],
}
const _: () = assert!(core::mem::size_of::<RegisterReviewsTreeV1Args>() == 32);

pub fn register_reviews_tree_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: &RegisterReviewsTreeV1Args,
) -> ProgramResult {
    let ctx = RegisterReviewsTreeV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.bubblegum_program.key != BUBBLEGUM_ID {
        return Err(MplAgentReputationError::InvalidBubblegumProgram.into());
    }
    if *ctx.accounts.compression_program.key != MPL_ACCOUNT_COMPRESSION_ID {
        return Err(MplAgentReputationError::InvalidCompressionProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentReputationError::InvalidSystemProgram.into());
    }

    let authority_bump = check_reviews_authority_pda(ctx.accounts.authority)?;

    let tree_bump = check_reviews_tree_pda(ctx.accounts.merkle_tree, args.tree_index)?;
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

    let index_bytes = args.tree_index.to_le_bytes();
    let tree_seeds: &[&[u8]] = &[REVIEWS_TREE_PREFIX, &index_bytes, &[tree_bump]];
    let authority_seeds: &[&[u8]] = &[REVIEWS_AUTHORITY_PREFIX, &[authority_bump]];

    invoke_signed(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.merkle_tree.key,
            rent_lamports,
            size as u64,
            &MPL_ACCOUNT_COMPRESSION_ID,
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.merkle_tree.clone(),
            ctx.accounts.system_program.clone(),
        ],
        &[tree_seeds],
    )?;

    CreateTreeConfigV2CpiBuilder::new(ctx.accounts.bubblegum_program)
        .tree_config(ctx.accounts.tree_config)
        .merkle_tree(ctx.accounts.merkle_tree)
        .payer(ctx.accounts.payer)
        .tree_creator(Some(ctx.accounts.authority))
        .log_wrapper(ctx.accounts.log_wrapper)
        .compression_program(ctx.accounts.compression_program)
        .system_program(ctx.accounts.system_program)
        .max_depth(args.max_depth)
        .max_buffer_size(args.max_buffer_size)
        .public(false)
        .invoke_signed(&[authority_seeds])?;

    msg!(
        "Registered reviews tree #{} at {}",
        args.tree_index,
        ctx.accounts.merkle_tree.key,
    );

    Ok(())
}

/// Compute the merkle tree account size in bytes, matching
/// spl-concurrent-merkle-tree's struct layout. `Path<MAX_DEPTH>` has no
/// leaf field — verified empirically against Bubblegum's canopy check.
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
