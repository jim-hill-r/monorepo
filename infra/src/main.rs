//! `infra`: infrastructure as code for the monorepo (decision 0007).
//!
//! `infra synth` writes `out/<stack>/main.tf.json` for every stack, and
//! `infra synth --check` fails when the committed files differ from a fresh
//! synth. OpenTofu plans and applies those files; this program never calls a
//! vendor API.

mod constructs;
mod stacks;
mod synth;
mod tfschema;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Infrastructure as code: synthesize Terraform JSON for OpenTofu")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Write out/<stack>/main.tf.json for every stack.
    Synth {
        /// Fail if the committed files differ from a fresh synth, instead of
        /// writing them.
        #[arg(long)]
        check: bool,
        /// The output directory.
        #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/out"))]
        out: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Synth { check, out } => {
            let stacks = stacks::all()?;
            if check {
                let stale = synth::stale(&stacks, &out)?;
                if !stale.is_empty() {
                    for path in &stale {
                        eprintln!("out of date: {}", path.display());
                    }
                    anyhow::bail!("run `cargo run -- synth` in infra/ and commit the result");
                }
            } else {
                synth::write(&stacks, &out)?;
                for stack in &stacks {
                    println!("wrote {}", stack.path(&out).display());
                }
            }
            Ok(())
        }
    }
}
