#![allow(unexpected_cfgs)]

use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::{error::MplAgentToolsError, processor};

entrypoint!(process_instruction);
fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::process_instruction(program_id, accounts, instruction_data) {
        // `PrintProgramError` and `DecodeError` were removed in solana-program 3.0,
        // so we manually decode the custom code back to our error enum and log its
        // message here — preserving the on-chain log behavior from solana 2.
        if let ProgramError::Custom(code) = error {
            if let Some(decoded) = MplAgentToolsError::from_u32(code) {
                msg!("{}", decoded);
            }
        }
        return Err(error);
    }
    Ok(())
}
