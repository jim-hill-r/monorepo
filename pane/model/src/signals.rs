//! The project signals pane can light from the snapshot alone: **Review**,
//! **Retros**, **Open issues** and **Standards**.
//!
//! These functions turn the snapshot's dates and lists into the ages and
//! counts that the rules in [`crate::health`] take, then apply those rules
//! with the snapshot's own [`Snapshot::thresholds`]. Ages are whole days
//! from a date to the day the snapshot was taken, so the result depends
//! only on the snapshot, never on a clock. The signals that need live
//! GitHub data (Build, Deploy and UAT, Pull requests) come later; see
//! `pane/issues/`. Standards reads the audit findings the snapshot was
//! built with ([`Snapshot::audit`]).

use crate::audit::Finding;
use crate::health::{self, Fix, ReviewStatus};
use crate::{HumanIssue, Link, ProjectStatus, REPO_URL, Signal, Snapshot, WorkItem, WorkKind};

/// Days in one review: the deep project review runs once a week.
const DAYS_PER_REVIEW: u32 = 7;

/// The snapshot-only signals for the project at `project` (a config root
/// such as `pane` or `cookbook/web`), in the plan's order.
pub fn project_signals(snapshot: &Snapshot, project: &str) -> Vec<Signal> {
    vec![
        review(snapshot, project),
        retros(snapshot, project),
        open_issues(snapshot, project),
        standards(snapshot, project),
    ]
}

/// **Review**: the newest deck for the project's top-level folder, since
/// the rotation reviews `cookbook`, not `cookbook/web`.
pub fn review(snapshot: &Snapshot, project: &str) -> Signal {
    let top = top_level(project);
    let today = snapshot.taken_at.as_str();
    let latest = snapshot
        .reviews
        .iter()
        .filter(|r| r.project == top)
        .max_by(|a, b| a.date.cmp(&b.date));
    let status = latest.map(|r| ReviewStatus {
        age_days: days_between(&r.date, today).unwrap_or(0),
        poor_areas: count(
            r.scorecard
                .iter()
                .filter(|row| row.rating.trim().eq_ignore_ascii_case("poor")),
        ),
        rotation_days: rotation_days(snapshot),
    });
    let mut signal = health::review(status, &snapshot.thresholds.review);
    if let Some(r) = latest {
        signal
            .evidence
            .push(file_link(format!("Review deck, {}", r.date), &r.path, None));
    }
    signal
}

/// **Retros**: the newest retro of the project. Its open actions have been
/// open since the retro's date.
pub fn retros(snapshot: &Snapshot, project: &str) -> Signal {
    let today = snapshot.taken_at.as_str();
    let latest = snapshot
        .retros
        .iter()
        .filter(|r| r.project.as_deref() == Some(project))
        .max_by(|a, b| a.date.cmp(&b.date));
    let ages: Option<Vec<u32>> = latest.map(|r| {
        let age = days_between(&r.date, today).unwrap_or(0);
        r.actions.iter().filter(|a| !a.done).map(|_| age).collect()
    });
    let mut signal = health::retros(ages.as_deref(), &snapshot.thresholds.retros);
    if let Some(r) = latest {
        signal
            .evidence
            .push(file_link(format!("Retro: {}", r.title), &r.path, None));
    }
    signal
}

/// **Open issues**: the project's `FIX` items, aged from their `added`
/// date, and the human issues whose `unblocks` names the project.
pub fn open_issues(snapshot: &Snapshot, project: &str) -> Signal {
    let today = snapshot.taken_at.as_str();
    let fixes: Vec<&WorkItem> = snapshot
        .work_items
        .iter()
        .filter(|w| w.kind == WorkKind::Fix && w.project.as_deref() == Some(project))
        .collect();
    let ages: Vec<Fix> = fixes
        .iter()
        .map(|w| Fix {
            age_days: w.added.as_deref().and_then(|d| days_between(d, today)),
            // Production smoke runs file their FIXes from live data, which
            // the snapshot doesn't have yet.
            from_smoke: false,
        })
        .collect();
    let blocking: Vec<&HumanIssue> = snapshot
        .human_issues
        .iter()
        .filter(|h| h.unblocks.as_deref().is_some_and(|u| names(u, project)))
        .collect();
    let mut signal = health::open_issues(&ages, blocking.len(), &snapshot.thresholds.open_issues);
    signal.evidence.extend(
        fixes
            .iter()
            .map(|w| file_link(format!("FIX: {}", w.text), &w.file, Some(w.line))),
    );
    signal.evidence.extend(
        blocking
            .iter()
            .map(|h| file_link(format!("Human issue: {}", h.text), &h.file, None)),
    );
    signal
}

