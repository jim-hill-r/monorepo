//! The rules. Per-project rules (COST-001 to COST-004, COST-008) look at one
//! file; account-wide rules (COST-005, COST-006) need every project's file.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::{Code, Cost, CostFile, Every, Finding, LimitsFile, Month, Plan, Radar, Scope};

/// Where each project keeps its cost analysis, relative to the project.
pub const COST_FILE: &str = "docs/cost.toml";

/// COST-008 warns when `reviewed` is older than this.
pub const REVIEW_MAX_DAYS: i64 = 180;

const LIMITS: &str = "standards/radar/limits.toml";
const RADAR: &str = "standards/radar/radar.json";

/// What a `decision` link points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Missing,
    /// The file exists but its `Status:` line isn't `Accepted`.
    NotAccepted,
    Accepted,
}

/// A project whose `docs/cost.toml` parsed.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    /// The top-level directory, such as `cookbook`.
    pub name: String,
    pub file: CostFile,
}

/// Checks one project's `docs/cost.toml`. `text` is `None` when the file
/// doesn't exist. Returns the findings and, when the file parsed, the
/// project for the account-wide rules.
pub fn check_project(
    name: &str,
    text: Option<&str>,
    radar: &Radar,
    limits: &LimitsFile,
    decision: &dyn Fn(&str) -> Decision,
    today: NaiveDate,
) -> (Vec<Finding>, Option<Project>) {
    let path = format!("{name}/{COST_FILE}");
    let mut findings = Vec::new();
    let mut add = |code, message: String| {
        findings.push(Finding {
            code,
            path: path.clone(),
            message,
        })
    };

    let Some(text) = text else {
        add(
            Code::Cost001,
            format!("{name} has no {COST_FILE}; see standards/docs/cost.md"),
        );
        return (findings, None);
    };
    let file = match crate::parse_cost_file(text) {
        Ok(file) => file,
        Err(e) => {
            add(Code::Cost002, e);
            return (findings, None);
        }
    };

    for service in &file.service {
        for key in service.usage.keys() {
            if !limits.limit.iter().any(|l| &l.key == key) {
                add(
                    Code::Cost002,
                    format!(
                        "{}: usage key `{key}` is not a limit in {LIMITS}",
                        service.blip
                    ),
                );
            }
        }
    }

    let check_vendor = |name: &str, what: &str, add: &mut dyn FnMut(Code, String)| match radar
        .vendor(name)
    {
        None => add(
            Code::Cost003,
            format!("{what} `{name}` is not a vendor blip in {RADAR}"),
        ),
        Some(blip) if !blip.is_approved() => add(
            Code::Cost003,
            format!(
                "{what} `{name}` is in {} on the radar; a vendor needs Jim's sign-off in a decision record first",
                blip.ring
            ),
        ),
        Some(_) => {}
    };
    for service in &file.service {
        check_vendor(&service.blip, "service", &mut add);
        if service.plan == Plan::Paid
            && radar
                .vendor(&service.blip)
                .is_some_and(|b| b.budget_cents() == 0)
        {
            add(
                Code::Cost004,
                format!(
                    "{} is on a paid plan, but its budget on the radar is $0",
                    service.blip
                ),
            );
        }
    }
    for cost in &file.cost {
        check_vendor(&cost.vendor, "cost vendor", &mut add);
        if cost.cents == 0 {
            continue;
        }
        let problem = match cost.decision.as_deref() {
            None => Some("has no `decision`".to_string()),
            Some(d) => match decision(d) {
                Decision::Missing => Some(format!("links decision {d}, which does not exist")),
                Decision::NotAccepted => Some(format!("links decision {d}, which is not accepted")),
                Decision::Accepted => None,
            },
        };
        if let Some(problem) = problem {
            add(
                Code::Cost004,
                format!(
                    "cost `{}` ({}) {problem}; any expense needs an accepted decision record",
                    cost.name,
                    money(cost.cents)
                ),
            );
        }
    }

    for service in &file.service {
        for (key, &used) in &service.usage {
            let Some(limit) = limits.limit.iter().find(|l| &l.key == key) else {
                continue;
            };
            if limit.scope != Scope::Project {
                continue;
            }
            if let Some(ceiling) = limit.ceiling()
                && used > ceiling
            {
                add(
                    Code::Cost005,
                    format!(
                        "{key}: expects {used} {}, over the {} of {ceiling}",
                        limit.unit,
                        if limit.budget.is_some() {
                            "budget"
                        } else {
                            "free limit"
                        }
                    ),
                );
            }
        }
    }

    let age = (today - file.reviewed).num_days();
    if age > REVIEW_MAX_DAYS {
        add(
            Code::Cost008,
            format!(
                "reviewed {} ({age} days ago); check it against the code and update `reviewed`",
                file.reviewed
            ),
        );
    }

    (
        findings,
        Some(Project {
            name: name.to_string(),
            file,
        }),
    )
}

