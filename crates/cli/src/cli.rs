use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Preflight: replay a fixed set of transactions against two builds of a
/// Solana program and report what changed.
#[derive(Parser)]
#[command(name = "preflight", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Replay the built-in transaction fixture against two program builds.
    Run {
        /// Path to the old (currently deployed) program's .so file.
        #[arg(long)]
        old: PathBuf,
        /// Path to the new (candidate) program's .so file.
        #[arg(long)]
        new: PathBuf,
        /// Directory to write transactions.json, report.md and
        /// report.json into.
        #[arg(long, default_value = "preflight-out")]
        out: PathBuf,
    },
    /// Build both bundled example programs, then run the same pipeline
    /// as `run`. The quickest way to see Preflight end to end.
    Demo {
        /// Directory to write the built programs and reports into.
        #[arg(long, default_value = "preflight-out")]
        out: PathBuf,
        /// platform-tools version passed to `cargo build-sbf`.
        ///
        /// Defaults to a version newer than the one `solana-install`
        /// selects automatically, because that default toolchain's
        /// bundled rustc is too old to build some of the example
        /// program's current dependencies. See the README for details.
        #[arg(long, default_value = "v1.54")]
        tools_version: String,
    },
}
