mod register;

use bytemuck::try_from_bytes;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentReputationError;
use crate::instruction::MplAgentReputationInstructionDiscriminant;

pub use register::{register_reputation_v1, RegisterReputationV1Args};

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
        Err(_) => Err(MplAgentReputationError::InvalidInstructionData.into()),
    }
}
