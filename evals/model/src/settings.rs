//! The manifest of settings made by hand (`standards/settings.toml`),
//! the human tasks that ask Jim to check them (`standards/docs/issues/human/settings-check-<id>.md`)
//! and his answers (`standards/settings-checks.jsonl`).
//! See "Human checks" in `standards/docs/evals.md`.
//!
//! ```toml
//! [[setting]]
//! id = "cloudflare-api-token"
//! vendor = "Cloudflare"
//! where = "dash.cloudflare.com > My Profile > API Tokens"
//! expect = "The token in the CLOUDFLARE_API_TOKEN repo secret has Pages: Edit."
//! why = "Without it every deploy fails."
//! every = "90d"
//! ```

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::ParseError;
use crate::ids::is_kebab_case;
use crate::results::{By, ResultStatus};

/// One hand-made setting. Never holds a secret's value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    /// Unique, kebab-case, stable: answers are keyed by it.
    pub id: String,
    /// Who hosts it, such as `GitHub` or `Cloudflare`.
    pub vendor: String,
    /// The dashboard page, or a link to it.
    #[serde(rename = "where")]
    pub location: String,
    /// What it must be, checkable in under a minute.
    pub expect: String,
    /// What breaks without it.
    pub why: String,
    /// How often to check it, in days: `30d`, `90d`.
    pub every: String,
}

impl Setting {
    /// The check interval. `parse_settings` has validated it.
    pub fn interval(&self) -> Duration {
        parse_every(&self.every).unwrap_or_else(|| Duration::days(30))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsFile {
    #[serde(default)]
    setting: Vec<Setting>,
}

/// Parses `every`: a whole number of days, `30d`.
pub fn parse_every(every: &str) -> Option<Duration> {
    let days: i64 = every.strip_suffix('d')?.parse().ok()?;
    (days > 0 && !every.starts_with('+')).then(|| Duration::days(days))
}

/// Parses and validates `standards/settings.toml`, returning every problem.
pub fn parse_settings(text: &str) -> Result<Vec<Setting>, Vec<ParseError>> {
    let file: SettingsFile = toml::from_str(text).map_err(|e| {
        let message = e.message().trim().to_string();
        let line = e.span().map(|span| {
            text.get(..span.start)
                .map_or(0, |before| before.matches('\n').count())
                + 1
        });
        vec![ParseError { line, message }]
    })?;

    let mut errors = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for setting in &file.setting {
        let mut problem = |message: String| {
            errors.push(ParseError::whole_file(format!(
                "setting `{}`: {message}",
                setting.id
            )));
        };
        if !is_kebab_case(&setting.id) {
            problem("id must be kebab-case".into());
        }
        if seen.contains(&setting.id.as_str()) {
            problem("id is used twice".into());
        }
        seen.push(&setting.id);
        for (field, value) in [
            ("vendor", &setting.vendor),
            ("where", &setting.location),
            ("expect", &setting.expect),
            ("why", &setting.why),
        ] {
            if value.trim().is_empty() {
                problem(format!("{field} is empty"));
            }
        }
        if parse_every(&setting.every).is_none() {
            problem(format!(
                "every `{}` must be a number of days, like 30d",
                setting.every
            ));
        }
    }
    if errors.is_empty() {
        Ok(file.setting)
    } else {
        Err(errors)
    }
}

/// One answer, a line of `standards/settings-checks.jsonl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingCheck {
    /// The setting's `id`.
    pub setting: String,
    /// `pass` for "ok", `fail` for anything wrong.
    pub status: ResultStatus,
    /// Jim's words, such as `ok` or what is wrong.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub at: DateTime<Utc>,
    pub by: By,
}

/// Why a check is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DueReason {
    /// Never checked.
    Never,
    /// The last answer is older than `every`.
    Expired,
    /// The last answer is older than twice `every`: the setting is at risk.
    Overdue,
    /// The last answer was "not ok", and nothing newer says it is fixed.
    Failed,
}

/// A setting that needs Jim to check it.
#[derive(Debug, Clone, PartialEq)]
pub struct DueCheck<'a> {
    pub setting: &'a Setting,
    pub last: Option<&'a SettingCheck>,
    pub reason: DueReason,
}

/// The latest answer for a setting, if any.
pub fn latest_check<'a>(setting: &str, checks: &'a [SettingCheck]) -> Option<&'a SettingCheck> {
    checks
        .iter()
        .filter(|c| c.setting == setting)
        .max_by_key(|c| c.at)
}

/// Every setting whose check is due at `now`, in manifest order.
pub fn due_checks<'a>(
    settings: &'a [Setting],
    checks: &'a [SettingCheck],
    now: DateTime<Utc>,
) -> Vec<DueCheck<'a>> {
    settings
        .iter()
        .filter_map(|setting| {
            let last = latest_check(&setting.id, checks);
            let reason = match last {
                None => DueReason::Never,
                Some(check) if check.status == ResultStatus::Fail => DueReason::Failed,
                Some(check) => {
                    let age = now - check.at;
                    if age >= setting.interval() * 2 {
                        DueReason::Overdue
                    } else if age >= setting.interval() {
                        DueReason::Expired
                    } else {
                        return None;
                    }
                }
            };
            Some(DueCheck {
                setting,
                last,
                reason,
            })
        })
        .collect()
}

