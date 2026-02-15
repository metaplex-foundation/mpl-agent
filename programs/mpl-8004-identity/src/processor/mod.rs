mod create;

use bytemuck::try_from_bytes;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::Mpl8004IdentityError;
use crate::instruction::Mpl8004IdentityInstructionDiscriminant;

pub use create::*;

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
        return Err(Mpl8004IdentityError::InvalidInstructionData.into());
    }

    // Route by discriminant (first byte).
    match Mpl8004IdentityInstructionDiscriminant::try_from(instruction_data[0]) {
        Ok(Mpl8004IdentityInstructionDiscriminant::Create) => {
            msg!("Instruction: Create");
            create(
                accounts,
                try_from_bytes(instruction_data)
                    .map_err(|_| Mpl8004IdentityError::InvalidInstructionData)?,
            )
        }
        Err(_) => Err(Mpl8004IdentityError::InvalidInstructionData.into()),
    }
}
