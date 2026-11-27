//! The `royalty` command. See `royalty/README.md`.

mod check;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "royalty",
    version,
    about = "Check royalty.toml files and work out what a consumer owes. Offline: no network calls."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check a royalty.toml (RTOML-001 onwards in docs/royalty-toml.md).
    /// Exits 1 on an error, or on a warning with --strict.
    Check {
        /// A royalty.toml, or a directory that holds one. Defaults to the
        /// current directory.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Treat warnings as errors.
        #[arg(long)]
        strict: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Check { path, strict } => check::run(&path, strict),
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("royalty: {error:#}");
            ExitCode::from(2)
        }
    }
}