/// The file name of a due check's human task in `standards/docs/issues/human/`.
pub fn human_issue_file_name(setting: &str) -> String {
    format!("settings-check-{setting}.md")
}

/// The human task file (`standards/docs/issues/human/settings-check-<id>.md`) for a due check,
/// added on `added` (`YYYY-MM-DD`).
///
/// It has the three sections every human task needs (ISS-004 in
/// `standards/scripts/issues.sh`), and links the one runbook for answering a
/// settings check.
pub fn human_issue_file(due: &DueCheck<'_>, covers: &[String], added: &str) -> String {
    format!(
        "---\nadded: {added}\nunblocks: \"an up-to-date check of {id} in standards/settings.toml\"\nrunbook: {SETTINGS_CHECK_RUNBOOK}\n---\n\n## Requirement\n\n{line}\n\n## Why it can't be automated\n\nThis setting is made by hand in {vendor} ({location}), and no file in the repo or API an agent can reach holds it, so only a person can look at it.\n\n## Runbook\n\n[Answer a settings check](../../../../{SETTINGS_CHECK_RUNBOOK})\n",
        id = due.setting.id,
        line = human_issue_line(due, covers),
        vendor = due.setting.vendor,
        location = due.setting.location,
    )
}

/// The runbook every settings check task links, from the repo root.
pub const SETTINGS_CHECK_RUNBOOK: &str = "standards/docs/runbooks/settings-check.md";

