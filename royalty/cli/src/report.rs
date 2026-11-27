//! `royalty report`: the scan and the calculation, written as
//! `royalty-report.json` (for tools) and `royalty-report.md` (for people).
//!
//! Deterministic (ROYL-006): families sorted by name, no timestamps but the
//! period, the same bytes for the same inputs. Private (ROYL-005): counts,
//! amounts and package names only; no file paths, no source, no names or
//! emails of the consumer's people. Payees are the producers' own public
//! names from their files.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};
use royalty_model::{Consumer, FamilyOwed, Method, Mode, Owed, RoyaltyFile, Status, TierChoice};
use serde::Serialize;

use crate::owed::{dollars, tier_label, work_out};
use crate::period::Month;
use crate::scan::{Scan, ScannedFamily};

/// The JSON format's version. A change that breaks readers bumps it.
pub const FORMAT: u32 = 1;

#[derive(Debug, Serialize)]
pub struct Report {
    pub format: u32,
    pub period: String,
    pub consumer: ConsumerPart,
    pub budget_cents: Option<u64>,
    pub total_cents: u64,
    pub over_budget: bool,
    pub families: Vec<FamilyPart>,
    pub no_terms: Vec<String>,
    pub notices: Vec<NoticePart>,
}

#[derive(Debug, Serialize)]
pub struct ConsumerPart {
    pub id: Option<String>,
    pub entity: String,
    pub class: String,
    pub stage: String,
}

#[derive(Debug, Serialize)]
pub struct FamilyPart {
    pub family: String,
    pub version: String,
    pub packages: Vec<String>,
    pub dependency: String,
    pub binding: String,
    /// The mode used: the producer's, if the consumer takes part in it.
    pub mode: String,
    pub tier: String,
    pub status: String,
    pub lines: Vec<LinePart>,
    pub before_caps_cents: u64,
    pub after_producer_caps_cents: u64,
    pub cents: u64,
    pub set_by: Option<String>,
    pub steps: Vec<StepPart>,
    pub over_budget: bool,
    pub binding_refused: bool,
    pub notes: Vec<String>,
    pub income_ceiling_cents: Option<u64>,
    pub payees: Vec<PayeePart>,
}

#[derive(Debug, Serialize)]
pub struct LinePart {
    pub unit: String,
    /// `null` when not measured.
    pub units: Option<u64>,
    pub declared: bool,
    pub charged: u64,
    pub millionths_of_a_cent: String,
}

#[derive(Debug, Serialize)]
pub struct StepPart {
    pub limit: String,
    pub cents: u64,
}

#[derive(Debug, Serialize)]
pub struct PayeePart {
    pub name: String,
    pub share: u64,
    pub cents: u64,
    pub github: Option<String>,
    pub methods: Vec<MethodPart>,
}

#[derive(Debug, Serialize)]
pub struct MethodPart {
    pub kind: String,
    /// The account, contact, URL or address.
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct NoticePart {
    pub package: String,
    pub message: String,
}

pub fn build(month: Month, consumer: &Consumer, scan: &Scan, owed: &Owed) -> Report {
    let families = owed
        .families
        .iter()
        .filter_map(|family| {
            let scanned = scan.families.iter().find(|s| s.family == family.family)?;
            Some(family_part(consumer, scanned, family))
        })
        .collect();
    Report {
        format: FORMAT,
        period: month.to_string(),
        consumer: ConsumerPart {
            id: consumer.id.clone(),
            entity: consumer.entity.as_str().into(),
            class: consumer.class.as_str().into(),
            stage: consumer.stage.as_str().into(),
        },
        budget_cents: owed.budget_cents,
        total_cents: owed.total_cents,
        over_budget: owed.over_budget,
        families,
        no_terms: scan.no_terms.clone(),
        notices: scan
            .notices
            .iter()
            .map(|n| NoticePart {
                package: n.package.clone(),
                message: n.message.clone(),
            })
            .collect(),
    }
}

fn status_word(status: Status) -> &'static str {
    match status {
        Status::Owed => "owed",
        Status::Free => "free",
        Status::NotBillable => "not-billable",
        Status::HeldBack => "held-back",
        Status::NotDue => "not-due",
    }
}

