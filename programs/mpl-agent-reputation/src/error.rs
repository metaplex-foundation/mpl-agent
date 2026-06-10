use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

/// On-chain error codes for the reputation program.
///
/// **Order is load-bearing.** `FromPrimitive` + `ProgramError::Custom(e as u32)`
/// turn each variant into its zero-based positional index — that integer is
/// what shows up in transaction logs as `custom program error: 0xNN` and is
/// what kinobi's auto-generated JS error map keys on.
///
/// Tests and JS bootstrap helpers (notably
/// `clients/js/test/_receiptsReviews.ts`) hardcode hex strings for specific
/// errors (e.g. `0xd` for `ReviewsCollectionAlreadyInitialized`). Reordering,
/// inserting, or removing variants without re-running `pnpm generate` AND
/// updating those hardcoded hex constants will silently break the mapping.
/// Add new variants at the end of the enum.
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

    /// 5 - Invalid review rating (must be 1..=5)
    #[error("Invalid review rating (must be 1..=5)")]
    InvalidReviewRating,

    /// 6 - Feedback URI missing or too long
    #[error("Feedback URI must be non-empty and within size limits")]
    FeedbackUriInvalid,

    /// 7 - Leaf owner does not match asset owner
    #[error("Leaf owner does not match the reviewed asset owner")]
    LeafOwnerMismatch,

    /// 8 - Invalid Bubblegum Program
    #[error("Invalid Bubblegum Program")]
    InvalidBubblegumProgram,

    /// 9 - Invalid Compression Program
    #[error("Invalid Compression Program")]
    InvalidCompressionProgram,

    /// 10 - A review already exists for this receipt
    #[error("A review already exists for this work receipt")]
    ReviewAlreadyExists,

    /// 11 - Invalid reviews collection PDA derivation
    #[error("Invalid reviews collection PDA derivation")]
    InvalidReviewsCollection,

    /// 12 - Invalid reviews authority PDA derivation
    #[error("Invalid reviews authority PDA derivation")]
    InvalidReviewsAuthority,

    /// 13 - Reviews collection already initialized
    #[error("Reviews collection already initialized")]
    ReviewsCollectionAlreadyInitialized,

    /// 14 - Invalid reviews tree PDA derivation
    #[error("Invalid reviews tree PDA derivation")]
    InvalidReviewsTreeDerivation,

    /// 15 - Receipts collection mismatch — supplied account is not the
    ///      canonical mpl-agent-tools receipts collection PDA.
    #[error(
        "Supplied receipts collection is not the canonical mpl-agent-tools receipts collection PDA"
    )]
    InvalidReceiptsCollection,

    /// 16 - Receipts tree mismatch — supplied account is not the
    ///      canonical mpl-agent-tools receipts tree PDA.
    #[error(
        "Supplied receipts merkle tree is not the canonical mpl-agent-tools receipts tree PDA"
    )]
    InvalidReceiptsTreeDerivation,
}

impl From<MplAgentReputationError> for ProgramError {
    fn from(e: MplAgentReputationError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
