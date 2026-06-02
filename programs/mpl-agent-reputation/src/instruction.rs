use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    InitializeReviewsConfigV1Args, LeaveReviewV1Args, RegisterReputationV1Args,
    RegisterReviewsTreeV1Args,
};

/// Instruction discriminants for routing.
/// The first byte of instruction data determines which instruction to execute.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MplAgentReputationInstructionDiscriminant {
    RegisterReputationV1 = 0,
    LeaveReviewV1 = 1,
    InitializeReviewsConfigV1 = 2,
    RegisterReviewsTreeV1 = 3,
}

impl TryFrom<u8> for MplAgentReputationInstructionDiscriminant {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MplAgentReputationInstructionDiscriminant::RegisterReputationV1),
            1 => Ok(MplAgentReputationInstructionDiscriminant::LeaveReviewV1),
            2 => Ok(MplAgentReputationInstructionDiscriminant::InitializeReviewsConfigV1),
            3 => Ok(MplAgentReputationInstructionDiscriminant::RegisterReviewsTreeV1),
            _ => Err(()),
        }
    }
}

/// Instruction enum for Shank IDL generation.
#[derive(Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum MplAgentReputationInstruction {
    /// Register an Agent Reputation.
    #[account(0, writable, name="agent_reputation", desc = "The agent reputation PDA")]
    #[account(1, writable, name="asset", desc = "The address of the Core asset")]
    #[account(2, writable, optional, name="collection", desc = "The address of the collection")]
    #[account(3, writable, signer, name="payer", desc = "The payer for additional rent")]
    #[account(4, optional, signer, name="authority", desc = "Authority for the collection. If not provided, the payer will be used.")]
    #[account(5, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(6, name="system_program", desc = "The system program")]
    RegisterReputationV1(RegisterReputationV1Args),

    /// Leave a star review for an Agent, backed by an existing work receipt
    /// cNFT owned by the reviewer. Mints a non-transferable review cNFT to
    /// the agent's wallet via Bubblegum CPI signed by the program config
    /// PDA. The reviews merkle tree must be at the canonical PDA address
    /// (`["reviews_tree", reviews_tree_index_le]`) and the reviews+receipts
    /// collections must match `program_config.{reviews,receipts}_collection`.
    /// A `ReviewRecordV1` PDA seeded with the receipt's bubblegum asset id
    /// gates against double-review. Any accounts beyond `system_program`
    /// are treated as the merkle proof path for `verify_leaf` CPI.
    #[account(0, writable, signer, name="payer", desc = "Pays for the review cNFT mint and the review record PDA")]
    #[account(1, signer, name="reviewer", desc = "The wallet leaving the review; must own the work receipt")]
    #[account(2, name="asset", desc = "The Core asset being reviewed (the agent)")]
    #[account(3, name="leaf_owner", desc = "The owner of the new review cNFT leaf - must equal asset.owner")]
    #[account(4, name="program_config", desc = "Singleton ReviewsConfigV1 PDA. Signs the Bubblegum CPI as tree creator/delegate via invoke_signed")]
    #[account(5, writable, name="tree_config", desc = "Bubblegum tree config PDA for the reviews tree")]
    #[account(6, writable, name="merkle_tree", desc = "Reviews merkle tree at PDA [\"reviews_tree\", reviews_tree_index_le]")]
    #[account(7, writable, name="core_collection", desc = "Reviews collection (must equal program_config.reviews_collection)")]
    #[account(8, name="mpl_core_cpi_signer", desc = "Bubblegum's mpl-core CPI signer PDA")]
    #[account(9, name="log_wrapper", desc = "MPL Noop / log wrapper program")]
    #[account(10, name="compression_program", desc = "MPL Account Compression program (used for both Bubblegum mint and verify_leaf CPI)")]
    #[account(11, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(12, name="bubblegum_program", desc = "The MPL Bubblegum program")]
    #[account(13, name="receipts_merkle_tree", desc = "Receipts Bubblegum merkle tree holding the receipt being referenced")]
    #[account(14, name="receipts_collection", desc = "Receipts collection (must equal program_config.receipts_collection)")]
    #[account(15, writable, name="review_record", desc = "ReviewRecordV1 PDA seeded with the receipt's bubblegum asset id - idempotency gate")]
    #[account(16, name="system_program", desc = "The system program")]
    LeaveReviewV1(LeaveReviewV1Args),

    /// Bootstrap: create the singleton ReviewsConfigV1 PDA and the
    /// canonical reviews collection (PDA, update authority = config PDA,
    /// BubblegumV2 + PermanentFreezeDelegate plugins). Captures the
    /// `receipts_collection` reference (created by agent-tools'
    /// initialize_tools_config_v1) for cross-program canonicalisation.
    #[account(0, writable, signer, name="admin", desc = "Bootstrapping admin; captured as the config authority")]
    #[account(1, writable, name="program_config", desc = "ReviewsConfigV1 PDA at [\"program_config\"]")]
    #[account(2, writable, name="reviews_collection", desc = "Reviews collection PDA at [\"reviews_collection\"]")]
    #[account(3, name="receipts_collection", desc = "The canonical receipts collection from agent-tools (recorded in config for later validation)")]
    #[account(4, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(5, name="system_program", desc = "The system program")]
    InitializeReviewsConfigV1(InitializeReviewsConfigV1Args),

    /// Register a new reviews merkle tree at PDA [\"reviews_tree\",
    /// next_tree_index_le]. Only `program_config.admin` may call this.
    #[account(0, writable, signer, name="admin", desc = "Must match program_config.admin")]
    #[account(1, writable, name="program_config", desc = "ReviewsConfigV1 PDA")]
    #[account(2, writable, name="merkle_tree", desc = "Reviews merkle tree PDA at [\"reviews_tree\", program_config.next_tree_index_le]")]
    #[account(3, writable, name="tree_config", desc = "Bubblegum tree config PDA (derived from merkle_tree)")]
    #[account(4, name="log_wrapper", desc = "MPL Noop / log wrapper program")]
    #[account(5, name="compression_program", desc = "MPL Account Compression program")]
    #[account(6, name="bubblegum_program", desc = "The MPL Bubblegum program")]
    #[account(7, name="system_program", desc = "The system program")]
    RegisterReviewsTreeV1(RegisterReviewsTreeV1Args),
}
