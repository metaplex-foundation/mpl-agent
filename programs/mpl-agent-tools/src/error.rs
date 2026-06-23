use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
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

    /// 15 - Invalid Bubblegum Program
    #[error("Invalid Bubblegum Program")]
    InvalidBubblegumProgram,

    /// 16 - Executive authority does not match delegate record
    #[error("Executive authority does not match the delegate record's authority")]
    ExecutiveAuthorityMismatch,

    /// 17 - Receipt URI missing or too long
    #[error("Receipt URI must be non-empty and within size limits")]
    ReceiptUriInvalid,

    /// 18 - Invalid receipts collection PDA / address
    #[error("Supplied collection is not the canonical receipts collection PDA")]
    InvalidReceiptsCollection,

    /// 19 - Invalid receipts authority PDA
    #[error("Supplied authority is not the canonical receipts authority PDA")]
    InvalidReceiptsAuthority,

    /// 20 - Receipts collection already initialized
    #[error("Receipts collection already exists")]
    ReceiptsCollectionAlreadyInitialized,

    /// 21 - Invalid receipts tree PDA derivation
    #[error("Invalid receipts tree PDA derivation")]
    InvalidReceiptsTreeDerivation,

    /// 22 - Invalid MPL Account Compression Program
    #[error("Invalid MPL Account Compression Program")]
    InvalidCompressionProgram,
}

impl From<MplAgentToolsError> for ProgramError {
    fn from(e: MplAgentToolsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
