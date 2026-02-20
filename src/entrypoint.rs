// ==============================
// src/entrypoint.rs
// ==============================
#![forbid(unsafe_code)]

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::processor::Processor;

solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, ix_data)
}
