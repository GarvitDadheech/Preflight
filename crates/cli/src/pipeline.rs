use std::path::Path;

use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use preflight_shared::{Fixture, Report, TxExecutionResult};

/// Runs the full record -> replay -> compare -> report pipeline against
/// two already-built program binaries, printing progress and a summary
/// to the terminal along the way.
pub fn run_pipeline(old_so: &Path, new_so: &Path, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create output directory {}", out_dir.display()))?;

    println!("{}", style("Preflight").bold());
    println!("  old program: {}", old_so.display());
    println!("  new program: {}", new_so.display());
    println!();

    let fixture = preflight_replay::generate_fixture();
    let fixture_path = out_dir.join("transactions.json");
    fixture
        .save(&fixture_path)
        .context("failed to persist transaction fixture")?;
    println!(
        "Recorded {} example transactions -> {}",
        fixture.transactions.len(),
        fixture_path.display()
    );
    println!();

    let old_results = replay_with_progress("Replaying against OLD program", old_so, &fixture)?;
    let new_results = replay_with_progress("Replaying against NEW program", new_so, &fixture)?;

    let report = preflight_comparator::compare(
        &old_so.display().to_string(),
        &new_so.display().to_string(),
        &fixture.transactions,
        &old_results,
        &new_results,
    )
    .context("failed to compare replay results")?;

    let (json_path, md_path) = preflight_report::write_reports(&report, out_dir)
        .context("failed to write report files")?;

    print_summary(&report);
    println!();
    println!("Full report written to:");
    println!("  {}", md_path.display());
    println!("  {}", json_path.display());

    Ok(())
}

fn replay_with_progress(
    label: &str,
    program: &Path,
    fixture: &Fixture,
) -> Result<Vec<TxExecutionResult>> {
    println!("{}", style(label).bold());
    let bar = ProgressBar::new(fixture.transactions.len() as u64);
    bar.set_style(
        ProgressStyle::with_template("  [{bar:30}] {pos}/{len}  {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    let results = preflight_replay::replay(program, fixture, |spec, result| {
        bar.set_message(format!(
            "{} ({})",
            spec.label,
            if result.success { "ok" } else { "failed" }
        ));
        bar.inc(1);
    })
    .with_context(|| format!("replay failed against {}", program.display()))?;

    bar.finish_and_clear();
    println!();
    Ok(results)
}

fn print_summary(report: &Report) {
    let s = &report.summary;
    println!("{}", style("Summary").bold());
    println!("  total transactions:     {}", s.total);
    println!("  unchanged:              {}", s.unchanged);
    println!("  compute units changed:  {}", s.compute_units_changed);
    println!(
        "  behavior changed:       {}",
        style(s.behavior_changed).yellow()
    );
    println!(
        "  error changed:          {}",
        style(s.error_changed).yellow()
    );
    println!("  new failures:           {}", style(s.new_failures).red());
    println!(
        "  new successes:          {}",
        style(s.new_successes).red()
    );
    println!();

    let score = style(s.safety_score).bold();
    let score = match s.safety_score {
        90..=100 => score.green(),
        70..=89 => score.yellow(),
        _ => score.red(),
    };
    println!("Safety score: {score}/100");
    println!("{}", preflight_report::verdict(s.safety_score));
}
