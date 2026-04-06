mod delegate_execution_v1;
mod register_executive_v1;
mod register_x402_v1;
mod revoke_execution_v1;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentToolsError;
use crate::instruction::MplAgentToolsInstructionDiscriminant;

pub use delegate_execution_v1::{delegate_execution_v1, DelegateExecutionV1Args};
pub use register_executive_v1::{register_executive_v1, RegisterExecutiveV1Args};
pub use register_x402_v1::{register_x402_v1, RegisterX402V1Args};
pub use revoke_execution_v1::{revoke_execution_v1, RevokeExecutionV1Args};

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
        return Err(MplAgentToolsError::InvalidInstructionData.into());
    }

    // Route by discriminant (first byte).
    match MplAgentToolsInstructionDiscriminant::try_from(instruction_data[0]) {
        Ok(MplAgentToolsInstructionDiscriminant::RegisterExecutorV1) => {
            msg!("Instruction: RegisterExecutiveV1");
            // Zero-copy: cast instruction data to args struct.
            let args: &RegisterExecutiveV1Args = bytemuck::from_bytes(
                &instruction_data[..core::mem::size_of::<RegisterExecutiveV1Args>()],
            );
            register_executive_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::DelegateExecutionV1) => {
            msg!("Instruction: DelegateExecutionV1");
            // Zero-copy: cast instruction data to args struct.
            let args: &DelegateExecutionV1Args = bytemuck::from_bytes(
                &instruction_data[..core::mem::size_of::<DelegateExecutionV1Args>()],
            );
            delegate_execution_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::RevokeExecutionV1) => {
            msg!("Instruction: RevokeExecutionV1");
            // Zero-copy: cast instruction data to args struct.
            let args: &RevokeExecutionV1Args = bytemuck::from_bytes(
                &instruction_data[..core::mem::size_of::<RevokeExecutionV1Args>()],
            );
            revoke_execution_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::RegisterX402V1) => {
            msg!("Instruction: RegisterX402V1");
            // Zero-copy: cast instruction data to args struct.
            let args: &RegisterX402V1Args = bytemuck::from_bytes(
                &instruction_data[..core::mem::size_of::<RegisterX402V1Args>()],
            );
            register_x402_v1(accounts, args)
        }
        Err(_) => Err(MplAgentToolsError::InvalidInstructionData.into()),
    }
}
