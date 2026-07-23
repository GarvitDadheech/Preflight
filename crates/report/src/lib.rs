//! Renders a [`Report`] into the two artifacts `preflight run` produces:
//! a machine-readable `report.json` and a human-readable `report.md`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use preflight_shared::{Report, Result};

/// Qualitative read on a safety score, used as the report's headline.
pub fn verdict(safety_score: u8) -> &'static str {
    match safety_score {
        90..=100 => "Safe to deploy - no meaningful behavioral differences detected.",
        70..=89 => "Low risk - minor differences detected. Review recommended before deploying.",
        40..=69 => "Elevated risk - behavioral changes detected. Review carefully before deploying.",
        _ => "High risk - regressions detected. Do not deploy without further review.",
    }
}

/// Serializes a report to pretty-printed JSON.
pub fn render_json(report: &Report) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

/// Renders a report as a Markdown document.
pub fn render_markdown(report: &Report) -> String {
    let mut out = String::new();
    let summary = &report.summary;

    writeln!(out, "# Preflight Regression Report").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "- **Old program:** `{}`", report.old_program).unwrap();
    writeln!(out, "- **New program:** `{}`", report.new_program).unwrap();
    writeln!(out, "- **Transactions replayed:** {}", summary.total).unwrap();
    writeln!(out).unwrap();

    writeln!(out, "## Safety score: {}/100", summary.safety_score).unwrap();
    writeln!(out).unwrap();
    writeln!(out, "{}", verdict(summary.safety_score)).unwrap();
    writeln!(out).unwrap();

    writeln!(out, "## Summary").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| Outcome | Count |").unwrap();
    writeln!(out, "|---|---|").unwrap();
    writeln!(out, "| Unchanged | {} |", summary.unchanged).unwrap();
    writeln!(out, "| Compute units changed | {} |", summary.compute_units_changed).unwrap();
    writeln!(out, "| Behavior changed | {} |", summary.behavior_changed).unwrap();
    writeln!(out, "| Error changed | {} |", summary.error_changed).unwrap();
    writeln!(out, "| New failures (regressions) | {} |", summary.new_failures).unwrap();
    writeln!(out, "| New successes | {} |", summary.new_successes).unwrap();
    writeln!(out).unwrap();

    let regressions: Vec<_> = report
        .comparisons
        .iter()
        .filter(|c| c.category.is_regression())
        .collect();

    if regressions.is_empty() {
        writeln!(out, "No behavioral regressions were detected among the replayed transactions.").unwrap();
    } else {
        writeln!(
            out,
            "## Behavioral differences ({} of {})",
            regressions.len(),
            summary.total
        )
        .unwrap();
        writeln!(out).unwrap();
        for comparison in &regressions {
            writeln!(
                out,
                "### `{}` - {}",
                comparison.label,
                comparison.category.label()
            )
            .unwrap();
            writeln!(out).unwrap();
            writeln!(out, "{}", comparison.description).unwrap();
            writeln!(out).unwrap();
            writeln!(out, "{}", comparison.notes).unwrap();
            writeln!(out).unwrap();
        }
    }

    writeln!(out, "## All transactions").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| # | Label | Outcome | Compute units (old -> new) | Notes |").unwrap();
    writeln!(out, "|---|---|---|---|---|").unwrap();
    for (i, comparison) in report.comparisons.iter().enumerate() {
        writeln!(
            out,
            "| {} | `{}` | {} | {} -> {} ({:+}) | {} |",
            i + 1,
            comparison.label,
            comparison.category.label(),
            comparison.old.compute_units_consumed,
            comparison.new.compute_units_consumed,
            comparison.compute_units_delta,
            comparison.notes.replace('|', "\\|"),
        )
        .unwrap();
    }
    writeln!(out).unwrap();

    writeln!(out, "## About this report").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "Preflight is a proof of concept. It replays a fixed set of {} example \
         transactions against both program builds inside an in-process local Solana \
         VM ([litesvm](https://github.com/LiteSVM/litesvm)) and diffs the results. \
         It is not a substitute for a full audit, and does not replay real historical \
         transactions from any live network.",
        summary.total
    )
    .unwrap();

    out
}

/// Writes both `report.json` and `report.md` into `dir`, returning their
/// paths.
pub fn write_reports(report: &Report, dir: &Path) -> Result<(PathBuf, PathBuf)> {
    std::fs::create_dir_all(dir)?;

    let json_path = dir.join("report.json");
    std::fs::write(&json_path, render_json(report)?)?;

    let md_path = dir.join("report.md");
    std::fs::write(&md_path, render_markdown(report))?;

    Ok((json_path, md_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use preflight_shared::{CounterStateSnapshot, RunSummary, TxComparison, TxExecutionResult};

    #[test]
    fn verdict_thresholds() {
        assert!(verdict(100).starts_with("Safe"));
        assert!(verdict(90).starts_with("Safe"));
        assert!(verdict(89).starts_with("Low risk"));
        assert!(verdict(70).starts_with("Low risk"));
        assert!(verdict(69).starts_with("Elevated risk"));
        assert!(verdict(40).starts_with("Elevated risk"));
        assert!(verdict(39).starts_with("High risk"));
        assert!(verdict(0).starts_with("High risk"));
    }

    fn sample_report() -> Report {
        Report {
            old_program: "old.so".to_string(),
            new_program: "new.so".to_string(),
            summary: RunSummary {
                total: 1,
                new_failures: 1,
                safety_score: 20,
                ..Default::default()
            },
            comparisons: vec![TxComparison {
                label: "increment_past_cap".to_string(),
                description: "Increment past the cap.".to_string(),
                category: preflight_shared::TxOutcomeCategory::NewFailure,
                old: TxExecutionResult {
                    label: "increment_past_cap".to_string(),
                    success: true,
                    error: None,
                    logs: vec![],
                    compute_units_consumed: 100,
                    counter_state: Some(CounterStateSnapshot {
                        is_initialized: true,
                        authority: "auth".to_string(),
                        value: 2000,
                    }),
                },
                new: TxExecutionResult {
                    label: "increment_past_cap".to_string(),
                    success: false,
                    error: Some("ValueExceedsMax".to_string()),
                    logs: vec![],
                    compute_units_consumed: 80,
                    counter_state: None,
                },
                compute_units_delta: -20,
                notes: "Succeeded on the old program but failed on the new program.".to_string(),
            }],
        }
    }

    #[test]
    fn markdown_includes_the_regression_and_the_score() {
        let markdown = render_markdown(&sample_report());
        assert!(markdown.contains("Safety score: 20/100"));
        assert!(markdown.contains("increment_past_cap"));
        assert!(markdown.contains("new failure"));
    }

    #[test]
    fn json_round_trips() {
        let report = sample_report();
        let json = render_json(&report).unwrap();
        let parsed: Report = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.summary.safety_score, report.summary.safety_score);
        assert_eq!(parsed.comparisons.len(), 1);
    }
}