fn family_part(consumer: &Consumer, scanned: &ScannedFamily, owed: &FamilyOwed) -> FamilyPart {
    let file: &RoyaltyFile = &scanned.file;
    let mode = if consumer.policy.modes.contains(&file.terms.mode) {
        file.terms.mode
    } else {
        Mode::Honor
    };
    FamilyPart {
        family: owed.family.clone(),
        version: scanned.version.clone(),
        packages: scanned.packages.clone(),
        dependency: owed.dependency.as_str().into(),
        binding: owed.binding.as_str().into(),
        mode: mode.as_str().into(),
        tier: match &owed.priced.tier {
            TierChoice::Tier { id, .. } => id.clone(),
            TierChoice::NonProduction => "non-production".into(),
            TierChoice::NoMatch => "none".into(),
        },
        status: status_word(owed.status).into(),
        lines: owed
            .priced
            .lines
            .iter()
            .map(|l| LinePart {
                unit: l.unit.as_str(),
                units: l.units,
                declared: l.declared,
                charged: l.charged,
                millionths_of_a_cent: l.millionths.to_string(),
            })
            .collect(),
        before_caps_cents: owed.before_caps(),
        after_producer_caps_cents: owed.after_producer_caps(),
        cents: owed.cents,
        set_by: owed.set_by().map(|l| l.as_str().into()),
        steps: owed
            .steps
            .iter()
            .map(|s| StepPart {
                limit: s.limit.as_str().into(),
                cents: s.cents,
            })
            .collect(),
        over_budget: owed.over_budget,
        binding_refused: owed.binding_refused,
        notes: {
            let mut notes = owed.notes.clone();
            if file.terms.trial_days > 0 && owed.before_caps() > 0 {
                notes.push(format!(
                    "the producer offers a {}-day trial from the first production use, which this report doesn't apply yet (it needs the payments ledger); owe nothing if the trial hasn't ended",
                    file.terms.trial_days
                ));
            }
            notes
        },
        income_ceiling_cents: file.income.as_ref().and_then(|i| i.ceiling_cents),
        payees: payees(file, owed.cents),
    }
}

/// Each payee's share of `cents`: `cents * share / total`, rounded down,
/// with the cents left over going one each to the payees in file order.
pub fn split(cents: u64, shares: &[u64]) -> Vec<u64> {
    let total: u64 = shares.iter().sum();
    if total == 0 {
        return vec![0; shares.len()];
    }
    let mut parts: Vec<u64> = shares
        .iter()
        .map(|s| u64::try_from(u128::from(cents) * u128::from(*s) / u128::from(total)).unwrap_or(0))
        .collect();
    let mut left = cents - parts.iter().sum::<u64>();
    for part in parts.iter_mut() {
        if left == 0 {
            break;
        }
        *part += 1;
        left -= 1;
    }
    parts
}

fn payees(file: &RoyaltyFile, cents: u64) -> Vec<PayeePart> {
    let shares: Vec<u64> = file.payees.iter().map(|p| p.share).collect();
    file.payees
        .iter()
        .zip(split(cents, &shares))
        .map(|(payee, cents)| PayeePart {
            name: payee.name.clone(),
            share: payee.share,
            cents,
            github: payee.github.clone(),
            methods: payee
                .methods
                .iter()
                .map(|m| {
                    let (kind, to) = match m {
                        Method::GithubSponsors { account } => (
                            "github-sponsors",
                            format!("https://github.com/sponsors/{account}"),
                        ),
                        Method::Invoice { contact } => ("invoice", contact.clone()),
                        Method::Link { url } => ("link", url.clone()),
                        Method::X402 { network, address } => {
                            ("x402", format!("{network}:{address}"))
                        }
                    };
                    MethodPart {
                        kind: kind.into(),
                        to,
                    }
                })
                .collect(),
        })
        .collect()
}

fn units_text(family: &FamilyPart) -> String {
    let parts: Vec<String> = family
        .lines
        .iter()
        .map(|l| match l.units {
            Some(n) => format!("{n} {}", l.unit),
            None => format!("{}: not measured", l.unit),
        })
        .collect();
    parts.join(", ")
}