/// What a cost adds to `month`, in cents. Yearly costs are spread over
/// twelve months (rounded up); one-off costs count only in their month.
pub fn monthly_cents(cost: &Cost, month: Month) -> u64 {
    if cost.since > month {
        return 0;
    }
    match cost.every {
        Every::Month => cost.cents,
        Every::Year => cost.cents.div_ceil(12),
        Every::Once if cost.since == month => cost.cents,
        Every::Once => 0,
    }
}

/// COST-005 for account-wide limits and COST-006, over every project that
/// has a parsed `docs/cost.toml`.
pub fn check_account(
    projects: &[Project],
    radar: &Radar,
    limits: &LimitsFile,
    month: Month,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for limit in limits.limit.iter().filter(|l| l.scope == Scope::Account) {
        let Some(ceiling) = limit.ceiling() else {
            continue;
        };
        let users: Vec<(&str, u64)> = projects
            .iter()
            .filter_map(|p| {
                let used: u64 = p
                    .file
                    .service
                    .iter()
                    .filter_map(|s| s.usage.get(&limit.key))
                    .sum();
                (used > 0).then_some((p.name.as_str(), used))
            })
            .collect();
        let total: u64 = users.iter().map(|(_, n)| n).sum();
        if total > ceiling {
            let by: Vec<String> = users.iter().map(|(p, n)| format!("{p} {n}")).collect();
            findings.push(Finding {
                code: Code::Cost005,
                path: LIMITS.to_string(),
                message: format!(
                    "{}: projects expect {total} {} in all ({}), over the {} of {ceiling}",
                    limit.key,
                    limit.unit,
                    by.join(", "),
                    if limit.budget.is_some() {
                        "budget"
                    } else {
                        "free limit"
                    }
                ),
            });
        }
    }

    let mut by_vendor: BTreeMap<&str, Vec<(&str, u64)>> = BTreeMap::new();
    for project in projects {
        for cost in &project.file.cost {
            let cents = monthly_cents(cost, month);
            if cents > 0 {
                by_vendor
                    .entry(cost.vendor.as_str())
                    .or_default()
                    .push((project.name.as_str(), cents));
            }
        }
    }
    for (vendor, items) in by_vendor {
        let Some(blip) = radar.vendor(vendor) else {
            continue; // COST-003 already reports it
        };
        let total: u64 = items.iter().map(|(_, c)| c).sum();
        if total > blip.budget_cents() {
            let by: Vec<String> = items
                .iter()
                .map(|(p, c)| format!("{p} {}", money(*c)))
                .collect();
            findings.push(Finding {
                code: Code::Cost006,
                path: RADAR.to_string(),
                message: format!(
                    "{vendor}: projects spend {} in {month} ({}), over its radar budget of {} a month",
                    money(total),
                    by.join(", "),
                    money(blip.budget_cents())
                ),
            });
        }
    }

    findings
}

pub(crate) fn money(cents: u64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_limits, parse_radar};

    const RADAR_JSON: &str = r#"[
      {"name": "Cloudflare Pages", "ring": "Adopt", "kind": "vendor", "budget": 0},
      {"name": "Squarespace", "ring": "Adopt", "kind": "vendor", "budget": 0},
      {"name": "Claude Code", "ring": "Adopt", "kind": "vendor", "budget": 25},
      {"name": "Neon", "ring": "Assess", "kind": "vendor", "budget": 0},
      {"name": "Rust", "ring": "Adopt", "kind": "technology", "budget": 0}
    ]"#;

    const LIMITS_TOML: &str = r#"
