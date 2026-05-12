use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    DelegateExecutionV1Args, InitializeToolsConfigV1Args, MintWorkReceiptV1Args,
    RegisterExecutiveV1Args, RegisterReceiptsTreeV1Args, RevokeExecutionV1Args,
};

/// Instruction discriminants for routing.
/// The first byte of instruction data determines which instruction to execute.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MplAgentToolsInstructionDiscriminant {
    RegisterExecutorV1 = 0,
    DelegateExecutionV1 = 1,
    RevokeExecutionV1 = 2,
    MintWorkReceiptV1 = 3,
    InitializeToolsConfigV1 = 4,
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
            4 => Ok(MplAgentToolsInstructionDiscriminant::InitializeToolsConfigV1),
            5 => Ok(MplAgentToolsInstructionDiscriminant::RegisterReceiptsTreeV1),
            _ => Err(()),
        }
    }
}

/// Instruction enum for Shank IDL generation.
/// Note: We keep Shank attributes for IDL generation but use zero-copy
/// for actual instruction deserialization in the processor.
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

    /// Mint a work receipt cNFT on behalf of an agent, owned (soulbound) by
    /// the client. Authority comes from an existing `ExecutionDelegateRecord`
    /// so any of the agent's executive delegates can issue receipts without
    /// the agent's owner key. The client co-signs the mint as a handshake.
    /// The merkle tree and core collection are program-managed: the tree
    /// must be at PDA ["receipts_tree", tree_index_le] and the collection
    /// must equal `program_config.collection`. The program signs the
    /// Bubblegum CPI as tree_creator_or_delegate via the program config PDA.
    #[account(0, writable, signer, name="payer", desc = "Pays for the cNFT mint")]
    #[account(1, signer, name="executive_authority", desc = "Executive's signing key (must match `authority` on the execution delegate record)")]
    #[account(2, name="execution_delegate_record", desc = "The delegate record proving the executive can act for this agent")]
    #[account(3, name="agent_asset", desc = "The Core asset representing the agent")]
    #[account(4, signer, name="client", desc = "The client receiving the receipt (co-signs the handshake)")]
    #[account(5, name="program_config", desc = "Singleton program config PDA. Signs the Bubblegum CPI as tree creator/delegate via invoke_signed")]
    #[account(6, writable, name="tree_config", desc = "Bubblegum tree config PDA for the receipts tree")]
    #[account(7, writable, name="merkle_tree", desc = "Receipts merkle tree at PDA [\"receipts_tree\", tree_index_le]")]
    #[account(8, writable, name="core_collection", desc = "Receipts collection (must equal program_config.collection)")]
    #[account(9, name="mpl_core_cpi_signer", desc = "Bubblegum's mpl-core CPI signer PDA")]
    #[account(10, name="log_wrapper", desc = "MPL Noop / log wrapper program")]
    #[account(11, name="compression_program", desc = "MPL Account Compression program")]
    #[account(12, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(13, name="bubblegum_program", desc = "The MPL Bubblegum program")]
    #[account(14, name="system_program", desc = "The system program")]
    MintWorkReceiptV1(MintWorkReceiptV1Args),

    /// Bootstrap the program: create the singleton `ProgramConfigV1` PDA
    /// and the canonical receipts collection (a Core collection whose
    /// update authority is the program config PDA, with `BubblegumV2` +
    /// `PermanentFreezeDelegate{frozen:true}` plugins). Idempotent fails
    /// if called twice — the program config can only ever be created once.
    #[account(0, writable, signer, name="admin", desc = "Bootstrapping admin; captured as the config authority for future tree registration")]
    #[account(1, writable, name="program_config", desc = "ProgramConfigV1 PDA at [\"program_config\"]")]
    #[account(2, writable, name="collection", desc = "Receipts collection PDA at [\"receipts_collection\"]")]
    #[account(3, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(4, name="system_program", desc = "The system program")]
    InitializeToolsConfigV1(InitializeToolsConfigV1Args),

    /// Register a new receipts merkle tree at PDA [\"receipts_tree\",
    /// next_tree_index_le]. Only `program_config.admin` may call this.
    /// Bumps `program_config.next_tree_index` on success. Caller picks
    /// `max_depth` / `max_buffer_size` / `canopy_depth`.
    #[account(0, writable, signer, name="admin", desc = "Must match program_config.admin")]
    #[account(1, writable, name="program_config", desc = "ProgramConfigV1 PDA")]
    #[account(2, writable, name="merkle_tree", desc = "Receipts merkle tree PDA at [\"receipts_tree\", program_config.next_tree_index_le]")]
    #[account(3, writable, name="tree_config", desc = "Bubblegum tree config PDA (derived from merkle_tree)")]
    #[account(4, name="log_wrapper", desc = "MPL Noop / log wrapper program")]
    #[account(5, name="compression_program", desc = "MPL Account Compression program")]
    #[account(6, name="bubblegum_program", desc = "The MPL Bubblegum program")]
    #[account(7, name="system_program", desc = "The system program")]
    RegisterReceiptsTreeV1(RegisterReceiptsTreeV1Args),
}