/// **Standards**: the project's own audit errors (findings in
/// [`Snapshot::audit`] owned by exactly this config root), then its open
/// `code-review` findings, which are its agent-generated `TODO`s outside any
/// epic (epics are planned phases, not findings). Grey ("Audit not run")
/// when the snapshot was built without the audit. Each error is evidence,
/// linking its file, or the project's folder.
pub fn standards(snapshot: &Snapshot, project: &str) -> Signal {
    let findings = count(snapshot.work_items.iter().filter(|w| {
        w.kind == WorkKind::Todo
            && w.project.as_deref() == Some(project)
            && w.epic.is_none()
            && w.tags.iter().any(|t| t == "agent-generated")
    }));
    let errors: Option<Vec<&Finding>> =
        snapshot.audit.as_ref().map(|a| a.errors(project).collect());
    let mut signal = health::standards(
        errors.as_ref().map(|e| count(e.iter())),
        findings,
        &snapshot.thresholds.standards,
    );
    signal.evidence.extend(errors.iter().flatten().map(|f| {
        let label = format!("error[{}] {}", f.code, f.message);
        match &f.file {
            Some(file) => file_link(label, file, None),
            None => Link {
                label,
                url: format!("{REPO_URL}/tree/main/{project}"),
            },
        }
    }));
    signal
}

/// One full review rotation in days: a week for each top-level project
/// that isn't on hold, and never less than a week.
pub fn rotation_days(snapshot: &Snapshot) -> u32 {
    let mut tops: Vec<&str> = snapshot
        .projects
        .iter()
        .filter(|p| p.status != ProjectStatus::OnHold)
        .map(|p| top_level(&p.path))
        .collect();
    tops.sort_unstable();
    tops.dedup();
    count(tops.iter()).max(1).saturating_mul(DAYS_PER_REVIEW)
}

/// Whole days from the date `from` to the date `to`. Each starts with
/// `YYYY-MM-DD` (an RFC 3339 time works too; its time is ignored). `None`
/// when either isn't a date; `0` when `to` is before `from`.
pub fn days_between(from: &str, to: &str) -> Option<u32> {
    let diff = day_number(to)? - day_number(from)?;
    Some(u32::try_from(diff.max(0)).unwrap_or(u32::MAX))
}

/// Days since 1970-01-01 of a `YYYY-MM-DD` prefix (Howard Hinnant's
/// days-from-civil algorithm).
fn day_number(date: &str) -> Option<i64> {
    let date = date.get(..10)?;
    let bytes = date.as_bytes();
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return None;
    }
    let num = |s: &str| -> Option<i64> {
        s.bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| s.parse().ok())
            .flatten()
    };
    let (y, m, d) = (num(&date[..4])?, num(&date[5..7])?, num(&date[8..10])?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// The top-level folder of a project path: `cookbook` for `cookbook/web`.
fn top_level(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}

/// Whether `text` names `project` as a whole word: not inside a longer
/// name such as `pane_model`, `my-pane` or `pane/web`.
fn names(text: &str, project: &str) -> bool {
    let part_of_name = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/');
    text.match_indices(project).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + project.len()..].chars().next();
        !before.is_some_and(part_of_name) && !after.is_some_and(part_of_name)
    })
}

fn file_link(label: String, path: &str, line: Option<u32>) -> Link {
    let anchor = line.map(|l| format!("#L{l}")).unwrap_or_default();
    Link {
        label,
        url: format!("{REPO_URL}/blob/main/{path}{anchor}"),
    }
}

