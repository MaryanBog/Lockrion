// ==============================
// src/state.rs  (BYTE-EXACT layout per State Layout v1.1)
// ==============================
#![forbid(unsafe_code)]

use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::LockrionError;

/// Must match State Layout v1.1 exactly:
/// IssuanceState size = 292 bytes; UserState size = 112 bytes. :contentReference[oaicite:3]{index=3}
pub const ISSUANCE_STATE_SIZE: usize = 292;
pub const USER_STATE_SIZE: usize = 112;

pub const STATE_VERSION: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuanceState {
    pub version: u8,            // 0
    pub bump: u8,               // 1
    pub issuer_address: Pubkey, // 2..34
    pub lock_mint: Pubkey,      // 34..66
    pub reward_mint: Pubkey,    // 66..98
    pub deposit_escrow: Pubkey, // 98..130
    pub reward_escrow: Pubkey,  // 130..162
    pub platform_treasury: Pubkey, // 162..194
    pub reserve_total: u128,    // 194..210
    pub start_ts: i64,          // 210..218
    pub maturity_ts: i64,       // 218..226
    pub claim_window: i64,      // 226..234
    pub final_day_index: u64,   // 234..242
    pub total_locked: u128,     // 242..258
    pub total_weight_accum: u128, // 258..274
    pub last_day_index: u64,    // 274..282
    pub reserve_funded: u8,     // 282
    pub sweep_executed: u8,     // 283
    pub reclaim_executed: u8,   // 284
    pub reserved_padding: [u8; 7], // 285..292
}

impl IssuanceState {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() != ISSUANCE_STATE_SIZE {
            return Err(LockrionError::InvalidAccountSize.into());
        }
        let version = input[0];
        if version != STATE_VERSION {
            return Err(LockrionError::InvalidStateVersion.into());
        }
        let bump = input[1];

