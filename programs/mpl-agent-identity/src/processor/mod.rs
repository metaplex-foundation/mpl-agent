mod register_identity_v1;
mod set_agent_token_v1;

pub use register_identity_v1::{register_identity_v1, RegisterIdentityV1Args};
pub use set_agent_token_v1::{set_agent_token_v1, SetAgentTokenV1Args};

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::error::MplAgentIdentityError;
use crate::instruction::MplAgentIdentityInstructionDiscriminant;

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
        Ok(MplAgentIdentityInstructionDiscriminant::SetAgentTokenV1) => {
            msg!("Instruction: SetAgentTokenV1");
            set_agent_token_v1(accounts, instruction_data)
        }
        Err(_) => Err(MplAgentIdentityError::InvalidInstructionData.into()),
    }
}
