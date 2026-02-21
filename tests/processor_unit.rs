// tests/processor_unit.rs

use lockrion_issuance_v1_1::{
    accumulator,
    error::LockrionError,
    state::{IssuanceState, UserState},
};

// -----------------------------
// Mocks
// -----------------------------
fn mock_issuance() -> IssuanceState {
    IssuanceState {
        version: lockrion_issuance_v1_1::state::STATE_VERSION,
        bump: 1,
        issuer_address: Default::default(),

        lock_mint: Default::default(),
        reward_mint: Default::default(),
        deposit_escrow: Default::default(),
        reward_escrow: Default::default(),
        platform_treasury: Default::default(),

        reserve_total: 1000,
        start_ts: 0,
        maturity_ts: 86400 * 10,
        claim_window: 86400,
        final_day_index: 10,

        total_locked: 0,
        total_weight_accum: 0,
        last_day_index: 0,

        reserve_funded: 1,
        sweep_executed: 0,
        reclaim_executed: 0,
        reserved_padding: [0u8; 7],
    }
}

fn mock_user() -> UserState {
    UserState {
        version: lockrion_issuance_v1_1::state::STATE_VERSION,
        bump: 1,
        issuance: Default::default(),
        participant: Default::default(),

        locked_amount: 0,
        user_weight_accum: 0,
        user_last_day_index: 0,

        reward_claimed: 0,
        reserved_padding: [0u8; 5],
    }
}

// -----------------------------
// Pure helpers (mirror contract logic)
// -----------------------------
fn finalize_global_pure(issuance: &mut IssuanceState, now: i64) -> Result<u64, LockrionError> {
    let raw = accumulator::raw_day_index(now, issuance.start_ts)?;
    let current = accumulator::bounded_day_index(raw, issuance.final_day_index);

    if current > issuance.last_day_index {
        let days_elapsed = current
            .checked_sub(issuance.last_day_index)
            .ok_or(LockrionError::ArithmeticUnderflow)?;

        let inc = issuance
            .total_locked
            .checked_mul(days_elapsed as u128)
            .ok_or(LockrionError::ArithmeticOverflow)?;

        issuance.total_weight_accum = issuance
            .total_weight_accum
            .checked_add(inc)
            .ok_or(LockrionError::ArithmeticOverflow)?;

        issuance.last_day_index = current;
    }

    if issuance.last_day_index > issuance.final_day_index {
        return Err(LockrionError::InvariantViolation);
    }

    Ok(current)
}

fn update_user_with_current_pure(user: &mut UserState, current: u64) -> Result<(), LockrionError> {
    if current > user.user_last_day_index {
        let days_elapsed_user = current
            .checked_sub(user.user_last_day_index)
            .ok_or(LockrionError::ArithmeticUnderflow)?;

        let inc = user
            .locked_amount
            .checked_mul(days_elapsed_user as u128)
            .ok_or(LockrionError::ArithmeticOverflow)?;

        user.user_weight_accum = user
            .user_weight_accum
            .checked_add(inc)
            .ok_or(LockrionError::ArithmeticOverflow)?;

        user.user_last_day_index = current;
    }
    Ok(())
}

fn reward_calc(reserve_total: u128, user_weight: u128, total_weight: u128) -> Result<u128, LockrionError> {
    if total_weight == 0 {
        return Err(LockrionError::DivisionByZero);
    }

    let num = reserve_total
        .checked_mul(user_weight)
        .ok_or(LockrionError::ArithmeticOverflow)?;

    Ok(num / total_weight)
}

// ==============================
// UT-GACC-01..04 (Global accumulator)
// ==============================

#[test]
fn ut_gacc_01_basic_accumulation() {
    let mut iss = mock_issuance();
    iss.total_locked = 100;

    finalize_global_pure(&mut iss, 86400 * 3).unwrap();

    assert_eq!(iss.total_weight_accum, 300);
    assert_eq!(iss.last_day_index, 3);
}

#[test]
fn ut_gacc_02_zero_days_elapsed() {
    let mut iss = mock_issuance();
    iss.total_locked = 100;

    finalize_global_pure(&mut iss, 0).unwrap();

    assert_eq!(iss.total_weight_accum, 0);
    assert_eq!(iss.last_day_index, 0);
}

#[test]
fn ut_gacc_03_bounded_to_final_day() {
    let mut iss = mock_issuance();
    iss.total_locked = 10;
    iss.final_day_index = 5;

    finalize_global_pure(&mut iss, 86400 * 100).unwrap();

    assert_eq!(iss.last_day_index, 5);
    assert_eq!(iss.total_weight_accum, 10 * 5);
}

