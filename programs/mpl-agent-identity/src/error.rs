use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum MplAgentIdentityError {
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

    /// 5 - Invalid Agent Token
    #[error("Invalid Agent Token")]
    InvalidAgentToken,

    /// 6 - Only the Asset Signer can set the agent token
    #[error("Only the Asset Signer can set the agent token")]
    OnlyAssetSignerCanSetAgentToken,

    /// 7 - Agent Token already set
    #[error("Agent Token already set")]
    AgentTokenAlreadySet,

    /// 8 - Invalid Agent Identity
    #[error("Invalid Agent Identity")]
    InvalidAgentIdentity,

    /// 9 - Agent Identity already registered
    #[error("Agent Identity already registered")]
    AgentIdentityAlreadyRegistered,
}

impl PrintProgramError for MplAgentIdentityError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<MplAgentIdentityError> for ProgramError {
    fn from(e: MplAgentIdentityError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MplAgentIdentityError {
    fn type_of() -> &'static str {
        "Mpl Agent Identity Error"
    }
}
