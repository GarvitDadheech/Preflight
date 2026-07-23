//! Compares two sets of transaction replay results (one per program
//! build) and classifies how each transaction's behavior did or did not
//! change.
//!
//! This crate is pure: it has no knowledge of Solana, litesvm, or the
//! filesystem. It only reasons about the [`TxExecutionResult`] values
//! `replay` produces.

use preflight_shared::{
    Error, Report, Result, RunSummary, TransactionSpec, TxComparison, TxExecutionResult,
    TxOutcomeCategory,
};

/// Penalty subtracted from a 100-point baseline safety score for each
/// occurrence of a category. Silent behavior changes and new failures
/// are weighted the heaviest because they are the regressions an upgrade
/// is least likely to be caught in manual testing.
fn penalty(category: TxOutcomeCategory) -> i64 {
    match category {
        TxOutcomeCategory::Unchanged => 0,
        TxOutcomeCategory::ComputeUnitsChanged => 3,
        TxOutcomeCategory::ErrorChanged => 8,
        TxOutcomeCategory::NewSuccess => 15,
        TxOutcomeCategory::NewFailure => 20,
        TxOutcomeCategory::BehaviorChanged => 20,
    }
}

/// Compares aligned old/new replay results and produces a full [`Report`].
///
/// `transactions`, `old_results` and `new_results` must all have the same
/// length and agree on transaction order (they do, as long as both replay
/// runs were given the same [`preflight_shared::Fixture`]).
pub fn compare(
    old_program: &str,
    new_program: &str,
    transactions: &[TransactionSpec],
    old_results: &[TxExecutionResult],
    new_results: &[TxExecutionResult],
) -> Result<Report> {
    if transactions.len() != old_results.len() || transactions.len() != new_results.len() {
        return Err(Error::Mismatch(format!(
            "expected {} results from each run, got {} old and {} new",
            transactions.len(),
            old_results.len(),
            new_results.len()
        )));
    }

    let mut comparisons = Vec::with_capacity(transactions.len());
    let mut summary = RunSummary {
        total: transactions.len(),
        ..Default::default()
    };
    let mut score: i64 = 100;

    for ((spec, old), new) in transactions.iter().zip(old_results).zip(new_results) {
        if spec.label != old.label || spec.label != new.label {
            return Err(Error::Mismatch(format!(
                "transaction order drifted: expected '{}', got old='{}' new='{}'",
                spec.label, old.label, new.label
            )));
        }

        let category = categorize(old, new);
        score -= penalty(category);

        match category {
            TxOutcomeCategory::Unchanged => summary.unchanged += 1,
            TxOutcomeCategory::ComputeUnitsChanged => summary.compute_units_changed += 1,
            TxOutcomeCategory::BehaviorChanged => summary.behavior_changed += 1,
            TxOutcomeCategory::ErrorChanged => summary.error_changed += 1,
            TxOutcomeCategory::NewFailure => summary.new_failures += 1,
            TxOutcomeCategory::NewSuccess => summary.new_successes += 1,
        }

        comparisons.push(TxComparison {
            label: spec.label.clone(),
            description: spec.description.clone(),
            category,
            compute_units_delta: new.compute_units_consumed as i64
                - old.compute_units_consumed as i64,
            notes: explain(category, old, new),
            old: old.clone(),
            new: new.clone(),
        });
    }

    summary.safety_score = score.clamp(0, 100) as u8;

    Ok(Report {
        old_program: old_program.to_string(),
        new_program: new_program.to_string(),
        summary,
        comparisons,
    })
}

fn categorize(old: &TxExecutionResult, new: &TxExecutionResult) -> TxOutcomeCategory {
    match (old.success, new.success) {
        (true, false) => TxOutcomeCategory::NewFailure,
        (false, true) => TxOutcomeCategory::NewSuccess,
        (false, false) => {
            if old.error == new.error {
                TxOutcomeCategory::Unchanged
            } else {
                TxOutcomeCategory::ErrorChanged
            }
        }
        (true, true) => {
            if old.counter_state != new.counter_state {
                TxOutcomeCategory::BehaviorChanged
            } else if old.compute_units_consumed != new.compute_units_consumed {
                TxOutcomeCategory::ComputeUnitsChanged
            } else {
                TxOutcomeCategory::Unchanged
            }
        }
    }
}

fn explain(category: TxOutcomeCategory, old: &TxExecutionResult, new: &TxExecutionResult) -> String {
    match category {
        TxOutcomeCategory::Unchanged if old.success => format!(
            "Succeeded on both versions with identical resulting state ({}).",
            describe_state(old)
        ),
        TxOutcomeCategory::Unchanged => format!(
            "Failed on both versions with the same error ({}).",
            old.error.as_deref().unwrap_or("unknown error")
        ),
        TxOutcomeCategory::ComputeUnitsChanged => format!(
            "Succeeded on both versions with identical resulting state ({}), but compute units \
             changed from {} to {} ({:+}).",
            describe_state(old),
            old.compute_units_consumed,
            new.compute_units_consumed,
            new.compute_units_consumed as i64 - old.compute_units_consumed as i64
        ),
        TxOutcomeCategory::BehaviorChanged => format!(
            "Succeeded on both versions but produced different account state: old -> {}, new -> {}.",
            describe_state(old),
            describe_state(new)
        ),
        TxOutcomeCategory::ErrorChanged => format!(
            "Failed on both versions but with different errors: old -> {}, new -> {}.",
            old.error.as_deref().unwrap_or("unknown error"),
            new.error.as_deref().unwrap_or("unknown error")
        ),
        TxOutcomeCategory::NewFailure => format!(
            "Succeeded on the old program ({}) but failed on the new program: {}.",
            describe_state(old),
            new.error.as_deref().unwrap_or("unknown error")
        ),
        TxOutcomeCategory::NewSuccess => format!(
            "Failed on the old program ({}) but succeeded on the new program ({}).",
            old.error.as_deref().unwrap_or("unknown error"),
            describe_state(new)
        ),
    }
}

