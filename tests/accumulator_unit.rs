use lockrion_issuance_v1_1::accumulator::*;
use lockrion_issuance_v1_1::error::LockrionError;

// =====================================================
// RAW DAY INDEX TESTS
// =====================================================

#[test]
fn ut_raw_day_before_start_returns_zero() {
    let start = 1_000_000;
    let block = 999_000;

    let idx = raw_day_index(block, start).unwrap();
    assert_eq!(idx, 0);
}

#[test]
fn ut_raw_day_within_first_day() {
    let start = 1_000_000;
    let block = start + 10_000; // < 86400

    let idx = raw_day_index(block, start).unwrap();
    assert_eq!(idx, 0);
}

#[test]
fn ut_raw_day_exact_boundary() {
    let start = 1_000_000;
    let block = start + ACCOUNTING_PERIOD;

    let idx = raw_day_index(block, start).unwrap();
    assert_eq!(idx, 1);
}

#[test]
fn ut_raw_day_multiple_days() {
    let start = 1_000_000;
    let block = start + ACCOUNTING_PERIOD * 5 + 1;

    let idx = raw_day_index(block, start).unwrap();
    assert_eq!(idx, 5);
}

// =====================================================
// ARITHMETIC SAFETY
// =====================================================

#[test]
fn ut_raw_day_overflow_guard() {
    let block = i64::MAX;
    let start = -1;

    let result = raw_day_index(block, start);

    assert!(matches!(result, Err(LockrionError::ArithmeticUnderflow)));
}

// =====================================================
// BOUNDED DAY INDEX TESTS
// =====================================================

#[test]
fn ut_bounded_index_normal() {
    let raw = 3;
    let final_day = 10;

    let bounded = bounded_day_index(raw, final_day);
    assert_eq!(bounded, 3);
}

#[test]
fn ut_bounded_index_clamped() {
    let raw = 15;
    let final_day = 10;

    let bounded = bounded_day_index(raw, final_day);
    assert_eq!(bounded, 10);
}

#[test]
fn ut_bound_01_clamp_after_final_day() {
    let raw = 20;
    let final_day = 5;

    let bounded = bounded_day_index(raw, final_day);
    assert_eq!(bounded, 5);
}

#[test]
fn ut_bound_02_idempotent_clamp() {
    let raw = 20;
    let final_day = 5;

    let first = bounded_day_index(raw, final_day);
    let second = bounded_day_index(first, final_day);

    assert_eq!(first, second);
}

#[test]
fn ut_same_01_same_timestamp_same_result() {
    let start = 1_000_000;
    let block = start + 10;

    let idx1 = raw_day_index(block, start).unwrap();
    let idx2 = raw_day_index(block, start).unwrap();

    assert_eq!(idx1, idx2);
}

#[test]
fn ut_same_02_repeated_call_deterministic() {
    let start = 1_000_000;
    let block = start + ACCOUNTING_PERIOD * 3;

    let first = raw_day_index(block, start).unwrap();
    let second = raw_day_index(block, start).unwrap();

    assert_eq!(first, second);
}

#[test]
fn ut_arith_03_negative_start_safe() {
    let start = -1_000;
    let block = 0;

    let idx = raw_day_index(block, start).unwrap();

    assert_eq!(idx, ((0 - (-1_000)) / ACCOUNTING_PERIOD) as u64);
}