mod register;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentIdentityError;
use crate::instruction::MplAgentIdentityInstructionDiscriminant;

pub use register::{register_identity_v1, RegisterIdentityV1Args};

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
        return Err(MplAgentIdentityError::InvalidInstructionData.into());
    }

    // Route by discriminant (first byte).
    match MplAgentIdentityInstructionDiscriminant::try_from(instruction_data[0]) {
        Ok(MplAgentIdentityInstructionDiscriminant::RegisterIdentityV1) => {
            msg!("Instruction: RegisterIdentityV1");
            register_identity_v1(accounts, instruction_data)
        }
        Err(_) => Err(MplAgentIdentityError::InvalidInstructionData.into()),
    }
}