        let issuer_address = Pubkey::new_from_array(input[2..34].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let lock_mint = Pubkey::new_from_array(input[34..66].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let reward_mint = Pubkey::new_from_array(input[66..98].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let deposit_escrow = Pubkey::new_from_array(input[98..130].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let reward_escrow = Pubkey::new_from_array(input[130..162].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let platform_treasury = Pubkey::new_from_array(input[162..194].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);

        let reserve_total = u128::from_le_bytes(input[194..210].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let start_ts = i64::from_le_bytes(input[210..218].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let maturity_ts = i64::from_le_bytes(input[218..226].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let claim_window = i64::from_le_bytes(input[226..234].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let final_day_index = u64::from_le_bytes(input[234..242].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);

        let total_locked = u128::from_le_bytes(input[242..258].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let total_weight_accum = u128::from_le_bytes(input[258..274].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let last_day_index = u64::from_le_bytes(input[274..282].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);

        let reserve_funded = input[282];
        let sweep_executed = input[283];
        let reclaim_executed = input[284];
        let reserved_padding: [u8; 7] = input[285..292].try_into().map_err(|_| LockrionError::InvalidAccountSize)?;

        Ok(Self {
            version, bump, issuer_address, lock_mint, reward_mint, deposit_escrow, reward_escrow, platform_treasury,
            reserve_total, start_ts, maturity_ts, claim_window, final_day_index,
            total_locked, total_weight_accum, last_day_index,
            reserve_funded, sweep_executed, reclaim_executed, reserved_padding,
        })
    }

    pub fn pack(&self, output: &mut [u8]) -> Result<(), ProgramError> {
        if output.len() != ISSUANCE_STATE_SIZE {
            return Err(LockrionError::InvalidAccountSize.into());
        }
        if self.version != STATE_VERSION {
            return Err(LockrionError::InvalidStateVersion.into());
        }

        output[0] = self.version;
        output[1] = self.bump;

        output[2..34].copy_from_slice(self.issuer_address.as_ref());
        output[34..66].copy_from_slice(self.lock_mint.as_ref());
        output[66..98].copy_from_slice(self.reward_mint.as_ref());
        output[98..130].copy_from_slice(self.deposit_escrow.as_ref());
        output[130..162].copy_from_slice(self.reward_escrow.as_ref());
        output[162..194].copy_from_slice(self.platform_treasury.as_ref());

        output[194..210].copy_from_slice(&self.reserve_total.to_le_bytes());
        output[210..218].copy_from_slice(&self.start_ts.to_le_bytes());
        output[218..226].copy_from_slice(&self.maturity_ts.to_le_bytes());
        output[226..234].copy_from_slice(&self.claim_window.to_le_bytes());
        output[234..242].copy_from_slice(&self.final_day_index.to_le_bytes());

        output[242..258].copy_from_slice(&self.total_locked.to_le_bytes());
        output[258..274].copy_from_slice(&self.total_weight_accum.to_le_bytes());
        output[274..282].copy_from_slice(&self.last_day_index.to_le_bytes());

        output[282] = self.reserve_funded;
        output[283] = self.sweep_executed;
        output[284] = self.reclaim_executed;

        output[285..292].copy_from_slice(&self.reserved_padding);
        Ok(())
    }

    #[inline]
    pub fn is_reserve_funded(&self) -> bool { self.reserve_funded == 1 }
    #[inline]
    pub fn is_sweep_executed(&self) -> bool { self.sweep_executed == 1 }
    #[inline]
    pub fn is_reclaim_executed(&self) -> bool { self.reclaim_executed == 1 }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserState {
    pub version: u8,         // 0
    pub bump: u8,            // 1
    pub issuance: Pubkey,    // 2..34
    pub participant: Pubkey, // 34..66
    pub locked_amount: u128, // 66..82
    pub user_weight_accum: u128, // 82..98
    pub user_last_day_index: u64, // 98..106
    pub reward_claimed: u8,  // 106
    pub reserved_padding: [u8; 5], // 107..112
}

impl UserState {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() != USER_STATE_SIZE {
            return Err(LockrionError::InvalidAccountSize.into());
        }
        let version = input[0];
        if version != STATE_VERSION {
            return Err(LockrionError::InvalidStateVersion.into());
        }
        let bump = input[1];

        let issuance = Pubkey::new_from_array(input[2..34].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let participant = Pubkey::new_from_array(input[34..66].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);

        let locked_amount = u128::from_le_bytes(input[66..82].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let user_weight_accum = u128::from_le_bytes(input[82..98].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);
        let user_last_day_index = u64::from_le_bytes(input[98..106].try_into().map_err(|_| LockrionError::InvalidAccountSize)?);

        let reward_claimed = input[106];
        let reserved_padding: [u8; 5] = input[107..112].try_into().map_err(|_| LockrionError::InvalidAccountSize)?;

        Ok(Self {
            version, bump, issuance, participant, locked_amount, user_weight_accum, user_last_day_index,
            reward_claimed, reserved_padding,
        })
    }

    pub fn pack(&self, output: &mut [u8]) -> Result<(), ProgramError> {
        if output.len() != USER_STATE_SIZE {
            return Err(LockrionError::InvalidAccountSize.into());
        }
        if self.version != STATE_VERSION {
            return Err(LockrionError::InvalidStateVersion.into());
        }

        output[0] = self.version;
        output[1] = self.bump;

        output[2..34].copy_from_slice(self.issuance.as_ref());
        output[34..66].copy_from_slice(self.participant.as_ref());

        output[66..82].copy_from_slice(&self.locked_amount.to_le_bytes());
        output[82..98].copy_from_slice(&self.user_weight_accum.to_le_bytes());
        output[98..106].copy_from_slice(&self.user_last_day_index.to_le_bytes());

        output[106] = self.reward_claimed;
        output[107..112].copy_from_slice(&self.reserved_padding);
        Ok(())
    }

    #[inline]
    pub fn is_reward_claimed(&self) -> bool { self.reward_claimed == 1 }
}
