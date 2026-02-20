// ==============================
// src/instruction.rs
// ==============================
#![forbid(unsafe_code)]

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub enum LockrionInstruction {
    /// fund_reserve(amount: u64) - amount must equal reserve_total (u128) in state;
    /// we still pass u64 for client convenience; program will validate exactness vs reserve_total.
    FundReserve { amount: u64 },

    /// deposit(amount: u64) - amount > 0
    Deposit { amount: u64 },

    /// claim_reward()
    ClaimReward,

    /// withdraw_deposit()
    WithdrawDeposit,

    /// sweep()
    Sweep,

    /// zero_participation_reclaim()
    ZeroParticipationReclaim,
}