// ==============================
// src/instruction.rs
// ==============================
#![forbid(unsafe_code)]

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub enum LockrionInstruction {
    /// InitIssuance(reserve_total, start_ts, maturity_ts)
    /// Creates and initializes issuance state account.
    InitIssuance {
        reserve_total: u128,
        start_ts: i64,
        maturity_ts: i64,
    },

    /// fund_reserve(amount: u64)
    /// amount must equal reserve_total (u128) in state.
    FundReserve {
        amount: u64,
    },

    /// deposit(amount: u64)
    Deposit {
        amount: u64,
    },

    /// claim_reward()
    ClaimReward,

    /// withdraw_deposit()
    WithdrawDeposit,

    /// sweep()
    Sweep,

    /// zero_participation_reclaim()
    ZeroParticipationReclaim,
}