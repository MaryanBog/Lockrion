// ==============================
// src/accumulator.rs (strict discrete day model)
// ==============================
#![forbid(unsafe_code)]

use crate::error::LockrionError;

/// accounting_period fixed to 86400 by profile; day_index is floor((t-start)/86400). :contentReference[oaicite:4]{index=4}
pub const ACCOUNTING_PERIOD: i64 = 86_400;

pub fn raw_day_index(block_ts: i64, start_ts: i64) -> Result<u64, LockrionError> {
    if block_ts < start_ts {
        return Ok(0);
    }
    let delta = block_ts
        .checked_sub(start_ts)
        .ok_or(LockrionError::ArithmeticUnderflow)?;
    // delta >= 0 here
    let d = delta / ACCOUNTING_PERIOD; // deterministic integer division
    if d < 0 {
        // should be unreachable given checks, but keep deterministic guard
        return Err(LockrionError::InvariantViolation);
    }
    Ok(d as u64)
}

pub fn bounded_day_index(raw: u64, final_day_index: u64) -> u64 {
    if raw > final_day_index { final_day_index } else { raw }
}
