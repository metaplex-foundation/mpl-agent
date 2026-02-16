use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum Mpl8004IdentityError {
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

impl PrintProgramError for Mpl8004IdentityError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<Mpl8004IdentityError> for ProgramError {
    fn from(e: Mpl8004IdentityError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for Mpl8004IdentityError {
    fn type_of() -> &'static str {
        "Mpl8004 Identity Error"
    }
}