fn describe_state(result: &TxExecutionResult) -> String {
    match &result.counter_state {
        Some(state) => format!("value={}", state.value),
        None => "no state".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use preflight_shared::{CounterInstructionSpec, CounterStateSnapshot, SignerRole};

    fn spec(label: &str) -> TransactionSpec {
        TransactionSpec {
            label: label.to_string(),
            description: "test transaction".to_string(),
            instruction: CounterInstructionSpec::Increment { amount: 1 },
            signer: SignerRole::Authority,
        }
    }

    fn ok(label: &str, value: u64, cu: u64) -> TxExecutionResult {
        TxExecutionResult {
            label: label.to_string(),
            success: true,
            error: None,
            logs: vec![],
            compute_units_consumed: cu,
            counter_state: Some(CounterStateSnapshot {
                is_initialized: true,
                authority: "11111111111111111111111111111111".to_string(),
                value,
            }),
        }
    }

    fn failed(label: &str, error: &str, cu: u64) -> TxExecutionResult {
        TxExecutionResult {
            label: label.to_string(),
            success: false,
            error: Some(error.to_string()),
            logs: vec![],
            compute_units_consumed: cu,
            counter_state: None,
        }
    }

    #[test]
    fn identical_runs_score_100_and_are_all_unchanged() {
        let specs = vec![spec("a"), spec("b")];
        let old = vec![ok("a", 10, 100), ok("b", 20, 100)];
        let new = old.clone();

        let report = compare("old.so", "new.so", &specs, &old, &new).unwrap();

        assert_eq!(report.summary.safety_score, 100);
        assert_eq!(report.summary.unchanged, 2);
        assert!(report
            .comparisons
            .iter()
            .all(|c| c.category == TxOutcomeCategory::Unchanged));
    }

    #[test]
    fn success_to_failure_is_a_new_failure_and_docks_the_score() {
        let specs = vec![spec("a")];
        let old = vec![ok("a", 10, 100)];
        let new = vec![failed("a", "Custom(1)", 90)];

        let report = compare("old.so", "new.so", &specs, &old, &new).unwrap();

        assert_eq!(report.comparisons[0].category, TxOutcomeCategory::NewFailure);
        assert_eq!(report.summary.new_failures, 1);
        assert!(report.summary.safety_score < 100);
    }

    #[test]
    fn same_success_different_state_is_behavior_changed() {
        let specs = vec![spec("a")];
        let old = vec![ok("a", 10, 100)];
        let new = vec![ok("a", 999, 100)];

        let report = compare("old.so", "new.so", &specs, &old, &new).unwrap();

        assert_eq!(
            report.comparisons[0].category,
            TxOutcomeCategory::BehaviorChanged
        );
    }

    #[test]
    fn same_success_different_cu_only_is_compute_units_changed() {
        let specs = vec![spec("a")];
        let old = vec![ok("a", 10, 100)];
        let new = vec![ok("a", 10, 150)];

        let report = compare("old.so", "new.so", &specs, &old, &new).unwrap();

        assert_eq!(
            report.comparisons[0].category,
            TxOutcomeCategory::ComputeUnitsChanged
        );
        assert_eq!(report.comparisons[0].compute_units_delta, 50);
    }

    #[test]
    fn both_fail_with_different_errors_is_error_changed() {
        let specs = vec![spec("a")];
        let old = vec![failed("a", "InvalidArgument", 50)];
        let new = vec![failed("a", "Custom(2)", 50)];

        let report = compare("old.so", "new.so", &specs, &old, &new).unwrap();

        assert_eq!(
            report.comparisons[0].category,
            TxOutcomeCategory::ErrorChanged
        );
    }

    #[test]
    fn both_fail_with_the_same_error_is_unchanged() {
        let specs = vec![spec("a")];
        let old = vec![failed("a", "InvalidArgument", 50)];
        let new = vec![failed("a", "InvalidArgument", 50)];

        let report = compare("old.so", "new.so", &specs, &old, &new).unwrap();

        assert_eq!(report.comparisons[0].category, TxOutcomeCategory::Unchanged);
    }

    #[test]
    fn failure_to_success_is_a_new_success() {
        let specs = vec![spec("a")];
        let old = vec![failed("a", "InvalidArgument", 50)];
        let new = vec![ok("a", 1, 60)];

        let report = compare("old.so", "new.so", &specs, &old, &new).unwrap();

        assert_eq!(
            report.comparisons[0].category,
            TxOutcomeCategory::NewSuccess
        );
    }

    #[test]
    fn mismatched_lengths_are_rejected() {
        let specs = vec![spec("a"), spec("b")];
        let old = vec![ok("a", 10, 100)];
        let new = vec![ok("a", 10, 100)];

        assert!(compare("old.so", "new.so", &specs, &old, &new).is_err());
    }
}
