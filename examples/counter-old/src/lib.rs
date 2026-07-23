//! Baseline ("old") version of the Preflight example counter program.
//!
//! See `../counter-new` for the modified version used to demonstrate
//! regression detection. The two crates are meant to be diffed directly
//! against each other.

// solana_program::entrypoint! expands to code referencing cfg values this
// crate never declares; harmless, but noisy without this.
#![allow(unexpected_cfgs)]

pub mod instruction;
pub mod processor;
pub mod state;

mod entrypoint;
