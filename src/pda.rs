// ==============================
// src/pda.rs (canonical seeds v1.1)
// ==============================
#![forbid(unsafe_code)]

use solana_program::pubkey::Pubkey;

pub const SEED_ISSUANCE: &[u8] = b"issuance";
pub const SEED_USER: &[u8] = b"user";

pub fn derive_issuance_pda(
    program_id: &Pubkey,
    issuer_address: &Pubkey,
    start_ts: i64,
    reserve_total: u128,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_ISSUANCE,
            issuer_address.as_ref(),
            &start_ts.to_le_bytes(),
            &reserve_total.to_le_bytes(),
        ],
        program_id,
    )
}

pub fn derive_user_pda(
    program_id: &Pubkey,
    issuance_pda: &Pubkey,
    participant: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_USER,
            issuance_pda.as_ref(),
            participant.as_ref(),
        ],
        program_id,
    )
}
