//! `royalty check`: read one royalty.toml and print its findings. In a
//! consumer's Cargo project it also runs the scan and fails when what is
//! owed breaks the consumer's budget or policy: the consumer's cost
//! ceiling, enforced on the pull request that would break it (ROYL-004).
//! It never touches the running program (ROYL-003).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use royalty_model::{Binding, Consumer, Level, OnExceed, Owed, parse};

use crate::owed::{dollars, work_out};
use crate::period::Month;

/// The file a path names: itself, or `royalty.toml` inside a directory.
pub fn royalty_file(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("royalty.toml")
    } else {
        path.to_path_buf()
    }
}

pub fn run(path: &Path, strict: bool, month: Month) -> Result<ExitCode> {
    let file = royalty_file(path);
    let text =
        fs::read_to_string(&file).with_context(|| format!("can't read {}", file.display()))?;
    let parsed = parse(&text);
    let shown = file.display();
    for finding in &parsed.findings {
        let mut line = format!(
            "{}[{}] {shown}",
            finding.level.as_str(),
            finding.code.as_str()
        );
        if !finding.at.is_empty() {
            line.push_str(&format!(" {}", finding.at));
        }
        line.push_str(&format!(": {}", finding.message));
        if let Some(requirement) = finding.requirement {
            line.push_str(&format!(" ({requirement})"));
        }
        println!("{line}");
    }
    let mut errors = parsed.findings.iter().filter(|f| f.is_error()).count();
    let warnings = parsed
        .findings
        .iter()
        .filter(|f| f.level == Level::Warning)
        .count();

    // The budget gate: only for a consumer's Cargo project.
    let dir = file.parent().unwrap_or(Path::new("."));
    let dir = if dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        dir
    };
    let mut scanned = false;
    if let Some(royalty) = parsed.valid()
        && royalty.consumer.is_some()
        && dir.join("Cargo.toml").is_file()
    {
        let (consumer, _scan, owed) = work_out(dir, month)?;
        let broken = gate(&consumer, &owed);
        for message in &broken {
            println!("error[RTOML-014] {shown}: {message} (ROYL-004)");
        }
        errors += broken.len();
        scanned = true;
        println!(
            "royalty check: {month}: {} owed{}",
            dollars(owed.total_cents),
            owed.budget_cents
                .map(|b| format!(" of a {} budget", dollars(b)))
                .unwrap_or_default()
        );
    }

    let fails = errors > 0 || (strict && warnings > 0);
    if errors == 0 && warnings == 0 {
        let meaning = match parsed.valid() {
            Some(f) if !f.has_terms && f.consumer.is_none() => " (no payment terms)",
            _ if scanned => " (within budget)",
            _ => "",
        };
        println!("royalty check: {shown} ok{meaning}");
    } else {
        println!("royalty check: {shown}: {errors} error(s), {warnings} warning(s)");
    }
    Ok(if fails {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

/// What breaks the consumer's budget or policy, one message per problem,
/// each naming the dependency and the limit.
pub fn gate(consumer: &Consumer, owed: &Owed) -> Vec<String> {
    let mut broken = Vec::new();
    let budget = &consumer.budget;
    for family in &owed.families {
        if family.binding_refused {
            let accepted: Vec<&str> = consumer
                .policy
                .bindings
                .iter()
                .map(|b| b.as_str())
                .collect();
            broken.push(format!(
                "{} asks for payment under binding \"{}\", which [consumer.policy] bindings ({}) does not accept",
                family.family,
                family.binding.as_str(),
                accepted.join(", ")
            ));
        }
        if family.over_budget
            && let Some(limit) = budget.per_producer_period_cents
            && family.cents > limit
        {
            broken.push(format!(
                "{} owes {}, over per_producer_period_cents ({}); binding \"{}\" is never scaled",
                family.family,
                dollars(family.cents),
                dollars(limit),
                family.binding.as_str()
            ));
        }
    }
    let total: u64 = owed.families.iter().map(|f| f.cents).sum();
    if let Some(limit) = owed.budget_cents
        && total > limit
    {
        let fixed: Vec<String> = owed
            .families
            .iter()
            .filter(|f| f.cents > 0)
            .filter(|f| budget.on_exceed == OnExceed::Fail || f.binding != Binding::Voluntary)
            .map(|f| format!("{} {}", f.family, dollars(f.cents)))
            .collect();
        let why = match budget.on_exceed {
            OnExceed::Fail => "on_exceed is \"fail\"".to_string(),
            OnExceed::Scale => "agreement and license amounts are never scaled".to_string(),
        };
        broken.push(format!(
            "the total owed, {}, is over the budget ({}) and {why}: {}",
            dollars(total),
            dollars(limit),
            fixed.join(", ")
        ));
    }
    broken
}
