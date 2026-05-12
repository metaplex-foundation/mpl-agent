mod leave_review;
mod program_config_ix;
mod register;
mod subsidy;

use bytemuck::try_from_bytes;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentReputationError;
use crate::instruction::MplAgentReputationInstructionDiscriminant;

pub use leave_review::{
    deserialize_leave_review_args, leave_review_v1, LeaveReviewV1Args, MAX_FEEDBACK_URI_LEN,
};
pub use program_config_ix::{
    cast_initialize_program_config_args, cast_register_reviews_tree_args,
    initialize_program_config_v1, register_reviews_tree_v1, InitializeReviewsConfigV1Args,
    RegisterReviewsTreeV1Args,
};
pub use register::{register_reputation_v1, RegisterReputationV1Args};
pub use subsidy::{
    deposit_subsidy_v1, deserialize_deposit_subsidy_args, deserialize_withdraw_subsidy_args,
    withdraw_subsidy_v1, DepositSubsidyV1Args, WithdrawSubsidyV1Args,
};

/// Process incoming instructions.
///
/// # Arguments
/// * `_program_id` - The program ID (unused but available for validation)
/// * `accounts` - The accounts required for the instruction
/// * `instruction_data` - The instruction data containing the discriminant and arguments
#[inline]
pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    // Ensure we have at least 1 byte for the discriminant.
    if instruction_data.is_empty() {
        return Err(MplAgentReputationError::InvalidInstructionData.into());
    }

    // Route by discriminant (first byte).
    match MplAgentReputationInstructionDiscriminant::try_from(instruction_data[0]) {
        Ok(MplAgentReputationInstructionDiscriminant::RegisterReputationV1) => {
            msg!("Instruction: RegisterReputationV1");
            register_reputation_v1(
                accounts,
                try_from_bytes(instruction_data)
                    .map_err(|_| MplAgentReputationError::InvalidInstructionData)?,
            )
        }
        Ok(MplAgentReputationInstructionDiscriminant::LeaveReviewV1) => {
            msg!("Instruction: LeaveReviewV1");
            let args = deserialize_leave_review_args(&instruction_data[1..])?;
            leave_review_v1(accounts, args)
        }
        Ok(MplAgentReputationInstructionDiscriminant::DepositSubsidyV1) => {
            msg!("Instruction: DepositSubsidyV1");
            let args = deserialize_deposit_subsidy_args(&instruction_data[1..])?;
            deposit_subsidy_v1(accounts, args)
        }
        Ok(MplAgentReputationInstructionDiscriminant::WithdrawSubsidyV1) => {
            msg!("Instruction: WithdrawSubsidyV1");
            let args = deserialize_withdraw_subsidy_args(&instruction_data[1..])?;
            withdraw_subsidy_v1(accounts, args)
        }
        Ok(MplAgentReputationInstructionDiscriminant::InitializeReviewsConfigV1) => {
            msg!("Instruction: InitializeReviewsConfigV1");
            let args = cast_initialize_program_config_args(instruction_data)?;
            initialize_program_config_v1(accounts, args)
        }
        Ok(MplAgentReputationInstructionDiscriminant::RegisterReviewsTreeV1) => {
            msg!("Instruction: RegisterReviewsTreeV1");
            let args = cast_register_reviews_tree_args(instruction_data)?;
            register_reviews_tree_v1(accounts, args)
        }
        Err(_) => Err(MplAgentReputationError::InvalidInstructionData.into()),
    }
}