fn count<T>(items: impl Iterator<Item = T>) -> u32 {
    u32::try_from(items.count()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Health, Project, Retro, RetroAction, Review, ScorecardRow, WorkSource};

    fn snapshot() -> Snapshot {
        let mut s = Snapshot::new("abc1234", "2026-11-27T12:00:00Z");
        for (path, status) in [
            ("pane", ProjectStatus::Priority),
            ("pane/web", ProjectStatus::Priority),
            ("cookbook", ProjectStatus::Active),
            ("luggage", ProjectStatus::OnHold),
        ] {
            s.projects.push(Project {
                path: path.into(),
                kind: "rust".into(),
                deploy_url: None,
                summary: None,
                status,
            });
        }
        s
    }

    fn fix(project: &str, added: Option<&str>) -> WorkItem {
        WorkItem {
            kind: WorkKind::Fix,
            text: "Broken thing.".into(),
            project: Some(project.into()),
            file: format!("{project}/issues/broken.md"),
            line: 7,
            source: WorkSource::IssuesFile,
            section: None,
            epic: None,
            tags: Vec::new(),
            added: added.map(str::to_owned),
        }
    }

    fn todo(project: &str, epic: Option<&str>, tags: &[&str]) -> WorkItem {
        WorkItem {
            kind: WorkKind::Todo,
            epic: epic.map(str::to_owned),
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            ..fix(project, Some("2026-11-27"))
        }
    }

    fn human(unblocks: &str) -> HumanIssue {
        HumanIssue {
            text: "Make a token.".into(),
            file: "issues/human/token.md".into(),
            line: 9,
            added: Some("2026-11-20".into()),
            unblocks: Some(unblocks.into()),
            why: None,
            runbook: None,
        }
    }

    fn deck(project: &str, date: &str, ratings: &[&str]) -> Review {
        Review {
            project: project.into(),
            date: date.into(),
            path: format!("docs/reviews/{date}-{project}/deck.md"),
            pr: None,
            signoffs: vec![],
            scorecard: ratings
                .iter()
                .map(|r| ScorecardRow {
                    area: "Area".into(),
                    rating: (*r).into(),
                    why: String::new(),
                })
                .collect(),
        }
    }

    fn retro(project: &str, date: &str, done: &[bool]) -> Retro {
        Retro {
            project: Some(project.into()),
            title: format!("Retro {date}"),
            date: date.into(),
            path: format!("{project}/docs/retros/{date}.md"),
            actions: done
                .iter()
                .map(|&done| RetroAction {
                    text: "Do it".into(),
                    done,
                })
                .collect(),
        }
    }

    #[test]
    fn days_between_counts_whole_days() {
        assert_eq!(days_between("2026-11-27", "2026-11-27"), Some(0));
        assert_eq!(days_between("2026-11-26", "2026-11-27T23:59:59Z"), Some(1));
        assert_eq!(days_between("2024-02-28", "2024-03-01"), Some(2));
        assert_eq!(days_between("2025-12-31", "2026-01-01"), Some(1));
        assert_eq!(days_between("1970-01-01", "2026-11-26"), Some(20_783));
    }

    #[test]
    fn days_between_is_zero_backwards_and_none_for_non_dates() {
        assert_eq!(days_between("2026-11-28", "2026-11-27"), Some(0));
        assert_eq!(days_between("soon", "2026-11-27"), None);
        assert_eq!(days_between("2026-13-01", "2026-11-27"), None);
        assert_eq!(days_between("2026-1-01", "2026-11-27"), None);
        assert_eq!(days_between("2026-11-27", ""), None);
    }

    #[test]
    fn rotation_is_a_week_per_top_level_project_not_on_hold() {
        // pane and pane/web are one project in the rotation; luggage is on hold.
        assert_eq!(rotation_days(&snapshot()), 14);
        assert_eq!(rotation_days(&Snapshot::new("a", "2026-11-27")), 7);
    }

    #[test]
    fn review_is_grey_until_the_project_is_reviewed() {
        let s = snapshot();
        let signal = review(&s, "pane");
        assert_eq!(signal.health, Health::Grey);
        assert!(signal.evidence.is_empty());
    }

    #[test]
    fn review_uses_the_newest_deck_of_the_top_level_project() {
        let mut s = snapshot();
        s.reviews
            .push(deck("pane", "2026-11-01", &["Poor", "Poor", "Poor"]));
        s.reviews
            .push(deck("pane", "2026-11-25", &["Good", "Fair"]));
        let signal = review(&s, "pane/web");
        assert_eq!(signal.health, Health::Green, "{}", signal.reason);
        assert_eq!(
            signal.evidence[0].url,
            format!("{REPO_URL}/blob/main/docs/reviews/2026-11-25-pane/deck.md")
        );
    }

    #[test]
    fn review_counts_poor_areas_and_ages_past_the_rotation() {
        let mut s = snapshot();
        s.reviews
            .push(deck("cookbook", "2026-11-25", &["Poor", "good"]));
        assert_eq!(review(&s, "cookbook").health, Health::Amber);
        s.reviews
            .push(deck("cookbook", "2026-11-26", &[" poor ", "POOR", "Poor"]));
        assert_eq!(review(&s, "cookbook").health, Health::Red);

        // Rotation is 14 days: 14 days old is still green, 15 is amber.
        let mut s = snapshot();
        s.reviews.push(deck("pane", "2026-11-13", &["Good"]));
        assert_eq!(review(&s, "pane").health, Health::Green);
        s.reviews[0].date = "2026-11-12".into();
        assert_eq!(review(&s, "pane").health, Health::Amber);
    }

    #[test]
    fn retros_age_open_actions_from_the_newest_retro() {
        let mut s = snapshot();
        assert_eq!(retros(&s, "pane").health, Health::Grey);

        s.retros.push(retro("pane", "2026-10-01", &[false]));
        assert_eq!(retros(&s, "pane").health, Health::Amber);

        // A newer retro with every action done replaces it.
        s.retros.push(retro("pane", "2026-11-20", &[true, true]));
        let signal = retros(&s, "pane");
        assert_eq!(signal.health, Health::Green);
        assert_eq!(signal.reason, "All actions done");
        assert!(
            signal.evidence[0]
                .url
                .ends_with("pane/docs/retros/2026-11-20.md")
        );

        // Another project's retro doesn't count.
        assert_eq!(retros(&s, "pane/web").health, Health::Grey);
    }

    #[test]
    fn retro_action_open_exactly_the_threshold_is_green() {
        let mut s = snapshot();
        s.retros.push(retro("pane", "2026-10-28", &[false]));
        assert_eq!(retros(&s, "pane").health, Health::Green);
    }

    #[test]
    fn open_issues_is_green_without_a_fix() {
        let mut s = snapshot();
        s.work_items.push(todo("pane", None, &[]));
        s.work_items.push(fix("cookbook", Some("2026-11-01")));
        assert_eq!(open_issues(&s, "pane").health, Health::Green);
    }

    #[test]
    fn open_issues_ages_fixes_from_their_added_date() {
        let mut s = snapshot();
        s.work_items.push(fix("pane", Some("2026-11-13")));
        let signal = open_issues(&s, "pane");
        assert_eq!(signal.health, Health::Amber, "14 days is not more than 14");
        assert_eq!(
            signal.evidence[0].url,
            format!("{REPO_URL}/blob/main/pane/issues/broken.md#L7")
        );

        s.work_items.push(fix("pane", Some("2026-11-12")));
        assert_eq!(open_issues(&s, "pane").health, Health::Red);
    }

    #[test]
    fn a_fix_of_unknown_age_is_amber_not_red() {
        let mut s = snapshot();
        s.work_items.push(fix("pane", None));
        s.work_items.push(fix("pane", Some("not a date")));
        assert_eq!(open_issues(&s, "pane").health, Health::Amber);
    }

    #[test]
    fn human_issues_block_the_project_they_name() {
        let mut s = snapshot();
        s.human_issues
            .push(human("pane Phase 3 (Cloudflare Access)"));
        s.human_issues.push(human("the `cookbook` sign-in"));
        let signal = open_issues(&s, "pane");
        assert_eq!(signal.health, Health::Amber);
        assert_eq!(signal.evidence.len(), 1);
        assert!(signal.evidence[0].label.starts_with("Human issue:"));
        assert_eq!(open_issues(&s, "cookbook").health, Health::Amber);
    }

    #[test]
    fn names_matches_whole_project_names_only() {
        assert!(names("pane", "pane"));
        assert!(names("unblocks pane, Phase 3", "pane"));
        assert!(names("(`pane`)", "pane"));
        assert!(!names("pane_model's tests", "pane"));
        assert!(!names("the pane/web page", "pane"));
        assert!(!names("windowpane", "pane"));
        assert!(!names("my-pane", "pane"));
        assert!(names("the pane/web page", "pane/web"));
    }

    #[test]
    fn standards_stays_grey_until_the_audit_is_in_the_snapshot() {
        let mut s = snapshot();
        for _ in 0..6 {
            s.work_items.push(todo("pane", None, &["agent-generated"]));
        }
        let signal = standards(&s, "pane");
        assert_eq!(signal.health, Health::Grey);
        assert_eq!(signal.reason, "Audit not run");
    }

    fn audited(output: &str) -> Snapshot {
        let mut s = snapshot();
        let roots: Vec<String> = s.projects.iter().map(|p| p.path.clone()).collect();
        s.audit = crate::audit::parse(output, &roots);
        s
    }

    #[test]
    fn standards_is_red_with_the_projects_own_audit_errors() {
        let s = audited(
            "error[STD-002] pane/web/mise.toml must include exactly one task file\n\
             error[NAM-002] pane/web: package name 'web' must be 'pane_web'\n\
             error[SEC-002] .gitignore does not ignore .env\n\
             warning[DOC-001] pane has no README.md\n\
             audit: 3 error(s)\n",
        );
        let web = standards(&s, "pane/web");
        assert_eq!(web.health, Health::Red);
        assert_eq!(web.reason, "2 audit error(s)");
        let urls: Vec<&str> = web.evidence.iter().map(|l| l.url.as_str()).collect();
        assert_eq!(
            urls,
            [
                "https://github.com/jimhillr/monorepo/blob/main/pane/web/mise.toml",
                "https://github.com/jimhillr/monorepo/tree/main/pane/web",
            ]
        );
        assert!(
            web.evidence[0]
                .label
                .starts_with("error[STD-002] pane/web/mise.toml")
        );

        // pane itself has only a warning, and the repo-wide error is no
        // project's.
        let pane = standards(&s, "pane");
        assert_eq!(pane.health, Health::Green);
        assert_eq!(pane.reason, "Audit clean");
        assert!(pane.evidence.is_empty());
    }

    #[test]
    fn standards_is_amber_with_a_clean_audit_and_many_review_findings() {
        let mut s = audited("audit: ok\n");
        assert_eq!(standards(&s, "pane").health, Health::Green);
        let amber = s.thresholds.standards.amber_review_findings;
        for _ in 0..amber {
            s.work_items.push(todo("pane", None, &["agent-generated"]));
        }
        let signal = standards(&s, "pane");
        assert_eq!(signal.health, Health::Amber);
        assert_eq!(signal.reason, format!("{amber} open code-review findings"));
    }

    #[test]
    fn standards_stays_grey_when_the_audit_did_not_finish() {
        let s = audited("error[STD-001] pane has no mise.toml\n");
        assert_eq!(standards(&s, "pane").health, Health::Grey);
    }

    #[test]
    fn project_signals_are_the_four_snapshot_signals_in_order() {
        let mut s = snapshot();
        s.work_items.push(fix("pane", Some("2026-11-01")));
        let signals = project_signals(&s, "pane");
        let names: Vec<&str> = signals.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["Review", "Retros", "Open issues", "Standards"]);
        let rollup = health::roll_up(&signals);
        assert_eq!(rollup.health, Health::Red);
        assert_eq!(rollup.coverage(), "1 of 4 signals");
    }

    #[test]
    fn signals_use_the_snapshot_thresholds() {
        let mut s = snapshot();
        s.work_items.push(fix("pane", Some("2026-11-25")));
        assert_eq!(open_issues(&s, "pane").health, Health::Amber);
        s.thresholds.open_issues.red_fix_open_days = 1;
        assert_eq!(open_issues(&s, "pane").health, Health::Red);
    }
}
