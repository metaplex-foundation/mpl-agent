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
    error::MplAgentToolsError,
    instruction::accounts::RegisterReceiptsTreeV1Accounts,
    state::{
        check_receipts_authority_pda, check_receipts_tree_pda, RECEIPTS_AUTHORITY_PREFIX,
        RECEIPTS_TREE_PREFIX,
    },
};

/// MPL Account Compression program id (owner of the merkle tree account).
const MPL_ACCOUNT_COMPRESSION_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey!("mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW");

/// Permissionless tree registration: the caller picks an unused
/// `tree_index` and pays the rent. The tree is created at PDA
/// `["receipts_tree", index_le]` and Bubblegum is configured with
/// `tree_creator = ["receipts_authority"]` PDA, so the program signs
/// every future mint.
///
/// First-come-first-served: if two callers race for the same index the
/// loser's CreateAccount fails because the account is already
/// initialised. The loser just retries with a higher index.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterReceiptsTreeV1Args {
    #[skip]
    pub discriminator: u8,
    /// Padding to align the `u64` field.
    #[padding]
    pub _pad: [u8; 7],
    pub tree_index: u64,
    pub max_depth: u32,
    pub max_buffer_size: u32,
    pub canopy_depth: u32,
    #[padding]
    pub _pad2: [u8; 4],
}
const _: () = assert!(core::mem::size_of::<RegisterReceiptsTreeV1Args>() == 32);

pub fn register_receipts_tree_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: &RegisterReceiptsTreeV1Args,
) -> ProgramResult {
    let ctx = RegisterReceiptsTreeV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.payer)?;

    if *ctx.accounts.bubblegum_program.key != BUBBLEGUM_ID {
        return Err(MplAgentToolsError::InvalidBubblegumProgram.into());
    }
    if *ctx.accounts.compression_program.key != MPL_ACCOUNT_COMPRESSION_ID {
        return Err(MplAgentToolsError::InvalidCompressionProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentToolsError::InvalidSystemProgram.into());
    }

    // Authority PDA — captured for invoke_signed at tree create time.
    let authority_bump = check_receipts_authority_pda(ctx.accounts.authority)?;

    // Tree must match the canonical PDA for the supplied tree_index.
    let tree_bump = check_receipts_tree_pda(ctx.accounts.merkle_tree, args.tree_index)?;
    if ctx.accounts.merkle_tree.data_len() != 0
        || *ctx.accounts.merkle_tree.owner != system_program::id()
    {
        return Err(MplAgentToolsError::InvalidAccountData.into());
    }

    let size = merkle_tree_account_size(
        args.max_depth as usize,
        args.max_buffer_size as usize,
        args.canopy_depth as usize,
    )?;
    let rent_lamports = Rent::get()?.minimum_balance(size);

    let index_bytes = args.tree_index.to_le_bytes();
    let tree_seeds: &[&[u8]] = &[RECEIPTS_TREE_PREFIX, &index_bytes, &[tree_bump]];
    let authority_seeds: &[&[u8]] = &[RECEIPTS_AUTHORITY_PREFIX, &[authority_bump]];

    // Allocate the merkle tree account at the PDA, owned by the
    // compression program. The tree-PDA signs CreateAccount via its own
    // seeds — that's only used here, never for subsequent mints.
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

    // CPI Bubblegum's CreateTreeConfigV2 with `tree_creator =
    // receipts_authority` PDA — this is the signer the program will
    // present at every subsequent mint.
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
        "Registered receipts tree #{} at {}",
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
    // ChangeLog<MAX_DEPTH>: root (32) + [Node; MAX_DEPTH] + index (u32) + _padding (u32)
    let change_log_size = NODE_SIZE + max_depth * NODE_SIZE + 4 + 4;
    let change_logs_total = change_log_size
        .checked_mul(max_buffer_size)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    // sequence_number, active_index, buffer_size (each u64) = 24
    let scalars = 24;
    // Path<MAX_DEPTH>: [Node; MAX_DEPTH] + index (u32) + _padding (u32)
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

pub fn cast_register_receipts_tree_args(
    data: &[u8],
) -> Result<&RegisterReceiptsTreeV1Args, ProgramError> {
    if data.len() < core::mem::size_of::<RegisterReceiptsTreeV1Args>() {
        return Err(MplAgentToolsError::InvalidInstructionData.into());
    }
    Ok(bytemuck::from_bytes(
        &data[..core::mem::size_of::<RegisterReceiptsTreeV1Args>()],
    ))
}