/// The report for people.
pub fn markdown(report: &Report, owed: &Owed) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Royalty report, {}\n\n", report.period));
    let id = report
        .consumer
        .id
        .as_deref()
        .map(|id| format!("`{id}`, "))
        .unwrap_or_default();
    out.push_str(&format!(
        "Consumer: {id}{}, {}, stage {}. Worked out by `royalty report` from the \
         dependency graph, git history and the consumer's `royalty.toml`; the \
         rules are in the royalty.toml reference.\n\n",
        report.consumer.entity, report.consumer.class, report.consumer.stage
    ));
    let budget = report
        .budget_cents
        .map(|b| format!(" of a {} budget", dollars(b)))
        .unwrap_or_default();
    out.push_str(&format!(
        "**Total owed: {}**{budget}.",
        dollars(report.total_cents)
    ));
    if report.over_budget {
        out.push_str(" **Over budget**: `royalty check` fails until a dependency is removed, a price is agreed or the budget is raised.");
    }
    out.push_str("\n\n## Families\n\n");
    if report.families.is_empty() {
        out.push_str(&format!(
            "No dependency has payment terms: {} package(s) scanned, all listed below.\n",
            report.no_terms.len()
        ));
    } else {
        out.push_str("| Family | Version | Dependency | Tier | Units | Before caps | After producer caps | Owed | Why |\n");
        out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    }
    for (family, owed_family) in report.families.iter().zip(&owed.families) {
        let mut why: Vec<String> = Vec::new();
        if let Some(set_by) = &family.set_by {
            why.push(format!("Set by {set_by}."));
        }
        if family.binding != "voluntary" && family.cents > 0 {
            why.push(format!("Binding `{}`: never scaled.", family.binding));
        }
        if family.over_budget {
            why.push("Over budget.".into());
        }
        if family.binding_refused {
            why.push(format!(
                "Binding \"{}\" is not accepted by the consumer's policy.",
                family.binding
            ));
        }
        if owed_family.status == Status::HeldBack {
            why.push("Held back: under the minimum payment.".into());
        }
        why.extend(family.notes.iter().map(|n| format!("Note: {n}.")));
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            family.family,
            family.version,
            family.dependency,
            tier_label(owed_family),
            units_text(family),
            dollars(family.before_caps_cents),
            dollars(family.after_producer_caps_cents),
            dollars(family.cents),
            why.join(" ")
        ));
    }

    let paying: Vec<&FamilyPart> = report.families.iter().filter(|f| f.cents > 0).collect();
    if !paying.is_empty() {
        out.push_str("\n## How to pay\n\n");
        for family in paying {
            out.push_str(&format!(
                "- `{}` ({}, {} mode): {}\n",
                family.family,
                family.binding,
                family.mode,
                dollars(family.cents)
            ));
            for payee in &family.payees {
                let how = payee
                    .methods
                    .first()
                    .map(|m| format!(" by {}: {}", m.kind, m.to))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  - {}: {}{how}\n",
                    payee.name,
                    dollars(payee.cents)
                ));
            }
        }
    }
    if !report.no_terms.is_empty() {
        out.push_str(&format!(
            "\n## No payment terms\n\n{}\n",
            report
                .no_terms
                .iter()
                .map(|p| format!("`{p}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !report.notices.is_empty() {
        out.push_str("\n## Notices\n\n");
        for notice in &report.notices {
            out.push_str(&format!("- `{}`: {}\n", notice.package, notice.message));
        }
    }
    out
}

pub fn run(dir: &Path, month: Month, out: &Path) -> Result<ExitCode> {
    let (consumer, scan, owed) = work_out(dir, month)?;
    let report = build(month, &consumer, &scan, &owed);
    let json = serde_json::to_string_pretty(&report)? + "\n";
    let md = markdown(&report, &owed);
    fs::create_dir_all(out).with_context(|| format!("can't create {}", out.display()))?;
    let json_path = out.join("royalty-report.json");
    let md_path = out.join("royalty-report.md");
    fs::write(&json_path, json).with_context(|| format!("can't write {}", json_path.display()))?;
    fs::write(&md_path, md).with_context(|| format!("can't write {}", md_path.display()))?;
    println!(
        "royalty report: {}: {} owed; wrote {} and {}",
        month,
        dollars(report.total_cents),
        json_path.display(),
        md_path.display()
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::split;

    #[test]
    fn shares_split_to_the_cent() {
        assert_eq!(split(1000, &[70, 30]), vec![700, 300]);
        assert_eq!(split(1001, &[1, 1]), vec![501, 500]);
        assert_eq!(split(100, &[1, 1, 1]), vec![34, 33, 33]);
        assert_eq!(split(0, &[1]), vec![0]);
    }
}
