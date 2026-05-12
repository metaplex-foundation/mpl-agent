mod delegate_execution_v1;
mod initialize_program_config_v1;
mod mint_work_receipt_v1;
mod register_executive_v1;
mod register_receipts_tree_v1;
mod revoke_execution_v1;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentToolsError;
use crate::instruction::MplAgentToolsInstructionDiscriminant;

pub use delegate_execution_v1::{delegate_execution_v1, DelegateExecutionV1Args};
pub use initialize_program_config_v1::{
    cast_args as cast_initialize_program_config_args, initialize_program_config_v1,
    receipts_collection_prefix, InitializeToolsConfigV1Args,
};
pub use mint_work_receipt_v1::{
    deserialize_mint_work_receipt_args, mint_work_receipt_v1, MintWorkReceiptV1Args,
    MAX_RECEIPT_URI_LEN,
};
pub use register_executive_v1::{register_executive_v1, RegisterExecutiveV1Args};
pub use register_receipts_tree_v1::{
    cast_register_receipts_tree_args, register_receipts_tree_v1, RegisterReceiptsTreeV1Args,
};
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
        Ok(MplAgentToolsInstructionDiscriminant::MintWorkReceiptV1) => {
            msg!("Instruction: MintWorkReceiptV1");
            // Variable-length borsh args follow the 1-byte discriminator.
            let args = deserialize_mint_work_receipt_args(&instruction_data[1..])?;
            mint_work_receipt_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::InitializeToolsConfigV1) => {
            msg!("Instruction: InitializeToolsConfigV1");
            let args = cast_initialize_program_config_args(instruction_data)?;
            initialize_program_config_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::RegisterReceiptsTreeV1) => {
            msg!("Instruction: RegisterReceiptsTreeV1");
            let args = cast_register_receipts_tree_args(instruction_data)?;
            register_receipts_tree_v1(accounts, args)
        }
        Err(_) => Err(MplAgentToolsError::InvalidInstructionData.into()),
    }
}
