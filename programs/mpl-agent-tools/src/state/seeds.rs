//! Stateless PDA derivations for the receipts collection, its program
//! signing authority, and the per-tree merkle accounts.
//!
//! Nothing in this module corresponds to a stored account — these are
//! all derivable from the program id and constant seeds. The program
//! signs CPIs by passing the matching seeds + bump to `invoke_signed`.

use mpl_utils::assert_derivation;
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::error::MplAgentToolsError;

/// Seeds for the canonical receipts collection (the MPL Core collection
/// asset every receipt cNFT lives in).
pub const RECEIPTS_COLLECTION_PREFIX: &[u8] = b"receipts_collection";

/// Seeds for the program's signing authority. This PDA is set as the
/// receipts collection's `update_authority` *and* as `tree_creator` on
/// every registered tree, so it is the only signer the program needs to
/// authenticate CPIs into mpl-core and mpl-bubblegum.
pub const RECEIPTS_AUTHORITY_PREFIX: &[u8] = b"receipts_authority";

/// Seeds for per-tree merkle accounts: `["receipts_tree", index_le]`.
pub const RECEIPTS_TREE_PREFIX: &[u8] = b"receipts_tree";

/// Verify the supplied account is at the canonical receipts collection
/// PDA and return its bump.
pub fn check_receipts_collection_pda(address: &AccountInfo) -> Result<u8, ProgramError> {
    assert_derivation(
        &crate::ID,
        address,
        &[RECEIPTS_COLLECTION_PREFIX],
        MplAgentToolsError::InvalidReceiptsCollection,
    )
}

/// Verify the supplied account is at the canonical receipts authority
/// PDA and return its bump.
pub fn check_receipts_authority_pda(address: &AccountInfo) -> Result<u8, ProgramError> {
    assert_derivation(
        &crate::ID,
        address,
        &[RECEIPTS_AUTHORITY_PREFIX],
        MplAgentToolsError::InvalidReceiptsAuthority,
    )
}

/// Verify the supplied account is the receipts tree PDA for `index`.
pub fn check_receipts_tree_pda(address: &AccountInfo, index: u64) -> Result<u8, ProgramError> {
    assert_derivation(
        &crate::ID,
        address,
        &[RECEIPTS_TREE_PREFIX, &index.to_le_bytes()],
        MplAgentToolsError::InvalidReceiptsTreeDerivation,
    )
}
