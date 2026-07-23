mod build;
mod cli;
mod pipeline;

use anyhow::Result;
use clap::Parser;
use console::style;

use cli::{Cli, Command};

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {err:#}", style("error:").red().bold());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run { old, new, out } => pipeline::run_pipeline(&old, &new, &out),
        Command::Demo { out, tools_version } => {
            let programs_dir = out.join("programs");

            println!("{}", style("Building examples/counter-old").bold());
            let old_so =
                build::build_example_program("counter-old", &programs_dir, &tools_version)?;
            println!("  -> {}", old_so.display());
            println!();

            println!("{}", style("Building examples/counter-new").bold());
            let new_so =
                build::build_example_program("counter-new", &programs_dir, &tools_version)?;
            println!("  -> {}", new_so.display());
            println!();

            pipeline::run_pipeline(&old_so, &new_so, &out)
        }
    }
}
