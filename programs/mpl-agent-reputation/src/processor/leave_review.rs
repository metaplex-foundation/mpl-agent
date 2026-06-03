use borsh::{BorshDeserialize, BorshSerialize};
use mpl_bubblegum::{
    instructions::MintV2CpiBuilder,
    types::{Creator, MetadataArgsV2, TokenStandard},
    utils::get_asset_id,
    ID as BUBBLEGUM_ID,
};
use mpl_core::types::Key as MplCoreKey;
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, keccak, msg, program::invoke,
    program_error::ProgramError, pubkey::Pubkey,
};
use solana_system_interface::program as system_program;

use crate::{
    error::MplAgentReputationError,
    instruction::accounts::LeaveReviewV1Accounts,
    state::{
        check_receipts_collection_pda, check_reviews_authority_pda, check_reviews_collection_pda,
        check_reviews_tree_pda, ReviewRecordV1, REVIEWS_AUTHORITY_PREFIX,
    },
};

/// Program ID of MPL Account Compression — the deployed compression program
/// Bubblegum V2 trees live in. We CPI into it to verify the work-receipt
/// leaf proof.
pub const MPL_ACCOUNT_COMPRESSION_ID: Pubkey =
    solana_program::pubkey!("mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW");

/// `verify_leaf` Anchor instruction discriminator =
/// `sha256("global:verify_leaf")[..8]`.
const VERIFY_LEAF_DISCRIMINATOR: [u8; 8] = [124, 220, 22, 223, 104, 10, 250, 224];

/// Number of named accounts in `LeaveReviewV1` (everything before the
/// merkle proof remaining accounts). Must stay in sync with the
/// `#[account]` list in `instruction.rs`.
const LEAVE_REVIEW_NAMED_ACCOUNTS: usize = 17;

/// Maximum length of the off-chain review JSON URI, in bytes.
pub const MAX_FEEDBACK_URI_LEN: usize = 200;

/// Arguments for the LeaveReviewV1 instruction.
///
/// The review references a work receipt by its `(merkle_tree, nonce)` pair —
/// the Bubblegum asset id is deterministically derived from those. Proof
/// data accompanies the call so we can validate the receipt on-chain.
#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, ShankType)]
pub struct LeaveReviewV1Args {
    /// Star rating, 1..=5.
    pub rating: u8,
    /// URI of the off-chain review JSON file.
    pub feedback_uri: String,
    /// Index of the reviews tree this review will be minted into (must
    /// match `["reviews_tree", reviews_tree_index_le]`).
    pub reviews_tree_index: u64,

    // --- Receipt merkle proof ------------------------------------------------
    /// Receipt leaf's nonce within its tree.
    pub receipt_nonce: u64,
    /// Receipt leaf's index within its tree.
    pub receipt_index: u32,
    /// Current root of the receipts merkle tree.
    pub receipt_root: [u8; 32],
    /// Hash of the receipt's MetadataArgsV2 + seller_fee_basis_points.
    pub receipt_data_hash: [u8; 32],
    /// Hash of the receipt's `asset_data` blob (DEFAULT_ASSET_DATA_HASH).
    pub receipt_asset_data_hash: [u8; 32],
    /// Receipt leaf flags.
    pub receipt_flags: u8,
}

