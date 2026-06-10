use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::from_bytes;
use mpl_bubblegum::{
    instructions::MintV2CpiBuilder,
    types::{Creator, MetadataArgsV2, TokenStandard},
    ID as BUBBLEGUM_ID,
};
use mpl_core::types::Key as MplCoreKey;
use mpl_utils::assert_signer;
use shank::ShankType;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
use solana_system_interface::program as system_program;

use crate::{
    error::MplAgentToolsError,
    instruction::accounts::MintWorkReceiptV1Accounts,
    state::{
        check_receipts_authority_pda, check_receipts_collection_pda, check_receipts_tree_pda,
        ExecutionDelegateRecordV1, RECEIPTS_AUTHORITY_PREFIX,
    },
};

/// Maximum length of the off-chain receipt URI, in bytes.
pub const MAX_RECEIPT_URI_LEN: usize = 200;

/// MPL Account Compression program id — pinned to defeat compression-
/// program spoofing.
const MPL_ACCOUNT_COMPRESSION_ID: solana_program::pubkey::Pubkey =
    solana_program::pubkey!("mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW");

#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, ShankType)]
pub struct MintWorkReceiptV1Args {
    /// URI of the off-chain receipt JSON.
    pub receipt_uri: String,
    /// Index of the receipts tree this receipt is being minted into.
    /// Must match the tree PDA seeds (`["receipts_tree", index_le]`).
    pub tree_index: u64,
}

pub fn mint_work_receipt_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: MintWorkReceiptV1Args,
) -> ProgramResult {
    let ctx = MintWorkReceiptV1Accounts::context(accounts)?;

    // Signers. The client is intentionally NOT a signer — coordinating a
    // real-time handshake with the receiving wallet is impractical. The
    // delegate record already authenticates the agent; the client is
    // recorded on the receipt (as leaf_owner + 0-share creator) so they
    // can later prove involvement when leaving a review.
    assert_signer(ctx.accounts.payer)?;
    assert_signer(ctx.accounts.executive_authority)?;

    // Agent asset = Core AssetV1.
    if ctx.accounts.agent_asset.owner != &mpl_core::ID
        || ctx.accounts.agent_asset.data_len() == 0
        || ctx.accounts.agent_asset.try_borrow_data()?[0] != MplCoreKey::AssetV1 as u8
    {
        return Err(MplAgentToolsError::InvalidCoreAsset.into());
    }

    // Execution delegate record = initialized and matches agent+executive.
    if ctx.accounts.execution_delegate_record.owner != &crate::ID
        || ctx.accounts.execution_delegate_record.data_len()
            < core::mem::size_of::<ExecutionDelegateRecordV1>()
    {
        return Err(MplAgentToolsError::ExecutionDelegateRecordMustBeInitialized.into());
    }
    {
        let dr_data = ctx.accounts.execution_delegate_record.try_borrow_data()?;
        let dr: &ExecutionDelegateRecordV1 =
            from_bytes(&dr_data[..core::mem::size_of::<ExecutionDelegateRecordV1>()]);
        if dr.agent_asset != *ctx.accounts.agent_asset.key {
            return Err(MplAgentToolsError::InvalidAgentIdentity.into());
        }
        if dr.authority != *ctx.accounts.executive_authority.key {
            return Err(MplAgentToolsError::ExecutiveAuthorityMismatch.into());
        }
    }

    // Stateless PDAs: verify all three canonical addresses, capture
    // the receipts_authority bump for invoke_signed below.
    let authority_bump = check_receipts_authority_pda(ctx.accounts.authority)?;
    check_receipts_tree_pda(ctx.accounts.merkle_tree, args.tree_index)?;
    check_receipts_collection_pda(ctx.accounts.core_collection)?;

    // Program program-id checks (defence-in-depth — Bubblegum should
    // assert these internally, but we pin them at the outer boundary).
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

    // Argument guard.
    if args.receipt_uri.is_empty() || args.receipt_uri.len() > MAX_RECEIPT_URI_LEN {
        return Err(MplAgentToolsError::ReceiptUriInvalid.into());
    }

    msg!(
        "Receipt agent={} client={} executive={} tree_index={}",
        ctx.accounts.agent_asset.key,
        ctx.accounts.client.key,
        ctx.accounts.executive_authority.key,
        args.tree_index,
    );

    let metadata = MetadataArgsV2 {
        name: "Agent Work Receipt".to_string(),
        symbol: "AGENTRCPT".to_string(),
        uri: args.receipt_uri,
        seller_fee_basis_points: 0,
        primary_sale_happened: false,
        is_mutable: false,
        token_standard: Some(TokenStandard::NonFungible),
        creators: vec![
            Creator {
                address: *ctx.accounts.agent_asset.key,
                verified: false,
                share: 100,
            },
            Creator {
                address: *ctx.accounts.client.key,
                verified: false,
                share: 0,
            },
        ],
        collection: Some(*ctx.accounts.core_collection.key),
    };

    // The receipts_authority PDA was registered as tree_creator at
    // RegisterReceiptsTreeV1 time AND is the collection's
    // update_authority. So signing the MintV2 CPI with this single PDA
    // satisfies both `tree_creator_or_delegate` and the default
    // `collection_authority` (which mpl-core checks against
    // update_authority).
    let authority_seeds: &[&[u8]] = &[RECEIPTS_AUTHORITY_PREFIX, &[authority_bump]];

    MintV2CpiBuilder::new(ctx.accounts.bubblegum_program)
        .tree_config(ctx.accounts.tree_config)
        .payer(ctx.accounts.payer)
        .tree_creator_or_delegate(Some(ctx.accounts.authority))
        .collection_authority(None)
        .leaf_owner(ctx.accounts.client)
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

pub fn deserialize_mint_work_receipt_args(
    data: &[u8],
) -> Result<MintWorkReceiptV1Args, ProgramError> {
    MintWorkReceiptV1Args::try_from_slice(data)
        .map_err(|_| MplAgentToolsError::InvalidInstructionData.into())
}
