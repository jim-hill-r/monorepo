//! The repo audit's findings (`mise run //:audit`, `standards/scripts/audit.sh`),
//! for the **Standards** signal.
//!
//! The audit prints one line per finding, `error[CODE] <subject> ...` or
//! `warning[CODE] <subject> ...`, where the subject is the project or file
//! the finding is about (`pane has no README.md`,
//! `pane/evals.toml: http eval ...`), and ends with `audit: ok` or
//! `audit: N error(s)`. [`parse`] reads that output and gives each finding
//! the project that owns its subject, so the snapshot can count each
//! project's errors.

use serde::{Deserialize, Serialize};

/// What the audit found when the snapshot was built.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Audit {
    /// Every finding, in the audit's order.
    #[serde(default)]
    pub findings: Vec<Finding>,
}

/// One audit finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub level: Level,
    /// The rule, such as `STD-001` or `DOC-003`.
    pub code: String,
    /// The config root that owns the subject, such as `pane/web`; `None`
    /// for a repo-wide finding.
    pub project: Option<String>,
    /// The subject when it is a file or folder inside the project (`:line`
    /// suffixes dropped), such as `pane/evals.toml`.
    pub file: Option<String>,
    /// The line after the `level[CODE]` tag.
    pub message: String,
}

/// An audit finding's level: errors fail the audit, warnings don't yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
}

impl Audit {
    /// The project's errors: findings owned by exactly `project`.
    pub fn errors<'a>(&'a self, project: &'a str) -> impl Iterator<Item = &'a Finding> + 'a {
        self.findings
            .iter()
            .filter(move |f| f.level == Level::Error && f.project.as_deref() == Some(project))
    }
}

/// Read the audit's output. `roots` are the config roots, used to give each
/// finding its project (the longest root that contains its subject).
///
/// `None` when the output has no closing `audit: ...` line, so an audit
/// that crashed part way never shows as clean.
pub fn parse(output: &str, roots: &[String]) -> Option<Audit> {
    let mut findings = Vec::new();
    let mut finished = false;
    for line in output.lines() {
        let line = line.trim_end();
        if line == "audit: ok" || (line.starts_with("audit: ") && line.ends_with("error(s)")) {
            finished = true;
        } else if let Some(finding) = finding(line, roots) {
            findings.push(finding);
        }
    }
    finished.then_some(Audit { findings })
}

/// One `level[CODE] message` line; `None` for any other line.
fn finding(line: &str, roots: &[String]) -> Option<Finding> {
    let (level, rest) = if let Some(rest) = line.strip_prefix("error[") {
        (Level::Error, rest)
    } else {
        (Level::Warning, line.strip_prefix("warning[")?)
    };
    let (code, message) = rest.split_once(']')?;
    let valid_code = !code.is_empty()
        && code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-');
    if !valid_code {
        return None;
    }
    let message = message.trim();
    let subject = subject(message);
    let project = roots
        .iter()
        .filter(|r| subject == r.as_str() || subject.starts_with(&format!("{r}/")))
        .max_by_key(|r| r.len())
        .cloned();
    let file = project
        .as_ref()
        .filter(|p| subject.len() > p.len())
        .map(|_| subject.to_owned());
    Some(Finding {
        level,
        code: code.to_owned(),
        project,
        file,
        message: message.to_owned(),
    })
}

/// The first word of a message, without a trailing `:` or `:<line>:`
/// (`pane/evals.toml:12:` is `pane/evals.toml`).
fn subject(message: &str) -> &str {
    let word = message.split_whitespace().next().unwrap_or_default();
    let mut subject = word.trim_end_matches(':');
    while let Some((head, tail)) = subject.rsplit_once(':') {
        if tail.is_empty() || !tail.chars().all(|c| c.is_ascii_digit()) {
            break;
        }
        subject = head;
    }
    subject.trim_end_matches('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots() -> Vec<String> {
        [
            "pane",
            "pane/web",
            "profiles",
            "profiles/jimhillr",
            "my_lang",
        ]
        .map(String::from)
        .to_vec()
    }

    const OUTPUT: &str = "\
warning[DOC-001] pane has no README.md
error[STD-002] pane/web/mise.toml must include exactly one standards/tasks/<type>.toml (found 2)
warning[DOC-002] profiles/jimhillr/README.md has no '# jimhillr' heading
warning[EVL-004] my_lang/docs/requirements.md:14: LANG-001 is a must with no eval
error[SEC-002] .gitignore does not ignore .env; add a line
error[NAM-001] pane_old: 'pane_old' is not snake_case
some other line from a sub-check
error[lower] not a finding
cost check: 3 warnings
audit: 3 error(s)
";

    #[test]
    fn parses_every_finding_with_its_project_and_file() -> Result<(), String> {
        let audit = parse(OUTPUT, &roots()).ok_or("a finished audit parses")?;
        let rows: Vec<_> = audit
            .findings
            .iter()
            .map(|f| {
                (
                    f.level,
                    f.code.as_str(),
                    f.project.as_deref(),
                    f.file.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                (Level::Warning, "DOC-001", Some("pane"), None),
                (
                    Level::Error,
                    "STD-002",
                    Some("pane/web"),
                    Some("pane/web/mise.toml")
                ),
                (
                    Level::Warning,
                    "DOC-002",
                    Some("profiles/jimhillr"),
                    Some("profiles/jimhillr/README.md")
                ),
                (
                    Level::Warning,
                    "EVL-004",
                    Some("my_lang"),
                    Some("my_lang/docs/requirements.md")
                ),
                // Repo-wide, and a name that only starts like a project.
                (Level::Error, "SEC-002", None, None),
                (Level::Error, "NAM-001", None, None),
            ]
        );
        assert_eq!(
            audit.findings[0].message, "pane has no README.md",
            "the message is the line after the tag"
        );
        Ok(())
    }

    #[test]
    fn errors_count_only_the_projects_own() -> Result<(), String> {
        let audit = parse(OUTPUT, &roots()).ok_or("parses")?;
        assert_eq!(audit.errors("pane/web").count(), 1);
        // pane/web's error is its own, not pane's; pane has only a warning.
        assert_eq!(audit.errors("pane").count(), 0);
        assert_eq!(audit.errors("my_lang").count(), 0);
        Ok(())
    }

    #[test]
    fn a_clean_audit_has_no_findings() {
        assert_eq!(
            parse("audit: ok\n", &roots()),
            Some(Audit::default()),
            "clean is Some, not None"
        );
    }

    #[test]
    fn an_unfinished_audit_is_none() {
        assert_eq!(
            parse("error[STD-001] pane has no mise.toml\n", &roots()),
            None
        );
        assert_eq!(parse("", &roots()), None);
    }

    #[test]
    fn subjects_drop_colons_and_line_numbers() {
        assert_eq!(subject("pane/x.md:12: says"), "pane/x.md");
        assert_eq!(subject("pane/x.md:12:3: says"), "pane/x.md");
        assert_eq!(subject("pane/mise.toml: bad"), "pane/mise.toml");
        assert_eq!(subject("pane: 'x' is not"), "pane");
        assert_eq!(subject("pane/ has"), "pane");
        assert_eq!(subject(""), "");
    }
}
