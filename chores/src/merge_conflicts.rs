//! `chores merge-conflicts`: which open PRs conflict with `main`, for the
//! `all-merge-conflicts.yml` workflow ("Keep it mergeable" in
//! standards/docs/pull-requests.md; Jim kept having to tell threads
//! "Conflicts", 2026-11-28).
//!
//! The workflow reads every open PR from the GitHub API into one JSON file;
//! this applies the rules and writes the plan. It reads only that file, so
//! the rules are tested without an account, and the workflow does the
//! labelling and commenting.
//!
//! | PR | Plan |
//! | --- | --- |
//! | Conflicts, not yet labelled | add `merge-conflict`; on an agent PR, also comment once so its thread wakes and merges `main` |
//! | Conflicts, already labelled | nothing (it was told) |
//! | Mergeable, still labelled | take the label off |
//! | GitHub hasn't worked it out yet (`mergeable` is null) | nothing until the next run |
//! | Dependabot's | nothing: Dependabot rebases its own PRs, and PR triage closes one someone else committed to |

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The label on a PR that conflicts with its base.
pub const LABEL: &str = "merge-conflict";
/// Hidden marker at the top of the comment, so a run can tell its own.
pub const MARKER: &str = "<!-- jimhillr-merge-conflicts -->";

/// One open PR, as `GET /repos/{repo}/pulls/{n}` returns it (only the
/// fields used).
#[derive(Debug, Default, Deserialize)]
pub struct Pull {
    pub number: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub draft: bool,
    /// `true`, `false`, or `null` while GitHub is still working it out.
    #[serde(default)]
    pub mergeable: Option<bool>,
    /// `clean`, `dirty` (conflicts), `blocked`, `behind`, `unknown`, ...
    #[serde(default)]
    pub mergeable_state: String,
    #[serde(default)]
    pub user: User,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub base: Base,
}

#[derive(Debug, Default, Deserialize)]
pub struct User {
    #[serde(default)]
    pub login: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Label {
    pub name: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Base {
    #[serde(rename = "ref", default)]
    pub name: String,
}

/// What the workflow does.
#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct Plan {
    /// PRs to label `merge-conflict`.
    pub label: Vec<u64>,
    /// Agent PRs to comment on (newly conflicting), each also in `label`.
    pub comment: Vec<Comment>,
    /// PRs to take the label off.
    pub unlabel: Vec<u64>,
    /// Every PR that conflicts now, labelled or not, for the summary.
    pub conflicting: Vec<Row>,
    /// PRs whose mergeability GitHub hasn't worked out yet.
    pub unknown: Vec<u64>,
}

/// A comment to post on one PR.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Comment {
    pub number: u64,
    pub body: String,
}

/// One conflicting PR in the summary.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Row {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub base: String,
    pub draft: bool,
}

/// Whether `login` is an agent account (`jimhillr-agt`), whose thread
/// subscribes to its PR's activity and so wakes on a comment.
pub fn is_agent(login: &str) -> bool {
    login.ends_with("-agt")
}

fn is_dependabot(login: &str) -> bool {
    login.starts_with("dependabot")
}

impl Pull {
    fn labelled(&self) -> bool {
        self.labels.iter().any(|l| l.name == LABEL)
    }

    /// `Some(true)` when it conflicts, `Some(false)` when it doesn't, `None`
    /// while GitHub hasn't worked it out.
    fn conflicts(&self) -> Option<bool> {
        match (self.mergeable, self.mergeable_state.as_str()) {
            (_, "dirty") | (Some(false), _) => Some(true),
            (Some(true), _) => Some(false),
            (None, _) => None,
        }
    }
}

/// The plan for these open PRs.
pub fn plan(pulls: &[Pull]) -> Plan {
    let mut plan = Plan::default();
    for pr in pulls {
        if is_dependabot(&pr.user.login) {
            continue;
        }
        match pr.conflicts() {
            None => plan.unknown.push(pr.number),
            Some(true) => {
                plan.conflicting.push(Row {
                    number: pr.number,
                    title: pr.title.clone(),
                    author: pr.user.login.clone(),
                    base: pr.base.name.clone(),
                    draft: pr.draft,
                });
                if !pr.labelled() {
                    plan.label.push(pr.number);
                    if is_agent(&pr.user.login) {
                        let base = if pr.base.name.is_empty() {
                            "main"
                        } else {
                            &pr.base.name
                        };
                        plan.comment.push(Comment {
                            number: pr.number,
                            body: comment(base),
                        });
                    }
                }
            }
            Some(false) => {
                if pr.labelled() {
                    plan.unlabel.push(pr.number);
                }
            }
        }
    }
    plan
}

/// The comment on a newly conflicting agent PR. It wakes the thread that
/// subscribed to the PR's activity; PR triage adopts it when no thread does.
pub fn comment(base: &str) -> String {
    format!(
        "{MARKER}\nThis PR now conflicts with `{base}`. The thread that owns it: convert it to \
         draft and turn off auto-merge, merge `origin/{base}`, resolve the conflict, push, and \
         once checks are green mark it ready again and ask Jim to approve the new head (his \
         earlier approval was dismissed by the push). \"Keep it mergeable\" in \
         `standards/docs/pull-requests.md` has the steps; PR triage adopts it if no thread does.\n\n\
         ---\n_Posted by the Merge conflicts workflow (`chores merge-conflicts`)._\n"
    )
}

