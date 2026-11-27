//! `chores gate`: the `Jim review` check (PR-004, Jim, 2026-11-27: "Lets
//! make sure no prs with red attention flags get auto merged. Cost changes
//! must be reviewed by jim").
//!
//! A PR needs Jim's own approval when any of these says it has a red
//! attention tag (`must-requirement-changed`) or changes cost:
//!
//! - the tags `chores badges` works out from the PR's diff, which the PR
//!   can't argue with;
//! - the tags its description's `## Attention` section declares;
//! - the `cost` label, which PR CI's Cost summary job sets when anything
//!   the repo uses or costs moves.
//!
//! Such a PR passes only once `jimhillr` has approved its head commit (and
//! hasn't asked for changes since), or when Jim opened it himself. The
//! workflow (`pull-request-jim-review.yml`) also labels it `needs-human`
//! and turns its auto-merge off. The rules are under "Jim review" in
//! `standards/docs/pull-requests.md`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::badges::{self, Attention, Tag};
use crate::pr_body;

/// Jim's own GitHub account, the only one whose approval passes the check.
pub const JIM: &str = "jimhillr";

/// The label the Cost summary job sets when anything the repo uses or
/// costs moves.
pub const COST_LABEL: &str = "cost";

impl Tag {
    /// Whether a PR with this tag needs Jim's own approval: every red tag,
    /// and `cost-changed`.
    pub fn needs_jim(self) -> bool {
        matches!(self, Tag::MustRequirementChanged | Tag::CostChanged)
    }
}

/// One review from `GET /repos/{repo}/pulls/{n}/reviews`, oldest first.
#[derive(Debug, Clone, Deserialize)]
pub struct Review {
    pub user: Option<User>,
    pub state: String,
    #[serde(default)]
    pub commit_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub login: String,
}

/// What the check decided.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Verdict {
    /// The PR has a red tag or a cost change, so it is Jim's to approve.
    pub needs_jim: bool,
    /// The check passes.
    pub pass: bool,
    /// Why it needs Jim, one line each.
    pub reasons: Vec<String>,
    /// One line for the job summary.
    pub line: String,
}

/// The tags a description's `## Attention` section declares; unknown ones
/// are left to the PR summary check.
pub fn body_tags(body: &str) -> Vec<Tag> {
    pr_body::section(body, "Attention")
        .map(|text| {
            text.lines()
                .filter_map(badges::bullet_tag)
                .filter_map(Tag::parse)
                .collect()
        })
        .unwrap_or_default()
}

/// Whether Jim's latest approval or change request is an approval of
/// `head`. A dismissed review or a comment doesn't change his last word.
pub fn jim_approved(reviews: &[Review], head: &str) -> bool {
    reviews
        .iter()
        .rfind(|r| {
            r.user.as_ref().is_some_and(|u| u.login == JIM)
                && matches!(
                    r.state.as_str(),
                    "APPROVED" | "CHANGES_REQUESTED" | "DISMISSED"
                )
        })
        .is_some_and(|r| r.state == "APPROVED" && r.commit_id.as_deref() == Some(head))
}

/// Everything the check reads about one PR.
pub struct Input<'a> {
    pub diff: &'a Attention,
    pub body: &'a str,
    pub labels: &'a [String],
    pub author: &'a str,
    pub head: &'a str,
    pub reviews: &'a [Review],
}

/// Decide the check for one PR.
pub fn decide(input: &Input<'_>) -> Verdict {
    let mut reasons = Vec::new();
    for tag in input.diff.tags() {
        if tag.needs_jim() {
            let what = match tag {
                Tag::MustRequirementChanged => badges::list(&input.diff.must, 5),
                _ => badges::list(&input.diff.cost, 5),
            };
            reasons.push(format!("the diff is {} ({what})", tag.as_str()));
        }
    }
    let declared: Vec<Tag> = body_tags(input.body);
    for tag in declared.into_iter().filter(|t| t.needs_jim()) {
        let line = format!("the description says {}", tag.as_str());
        if !reasons.contains(&line) {
            reasons.push(line);
        }
    }
    if input.labels.iter().any(|l| l == COST_LABEL) {
        reasons.push(format!(
            "labelled `{COST_LABEL}` by the Cost summary (something the repo uses or costs moved)"
        ));
    }
    let needs_jim = !reasons.is_empty();
    let short = |s: &str| s.chars().take(7).collect::<String>();
    let (pass, line) = if !needs_jim {
        (
            true,
            "pass: no red attention tag and no cost change".to_string(),
        )
    } else if input.author == JIM {
        (
            true,
            format!("pass: Jim opened it ({})", reasons.join("; ")),
        )
    } else if jim_approved(input.reviews, input.head) {
        (
            true,
            format!(
                "pass: Jim approved {} ({})",
                short(input.head),
                reasons.join("; ")
            ),
        )
    } else {
        (
            false,
            format!(
                "block: needs Jim's own approval of {}, and no auto-merge ({})",
                short(input.head),
                reasons.join("; ")
            ),
        )
    };
    Verdict {
        needs_jim,
        pass,
        reasons,
        line,
    }
}

/// What `chores gate` was given on the command line.
pub struct Args {
    pub base: String,
    pub body: Option<PathBuf>,
    pub labels: Vec<String>,
    pub author: String,
    pub head: String,
    pub reviews: Option<PathBuf>,
    pub json: Option<PathBuf>,
}

fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

