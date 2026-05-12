use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
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

    /// 5 - Agent Reputation already registered
    #[error("Agent Reputation already registered")]
    AgentReputationAlreadyRegistered,

    /// 6 - Invalid review rating (must be 1..=5)
    #[error("Invalid review rating (must be 1..=5)")]
    InvalidReviewRating,

    /// 7 - Feedback URI missing or too long
    #[error("Feedback URI must be non-empty and within size limits")]
    FeedbackUriInvalid,

    /// 8 - Leaf owner does not match asset owner
    #[error("Leaf owner does not match the reviewed asset owner")]
    LeafOwnerMismatch,

    /// 9 - Invalid Bubblegum Program
    #[error("Invalid Bubblegum Program")]
    InvalidBubblegumProgram,

    /// 10 - Invalid Compression Program
    #[error("Invalid Compression Program")]
    InvalidCompressionProgram,

    /// 11 - A review already exists for this receipt
    #[error("A review already exists for this work receipt")]
    ReviewAlreadyExists,

    /// 12 - Subsidy pool already initialized
    #[error("Subsidy pool already initialized for this agent")]
    SubsidyPoolAlreadyInitialized,

    /// 13 - Unauthorized subsidy withdrawal
    #[error("Subsidy withdrawal requires the pool's withdraw authority signature")]
    UnauthorizedSubsidyWithdrawal,

    /// 14 - Subsidy pool not initialized
    #[error("Subsidy pool not initialized for this agent")]
    SubsidyPoolNotInitialized,

    /// 15 - Invalid Program Config PDA derivation
    #[error("Invalid Program Config PDA derivation")]
    InvalidProgramConfigDerivation,

    /// 16 - Program Config not initialized
    #[error("Program Config not initialized")]
    ProgramConfigNotInitialized,

    /// 17 - Program Config already initialized
    #[error("Program Config already initialized")]
    ProgramConfigAlreadyInitialized,

    /// 18 - Invalid reviews tree PDA derivation
    #[error("Invalid reviews tree PDA derivation")]
    InvalidReviewsTreeDerivation,

    /// 19 - Reviews collection mismatch
    #[error("Supplied reviews collection does not match config.reviews_collection")]
    InvalidReviewsCollection,

    /// 20 - Receipts collection mismatch
    #[error("Supplied receipts collection does not match config.receipts_collection")]
    InvalidReceiptsCollection,

    /// 21 - Unauthorized admin
    #[error("Signer is not the program config admin")]
    UnauthorizedAdmin,

    /// 22 - Invalid log wrapper program
    #[error("Invalid log wrapper program")]
    InvalidLogWrapperProgram,
}

impl PrintProgramError for MplAgentReputationError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<MplAgentReputationError> for ProgramError {
    fn from(e: MplAgentReputationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MplAgentReputationError {
    fn type_of() -> &'static str {
        "Mpl Agent Reputation Error"
    }
}