pub fn leave_review_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: LeaveReviewV1Args,
) -> ProgramResult {
    /****************************************************/
    /****************** Account Setup *******************/
    /****************************************************/

    let ctx = LeaveReviewV1Accounts::context(accounts)?;
    // Remaining accounts after the named ones form the merkle proof path.
    let proof_accounts = if accounts.len() > LEAVE_REVIEW_NAMED_ACCOUNTS {
        &accounts[LEAVE_REVIEW_NAMED_ACCOUNTS..]
    } else {
        &[][..]
    };

    /****************************************************/
    /****************** Account Guards ******************/
    /****************************************************/

    assert_signer(ctx.accounts.payer)?;
    assert_signer(ctx.accounts.reviewer)?;

    // Validate the agent asset is an MPL Core AssetV1 and read its owner.
    if ctx.accounts.asset.owner != &mpl_core::ID {
        return Err(MplAgentReputationError::InvalidCoreAsset.into());
    }
    {
        let asset_data = ctx.accounts.asset.try_borrow_data()?;
        // BaseAssetV1 = key (1) + owner (32) + ... so any legitimate
        // AssetV1 is at least 33 bytes. Reject short buffers explicitly
        // so the `asset_data[1..33]` slice below cannot panic.
        if asset_data.len() < 33 || asset_data[0] != MplCoreKey::AssetV1 as u8 {
            return Err(MplAgentReputationError::InvalidCoreAsset.into());
        }
        let asset_owner = Pubkey::new_from_array(
            asset_data[1..33]
                .try_into()
                .map_err(|_| MplAgentReputationError::InvalidCoreAsset)?,
        );
        // The leaf owner of the review cNFT must be the agent's wallet.
        if *ctx.accounts.leaf_owner.key != asset_owner {
            return Err(MplAgentReputationError::LeafOwnerMismatch.into());
        }
    }

    // Validate program account keys.
    if *ctx.accounts.mpl_core_program.key != mpl_core::ID {
        return Err(MplAgentReputationError::InvalidMplCoreProgram.into());
    }
    if *ctx.accounts.bubblegum_program.key != BUBBLEGUM_ID {
        return Err(MplAgentReputationError::InvalidBubblegumProgram.into());
    }
    if *ctx.accounts.compression_program.key != MPL_ACCOUNT_COMPRESSION_ID {
        return Err(MplAgentReputationError::InvalidCompressionProgram.into());
    }
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MplAgentReputationError::InvalidSystemProgram.into());
    }

    // Reviews collection + authority + tree: all canonical PDAs.
    check_reviews_collection_pda(ctx.accounts.core_collection)?;
    let authority_bump = check_reviews_authority_pda(ctx.accounts.authority)?;
    check_reviews_tree_pda(ctx.accounts.merkle_tree, args.reviews_tree_index)?;
    // Receipts collection: canonical PDA from mpl-agent-tools.
    check_receipts_collection_pda(ctx.accounts.receipts_collection)?;

    /****************************************************/
    /***************** Argument Guards ******************/
    /****************************************************/

    if args.rating == 0 || args.rating > 5 {
        return Err(MplAgentReputationError::InvalidReviewRating.into());
    }
    if args.feedback_uri.is_empty() || args.feedback_uri.len() > MAX_FEEDBACK_URI_LEN {
        return Err(MplAgentReputationError::FeedbackUriInvalid.into());
    }

    /****************************************************/
    /************ Verify Work-Receipt Proof *************/
    /****************************************************/
    //
    // The receipts cNFT lives in `receipts_merkle_tree`. We reconstruct its
    // LeafSchemaV2 hash using:
    //   - id           = bubblegum_asset_id(receipts_merkle_tree, nonce)
    //   - owner        = reviewer            (the leaf belongs to the wallet leaving the review)
    //   - delegate     = reviewer            (MintV2 with leaf_delegate=None defaults to owner)
    //   - collection_hash = keccak(receipts_collection_pubkey)
    //   - data_hash / creator_hash / asset_data_hash / flags = caller-supplied
    //
    // The CPI to MPL Account Compression's `verify_leaf` then proves this
    // exact leaf hash is in the tree at the supplied index using the
    // remaining_accounts as the proof path.

    let receipts_merkle_tree = ctx.accounts.receipts_merkle_tree.key;
    let receipts_collection = ctx.accounts.receipts_collection.key;
    let receipt_asset_id = get_asset_id(receipts_merkle_tree, args.receipt_nonce);
    let receipt_owner = *ctx.accounts.reviewer.key;
    let receipt_delegate = receipt_owner;
    let collection_hash = keccak::hashv(&[receipts_collection.as_ref()]).to_bytes();

    // Bind the receipt to the reviewed agent: MintWorkReceiptV1 always
    // writes `creators = [{address: agent_asset, verified: false, share:
    // 100}]`, so we can compute the expected creator_hash on-chain from
    // `ctx.accounts.asset.key`. This prevents replay of a real receipt
    // for AgentA against a review for AgentB — the reconstructed leaf
    // hash here would no longer match the receipt's actual leaf in the
    // tree, so verify_leaf_cpi rejects.
    let expected_creator_hash = keccak::hashv(&[
        ctx.accounts.asset.key.as_ref(),
        &[0u8],   // verified = false
        &[100u8], // share = 100
    ])
    .to_bytes();

    let leaf_hash = keccak::hashv(&[
        // LeafSchemaV2 version byte
        &[2u8],
        receipt_asset_id.as_ref(),
        receipt_owner.as_ref(),
        receipt_delegate.as_ref(),
        &args.receipt_nonce.to_le_bytes(),
        &args.receipt_data_hash,
        &expected_creator_hash,
        &collection_hash,
        &args.receipt_asset_data_hash,
        &[args.receipt_flags],
    ])
    .to_bytes();

    verify_leaf_cpi(
        ctx.accounts.compression_program,
        ctx.accounts.receipts_merkle_tree,
        args.receipt_root,
        leaf_hash,
        args.receipt_index,
        proof_accounts,
    )?;

    /****************************************************/
    /************ Create Review Record PDA **************/
    /****************************************************/
    //
    // The PDA's existence is the idempotency guarantee — a second
    // `LeaveReviewV1` against the same receipt fails at account creation.

    let record_bump =
        ReviewRecordV1::check_pda_derivation(ctx.accounts.review_record, &receipt_asset_id)?;

    // Pre-flight check so we return a clean error rather than the
    // system-program create-account failure when re-reviewing.
    if ctx.accounts.review_record.data_len() != 0
        || *ctx.accounts.review_record.owner != system_program::id()
    {
        return Err(MplAgentReputationError::ReviewAlreadyExists.into());
    }

    ReviewRecordV1::create_account(
        ctx.accounts.review_record,
        ctx.accounts.system_program,
        ctx.accounts.payer,
        &receipt_asset_id,
        record_bump,
    )?;

    {
        let mut data = ctx.accounts.review_record.try_borrow_mut_data()?;
        let record: &mut ReviewRecordV1 =
            bytemuck::from_bytes_mut(&mut data[..core::mem::size_of::<ReviewRecordV1>()]);
        record.initialize(record_bump, ctx.accounts.reviewer.key, &receipt_asset_id);
    }

    /****************************************************/
    /**************** Mint review cNFT ******************/
    /****************************************************/

    msg!(
        "Review rating={} reviewer={} agent={} receipt={}",
        args.rating,
        ctx.accounts.reviewer.key,
        ctx.accounts.asset.key,
        receipt_asset_id,
    );

    let metadata = MetadataArgsV2 {
        name: format!("Agent Feedback ({}★)", args.rating),
        symbol: "AGENTFB".to_string(),
        uri: args.feedback_uri,
        seller_fee_basis_points: 0,
        primary_sale_happened: false,
        is_mutable: false,
        token_standard: Some(TokenStandard::NonFungible),
        creators: vec![Creator {
            address: *ctx.accounts.reviewer.key,
            verified: false,
            share: 100,
        }],
        collection: Some(*ctx.accounts.core_collection.key),
    };

    // The reviews_authority PDA was registered as tree_creator at
    // RegisterReviewsTreeV1 time AND is the reviews collection's
    // update_authority. Signing the MintV2 CPI with this single PDA
    // satisfies both `tree_creator_or_delegate` and the default
    // `collection_authority`.
    let authority_seeds: &[&[u8]] = &[REVIEWS_AUTHORITY_PREFIX, &[authority_bump]];

    MintV2CpiBuilder::new(ctx.accounts.bubblegum_program)
        .tree_config(ctx.accounts.tree_config)
        .payer(ctx.accounts.payer)
        .tree_creator_or_delegate(Some(ctx.accounts.authority))
        .collection_authority(None)
        .leaf_owner(ctx.accounts.leaf_owner)
        .leaf_delegate(None)
        .merkle_tree(ctx.accounts.merkle_tree)
        .core_collection(Some(ctx.accounts.core_collection))
        .mpl_core_cpi_signer(Some(ctx.accounts.mpl_core_cpi_signer))
        .log_wrapper(ctx.accounts.log_wrapper)
        .compression_program(ctx.accounts.compression_program)
        .mpl_core_program(ctx.accounts.mpl_core_program)
        .system_program(ctx.accounts.system_program)
        .metadata(metadata)
        .invoke_signed(&[authority_seeds])?;

    Ok(())
}

