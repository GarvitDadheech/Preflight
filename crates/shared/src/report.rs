use serde::{Deserialize, Serialize};

use crate::execution::TxExecutionResult;

/// How a single transaction's outcome differs between the old and new
/// program builds.
///
/// Ordered roughly from "fine" to "concerning"; `comparator` picks the
/// first category that applies when several conditions overlap.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxOutcomeCategory {
    /// Same success/failure, same resulting state or error, same compute
    /// units.
    Unchanged,
    /// Same success/failure and same resulting state, but compute unit
    /// consumption differs.
    ComputeUnitsChanged,
    /// Both versions succeeded, but the resulting account state differs.
    BehaviorChanged,
    /// Both versions failed, but with a different error.
    ErrorChanged,
    /// Succeeded on the old program, failed on the new one. A regression.
    NewFailure,
    /// Failed on the old program, succeeded on the new one.
    NewSuccess,
}

impl TxOutcomeCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::ComputeUnitsChanged => "compute units changed",
            Self::BehaviorChanged => "behavior changed",
            Self::ErrorChanged => "error changed",
            Self::NewFailure => "new failure",
            Self::NewSuccess => "new success",
        }
    }

    /// Whether this category represents a meaningful behavioral
    /// regression worth calling out, as opposed to background noise like
    /// a compute unit shift.
    pub fn is_regression(self) -> bool {
        matches!(
            self,
            Self::BehaviorChanged | Self::ErrorChanged | Self::NewFailure | Self::NewSuccess
        )
    }
}

/// Full comparison of one transaction's execution against both program
/// builds.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxComparison {
    pub label: String,
    pub description: String,
    pub category: TxOutcomeCategory,
    pub old: TxExecutionResult,
    pub new: TxExecutionResult,
    pub compute_units_delta: i64,
    /// Plain-language explanation of what changed and why, e.g. "old
    /// succeeded (value=2000), new failed with Custom program error 0x1".
    pub notes: String,
}

/// Aggregate counts and the overall safety score for a run.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RunSummary {
    pub total: usize,
    pub unchanged: usize,
    pub compute_units_changed: usize,
    pub behavior_changed: usize,
    pub error_changed: usize,
    pub new_failures: usize,
    pub new_successes: usize,
    /// 0 (unsafe to deploy) to 100 (no observed behavioral difference).
    pub safety_score: u8,
}

/// The complete result of a `preflight run`, in the shape written to
/// `report.json` and rendered into `report.md`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Report {
    pub old_program: String,
    pub new_program: String,
    pub summary: RunSummary,
    pub comparisons: Vec<TxComparison>,
}
