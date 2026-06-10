use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    CreateReceiptsCollectionV1Args, DelegateExecutionV1Args, MintWorkReceiptV1Args,
    RegisterExecutiveV1Args, RegisterReceiptsTreeV1Args, RevokeExecutionV1Args,
};

/// Instruction discriminants for routing.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MplAgentToolsInstructionDiscriminant {
    RegisterExecutorV1 = 0,
    DelegateExecutionV1 = 1,
    RevokeExecutionV1 = 2,
    MintWorkReceiptV1 = 3,
    CreateReceiptsCollectionV1 = 4,
    RegisterReceiptsTreeV1 = 5,
}

impl TryFrom<u8> for MplAgentToolsInstructionDiscriminant {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MplAgentToolsInstructionDiscriminant::RegisterExecutorV1),
            1 => Ok(MplAgentToolsInstructionDiscriminant::DelegateExecutionV1),
            2 => Ok(MplAgentToolsInstructionDiscriminant::RevokeExecutionV1),
            3 => Ok(MplAgentToolsInstructionDiscriminant::MintWorkReceiptV1),
            4 => Ok(MplAgentToolsInstructionDiscriminant::CreateReceiptsCollectionV1),
            5 => Ok(MplAgentToolsInstructionDiscriminant::RegisterReceiptsTreeV1),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum MplAgentToolsInstruction {
    /// Register an Agent Executive.
    #[account(0, writable, name="executive_profile", desc = "The executive profile")]
    #[account(1, writable, signer, name="payer", desc = "The payer for additional rent")]
    #[account(2, optional, signer, name="authority", desc = "Authority the executive signs with when executing agent actions")]
    #[account(3, name="system_program", desc = "The system program")]
    RegisterExecutiveV1(RegisterExecutiveV1Args),

    /// Delegate Executive Permission for an Agent Asset.
    #[account(0, name="executive_profile", desc = "The executive profile")]
    #[account(1, name="agent_asset", desc = "The agent asset")]
    #[account(2, name="agent_identity", desc = "The agent identity")]
    #[account(3, writable, name="execution_delegate_record", desc = "The execution delegate record")]
    #[account(4, writable, signer, name="payer", desc = "The payer for additional rent")]
    #[account(5, optional, signer, name="authority", desc = "Authority the executive signs with when executing agent actions")]
    #[account(6, name="system_program", desc = "The system program")]
    DelegateExecutionV1(DelegateExecutionV1Args),

    /// Revoke an Execution Delegate for an Agent Asset.
    #[account(0, writable, name="execution_delegate_record", desc = "The execution delegate record to close")]
    #[account(1, name="agent_asset", desc = "The agent asset")]
    #[account(2, writable, name="destination", desc = "The destination for the refunded rent")]
    #[account(3, writable, signer, name="payer", desc = "The payer")]
    #[account(4, optional, signer, name="authority", desc = "Authority — must be asset owner or executive authority")]
    #[account(5, name="system_program", desc = "The system program")]
    RevokeExecutionV1(RevokeExecutionV1Args),

    /// Mint a work receipt cNFT to a client wallet on behalf of an agent.
    /// Auth comes from an existing `ExecutionDelegateRecord`; the client
    /// does NOT co-sign — in practice a real-time handshake is too costly
    /// to coordinate. The client is recorded as a creator on the receipt
    /// (share 0) so they can later prove ownership / participation when
    /// leaving a review. The receipts authority PDA (constant program
    /// signer) authorizes both the tree write and the collection add —
    /// caller does not need to provide an admin signer.
    #[account(0, writable, signer, name="payer", desc = "Pays for the cNFT mint")]
    #[account(1, signer, name="executive_authority", desc = "Executive's signing key (must match `authority` on the execution delegate record)")]
    #[account(2, name="execution_delegate_record", desc = "The delegate record proving the executive can act for this agent")]
    #[account(3, name="agent_asset", desc = "The Core asset representing the agent")]
    #[account(4, name="client", desc = "The client receiving the receipt (leaf owner + 0-share creator)")]
    #[account(5, name="authority", desc = "Receipts authority PDA at [\"receipts_authority\"] — signs CPI via invoke_signed")]
    #[account(6, writable, name="tree_config", desc = "Bubblegum tree config PDA for the receipts tree")]
    #[account(7, writable, name="merkle_tree", desc = "Receipts merkle tree at PDA [\"receipts_tree\", tree_index_le]")]
    #[account(8, writable, name="core_collection", desc = "Canonical receipts collection PDA at [\"receipts_collection\"]")]
    #[account(9, name="mpl_core_cpi_signer", desc = "Bubblegum's mpl-core CPI signer PDA")]
    #[account(10, name="log_wrapper", desc = "MPL Noop / log wrapper program")]
    #[account(11, name="compression_program", desc = "MPL Account Compression program")]
    #[account(12, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(13, name="bubblegum_program", desc = "The MPL Bubblegum program")]
    #[account(14, name="system_program", desc = "The system program")]
    MintWorkReceiptV1(MintWorkReceiptV1Args),

    /// Permissionless idempotent bootstrap: create the canonical
    /// receipts collection at [\"receipts_collection\"] PDA with
    /// update_authority = [\"receipts_authority\"] PDA. Anyone may
    /// call. A hostile first caller cannot capture authority because
    /// it's program-derived, not caller-derived. Second call fails
    /// because the collection account is already initialized.
    #[account(0, writable, signer, name="payer", desc = "Funds the collection's rent")]
    #[account(1, writable, name="collection", desc = "Receipts collection PDA at [\"receipts_collection\"]")]
    #[account(2, name="authority", desc = "Receipts authority PDA at [\"receipts_authority\"] — becomes the collection's update_authority")]
    #[account(3, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(4, name="system_program", desc = "The system program")]
    CreateReceiptsCollectionV1(CreateReceiptsCollectionV1Args),

    /// Permissionless tree registration: caller picks an unused
    /// `tree_index` and pays the rent. Tree is created at PDA
    /// [\"receipts_tree\", tree_index_le]. Bubblegum is configured with
    /// `tree_creator = [\"receipts_authority\"]` PDA so MintWorkReceiptV1
    /// can sign every future mint without the original creator.
    #[account(0, writable, signer, name="payer", desc = "Funds the tree rent")]
    #[account(1, name="authority", desc = "Receipts authority PDA at [\"receipts_authority\"] — set as tree_creator")]
    #[account(2, writable, name="merkle_tree", desc = "Receipts merkle tree PDA at [\"receipts_tree\", tree_index_le]")]
    #[account(3, writable, name="tree_config", desc = "Bubblegum tree config PDA (derived from merkle_tree)")]
    #[account(4, name="log_wrapper", desc = "MPL Noop / log wrapper program")]
    #[account(5, name="compression_program", desc = "MPL Account Compression program")]
    #[account(6, name="bubblegum_program", desc = "The MPL Bubblegum program")]
    #[account(7, name="system_program", desc = "The system program")]
    RegisterReceiptsTreeV1(RegisterReceiptsTreeV1Args),
}
