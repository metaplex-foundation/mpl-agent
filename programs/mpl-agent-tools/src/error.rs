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

    /// 5 - Executor Profile must be uninitialized
    #[error("Executor Profile must be uninitialized")]
    ExecutorProfileMustBeUninitialized,

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
