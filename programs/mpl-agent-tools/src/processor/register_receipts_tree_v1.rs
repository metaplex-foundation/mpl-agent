use bytemuck::{Pod, Zeroable};
use mpl_bubblegum::{instructions::CreateTreeConfigV2CpiBuilder, ID as BUBBLEGUM_ID};
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::{
    error::MplAgentToolsError,
    instruction::accounts::RegisterReceiptsTreeV1Accounts,
    state::{check_receipts_tree_pda, ToolsConfigV1, RECEIPTS_TREE_PREFIX},
};

/// MPL Account Compression program id (owner of the merkle tree account).
const MPL_ACCOUNT_COMPRESSION_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey!("mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW");

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct RegisterReceiptsTreeV1Args {
    #[skip]
    pub discriminator: u8,
    /// Padding to align the following `u32` fields.
    #[padding]
    pub _pad: [u8; 3],
    pub max_depth: u32,
    pub max_buffer_size: u32,
    pub canopy_depth: u32,
}
const _: () = assert!(core::mem::size_of::<RegisterReceiptsTreeV1Args>() == 16);

pub fn register_receipts_tree_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: &RegisterReceiptsTreeV1Args,
) -> ProgramResult {
    let ctx = RegisterReceiptsTreeV1Accounts::context(accounts)?;

    assert_signer(ctx.accounts.admin)?;

    if *ctx.accounts.bubblegum_program.key != BUBBLEGUM_ID {
        return Err(MplAgentToolsError::InvalidBubblegumProgram.into());
    }
    if *ctx.accounts.compression_program.key != MPL_ACCOUNT_COMPRESSION_ID {
        return Err(MplAgentToolsError::InvalidCompressionProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentToolsError::InvalidSystemProgram.into());
    }

    // Verify program_config is initialized and admin matches.
    let config_bump = ToolsConfigV1::check_pda_derivation(ctx.accounts.program_config)?;
    if ctx.accounts.program_config.owner != &crate::ID
        || ctx.accounts.program_config.data_len() < core::mem::size_of::<ToolsConfigV1>()
    {
        return Err(MplAgentToolsError::ProgramConfigNotInitialized.into());
    }

    let next_tree_index: u64 = {
        let cfg_data = ctx.accounts.program_config.try_borrow_data()?;
        let cfg: &ToolsConfigV1 =
            bytemuck::from_bytes(&cfg_data[..core::mem::size_of::<ToolsConfigV1>()]);
        if cfg.admin != *ctx.accounts.admin.key {
            return Err(MplAgentToolsError::UnauthorizedAdmin.into());
        }
        cfg.next_tree_index
    };

    // Verify merkle_tree matches the canonical PDA for next_tree_index.
    let tree_bump = check_receipts_tree_pda(ctx.accounts.merkle_tree, next_tree_index)?;
    if ctx.accounts.merkle_tree.data_len() != 0
        || *ctx.accounts.merkle_tree.owner != system_program::id()
    {
        return Err(MplAgentToolsError::InvalidAccountData.into());
    }

    // Compute the on-chain merkle tree byte size (matches spl-account-
    // compression's getMerkleTreeSize) and allocate the account at the
    // tree PDA address with mpl-account-compression as the owner.
    let tree_size = merkle_tree_account_size(
        args.max_depth as usize,
        args.max_buffer_size as usize,
        args.canopy_depth as usize,
    )?;
    let rent_lamports = Rent::get()?.minimum_balance(tree_size);

    let index_bytes = next_tree_index.to_le_bytes();
    let tree_signer_seeds: &[&[u8]] = &[RECEIPTS_TREE_PREFIX, &index_bytes, &[tree_bump]];

    invoke_signed(
        &system_instruction::create_account(
            ctx.accounts.admin.key,
            ctx.accounts.merkle_tree.key,
            rent_lamports,
            tree_size as u64,
            &MPL_ACCOUNT_COMPRESSION_ID,
        ),
        &[
            ctx.accounts.admin.clone(),
            ctx.accounts.merkle_tree.clone(),
            ctx.accounts.system_program.clone(),
        ],
        &[tree_signer_seeds],
    )?;

    // CPI Bubblegum's CreateTreeConfigV2 with tree_creator = program_config
    // PDA (signed via config bump). The tree is now controlled by this
    // program for every future mint.
    let config_signer_seeds: &[&[u8]] = &[ToolsConfigV1::PREFIX, &[config_bump]];

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
        .invoke_signed(&[config_signer_seeds])?;

    // Bump next_tree_index in the config so the next call addresses a
    // fresh PDA.
    {
        let mut cfg_data = ctx.accounts.program_config.try_borrow_mut_data()?;
        let cfg: &mut ToolsConfigV1 =
            bytemuck::from_bytes_mut(&mut cfg_data[..core::mem::size_of::<ToolsConfigV1>()]);
        cfg.next_tree_index = next_tree_index
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    msg!(
        "Registered receipts tree #{} at {}",
        next_tree_index,
        ctx.accounts.merkle_tree.key,
    );

    Ok(())
}

/// Compute the merkle tree account size in bytes, matching
/// spl-account-compression's `getMerkleTreeSize(maxDepth, maxBuffer,
/// canopyDepth)`. Header is 88 bytes; tree body layout is fixed by
/// max_depth + max_buffer_size; canopy is `(2^(canopy_depth+1) - 2) * 32`
/// bytes, or zero when canopy_depth == 0.
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
    // Path<MAX_DEPTH>: [Node; MAX_DEPTH] + index + _padding (no leaf field
    // in spl-concurrent-merkle-tree's Path type — verified empirically
    // against bubblegum's create_tree canopy check)
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

pub fn cast_register_receipts_tree_args<'a>(
    data: &'a [u8],
) -> Result<&'a RegisterReceiptsTreeV1Args, ProgramError> {
    if data.len() < core::mem::size_of::<RegisterReceiptsTreeV1Args>() {
        return Err(MplAgentToolsError::InvalidInstructionData.into());
    }
    Ok(bytemuck::from_bytes(
        &data[..core::mem::size_of::<RegisterReceiptsTreeV1Args>()],
    ))
}
