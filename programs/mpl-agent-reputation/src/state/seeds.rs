//! Stateless PDA derivations for the reviews collection, the program's
//! signing authority, and per-tree merkle accounts. Also carries the
//! cross-program checks (receipts collection + receipts tree) that bind
//! LeaveReviewV1 to canonical mpl-agent-tools state.

use mpl_utils::assert_derivation;
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, pubkey, pubkey::Pubkey,
};

use crate::error::MplAgentReputationError;

/// Seeds for the canonical reviews collection.
pub const REVIEWS_COLLECTION_PREFIX: &[u8] = b"reviews_collection";

/// Seeds for the program's signing authority. Update authority on the
/// reviews collection AND tree_creator on every reviews tree.
pub const REVIEWS_AUTHORITY_PREFIX: &[u8] = b"reviews_authority";

/// Seeds for per-tree merkle accounts: `["reviews_tree", index_le]`.
pub const REVIEWS_TREE_PREFIX: &[u8] = b"reviews_tree";

/// Seeds for the canonical receipts collection (lives under mpl-agent-tools).
pub const RECEIPTS_COLLECTION_PREFIX: &[u8] = b"receipts_collection";

/// Seeds for per-tree receipts merkle accounts under mpl-agent-tools:
/// `["receipts_tree", index_le]`.
pub const RECEIPTS_TREE_PREFIX: &[u8] = b"receipts_tree";

/// Program id of mpl-agent-tools. Inlined as a constant (rather than
/// pulled from `mpl_agent_tools::ID` via the rust-tools client crate) to
/// keep this program free of generated client code. Must match the
/// `declare_id!` in `programs/mpl-agent-tools/src/lib.rs`.
pub const MPL_AGENT_TOOLS_ID: Pubkey = pubkey!("TLREGni9ZEyGC3vnPZtqUh95xQ8oPqJSvNjvB7FGK8S");

/// Program id of MPL Account Compression — the on-chain home of every
/// Bubblegum V2 tree. Pinned at the boundary of every CPI into it
/// (verify_leaf, CreateTreeConfigV2) to defeat compression-program
/// spoofing.
pub const MPL_ACCOUNT_COMPRESSION_ID: Pubkey =
    pubkey!("mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW");

pub fn check_reviews_collection_pda(address: &AccountInfo) -> Result<u8, ProgramError> {
    assert_derivation(
        &crate::ID,
        address,
        &[REVIEWS_COLLECTION_PREFIX],
        MplAgentReputationError::InvalidReviewsCollection,
    )
}

pub fn check_reviews_authority_pda(address: &AccountInfo) -> Result<u8, ProgramError> {
    assert_derivation(
        &crate::ID,
        address,
        &[REVIEWS_AUTHORITY_PREFIX],
        MplAgentReputationError::InvalidReviewsAuthority,
    )
}

pub fn check_reviews_tree_pda(address: &AccountInfo, index: u64) -> Result<u8, ProgramError> {
    assert_derivation(
        &crate::ID,
        address,
        &[REVIEWS_TREE_PREFIX, &index.to_le_bytes()],
        MplAgentReputationError::InvalidReviewsTreeDerivation,
    )
}

/// Verify the supplied account is the canonical receipts collection PDA
/// (derived from `mpl-agent-tools`'s program id + `b"receipts_collection"`).
/// This is the cross-program canonicalization check — LeaveReviewV1 must
/// reference a receipt minted by mpl-agent-tools.
pub fn check_receipts_collection_pda(address: &AccountInfo) -> Result<u8, ProgramError> {
    assert_derivation(
        &MPL_AGENT_TOOLS_ID,
        address,
        &[RECEIPTS_COLLECTION_PREFIX],
        MplAgentReputationError::InvalidReceiptsCollection,
    )
}

/// Verify the supplied account is the canonical receipts tree PDA for
/// `index` (derived from `mpl-agent-tools`'s program id +
/// `[b"receipts_tree", index_le]`). Without this check a caller could pass
/// any attacker-controlled compression tree and forge a receipt leaf,
/// bypassing the work-receipt gate on LeaveReviewV1.
pub fn check_receipts_tree_pda(address: &AccountInfo, index: u64) -> Result<u8, ProgramError> {
    assert_derivation(
        &MPL_AGENT_TOOLS_ID,
        address,
        &[RECEIPTS_TREE_PREFIX, &index.to_le_bytes()],
        MplAgentReputationError::InvalidReceiptsTreeDerivation,
    )
}

/// Helper: derive the receipts collection address without account check.
pub fn receipts_collection_address() -> Pubkey {
    let (address, _bump) =
        Pubkey::find_program_address(&[RECEIPTS_COLLECTION_PREFIX], &MPL_AGENT_TOOLS_ID);
    address
}
