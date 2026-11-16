mod create_reviews_collection_v1;
mod leave_review;
mod register_reviews_tree_v1;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentReputationError;
use crate::instruction::MplAgentReputationInstructionDiscriminant;

pub use create_reviews_collection_v1::{
    cast_create_reviews_collection_args, create_reviews_collection_v1,
    CreateReviewsCollectionV1Args,
};
pub use leave_review::{leave_review_v1, LeaveReviewV1Args, MAX_FEEDBACK_URI_LEN};
pub use register_reviews_tree_v1::{
    cast_register_reviews_tree_args, register_reviews_tree_v1, RegisterReviewsTreeV1Args,
};

/// Process incoming instructions.
#[inline]
pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(MplAgentReputationError::InvalidInstructionData.into());
    }

    match MplAgentReputationInstructionDiscriminant::try_from(instruction_data[0]) {
        Ok(MplAgentReputationInstructionDiscriminant::LeaveReviewV1) => {
            msg!("Instruction: LeaveReviewV1");
            leave_review_v1(accounts, instruction_data)
        }
        Ok(MplAgentReputationInstructionDiscriminant::CreateReviewsCollectionV1) => {
            msg!("Instruction: CreateReviewsCollectionV1");
            let args = cast_create_reviews_collection_args(instruction_data)?;
            create_reviews_collection_v1(accounts, args)
        }
        Ok(MplAgentReputationInstructionDiscriminant::RegisterReviewsTreeV1) => {
            msg!("Instruction: RegisterReviewsTreeV1");
            let args = cast_register_reviews_tree_args(instruction_data)?;
            register_reviews_tree_v1(accounts, args)
        }
        Err(_) => Err(MplAgentReputationError::InvalidInstructionData.into()),
    }
}