#[test]
fn ut_gacc_04_overflow_guard() {
    let mut iss = mock_issuance();
    iss.total_locked = u128::MAX;

    // make days_elapsed >= 2 so mul overflows: MAX * 2
    let r = finalize_global_pure(&mut iss, 86400 * 2);

    assert!(matches!(r, Err(LockrionError::ArithmeticOverflow)));
}

// ==============================
// UT-UACC-01..04 (Per-user accumulator)
// ==============================

#[test]
fn ut_uacc_01_basic_accumulation() {
    let mut user = mock_user();
    user.locked_amount = 50;
    user.user_last_day_index = 1;

    update_user_with_current_pure(&mut user, 4).unwrap();

    assert_eq!(user.user_weight_accum, 150);
    assert_eq!(user.user_last_day_index, 4);
}

#[test]
fn ut_uacc_02_zero_days_elapsed() {
    let mut user = mock_user();
    user.locked_amount = 50;
    user.user_last_day_index = 4;
    user.user_weight_accum = 777;

    update_user_with_current_pure(&mut user, 4).unwrap();

    assert_eq!(user.user_weight_accum, 777);
    assert_eq!(user.user_last_day_index, 4);
}

#[test]
fn ut_uacc_03_already_final_day_no_change() {
    let mut user = mock_user();
    user.locked_amount = 123;
    user.user_last_day_index = 5;
    user.user_weight_accum = 9;

    update_user_with_current_pure(&mut user, 5).unwrap();

    assert_eq!(user.user_weight_accum, 9);
    assert_eq!(user.user_last_day_index, 5);
}

#[test]
fn ut_uacc_04_overflow_guard() {
    let mut user = mock_user();
    user.locked_amount = u128::MAX;
    user.user_last_day_index = 0;

    let r = update_user_with_current_pure(&mut user, 2);

    assert!(matches!(r, Err(LockrionError::ArithmeticOverflow)));
}

// ==============================
// UT-BOUND-01..02 (Bounded accumulation)
// ==============================

#[test]
fn ut_bound_01_day_index_clamps() {
    assert_eq!(accumulator::bounded_day_index(20, 5), 5);
}

#[test]
fn ut_bound_02_clamp_idempotent() {
    let a = accumulator::bounded_day_index(20, 5);
    let b = accumulator::bounded_day_index(a, 5);
    assert_eq!(a, b);
}

// ==============================
// UT-SAME-01..02 (Same-day determinism)
// ==============================

#[test]
fn ut_same_01_same_timestamp_same_day_index() {
    let start = 1_000_000;
    let block = start + 123;
    let a = accumulator::raw_day_index(block, start).unwrap();
    let b = accumulator::raw_day_index(block, start).unwrap();
    assert_eq!(a, b);
}

#[test]
fn ut_same_02_two_calls_deterministic() {
    let start = 0;
    let block = accumulator::ACCOUNTING_PERIOD * 7 + 9;
    let a = accumulator::raw_day_index(block, start).unwrap();
    let b = accumulator::raw_day_index(block, start).unwrap();
    assert_eq!(a, b);
}

// ==============================
// UT-REWARD-01..04 (Reward formula)
// ==============================

#[test]
fn ut_reward_01_proportional() {
    let r = reward_calc(1000, 25, 100).unwrap();
    assert_eq!(r, 250);
}

#[test]
fn ut_reward_02_floor_rounding() {
    let r = reward_calc(1000, 1, 3).unwrap();
    assert_eq!(r, 333);
}

#[test]
fn ut_reward_03_division_by_zero() {
    let r = reward_calc(1000, 1, 0);
    assert!(matches!(r, Err(LockrionError::DivisionByZero)));
}

#[test]
fn ut_reward_04_overflow_guard() {
    let r = reward_calc(u128::MAX, u128::MAX, 1);
    assert!(matches!(r, Err(LockrionError::ArithmeticOverflow)));
}

// ==============================
// UT-ARITH-01..03 (Arithmetic safety)
// ==============================

#[test]
fn ut_arith_01_add_overflow() {
    assert!(u128::MAX.checked_add(1).is_none());
}

#[test]
fn ut_arith_02_sub_underflow() {
    assert!(0u128.checked_sub(1).is_none());
}

#[test]
fn ut_arith_03_negative_ts_returns_zero_day() {
    let r = accumulator::raw_day_index(-10, 0).unwrap();
    assert_eq!(r, 0);
}