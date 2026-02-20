// ==============================
// src/error.rs
// ==============================
#![forbid(unsafe_code)]

use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[repr(u32)]
pub enum LockrionError {
    // 0–9: Instruction
    #[error("Invalid instruction")]
    InvalidInstruction = 0,

    // 10–19: Funding
    #[error("Reserve already funded")]
    ReserveAlreadyFunded = 10,
    #[error("Reserve not funded")]
    ReserveNotFunded = 11,
    #[error("Invalid funding amount")]
    InvalidFundingAmount = 12,
    #[error("Funding window closed")]
    FundingWindowClosed = 13,

    // 20–29: Deposit
    #[error("Deposit window not started")]
    DepositWindowNotStarted = 20,
    #[error("Deposit window closed")]
    DepositWindowClosed = 21,
    #[error("Deposit window not closed")]
    DepositWindowNotClosed = 22,
    #[error("Invalid amount")]
    InvalidAmount = 23,

    // 30–39: Claim
    #[error("Claim window not started")]
    ClaimWindowNotStarted = 30,
    #[error("Claim window closed")]
    ClaimWindowClosed = 31,
    #[error("Already claimed")]
    AlreadyClaimed = 32,

    // 40–49: Sweep / Reclaim
    #[error("Sweep already executed")]
    SweepAlreadyExecuted = 40,
    #[error("Reclaim already executed")]
    ReclaimAlreadyExecuted = 41,
    #[error("No participation")]
    NoParticipation = 42,

    // 50–59: Auth / Accounts
    #[error("Unauthorized caller")]
    UnauthorizedCaller = 50,
    #[error("Invalid PDA")]
    InvalidPda = 51,
    #[error("Invalid token program")]
    InvalidTokenProgram = 52,
    #[error("Invalid mint")]
    InvalidMint = 53,
    #[error("Invalid authority")]
    InvalidAuthority = 54,
    #[error("Invalid escrow account")]
    InvalidEscrowAccount = 55,
    #[error("Invalid platform treasury")]
    InvalidPlatformTreasury = 56,
    #[error("Invalid user state account")]
    InvalidUserStateAccount = 57,

    // 60–69: Math
    #[error("Arithmetic overflow")]
    ArithmeticOverflow = 60,
    #[error("Arithmetic underflow")]
    ArithmeticUnderflow = 61,
    #[error("Division by zero")]
    DivisionByZero = 62,
    #[error("Invariant violation")]
    InvariantViolation = 63,

    // 70–79: Layout
    #[error("Invalid state version")]
    InvalidStateVersion = 70,
    #[error("Invalid account size")]
    InvalidAccountSize = 71,
}

impl From<LockrionError> for ProgramError {
    fn from(e: LockrionError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
