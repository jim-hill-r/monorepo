//! `infra`: infrastructure as code for the monorepo (decision 0007).
//!
//! `infra synth` writes `out/<stack>/main.tf.json` for every stack, and
//! `infra synth --check` fails when the committed files differ from a fresh
//! synth. OpenTofu plans and applies those files; this program never calls a
//! vendor API. `infra guard <plan.json>` checks a saved plan's JSON against
//! the plan guards (IAC-001 to IAC-004) before it may be applied.

mod constructs;
mod guard;
mod stacks;
mod synth;
mod tfschema;

use anyhow::Context;
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
    /// Print the `-backend-config` arguments for a stack's state URL on the
    /// state Worker, for `tofu init` in the workflows.
    BackendConfig {
        /// The stack, such as `auth0`.
        stack: String,
        /// The state Worker's origin, such as `https://jimhillr-tofu-state.<subdomain>.workers.dev`.
        origin: String,
    },
    /// Check a plan (`tofu show -json plan.tfplan > plan.json`) against the
    /// plan guards; fails on any finding.
    Guard {
        /// The plan's JSON.
        plan: PathBuf,
        /// The free-tier budgets.
        #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../standards/radar/limits.toml"))]
        limits: PathBuf,
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
        Command::BackendConfig { stack, origin } => {
            for arg in synth::backend_config_args(&stack, &origin)? {
                println!("{arg}");
            }
            Ok(())
        }
        Command::Guard { plan, limits } => {
            let text = std::fs::read_to_string(&plan)
                .with_context(|| format!("reading {}", plan.display()))?;
            let json: serde_json::Value = serde_json::from_str(&text)
                .with_context(|| format!("{} is not plan JSON", plan.display()))?;
            let findings = guard::check(&json, &guard::read_budgets(&limits)?);
            for finding in &findings {
                eprintln!("{finding}");
            }
            if !findings.is_empty() {
                anyhow::bail!(
                    "{} plan guard finding(s) in {}",
                    findings.len(),
                    plan.display()
                );
            }
            println!("guard: ok ({})", plan.display());
            Ok(())
        }
    }
}
