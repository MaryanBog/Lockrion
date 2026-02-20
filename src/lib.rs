// ==============================
// src/lib.rs
// ==============================
#![deny(warnings)]
#![forbid(unsafe_code)]

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod pda;
pub mod accumulator;

solana_program::declare_id!("GyJD65QDSNaskfNEpYaxJokog84ZjAx84nvm62NzS4wj"); // TODO: replace