/// Run `chores gate`: print the verdict line and say whether it passed.
pub fn run(root: &Path, args: &Args) -> Result<bool> {
    let diff = badges::for_repo(root, &args.base)?;
    let body = args
        .body
        .as_deref()
        .map(read)
        .transpose()?
        .unwrap_or_default();
    let reviews: Vec<Review> = match &args.reviews {
        Some(p) => {
            serde_json::from_str(&read(p)?).with_context(|| format!("parsing {}", p.display()))?
        }
        None => Vec::new(),
    };
    let labels: Vec<String> = args
        .labels
        .iter()
        .filter(|l| !l.is_empty())
        .cloned()
        .collect();
    let verdict = decide(&Input {
        diff: &diff,
        body: &body,
        labels: &labels,
        author: &args.author,
        head: &args.head,
        reviews: &reviews,
    });
    if let Some(p) = &args.json {
        std::fs::write(p, serde_json::to_string_pretty(&verdict)?)
            .with_context(|| format!("writing {}", p.display()))?;
    }
    println!("{}", verdict.line);
    Ok(verdict.pass)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "abcdef1234";

    fn review(login: &str, state: &str, commit: &str) -> Review {
        Review {
            user: Some(User {
                login: login.to_string(),
            }),
            state: state.to_string(),
            commit_id: Some(commit.to_string()),
        }
    }

    fn verdict(
        diff: &Attention,
        body: &str,
        labels: &[&str],
        author: &str,
        reviews: &[Review],
    ) -> Verdict {
        let labels: Vec<String> = labels.iter().map(|s| (*s).to_string()).collect();
        decide(&Input {
            diff,
            body,
            labels: &labels,
            author,
            head: HEAD,
            reviews,
        })
    }

    fn cost_diff() -> Attention {
        Attention {
            cost: vec!["pane/docs/cost.toml".to_string()],
            ..Attention::default()
        }
    }

    fn must_diff() -> Attention {
        Attention {
            must: vec!["COOK-001 (cookbook/docs/requirements.md)".to_string()],
            ..Attention::default()
        }
    }

    #[test]
    fn every_red_tag_and_cost_changed_need_jim() {
        for tag in Tag::ALL {
            let want = tag.emoji() == "🔴" || tag == Tag::CostChanged;
            assert_eq!(tag.needs_jim(), want, "{}", tag.as_str());
        }
    }

    #[test]
    fn a_quiet_pr_passes() {
        let v = verdict(&Attention::default(), "", &[], "jimhillr-agt", &[]);
        assert!(v.pass && !v.needs_jim, "{v:?}");
    }

    #[test]
    fn amber_and_threat_tags_alone_pass() {
        let diff = Attention {
            other: vec!["COOK-003".to_string()],
            risk: vec!["pane/docs/threat.toml".to_string()],
            ..Attention::default()
        };
        let v = verdict(&diff, "", &[], "jimhillr-agt", &[]);
        assert!(v.pass && !v.needs_jim, "{v:?}");
    }

    #[test]
    fn a_cost_change_waits_for_jim() {
        let v = verdict(&cost_diff(), "", &[], "jimhillr-agt", &[]);
        assert!(!v.pass && v.needs_jim, "{v:?}");
        assert!(
            v.line
                .starts_with("block: needs Jim's own approval of abcdef1"),
            "{}",
            v.line
        );
        assert!(v.line.contains("pane/docs/cost.toml"), "{}", v.line);
    }

    #[test]
    fn a_must_requirement_change_waits_for_jim() {
        let v = verdict(&must_diff(), "", &[], "jimhillr-agt", &[]);
        assert!(!v.pass, "{v:?}");
        assert!(
            v.line.contains("must-requirement-changed (`COOK-001"),
            "{}",
            v.line
        );
    }

    #[test]
    fn the_description_and_the_cost_label_count_too() {
        let body = "## Attention\n\n- 🔴 must-requirement-changed: COOK-001\n- 🟢 cost-unchanged\n\n## What changed\n\nx\n";
        let v = verdict(&Attention::default(), body, &[], "jimhillr-agt", &[]);
        assert!(!v.pass, "{v:?}");
        assert_eq!(v.reasons, ["the description says must-requirement-changed"]);
        let v = verdict(&Attention::default(), "", &["cost"], "jimhillr-agt", &[]);
        assert!(!v.pass && v.reasons[0].contains("`cost`"), "{v:?}");
    }

    #[test]
    fn only_jims_approval_of_the_head_passes() {
        let diff = cost_diff();
        let ok = |reviews: &[Review]| verdict(&diff, "", &[], "jimhillr-agt", reviews).pass;
        assert!(ok(&[review("jimhillr", "APPROVED", HEAD)]));
        assert!(!ok(&[review("jimhillr-reviewer", "APPROVED", HEAD)]));
        assert!(!ok(&[review("jimhillr", "APPROVED", "0000000")]));
        assert!(!ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr", "CHANGES_REQUESTED", HEAD)
        ]));
        assert!(!ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr", "DISMISSED", HEAD)
        ]));
        assert!(ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr", "COMMENTED", HEAD)
        ]));
    }

    #[test]
    fn jims_own_pr_passes() {
        let v = verdict(&cost_diff(), "", &[], "jimhillr", &[]);
        assert!(v.pass && v.needs_jim, "{v:?}");
    }

    #[test]
    fn body_tags_skip_unknown_and_other_sections() {
        let body = "## Attention\n\n- 🟠 cost-changed: x\n- 🟣 made-up\n\n## Details\n\n- 🔴 must-requirement-changed\n";
        assert_eq!(body_tags(body), [Tag::CostChanged]);
        assert!(body_tags("no sections").is_empty());
    }
}
