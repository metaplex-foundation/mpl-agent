use shank::{ShankContext, ShankInstruction};

use crate::processor::{DelegateExecutionV1Args, RegisterExecutiveV1Args};

/// Instruction discriminants for routing.
/// The first byte of instruction data determines which instruction to execute.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MplAgentToolsInstructionDiscriminant {
    RegisterExecutorV1 = 0,
    DelegateExecutionV1 = 1,
}

impl TryFrom<u8> for MplAgentToolsInstructionDiscriminant {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MplAgentToolsInstructionDiscriminant::RegisterExecutorV1),
            1 => Ok(MplAgentToolsInstructionDiscriminant::DelegateExecutionV1),
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
}
