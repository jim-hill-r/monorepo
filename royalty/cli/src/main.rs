//! The `royalty` command. See `royalty/README.md`.

mod check;
mod owed;
mod scan;

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
    /// Scan a consumer's Cargo dependencies for royalty.toml files and
    /// print what is owed this period. Runs `cargo metadata --locked
    /// --offline`, so it needs a Cargo.lock and the sources on disk.
    Scan {
        /// The consumer's directory (with Cargo.toml, Cargo.lock and
        /// optionally royalty.toml). Defaults to the current directory.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Check { path, strict } => check::run(&path, strict),
        Command::Scan { path } => owed::run(&path),
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("royalty: {error:#}");
            ExitCode::from(2)
        }
    }
}
