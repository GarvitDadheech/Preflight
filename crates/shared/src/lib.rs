//! Data types shared across the Preflight workspace: the transaction
//! fixture format, per-transaction execution results, and the report
//! structures produced by comparing two runs.
//!
//! This crate holds plain, serializable data only. It has no dependency
//! on the Solana runtime or on litesvm; `replay` is responsible for
//! turning a [`fixture::Fixture`] into real transactions and back into
//! [`execution::TxExecutionResult`]s.

pub mod error;
pub mod execution;
pub mod fixture;
pub mod report;

pub use error::{Error, Result};
pub use execution::{CounterStateSnapshot, TxExecutionResult};
pub use fixture::{CounterInstructionSpec, Fixture, KeypairBytes, SignerRole, TransactionSpec};
pub use report::{Report, RunSummary, TxComparison, TxOutcomeCategory};
