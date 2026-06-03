mod create_receipts_collection_v1;
mod delegate_execution_v1;
mod mint_work_receipt_v1;
mod register_executive_v1;
mod register_receipts_tree_v1;
mod revoke_execution_v1;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentToolsError;
use crate::instruction::MplAgentToolsInstructionDiscriminant;

pub use create_receipts_collection_v1::{
    cast_create_receipts_collection_args, create_receipts_collection_v1,
    CreateReceiptsCollectionV1Args,
};
pub use delegate_execution_v1::{delegate_execution_v1, DelegateExecutionV1Args};
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
#[inline]
pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(MplAgentToolsError::InvalidInstructionData.into());
    }

    match MplAgentToolsInstructionDiscriminant::try_from(instruction_data[0]) {
        Ok(MplAgentToolsInstructionDiscriminant::RegisterExecutorV1) => {
            msg!("Instruction: RegisterExecutiveV1");
            let args: &RegisterExecutiveV1Args = bytemuck::from_bytes(
                &instruction_data[..core::mem::size_of::<RegisterExecutiveV1Args>()],
            );
            register_executive_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::DelegateExecutionV1) => {
            msg!("Instruction: DelegateExecutionV1");
            let args: &DelegateExecutionV1Args = bytemuck::from_bytes(
                &instruction_data[..core::mem::size_of::<DelegateExecutionV1Args>()],
            );
            delegate_execution_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::RevokeExecutionV1) => {
            msg!("Instruction: RevokeExecutionV1");
            let args: &RevokeExecutionV1Args = bytemuck::from_bytes(
                &instruction_data[..core::mem::size_of::<RevokeExecutionV1Args>()],
            );
            revoke_execution_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::MintWorkReceiptV1) => {
            msg!("Instruction: MintWorkReceiptV1");
            let args = deserialize_mint_work_receipt_args(&instruction_data[1..])?;
            mint_work_receipt_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::CreateReceiptsCollectionV1) => {
            msg!("Instruction: CreateReceiptsCollectionV1");
            let args = cast_create_receipts_collection_args(instruction_data)?;
            create_receipts_collection_v1(accounts, args)
        }
        Ok(MplAgentToolsInstructionDiscriminant::RegisterReceiptsTreeV1) => {
            msg!("Instruction: RegisterReceiptsTreeV1");
            let args = cast_register_receipts_tree_args(instruction_data)?;
            register_receipts_tree_v1(accounts, args)
        }
        Err(_) => Err(MplAgentToolsError::InvalidInstructionData.into()),
    }
}