/// Construct + invoke MPL Account Compression's `verify_leaf` instruction
/// without depending on the SDK crate (whose newer versions conflict with
/// our pinned solana-program 2.3). Anchor wire format:
///   data = [discriminator(8) || root(32) || leaf(32) || index(u32 LE)]
///   accounts = [merkle_tree(read), ...proof_nodes(read)]
fn verify_leaf_cpi<'a>(
    compression_program: &AccountInfo<'a>,
    merkle_tree: &AccountInfo<'a>,
    root: [u8; 32],
    leaf: [u8; 32],
    index: u32,
    proof_accounts: &[AccountInfo<'a>],
) -> ProgramResult {
    let mut data = Vec::with_capacity(8 + 32 + 32 + 4);
    data.extend_from_slice(&VERIFY_LEAF_DISCRIMINATOR);
    data.extend_from_slice(&root);
    data.extend_from_slice(&leaf);
    data.extend_from_slice(&index.to_le_bytes());

    let mut metas = Vec::with_capacity(1 + proof_accounts.len());
    metas.push(solana_program::instruction::AccountMeta::new_readonly(
        *merkle_tree.key,
        false,
    ));
    for proof in proof_accounts {
        metas.push(solana_program::instruction::AccountMeta::new_readonly(
            *proof.key, false,
        ));
    }

    let ix = solana_program::instruction::Instruction {
        program_id: MPL_ACCOUNT_COMPRESSION_ID,
        accounts: metas,
        data,
    };

    let mut infos = Vec::with_capacity(2 + proof_accounts.len());
    infos.push(compression_program.clone());
    infos.push(merkle_tree.clone());
    for proof in proof_accounts {
        infos.push(proof.clone());
    }

    invoke(&ix, &infos)
}

/// Deserialize args from the instruction data slice (after the discriminator).
pub fn deserialize_leave_review_args(data: &[u8]) -> Result<LeaveReviewV1Args, ProgramError> {
    LeaveReviewV1Args::try_from_slice(data)
        .map_err(|_| MplAgentReputationError::InvalidInstructionData.into())
}
