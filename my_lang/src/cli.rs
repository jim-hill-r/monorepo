//! The `my_lang` command line: `build` compiles a program to a native
//! executable and `run` runs it straight away with the JIT.
//!
//! ```text
//! my_lang build answer.my            # writes ./answer
//! my_lang build answer.my -o out/a   # writes out/a
//! my_lang run answer.my              # prints main's result
//! my_lang run add.my 3 4             # passes 3 and 4 to main(a, b)
//! ```
//!
//! The binary in `src/main.rs` only parses arguments and reports errors; the
//! work is done here so tests can call it directly.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use inkwell::context::Context;

use crate::aot::{self, AotError, ENTRY_FUNCTION};
use crate::jit::{JitError, JitRunner};

/// Compile and run my_lang programs.
#[derive(Debug, Parser)]
#[command(name = "my_lang", version, about)]
pub struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub command: Command,
}

/// The `my_lang` subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compile a program to a native executable that prints main's result.
    Build {
        /// The my_lang source file.
        file: PathBuf,
        /// Where to write the executable [default: the source file's name
        /// without its extension, next to it].
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compile a program with the JIT, call main and print its result.
    Run {
        /// The my_lang source file.
        file: PathBuf,
        /// Integer arguments for main, one per parameter (at most 8).
        #[arg(allow_negative_numbers = true)]
        args: Vec<i64>,
    },
}

/// Errors the command line reports.
#[derive(Debug)]
pub enum CliError {
    /// The source file could not be read.
    Read {
        /// The file that was asked for.
        path: PathBuf,
        /// Why reading it failed.
        error: std::io::Error,
    },
    /// No output path was given and none can be derived from the source path.
    NoOutputName(PathBuf),
    /// `build` failed.
    Build(AotError),
    /// `run` failed.
    Run(JitError),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Read { path, error } => {
                write!(f, "could not read {}: {error}", path.display())
            }
            CliError::NoOutputName(path) => write!(
                f,
                "can't name an executable after {}; pass -o <output>",
                path.display()
            ),
            CliError::Build(e) => write!(f, "{e}"),
            CliError::Run(JitError::FunctionNotFound(name)) if name == ENTRY_FUNCTION => {
                write!(f, "no entry function '{name}' in the program")
            }
            CliError::Run(JitError::ArgumentCountMismatch { expected, given }) => write!(
                f,
                "'{ENTRY_FUNCTION}' takes {expected} arguments, but {given} were given"
            ),
            CliError::Run(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CliError {}

/// What a successful command produced.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// `build` wrote this executable.
    Built(PathBuf),
    /// `run` returned this value from `main`.
    Ran(i64),
}

/// Carry out `command`.
///
/// # Errors
///
/// Returns a [`CliError`] if the source can't be read, or compiling, linking
/// or running it fails.
pub fn execute(command: &Command) -> Result<Outcome, CliError> {
    match command {
        Command::Build { file, output } => {
            let output = match output {
                Some(path) => path.clone(),
                None => default_output(file)?,
            };
            let source = read_source(file)?;
            aot::build_executable(&source, &output).map_err(CliError::Build)?;
            Ok(Outcome::Built(output))
        }
        Command::Run { file, args } => {
            let source = read_source(file)?;
            run_source(&source, args).map(Outcome::Ran)
        }
    }
}

/// JIT-compile `source` and call its `main` with `args`.
///
/// # Errors
///
/// Returns [`CliError::Run`] if compiling fails, there is no `main`, or
/// `args` doesn't match `main`'s parameters.
pub fn run_source(source: &str, args: &[i64]) -> Result<i64, CliError> {
    let context = Context::create();
    let runner = JitRunner::new(&context, source).map_err(CliError::Run)?;
    runner.call(ENTRY_FUNCTION, args).map_err(CliError::Run)
}

/// The executable `build` writes when no `-o` is given: the source path
/// without its extension, such as `answer` for `answer.my`.
///
/// # Errors
///
/// Returns [`CliError::NoOutputName`] if that would be the source file
/// itself (a source file with no extension) or `file` has no file name.
pub fn default_output(file: &Path) -> Result<PathBuf, CliError> {
    let output = file.with_extension("");
    if output.file_name().is_none() || output == file {
        return Err(CliError::NoOutputName(file.to_path_buf()));
    }
    Ok(output)
}

fn read_source(path: &Path) -> Result<String, CliError> {
    std::fs::read_to_string(path).map_err(|error| CliError::Read {
        path: path.to_path_buf(),
        error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_output_drops_the_extension() {
        assert_eq!(
            default_output(Path::new("dir/answer.my")).ok(),
            Some(PathBuf::from("dir/answer"))
        );
    }

    #[test]
    fn default_output_refuses_to_overwrite_the_source() {
        assert!(matches!(
            default_output(Path::new("answer")),
            Err(CliError::NoOutputName(_))
        ));
    }

    #[test]
    fn run_source_calls_main_with_arguments() {
        let source = "function main(a, b) { return a - b }";
        assert_eq!(run_source(source, &[10, 3]).ok(), Some(7));
    }

    #[test]
    fn arguments_parse_including_negatives() {
        let cli = Cli::try_parse_from(["my_lang", "run", "p.my", "-5", "2"]);
        assert!(matches!(
            cli.map(|c| c.command),
            Ok(Command::Run { args, .. }) if args == vec![-5, 2]
        ));
    }
}