[[limit]]
key = "pages_projects"
blip = "Cloudflare Pages"
item = "Pages projects"
free = 100
budget = 2
scope = "account"
unit = "projects"
past = "x"
source = "x"
checked = "2026-11-26"

[[limit]]
key = "pages_files_per_site"
blip = "Cloudflare Pages"
item = "Files per site"
free = 20000
scope = "project"
unit = "files"
past = "x"
source = "x"
checked = "2026-11-26"
"#;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 26).expect("date")
    }

    fn accepted(_: &str) -> Decision {
        Decision::Accepted
    }

    fn check_with(text: Option<&str>, decision: &dyn Fn(&str) -> Decision) -> Vec<Finding> {
        let radar = parse_radar(RADAR_JSON).expect("radar");
        let limits = parse_limits(LIMITS_TOML).expect("limits");
        check_project("cookbook", text, &radar, &limits, decision, today()).0
    }

    fn check(text: &str) -> Vec<Finding> {
        check_with(Some(text), &accepted)
    }

    fn codes(findings: &[Finding]) -> Vec<&'static str> {
        findings.iter().map(|f| f.code.as_str()).collect()
    }

    const HEAD: &str = "reviewed = \"2026-11-26\"\nsummary = \"x\"\n";

    fn pages(usage: &str) -> String {
        format!(
            "{HEAD}[[service]]\nblip = \"Cloudflare Pages\"\nplan = \"free\"\nwhy = \"x\"\n[service.usage]\n{usage}\n"
        )
    }

    fn domain(cents: u64, every: &str, since: &str, decision: &str) -> String {
        format!(
            "[[cost]]\nname = \"domain\"\nvendor = \"Squarespace\"\ncents = {cents}\nevery = \"{every}\"\nsince = \"{since}\"\n{decision}\n"
        )
    }

    #[test]
    fn test_clean_file_has_no_findings() {
        assert!(check(&pages("pages_projects = 1")).is_empty());
        assert!(check(HEAD).is_empty());
    }

    #[test]
    fn test_cost_001_missing_file() {
        let findings = check_with(None, &accepted);
        assert_eq!(codes(&findings), ["COST-001"]);
        assert_eq!(findings[0].path, "cookbook/docs/cost.toml");
    }

    #[test]
    fn test_cost_002_bad_file_and_unknown_usage_key() {
        assert_eq!(codes(&check("summary = 1")), ["COST-002"]);
        let findings = check(&pages("pages_sites = 1"));
        assert_eq!(codes(&findings), ["COST-002"]);
        assert!(findings[0].message.contains("pages_sites"));
    }

    #[test]
    fn test_cost_003_unknown_technology_and_unapproved_vendors() {
        for blip in ["Vercel", "Rust", "Neon"] {
            let text =
                format!("{HEAD}[[service]]\nblip = \"{blip}\"\nplan = \"free\"\nwhy = \"x\"\n");
            assert_eq!(codes(&check(&text)), ["COST-003"], "{blip}");
        }
        let text = format!(
            "{HEAD}[[cost]]\nname = \"x\"\nvendor = \"Neon\"\ncents = 0\nevery = \"month\"\nsince = \"2026-11\"\n"
        );
        assert!(check(&text)[0].message.contains("Assess"));
    }

    #[test]
    fn test_cost_004_paid_plan_on_a_zero_budget() {
        let text = pages("").replace("plan = \"free\"", "plan = \"paid\"");
        assert_eq!(codes(&check(&text)), ["COST-004"]);
        let claude =
            format!("{HEAD}[[service]]\nblip = \"Claude Code\"\nplan = \"paid\"\nwhy = \"x\"\n");
        assert!(check(&claude).is_empty(), "Claude Code has a budget");
    }

    #[test]
    fn test_cost_004_money_needs_an_accepted_decision() {
        let none = format!("{HEAD}{}", domain(1200, "year", "2026-12", ""));
        assert_eq!(codes(&check(&none)), ["COST-004"]);
        let link = "decision = \"standards/docs/decisions/0009-domain.md\"";
        let linked = format!("{HEAD}{}", domain(1200, "year", "2026-12", link));
        assert!(check(&linked).is_empty());
        for (state, want) in [
            (Decision::Missing, "does not exist"),
            (Decision::NotAccepted, "not accepted"),
        ] {
            let findings = check_with(Some(&linked), &move |_| state);
            assert_eq!(codes(&findings), ["COST-004"]);
            assert!(findings[0].message.contains(want), "{}", findings[0]);
        }
        let free = format!("{HEAD}{}", domain(0, "year", "2026-12", ""));
        assert!(check(&free).is_empty(), "$0 needs no decision");
    }

    #[test]
    fn test_cost_005_project_scope() {
        assert_eq!(
            codes(&check(&pages("pages_files_per_site = 20001"))),
            ["COST-005"]
        );
        assert!(check(&pages("pages_files_per_site = 20000")).is_empty());
    }

    #[test]
    fn test_cost_008_old_review_is_a_warning_only() {
        let old = "reviewed = \"2026-01-01\"\nsummary = \"x\"\n";
        let findings = check(old);
        assert_eq!(codes(&findings), ["COST-008"]);
        assert!(!findings[0].code.can_fail());
    }

    fn project(name: &str, text: &str) -> Project {
        Project {
            name: name.to_string(),
            file: crate::parse_cost_file(text).expect("parses"),
        }
    }

    fn account(projects: &[Project]) -> Vec<Finding> {
        let radar = parse_radar(RADAR_JSON).expect("radar");
        let limits = parse_limits(LIMITS_TOML).expect("limits");
        check_account(
            projects,
            &radar,
            &limits,
            Month {
                year: 2026,
                month: 12,
            },
        )
    }

    #[test]
    fn test_cost_005_account_scope_sums_projects() {
        let two = [
            project("cookbook", &pages("pages_projects = 1")),
            project("pane", &pages("pages_projects = 1")),
        ];
        assert!(account(&two).is_empty());
        let three = [
            project("cookbook", &pages("pages_projects = 1")),
            project("pane", &pages("pages_projects = 2")),
        ];
        let findings = account(&three);
        assert_eq!(codes(&findings), ["COST-005"]);
        assert!(
            findings[0].message.contains("cookbook 1, pane 2"),
            "{}",
            findings[0]
        );
    }

    #[test]
    fn test_cost_006_money_over_the_radar_budget() {
        let link = "decision = \"d.md\"";
        let findings = account(&[project(
            "cookbook",
            &format!("{HEAD}{}", domain(1200, "year", "2026-12", link)),
        )]);
        assert_eq!(codes(&findings), ["COST-006"]);
        assert!(findings[0].message.contains("$1.00"), "{}", findings[0]);
        let later = format!("{HEAD}{}", domain(1200, "year", "2027-01", link));
        assert!(account(&[project("cookbook", &later)]).is_empty());
    }

    #[test]
    fn test_monthly_cents_spreads_years_and_counts_once_in_its_month() {
        let cost = |every, cents| Cost {
            name: "x".into(),
            vendor: "Squarespace".into(),
            cents,
            every,
            since: Month {
                year: 2026,
                month: 11,
            },
            decision: None,
            notes: None,
        };
        let nov = Month {
            year: 2026,
            month: 11,
        };
        let dec = Month {
            year: 2026,
            month: 12,
        };
        let oct = Month {
            year: 2026,
            month: 10,
        };
        assert_eq!(monthly_cents(&cost(Every::Month, 500), dec), 500);
        assert_eq!(monthly_cents(&cost(Every::Year, 1200), dec), 100);
        assert_eq!(monthly_cents(&cost(Every::Year, 1201), dec), 101);
        assert_eq!(monthly_cents(&cost(Every::Once, 999), nov), 999);
        assert_eq!(monthly_cents(&cost(Every::Once, 999), dec), 0);
        assert_eq!(monthly_cents(&cost(Every::Month, 500), oct), 0);
    }
}
