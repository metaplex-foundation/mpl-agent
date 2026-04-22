use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum MplAgentToolsError {
    /// 0 - Invalid System Program
    #[error("Invalid System Program")]
    InvalidSystemProgram,

    /// 1 - Invalid instruction data
    #[error("Invalid instruction data")]
    InvalidInstructionData,

    /// 2 - Invalid account data
    #[error("Invalid account data")]
    InvalidAccountData,

    /// 3 - Invalid MPL Core Program
    #[error("Invalid MPL Core Program")]
    InvalidMplCoreProgram,

    /// 4 - Invalid Asset
    #[error("Invalid Core Asset")]
    InvalidCoreAsset,

    /// 5 - Executive Profile must be uninitialized
    #[error("Executive Profile must be uninitialized")]
    ExecutiveProfileMustBeUninitialized,

    /// 6 - Invalid Execution Delegate Record Derivation
    #[error("Invalid Execution Delegate Record Derivation")]
    InvalidExecutionDelegateRecordDerivation,

    /// 7 - Execution Delegate Record must be uninitialized
    #[error("Execution Delegate Record must be uninitialized")]
    ExecutionDelegateRecordMustBeUninitialized,

    /// 8 - Invalid Agent Identity
    #[error("Invalid Agent Identity")]
    InvalidAgentIdentity,

    /// 9 - Agent Identity not registered
    #[error("Agent Identity not registered")]
    AgentIdentityNotRegistered,

    /// 10 - Asset owner must be the one to delegate execution
    #[error("Asset owner must be the one to delegate execution")]
    AssetOwnerMustBeTheOneToDelegateExecution,

    /// 11 - Invalid Executive Profile Derivation
    #[error("Invalid Executive Profile Derivation")]
    InvalidExecutiveProfileDerivation,

    /// 12 - Execution Delegate Record must be initialized
    #[error("Execution Delegate Record must be initialized")]
    ExecutionDelegateRecordMustBeInitialized,

    /// 13 - Authority must be asset owner or executive to revoke
    #[error("Authority must be asset owner or executive to revoke")]
    UnauthorizedRevoke,

    /// 14 - Executive Profile must be initialized
    #[error("Executive Profile must be initialized")]
    ExecutiveProfileMustBeInitialized,
}

impl PrintProgramError for MplAgentToolsError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<MplAgentToolsError> for ProgramError {
    fn from(e: MplAgentToolsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MplAgentToolsError {
    fn type_of() -> &'static str {
        "Mpl Agent Tools Error"
    }
}