/// One sentence asking Jim to check a due setting: the body of its human
/// task. `covers` lists the requirements whose `human-check` evals name the
/// setting.
pub fn human_issue_line(due: &DueCheck<'_>, covers: &[String]) -> String {
    let setting = due.setting;
    let last = match (due.reason, due.last) {
        (DueReason::Failed, Some(check)) => format!(
            "Last answer {}: not ok ({})",
            check.at.format("%Y-%m-%d"),
            check.note.as_deref().unwrap_or("no note")
        ),
        (_, Some(check)) => format!("Last checked {}", check.at.format("%Y-%m-%d")),
        (_, None) => "Never checked".to_string(),
    };
    let covers = if covers.is_empty() {
        String::new()
    } else {
        format!(" Covers {}.", covers.join(", "))
    };
    format!(
        "Check {}: {} ({}: {}). Reply ok or what is wrong. (Periodic, every {}. {last}.{covers})",
        setting.id,
        setting.expect.trim_end_matches('.'),
        setting.vendor,
        setting.location,
        setting.every,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
[[setting]]
id = "cloudflare-api-token"
vendor = "Cloudflare"
where = "My Profile > API Tokens"
expect = "The deploy token has Pages: Edit."
why = "Deploys fail without it."
every = "90d"

[[setting]]
id = "claude-connectors"
vendor = "Claude"
where = "claude.ai > Settings > Connectors"
expect = "GitHub is connected."
why = "Agents can't open PRs without it."
every = "30d"
"#;

    fn settings() -> Vec<Setting> {
        match parse_settings(MANIFEST) {
            Ok(s) => s,
            Err(e) => panic!("{e:?}"),
        }
    }

    fn at(day: u32) -> DateTime<Utc> {
        match format!("2026-01-{day:02}T12:00:00Z").parse() {
            Ok(t) => t,
            Err(e) => panic!("{e}"),
        }
    }

    fn answer(setting: &str, status: ResultStatus, day: u32) -> SettingCheck {
        SettingCheck {
            setting: setting.into(),
            status,
            note: Some(
                if status == ResultStatus::Pass {
                    "ok"
                } else {
                    "token expired"
                }
                .into(),
            ),
            at: at(day),
            by: By::Jim,
        }
    }

    #[test]
    fn parses_the_manifest() {
        let settings = settings();
        assert_eq!(settings.len(), 2);
        assert_eq!(settings[0].location, "My Profile > API Tokens");
        assert_eq!(settings[0].interval(), Duration::days(90));
    }

    #[test]
    fn reports_bad_entries() {
        let bad = r#"
[[setting]]
id = "Bad_Id"
vendor = ""
where = "x"
expect = "x"
why = "x"
every = "3w"

[[setting]]
id = "Bad_Id"
vendor = "x"
where = "x"
expect = "x"
why = "x"
every = "0d"
"#;
        let errors = match parse_settings(bad) {
            Ok(s) => panic!("parsed {s:?}"),
            Err(e) => e,
        };
        let messages: Vec<String> = errors.iter().map(ToString::to_string).collect();
        assert_eq!(
            messages,
            vec![
                "setting `Bad_Id`: id must be kebab-case",
                "setting `Bad_Id`: vendor is empty",
                "setting `Bad_Id`: every `3w` must be a number of days, like 30d",
                "setting `Bad_Id`: id must be kebab-case",
                "setting `Bad_Id`: id is used twice",
                "setting `Bad_Id`: every `0d` must be a number of days, like 30d",
            ]
        );
    }

    #[test]
    fn rejects_unknown_fields_such_as_a_value() {
        let text = "[[setting]]\nid = \"a\"\nvendor = \"x\"\nwhere = \"x\"\nexpect = \"x\"\nwhy = \"x\"\nevery = \"30d\"\nvalue = \"sk-123\"\n";
        let errors = parse_settings(text).err().unwrap_or_default();
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].message.contains("unknown field `value`"),
            "{errors:?}"
        );
        assert_eq!(errors[0].line, Some(8));
    }

    #[test]
    fn every_is_whole_days() {
        assert_eq!(parse_every("30d"), Some(Duration::days(30)));
        for bad in ["30", "d", "0d", "-1d", "+5d", "1.5d", "7h"] {
            assert_eq!(parse_every(bad), None, "{bad}");
        }
    }

    #[test]
    fn due_checks_follow_every_and_the_latest_answer() {
        let settings = settings();
        let checks = vec![
            answer("cloudflare-api-token", ResultStatus::Pass, 1),
            answer("claude-connectors", ResultStatus::Pass, 1),
        ];
        // Day 20: nothing is due.
        assert!(due_checks(&settings, &checks, at(20)).is_empty());

        // Day 31 (30 days on): the 30d check is due, the 90d one isn't.
        let due = due_checks(&settings, &checks, at(31));
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].setting.id, "claude-connectors");
        assert_eq!(due[0].reason, DueReason::Expired);

        // A newer "not ok" makes it due again at once, until a newer ok.
        let mut checks = checks;
        checks.push(answer("cloudflare-api-token", ResultStatus::Fail, 10));
        let due = due_checks(&settings, &checks, at(11));
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].reason, DueReason::Failed);
        checks.push(answer("cloudflare-api-token", ResultStatus::Pass, 12));
        assert!(due_checks(&settings, &checks, at(13)).is_empty());
    }

    #[test]
    fn never_checked_and_overdue() {
        let settings = settings();
        let due = due_checks(&settings, &[], at(1));
        assert_eq!(due.len(), 2);
        assert!(due.iter().all(|d| d.reason == DueReason::Never));

        let checks = vec![answer("claude-connectors", ResultStatus::Pass, 1)];
        let now = at(1) + Duration::days(61);
        let due = due_checks(&settings[1..], &checks, now);
        assert_eq!(due[0].reason, DueReason::Overdue);
    }

    #[test]
    fn writes_the_human_issue_line() {
        let settings = settings();
        let checks = vec![answer("cloudflare-api-token", ResultStatus::Fail, 3)];
        let due = due_checks(&settings, &checks, at(4));
        assert_eq!(
            human_issue_line(&due[0], &["COOK-003".into()]),
            "Check cloudflare-api-token: The deploy token has Pages: Edit (Cloudflare: My Profile > API Tokens). Reply ok or what is wrong. (Periodic, every 90d. Last answer 2026-01-03: not ok (token expired). Covers COOK-003.)"
        );
        assert_eq!(
            human_issue_line(&due[1], &[]),
            "Check claude-connectors: GitHub is connected (Claude: claude.ai > Settings > Connectors). Reply ok or what is wrong. (Periodic, every 30d. Never checked.)"
        );
    }

    #[test]
    fn writes_the_human_issue_file() {
        let settings = settings();
        let due = due_checks(&settings[1..], &[], at(1));
        assert_eq!(
            human_issue_file_name("claude-connectors"),
            "settings-check-claude-connectors.md"
        );
        assert_eq!(
            human_issue_file(&due[0], &[], "2026-01-01"),
            "---\nadded: 2026-01-01\nunblocks: \"an up-to-date check of claude-connectors in standards/settings.toml\"\nrunbook: standards/docs/runbooks/settings-check.md\n---\n\n## Requirement\n\nCheck claude-connectors: GitHub is connected (Claude: claude.ai > Settings > Connectors). Reply ok or what is wrong. (Periodic, every 30d. Never checked.)\n\n## Why it can't be automated\n\nThis setting is made by hand in Claude (claude.ai > Settings > Connectors), and no file in the repo or API an agent can reach holds it, so only a person can look at it.\n\n## Runbook\n\n[Answer a settings check](../../../../standards/docs/runbooks/settings-check.md)\n"
        );
    }

    #[test]
    fn reads_and_writes_an_answer_line() {
        let line = r#"{"setting":"claude-connectors","status":"pass","note":"ok","at":"2026-01-01T12:00:00Z","by":"jim"}"#;
        let check: SettingCheck = match serde_json::from_str(line) {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(check, answer("claude-connectors", ResultStatus::Pass, 1));
        assert_eq!(serde_json::to_string(&check).ok().as_deref(), Some(line));
    }
}