/// The run summary, in Markdown.
pub fn render(plan: &Plan) -> String {
    let mut out = String::from("## Merge conflicts\n\n");
    if plan.conflicting.is_empty() {
        out.push_str("No open PR conflicts with its base.\n");
    } else {
        out.push_str("| PR | Author | Base | Draft | Title |\n| --- | --- | --- | --- | --- |\n");
        for r in &plan.conflicting {
            let _ = writeln!(
                out,
                "| #{} | {} | {} | {} | {} |",
                r.number,
                r.author,
                r.base,
                if r.draft { "yes" } else { "no" },
                r.title.replace('|', "\\|")
            );
        }
    }
    let list = |v: &[u64]| {
        if v.is_empty() {
            "none".to_string()
        } else {
            v.iter()
                .map(|n| format!("#{n}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    };
    let _ = write!(
        out,
        "\nLabel `{LABEL}`: {}. Comment: {}. Unlabel: {}. Not worked out yet: {}.\n",
        list(&plan.label),
        list(&plan.comment.iter().map(|c| c.number).collect::<Vec<_>>()),
        list(&plan.unlabel),
        list(&plan.unknown)
    );
    out
}

/// Read the PRs, write the plan as JSON when asked, and return it.
pub fn run(pulls: &Path, json: Option<&Path>) -> Result<Plan> {
    let text =
        std::fs::read_to_string(pulls).with_context(|| format!("reading {}", pulls.display()))?;
    let pulls: Vec<Pull> =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", pulls.display()))?;
    let plan = plan(&pulls);
    if let Some(path) = json {
        std::fs::write(path, serde_json::to_string_pretty(&plan)?)
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pulls(json: &str) -> Vec<Pull> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn labels_and_comments_a_new_agent_conflict() {
        let p = plan(&pulls(
            r#"[{"number": 7, "title": "t", "mergeable": false, "mergeable_state": "dirty",
                 "user": {"login": "jimhillr-agt"}, "labels": [], "base": {"ref": "main"}}]"#,
        ));
        assert_eq!(p.label, vec![7]);
        assert_eq!(p.comment.len(), 1);
        assert_eq!(p.comment[0].number, 7);
        assert!(p.comment[0].body.contains("origin/main"));
        assert!(p.unlabel.is_empty());
        assert_eq!(p.conflicting.len(), 1);
    }

    #[test]
    fn labels_a_human_conflict_without_commenting() {
        let p = plan(&pulls(
            r#"[{"number": 8, "mergeable": false, "user": {"login": "jimhillr"}}]"#,
        ));
        assert_eq!(p.label, vec![8]);
        assert!(p.comment.is_empty());
    }

    #[test]
    fn says_nothing_twice() {
        let p = plan(&pulls(
            r#"[{"number": 9, "mergeable": false, "mergeable_state": "dirty",
                 "user": {"login": "jimhillr-agt"}, "labels": [{"name": "merge-conflict"}]}]"#,
        ));
        assert!(p.label.is_empty());
        assert!(p.comment.is_empty());
        assert_eq!(p.conflicting.len(), 1);
    }

    #[test]
    fn unlabels_once_resolved() {
        let p = plan(&pulls(
            r#"[{"number": 10, "mergeable": true, "mergeable_state": "blocked",
                 "user": {"login": "jimhillr-agt"}, "labels": [{"name": "merge-conflict"}]},
                {"number": 11, "mergeable": true, "mergeable_state": "behind",
                 "user": {"login": "jimhillr-agt"}}]"#,
        ));
        assert_eq!(p.unlabel, vec![10]);
        assert!(p.label.is_empty());
        assert!(p.conflicting.is_empty());
    }

    #[test]
    fn waits_while_github_works_it_out() {
        let p = plan(&pulls(
            r#"[{"number": 12, "mergeable": null, "mergeable_state": "unknown",
                 "user": {"login": "jimhillr-agt"}, "labels": [{"name": "merge-conflict"}]}]"#,
        ));
        assert_eq!(p.unknown, vec![12]);
        assert!(p.unlabel.is_empty() && p.label.is_empty());
    }

    #[test]
    fn leaves_dependabot_alone() {
        let p = plan(&pulls(
            r#"[{"number": 13, "mergeable": false, "user": {"login": "dependabot[bot]"}}]"#,
        ));
        assert_eq!(p, Plan::default());
    }

    #[test]
    fn comment_carries_the_marker_and_base() {
        let c = comment("main");
        assert!(c.starts_with(MARKER));
        assert!(c.contains("origin/main"));
    }

    #[test]
    fn render_lists_conflicts_and_actions() {
        let p = plan(&pulls(
            r#"[{"number": 7, "title": "a | b", "mergeable": false,
                 "user": {"login": "jimhillr-agt"}, "base": {"ref": "main"}}]"#,
        ));
        let out = render(&p);
        assert!(out.contains("| #7 | jimhillr-agt | main | no | a \\| b |"));
        assert!(out.contains("Label `merge-conflict`: #7. Comment: #7."));
        assert!(render(&Plan::default()).contains("No open PR conflicts"));
    }
}
