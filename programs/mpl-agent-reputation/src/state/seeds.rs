//! Stateless PDA derivations for the reviews collection, the program's
//! signing authority, and per-tree merkle accounts.

use mpl_utils::assert_derivation;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::error::MplAgentReputationError;

/// Seeds for the canonical reviews collection.
pub const REVIEWS_COLLECTION_PREFIX: &[u8] = b"reviews_collection";

/// Seeds for the program's signing authority. Update authority on the
/// reviews collection AND tree_creator on every reviews tree.
pub const REVIEWS_AUTHORITY_PREFIX: &[u8] = b"reviews_authority";

/// Seeds for per-tree merkle accounts: `["reviews_tree", index_le]`.
pub const REVIEWS_TREE_PREFIX: &[u8] = b"reviews_tree";

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
/// (derived from `mpl_agent_tools`'s program id + `b"receipts_collection"`).
/// This is the cross-program canonicalization check — LeaveReviewV1 must
/// reference a receipt minted by mpl-agent-tools.
pub fn check_receipts_collection_pda(address: &AccountInfo) -> Result<u8, ProgramError> {
    const RECEIPTS_COLLECTION_PREFIX: &[u8] = b"receipts_collection";
    assert_derivation(
        &mpl_agent_tools::ID,
        address,
        &[RECEIPTS_COLLECTION_PREFIX],
        MplAgentReputationError::InvalidReceiptsCollection,
    )
}

/// Verify the supplied account is the canonical receipts tree PDA for
/// `index` (derived from `mpl_agent_tools`'s program id +
/// `[b"receipts_tree", index_le]`). Without this check a caller could pass
/// any attacker-controlled compression tree and forge a receipt leaf,
/// bypassing the work-receipt gate on LeaveReviewV1.
pub fn check_receipts_tree_pda(address: &AccountInfo, index: u64) -> Result<u8, ProgramError> {
    const RECEIPTS_TREE_PREFIX: &[u8] = b"receipts_tree";
    assert_derivation(
        &mpl_agent_tools::ID,
        address,
        &[RECEIPTS_TREE_PREFIX, &index.to_le_bytes()],
        MplAgentReputationError::InvalidReceiptsTreeDerivation,
    )
}

/// Helper: derive the receipts collection address without account check.
pub fn receipts_collection_address() -> Pubkey {
    const RECEIPTS_COLLECTION_PREFIX: &[u8] = b"receipts_collection";
    let (address, _bump) =
        Pubkey::find_program_address(&[RECEIPTS_COLLECTION_PREFIX], &mpl_agent_tools::ID);
    address
}
