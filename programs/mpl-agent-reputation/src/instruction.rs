use shank::{ShankContext, ShankInstruction};

use crate::processor::RegisterReputationV1Args;

/// Instruction discriminants for routing.
/// The first byte of instruction data determines which instruction to execute.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MplAgentReputationInstructionDiscriminant {
    RegisterReputationV1 = 0,
}

impl TryFrom<u8> for MplAgentReputationInstructionDiscriminant {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MplAgentReputationInstructionDiscriminant::RegisterReputationV1),
            _ => Err(()),
        }
    }
}

/// Instruction enum for Shank IDL generation.
/// Note: We keep Shank attributes for IDL generation but use zero-copy
/// for actual instruction deserialization in the processor.
#[derive(Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum MplAgentReputationInstruction {
    /// Register an Agent Reputation.
    #[account(0, writable, name="agent_reputation", desc = "The agent reputation PDA")]
    #[account(1, writable, name="collection_reputation_config", desc = "The collection reputation config")]
    #[account(2, writable, name="asset", desc = "The address of the Core asset")]
    #[account(3, writable, name="collection", desc = "The address of the collection")]
    #[account(4, writable, signer, name="payer", desc = "The payer for additional rent")]
    #[account(5, optional, signer, name="authority", desc = "Authority for the collection. If not provided, the payer will be used.")]
    #[account(6, name="mpl_core_program", desc = "The MPL Core program")]
    #[account(7, name="system_program", desc = "The system program")]
    RegisterReputationV1(RegisterReputationV1Args),
}
