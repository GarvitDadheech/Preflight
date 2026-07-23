use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Directory containing `examples/counter-old` and `examples/counter-new`,
/// resolved relative to this crate's own location so `preflight demo`
/// works regardless of the caller's current directory.
fn examples_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples"))
}

/// Builds one of the bundled example program crates with `cargo
/// build-sbf`, writing its `.so` into `sbf_out_dir`, and returns the
/// path to the resulting file.
pub fn build_example_program(
    crate_name: &str,
    sbf_out_dir: &Path,
    tools_version: &str,
) -> Result<PathBuf> {
    let manifest_path = examples_dir().join(crate_name).join("Cargo.toml");
    if !manifest_path.exists() {
        bail!(
            "expected to find example program manifest at {}",
            manifest_path.display()
        );
    }
    std::fs::create_dir_all(sbf_out_dir)?;

    let status = Command::new("cargo")
        .arg("build-sbf")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--sbf-out-dir")
        .arg(sbf_out_dir)
        .arg("--tools-version")
        .arg(tools_version)
        .status()
        .with_context(|| "failed to invoke `cargo build-sbf` (is the Solana CLI installed?)")?;

    if !status.success() {
        bail!("`cargo build-sbf` failed for {crate_name}");
    }

    let so_path = sbf_out_dir.join(format!("{}.so", crate_name.replace('-', "_")));
    if !so_path.exists() {
        bail!(
            "`cargo build-sbf` reported success but {} was not produced",
            so_path.display()
        );
    }
    Ok(so_path)
}
