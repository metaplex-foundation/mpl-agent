use bytemuck::{Pod, Zeroable};
use mpl_bubblegum::{instructions::BurnV2CpiBuilder, ID as BUBBLEGUM_ID};
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
use solana_system_interface::program as system_program;

use crate::{
    error::MplAgentToolsError,
    instruction::accounts::CloseWorkReceiptV1Accounts,
    state::{
        check_receipts_authority_pda, check_receipts_collection_pda, check_receipts_tree_pda,
        RECEIPTS_AUTHORITY_PREFIX,
    },
};

/// MPL Account Compression program id — pinned to defeat spoofing.
const MPL_ACCOUNT_COMPRESSION_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey!("mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW");

/// Arguments for `CloseWorkReceiptV1`. Mirrors Bubblegum's
/// `BurnV2InstructionArgs` exactly — the caller supplies the leaf proof
/// data the compression program will verify. Fixed-size; uses the
/// repo's Pod/zero-copy ABI convention.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable, ShankType)]
pub struct CloseWorkReceiptV1Args {
    #[skip]
    pub discriminator: u8,
    /// Padding to align the `u64` field that follows.
    #[padding]
    pub _pad: [u8; 7],
    /// Index of the receipts tree the receipt lives in.
    pub tree_index: u64,
    /// Current root of the receipts merkle tree.
    pub root: [u8; 32],
    /// Hash of the receipt's MetadataArgsV2 + seller_fee_basis_points.
    pub data_hash: [u8; 32],
    /// Hash of the receipt's creators array.
    pub creator_hash: [u8; 32],
    /// Hash of the receipt's `asset_data` blob.
    pub asset_data_hash: [u8; 32],
    /// Leaf flags.
    pub flags: u8,
    /// Padding to align the `u64` field that follows.
    #[padding]
    pub _pad2: [u8; 7],
    /// Receipt leaf's nonce within its tree.
    pub nonce: u64,
    /// Receipt leaf's index within its tree.
    pub index: u32,
    /// Tail padding for 8-byte alignment.
    #[padding]
    pub _pad3: [u8; 4],
}
const _: () = assert!(core::mem::size_of::<CloseWorkReceiptV1Args>() == 168);
const _: () = assert!(core::mem::size_of::<CloseWorkReceiptV1Args>() % 8 == 0);
const _: () = assert!(core::mem::align_of::<CloseWorkReceiptV1Args>() == 8);

/// Number of named accounts in `CloseWorkReceiptV1` (everything before
/// the merkle proof remaining accounts). Must stay in sync with the
/// `#[account]` list in `instruction.rs`.
const CLOSE_WORK_RECEIPT_NAMED_ACCOUNTS: usize = 12;

/// Burn (close) a work-receipt cNFT. Authorized at the cNFT level by the
/// collection's `PermanentBurnDelegate` plugin whose authority is the
/// `UpdateAuthority` — i.e. the program's receipts_authority PDA. The
/// program enforces a single rule on top: the caller must sign as the
/// leaf owner. Anyone passing a valid proof + a matching signature can
/// close their own receipts (typical use: cleaning up spam).
pub fn close_work_receipt_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: &CloseWorkReceiptV1Args,
) -> ProgramResult {
    let ctx = CloseWorkReceiptV1Accounts::context(accounts)?;
    let proof_accounts = if accounts.len() > CLOSE_WORK_RECEIPT_NAMED_ACCOUNTS {
        &accounts[CLOSE_WORK_RECEIPT_NAMED_ACCOUNTS..]
    } else {
        &[][..]
    };

    assert_signer(ctx.accounts.payer)?;
    assert_signer(ctx.accounts.leaf_owner)?;

    if *ctx.accounts.mpl_core_program.key != mpl_core::ID {
        return Err(MplAgentToolsError::InvalidMplCoreProgram.into());
    }
    if *ctx.accounts.bubblegum_program.key != BUBBLEGUM_ID {
        return Err(MplAgentToolsError::InvalidBubblegumProgram.into());
    }
    if *ctx.accounts.compression_program.key != MPL_ACCOUNT_COMPRESSION_ID {
        return Err(MplAgentToolsError::InvalidCompressionProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentToolsError::InvalidSystemProgram.into());
    }

    let authority_bump = check_receipts_authority_pda(ctx.accounts.authority)?;
    check_receipts_tree_pda(ctx.accounts.merkle_tree, args.tree_index)?;
    check_receipts_collection_pda(ctx.accounts.core_collection)?;

    msg!(
        "Close receipt leaf_owner={} tree_index={} nonce={}",
        ctx.accounts.leaf_owner.key,
        args.tree_index,
        args.nonce,
    );

    let authority_seeds: &[&[u8]] = &[RECEIPTS_AUTHORITY_PREFIX, &[authority_bump]];

    let mut burn = BurnV2CpiBuilder::new(ctx.accounts.bubblegum_program);
    burn.tree_config(ctx.accounts.tree_config)
        .payer(ctx.accounts.payer)
        .authority(Some(ctx.accounts.authority))
        .leaf_owner(ctx.accounts.leaf_owner)
        .leaf_delegate(None)
        .merkle_tree(ctx.accounts.merkle_tree)
        .core_collection(Some(ctx.accounts.core_collection))
        .mpl_core_cpi_signer(Some(ctx.accounts.mpl_core_cpi_signer))
        .log_wrapper(ctx.accounts.log_wrapper)
        .compression_program(ctx.accounts.compression_program)
        .mpl_core_program(ctx.accounts.mpl_core_program)
        .system_program(ctx.accounts.system_program)
        .root(args.root)
        .data_hash(args.data_hash)
        .creator_hash(args.creator_hash)
        .asset_data_hash(args.asset_data_hash)
        .flags(args.flags)
        .nonce(args.nonce)
        .index(args.index);

    for proof in proof_accounts {
        burn.add_remaining_account(proof, false, false);
    }
    burn.invoke_signed(&[authority_seeds])?;

    Ok(())
}

pub fn cast_close_work_receipt_args(data: &[u8]) -> Result<&CloseWorkReceiptV1Args, ProgramError> {
    if data.len() < core::mem::size_of::<CloseWorkReceiptV1Args>() {
        return Err(MplAgentToolsError::InvalidInstructionData.into());
    }
    Ok(bytemuck::from_bytes(
        &data[..core::mem::size_of::<CloseWorkReceiptV1Args>()],
    ))
}
