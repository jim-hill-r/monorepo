//! `royalty scan`: scan a consumer's dependencies and print what is owed.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};
use royalty_model::{Consumer, FamilyUse, Owed, Status, TierChoice, Unit, owe, parse};

use crate::period::Month;
use crate::scan::{Scan, scan};
use crate::seats;

/// The consumer's side from `dir/royalty.toml`, or the defaults when there
/// is none (a person in development: nothing is owed).
pub fn load_consumer(dir: &Path) -> Result<Consumer> {
    let path = dir.join("royalty.toml");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(Consumer::default());
    };
    let parsed = parse(&text);
    let file = parsed.valid().with_context(|| {
        format!(
            "{} has errors; run royalty check on it first",
            path.display()
        )
    })?;
    Ok(file.consumer_or_default())
}

/// Scans `dir` and works out what is owed in `month`.
pub fn work_out(dir: &Path, month: Month) -> Result<(Consumer, Scan, Owed)> {
    let consumer = load_consumer(dir)?;
    let mut scan = scan(dir, &consumer.deployments)?;
    for family in &mut scan.families {
        let period = family.file.terms.period;
        family.usage.due = month.is_due(period);
        if family.usage.due
            && let Some(seats) = seats::count(dir, month.period_start(period), month.next())
        {
            family.usage.measured.insert(Unit::Seat, seats);
        }
    }
    let uses: Vec<FamilyUse<'_>> = scan
        .families
        .iter()
        .map(|f| FamilyUse {
            family: f.family.clone(),
            producer: &f.file,
            usage: f.usage.clone(),
        })
        .collect();
    let owed = owe(&consumer, &uses);
    Ok((consumer, scan, owed))
}

/// `$12.34`.
pub fn dollars(cents: u64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

pub fn tier_label(owed: &royalty_model::FamilyOwed) -> String {
    match (&owed.status, &owed.priced.tier) {
        (Status::NotBillable, _) => "not billable (transitive)".into(),
        (Status::NotDue, _) => "not due this month".into(),
        (_, TierChoice::NonProduction) => "non-production (free)".into(),
        (_, TierChoice::NoMatch) => "no tier matched (free)".into(),
        (_, TierChoice::Tier { id, .. }) => id.clone(),
    }
}

pub fn run(dir: &Path, month: Month) -> Result<ExitCode> {
    let (consumer, scan, owed) = work_out(dir, month)?;
    println!(
        "royalty scan: {} ({}, {}), {month}",
        dir.display(),
        consumer.entity.as_str(),
        consumer.stage.as_str()
    );
    for family in &owed.families {
        let scanned = scan.families.iter().find(|f| f.family == family.family);
        let version = scanned.map(|f| f.version.as_str()).unwrap_or("");
        let mut line = format!(
            "  {} {} ({}): {}, {}",
            family.family,
            version,
            family.dependency.as_str(),
            tier_label(family),
            dollars(family.cents)
        );
        if family.cents != family.before_caps() {
            line.push_str(&format!(" (priced at {})", dollars(family.before_caps())));
        }
        if let Some(limit) = family.set_by() {
            line.push_str(&format!(", set by {}", limit.as_str()));
        }
        if family.status == Status::HeldBack {
            line.push_str(", held back: under the minimum payment");
        }
        if family.over_budget {
            line.push_str(", OVER BUDGET");
        }
        if family.binding_refused {
            line.push_str(&format!(
                ", binding \"{}\" not accepted by [consumer.policy]",
                family.binding.as_str()
            ));
        }
        println!("{line}");
        if let Some(f) = scanned.filter(|f| f.packages.len() > 1) {
            println!("    packages: {}", f.packages.join(", "));
        }
        for line in family.priced.lines.iter().filter(|l| l.units.is_none()) {
            println!("    {}: not measured", line.unit.as_str());
        }
        for note in &family.notes {
            println!("    note: {note}");
        }
    }
    if !scan.no_terms.is_empty() {
        println!("  no terms: {}", scan.no_terms.join(", "));
    }
    for notice in &scan.notices {
        println!("  notice: {}: {}", notice.package, notice.message);
    }
    let budget = owed
        .budget_cents
        .map(|b| format!(" of a {} budget", dollars(b)))
        .unwrap_or_default();
    println!("total: {}{budget}", dollars(owed.total_cents));
    Ok(ExitCode::SUCCESS)
}
