mod register;

use bytemuck::try_from_bytes;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentValidationError;
use crate::instruction::MplAgentValidationInstructionDiscriminant;

pub use register::{register_validation_v1, RegisterValidationV1Args};

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
        return Err(MplAgentValidationError::InvalidInstructionData.into());
    }

    // Route by discriminant (first byte).
    match MplAgentValidationInstructionDiscriminant::try_from(instruction_data[0]) {
        Ok(MplAgentValidationInstructionDiscriminant::RegisterValidationV1) => {
            msg!("Instruction: RegisterValidationV1");
            register_validation_v1(
                accounts,
                try_from_bytes(instruction_data)
                    .map_err(|_| MplAgentValidationError::InvalidInstructionData)?,
            )
        }
        Err(_) => Err(MplAgentValidationError::InvalidInstructionData.into()),
    }
}
