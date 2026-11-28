//! The `my_lang` binary. See [`my_lang::cli`] for the commands.

use std::process::ExitCode;

use clap::Parser;
use my_lang::cli::{self, Cli, Outcome};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli::execute(&cli.command) {
        Ok(Outcome::Ran(value)) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Ok(Outcome::Built(_)) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("my_lang: {e}");
            ExitCode::FAILURE
        }
    }
}
