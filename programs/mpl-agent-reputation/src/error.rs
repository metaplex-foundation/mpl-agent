use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum MplAgentReputationError {
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
}

impl From<MplAgentReputationError> for ProgramError {
    fn from(e: MplAgentReputationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
