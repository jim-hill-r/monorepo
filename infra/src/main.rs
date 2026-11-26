//! `infra`: infrastructure as code for the monorepo (decision 0007).
//!
//! `infra synth` writes `out/<stack>/main.tf.json` for every stack, and
//! `infra synth --check` fails when the committed files differ from a fresh
//! synth. OpenTofu plans and applies those files; this program never calls a
//! vendor API. `infra guard <plan.json>` checks a saved plan's JSON against
//! the plan guards (IAC-001 to IAC-005) before it may be applied, and
//! reports whether it changes a credential (decision 0010).

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
    /// Check a plan (`tofu show -json plan.tfplan > plan.json`) against the
    /// plan guards; fails on any finding.
    Guard {
        /// The plan's JSON.
        plan: PathBuf,
        /// The free-tier budgets.
        #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../standards/radar/limits.toml"))]
        limits: PathBuf,
        /// Write the plan's credential changes here as JSON,
        /// `{"credential_change": bool, "changes": [...]}`, so the workflow
        /// applies it in `infra-credentials` when true (decision 0010).
        #[arg(long)]
        report: Option<PathBuf>,
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
        Command::Guard {
            plan,
            limits,
            report,
        } => {
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
            let changes = guard::credential_changes(&json);
            for change in &changes {
                println!(
                    "credential change: {} ({}): {}",
                    change.address,
                    change.actions.join(", "),
                    change.attributes.join(", ")
                );
            }
            if let Some(report) = report {
                let body = serde_json::json!({
                    "credential_change": !changes.is_empty(),
                    "changes": changes,
                });
                std::fs::write(&report, serde_json::to_string_pretty(&body)? + "\n")
                    .with_context(|| format!("writing {}", report.display()))?;
            }
            println!(
                "guard: ok ({}){}",
                plan.display(),
                if changes.is_empty() {
                    ""
                } else {
                    "; credential changes need Jim's second approval in infra-credentials"
                }
            );
            Ok(())
        }
    }
}
