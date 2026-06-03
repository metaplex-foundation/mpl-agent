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

    /// 12 - Invalid reviews collection PDA derivation
    #[error("Invalid reviews collection PDA derivation")]
    InvalidReviewsCollection,

    /// 13 - Invalid reviews authority PDA derivation
    #[error("Invalid reviews authority PDA derivation")]
    InvalidReviewsAuthority,

    /// 14 - Reviews collection already initialized
    #[error("Reviews collection already initialized")]
    ReviewsCollectionAlreadyInitialized,

    /// 15 - Invalid reviews tree PDA derivation
    #[error("Invalid reviews tree PDA derivation")]
    InvalidReviewsTreeDerivation,

    /// 16 - Receipts collection mismatch — supplied account is not the
    ///      canonical mpl-agent-tools receipts collection PDA.
    #[error(
        "Supplied receipts collection is not the canonical mpl-agent-tools receipts collection PDA"
    )]
    InvalidReceiptsCollection,
}

impl From<MplAgentReputationError> for ProgramError {
    fn from(e: MplAgentReputationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
