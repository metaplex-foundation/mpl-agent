use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    CreateReviewsCollectionV1Args, LeaveReviewV1Args, RegisterReviewsTreeV1Args,
};

/// Instruction discriminants for routing.
/// The first byte of instruction data determines which instruction to execute.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MplAgentReputationInstructionDiscriminant {
    LeaveReviewV1 = 0,
    CreateReviewsCollectionV1 = 1,
    RegisterReviewsTreeV1 = 2,
}

impl TryFrom<u8> for MplAgentReputationInstructionDiscriminant {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MplAgentReputationInstructionDiscriminant::LeaveReviewV1),
            1 => Ok(MplAgentReputationInstructionDiscriminant::CreateReviewsCollectionV1),
            2 => Ok(MplAgentReputationInstructionDiscriminant::RegisterReviewsTreeV1),
            _ => Err(()),
        }
    }
}

/// Instruction enum for Shank IDL generation.
#[derive(Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum MplAgentReputationInstruction {
    /// Leave a star review for an Agent, backed by an existing work receipt
    /// cNFT owned by the reviewer. Mints a non-transferable review cNFT to
    /// the agent's wallet via Bubblegum CPI signed by the reviews authority
    /// PDA. Reviews+receipts collections + reviews tree must all be the
    /// canonical PDAs (cross-program for receipts). A `ReviewRecordV1` PDA
    /// seeded with the receipt's bubblegum asset id gates against
    /// double-review. Any accounts beyond `system_program` are treated as
    /// the merkle proof path for the receipt `verify_leaf` CPI.
    #[account(0, writable, signer, name="payer", desc = "Pays for the review cNFT mint and the review record PDA")]
    #[account(1, signer, name="reviewer", desc = "The wallet leaving the review; must own the work receipt")]
    #[account(2, name="asset", desc = "The Core asset being reviewed (the agent)")]
    #[account(3, name="leaf_owner", desc = "The owner of the new review cNFT leaf - must equal asset.owner")]
    #[account(4, name="authority", desc = "Reviews authority PDA at [\"reviews_authority\"] — signs the Bubblegum CPI as tree_creator/collection_authority via invoke_signed")]
    #[account(5, writable, name="tree_config", desc = "Bubblegum tree config PDA for the reviews tree")]
    #[account(6, writable, name="merkle_tree", desc = "Reviews merkle tree at PDA [\"reviews_tree\", reviews_tree_index_le]")]
    #[account(7, writable, name="core_collection", desc = "Reviews collection PDA at [\"reviews_collection\"]")]
    #[account(8, name="mpl_core_cpi_signer", desc = "Bubblegum's mpl-core CPI signer PDA")]
    #[account(9, name="log_wrapper", desc = "MPL Noop / log wrapper program")]
    #[account(10, name="compression_program", desc = "MPL Account Compression program (used for both Bubblegum mint and verify_leaf CPI)")]
    #[account(11, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(12, name="bubblegum_program", desc = "The MPL Bubblegum program")]
    #[account(13, name="receipts_merkle_tree", desc = "Receipts Bubblegum merkle tree holding the receipt being referenced")]
    #[account(14, name="receipts_collection", desc = "Canonical receipts collection PDA from mpl-agent-tools at [\"receipts_collection\"]")]
    #[account(15, writable, name="review_record", desc = "ReviewRecordV1 PDA seeded with the receipt's bubblegum asset id - idempotency gate")]
    #[account(16, name="system_program", desc = "The system program")]
    LeaveReviewV1(LeaveReviewV1Args),

    /// Permissionless idempotent bootstrap: create the canonical reviews
    /// collection at [\"reviews_collection\"] PDA with update_authority =
    /// [\"reviews_authority\"] PDA. Anyone may call. A hostile first caller
    /// cannot capture authority because it's program-derived, not
    /// caller-derived. Second call fails because the collection account is
    /// already initialized.
    #[account(0, writable, signer, name="payer", desc = "Funds the collection's rent")]
    #[account(1, writable, name="collection", desc = "Reviews collection PDA at [\"reviews_collection\"]")]
    #[account(2, name="authority", desc = "Reviews authority PDA at [\"reviews_authority\"] — becomes the collection's update_authority")]
    #[account(3, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(4, name="system_program", desc = "The system program")]
    CreateReviewsCollectionV1(CreateReviewsCollectionV1Args),

    /// Permissionless tree registration: caller picks an unused
    /// `tree_index` and pays the rent. Tree is created at PDA
    /// [\"reviews_tree\", tree_index_le]. Bubblegum is configured with
    /// `tree_creator = [\"reviews_authority\"]` PDA so LeaveReviewV1 can sign
    /// every future mint without the original creator.
    #[account(0, writable, signer, name="payer", desc = "Funds the tree rent")]
    #[account(1, name="authority", desc = "Reviews authority PDA at [\"reviews_authority\"] — set as tree_creator")]
    #[account(2, writable, name="merkle_tree", desc = "Reviews merkle tree PDA at [\"reviews_tree\", tree_index_le]")]
    #[account(3, writable, name="tree_config", desc = "Bubblegum tree config PDA (derived from merkle_tree)")]
    #[account(4, name="log_wrapper", desc = "MPL Noop / log wrapper program")]
    #[account(5, name="compression_program", desc = "MPL Account Compression program")]
    #[account(6, name="bubblegum_program", desc = "The MPL Bubblegum program")]
    #[account(7, name="system_program", desc = "The system program")]
    RegisterReviewsTreeV1(RegisterReviewsTreeV1Args),
}
