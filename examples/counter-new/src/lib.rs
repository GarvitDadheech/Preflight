//! Modified ("new") version of the Preflight example counter program.
//!
//! Diff this crate against `../counter-old` to see the intentional
//! behavior changes: `error.rs` is new, and `processor.rs` adds a value
//! cap and turns a silently-clamped decrement into a hard error. `state.rs`,
//! `instruction.rs` and `entrypoint.rs` are untouched.

// solana_program::entrypoint! expands to code referencing cfg values this
// crate never declares; harmless, but noisy without this.
#![allow(unexpected_cfgs)]

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

mod entrypoint;
