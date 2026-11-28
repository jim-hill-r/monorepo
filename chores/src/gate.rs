//! `chores gate`: the `Human review` check (PR-004, Jim, 2026-11-27: "Lets
//! make sure no prs with red attention flags get auto merged. Cost changes
//! must be reviewed by jim", then "Lets make it needs human review").
//!
//! A PR needs a human's approval when any of these says it has a red
//! attention tag (`must-requirement-changed`) or changes cost:
//!
//! - the tags `chores badges` works out from the PR's diff, which the PR
//!   can't argue with;
//! - the tags its description's `## Attention` section declares;
//! - the `cost` label, which PR CI's Cost summary job sets when anything
//!   the repo uses or costs moves.
//!
//! Such a PR passes only once a human has approved its head commit (and no
//! human has asked for changes since), or when a human opened it. Agent
//! accounts (`-agt`), `jimhillr-reviewer` and bots never count.
//!
//! The verdict is a commit status, not a failing check (Jim, 2026-11-28:
//! "I don't think that should block the merge once I approve"): `pending`
//! while it waits for a human, `success` once one approves, and `failure`
//! only when a human asked for changes. One status context replaces itself
//! on every run, so an approval leaves no stale red check behind. The
//! workflow (`pull-request-human-review.yml`) posts it, and labels a held
//! PR `human-review` until a human approves. It leaves auto-merge on, so
//! the approval merges the PR (Jim, 2026-11-28: "Yes, merge on my
//! approval"). The rules are under
//! "Human review" in `standards/docs/pull-requests.md`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::badges::{self, Attention, Tag};
use crate::pr_body;

/// The reviewer account (decision 0014): it approves by written rules, so
/// it is never a human review.
pub const REVIEWER: &str = "jimhillr-reviewer";

/// Whether a GitHub login is a person: not an agent account (`-agt`), not
/// the reviewer account and not a bot or app (`[bot]`, `app/`, Copilot).
pub fn is_human(login: &str) -> bool {
    let lower = login.to_ascii_lowercase();
    !lower.is_empty()
        && !lower.ends_with("-agt")
        && lower != REVIEWER
        && !lower.ends_with("[bot]")
        && !lower.starts_with("app/")
        && lower != "copilot"
}

/// The label the Cost summary job sets when anything the repo uses or
/// costs moves.
pub const COST_LABEL: &str = "cost";

impl Tag {
    /// Whether a PR with this tag needs a human's approval: every red tag,
    /// and `cost-changed`.
    pub fn needs_human(self) -> bool {
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
    /// The reviewer's role on the repo: `OWNER`, `MEMBER`, `COLLABORATOR`,
    /// `CONTRIBUTOR`, `NONE` and so on.
    #[serde(default)]
    pub author_association: Option<String>,
}

/// Whether a repo role makes someone a collaborator (Jim, 2026-11-27: "Any
/// human collaborator is fine"): the owner, an org member or an invited
/// collaborator. Anyone else's review never counts.
pub fn is_collaborator(association: Option<&str>) -> bool {
    matches!(association, Some("OWNER" | "MEMBER" | "COLLABORATOR"))
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub login: String,
}

/// The `Human review` commit status the workflow posts.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// Nothing holds it, a human opened it, or a human approved its head.
    Success,
    /// Waiting for a human's approval: yellow, not red.
    Pending,
    /// A human asked for changes.
    Failure,
}

/// The most characters GitHub keeps of a commit status description.
pub const DESCRIPTION_CHARS: usize = 140;

/// What the check decided.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Verdict {
    /// The PR has a red tag or a cost change, so a human must approve it.
    pub needs_human: bool,
    /// The check passes.
    pub pass: bool,
    /// The commit status to post.
    pub state: State,
    /// The status description, at most [`DESCRIPTION_CHARS`] characters.
    pub description: String,
    /// Why it needs a human, one line each.
    pub reasons: Vec<String>,
    /// One line for the job summary.
    pub line: String,
}

/// The tags a description's `## Attention` section declares; unknown ones
/// are left to the PR summary check.
pub fn body_tags(body: &str) -> Vec<Tag> {
    pr_body::section(body, "Attention").map_or_else(Vec::new, |text| {
        text.lines()
            .filter_map(badges::bullet_tag)
            .filter_map(Tag::parse)
            .collect()
    })
}

/// Where the humans' reviews leave a PR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Humans {
    /// A human approved the head and none asks for changes.
    Approved,
    /// A human's last word is "changes requested".
    ChangesRequested,
    /// Neither: no human has approved the head yet.
    Waiting,
}

/// Where the humans' reviews leave `head`: approved once a human approved
/// it with no human change request after, where each human's last word
/// counts, a dismissed review withdraws theirs, and a comment changes
/// nothing. Only humans count.
pub fn humans(reviews: &[Review], head: &str) -> Humans {
    let mut last: Vec<(&str, &Review)> = Vec::new();
    for r in reviews {
        let Some(login) = r.user.as_ref().map(|u| u.login.as_str()) else {
            continue;
        };
        if !is_human(login)
            || !is_collaborator(r.author_association.as_deref())
            || !matches!(
                r.state.as_str(),
                "APPROVED" | "CHANGES_REQUESTED" | "DISMISSED"
            )
        {
            continue;
        }
        last.retain(|(l, _)| *l != login);
        last.push((login, r));
    }
    if last.iter().any(|(_, r)| r.state == "CHANGES_REQUESTED") {
        Humans::ChangesRequested
    } else if last
        .iter()
        .any(|(_, r)| r.state == "APPROVED" && r.commit_id.as_deref() == Some(head))
    {
        Humans::Approved
    } else {
        Humans::Waiting
    }
}

/// Cut `text` to at most `max` characters, ending with an ellipsis when
/// it was longer.
fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Everything the check reads about one PR.
pub struct Input<'a> {
    pub diff: &'a Attention,
    pub body: &'a str,
    pub labels: &'a [String],
    pub author: &'a str,
    /// The PR author's repo role (`author_association`).
    pub author_association: &'a str,
    pub head: &'a str,
    pub reviews: &'a [Review],
}

/// Decide the check for one PR.
pub fn decide(input: &Input<'_>) -> Verdict {
    let mut reasons = Vec::new();
    for tag in input.diff.tags() {
        if tag.needs_human() {
            let what = match tag {
                Tag::MustRequirementChanged => badges::list(&input.diff.must, 5),
                _ => badges::list(&input.diff.cost, 5),
            };
            reasons.push(format!("the diff is {} ({what})", tag.as_str()));
        }
    }
    let declared: Vec<Tag> = body_tags(input.body);
    for tag in declared.into_iter().filter(|t| t.needs_human()) {
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
    let needs_human = !reasons.is_empty();
    let short = |s: &str| s.chars().take(7).collect::<String>();
    let head = short(input.head);
    let why = reasons.join("; ");
    let tags = || {
        let mut t: Vec<&str> = Vec::new();
        for r in &reasons {
            let tag = if r.contains(Tag::MustRequirementChanged.as_str()) {
                Tag::MustRequirementChanged.as_str()
            } else {
                Tag::CostChanged.as_str()
            };
            if !t.contains(&tag) {
                t.push(tag);
            }
        }
        t.join(", ")
    };
    let (state, line, description) = if !needs_human {
        (
            State::Success,
            "pass: no red attention tag and no cost change".to_string(),
            "No red attention tag or cost change".to_string(),
        )
    } else if is_human(input.author) && is_collaborator(Some(input.author_association)) {
        (
            State::Success,
            format!("pass: {} opened it ({why})", input.author),
            format!("{} opened it ({})", input.author, tags()),
        )
    } else {
        match humans(input.reviews, input.head) {
            Humans::Approved => (
                State::Success,
                format!("pass: a human approved {head} ({why})"),
                format!("A human approved {head} ({})", tags()),
            ),
            Humans::ChangesRequested => (
                State::Failure,
                format!("block: a human asked for changes ({why})"),
                format!("A human asked for changes ({})", tags()),
            ),
            Humans::Waiting => (
                State::Pending,
                format!("wait: needs a human's approval of {head} ({why})"),
                format!("Waiting for a human to approve {head} ({})", tags()),
            ),
        }
    };
    Verdict {
        needs_human,
        pass: state == State::Success,
        state,
        description: clip(&description, DESCRIPTION_CHARS),
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
    pub author_association: String,
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
        author_association: &args.author_association,
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
        review_as(login, state, commit, "COLLABORATOR")
    }

    fn review_as(login: &str, state: &str, commit: &str, association: &str) -> Review {
        Review {
            user: Some(User {
                login: login.to_string(),
            }),
            state: state.to_string(),
            commit_id: Some(commit.to_string()),
            author_association: Some(association.to_string()),
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
            author_association: "OWNER",
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
            assert_eq!(tag.needs_human(), want, "{}", tag.as_str());
        }
    }

    #[test]
    fn a_quiet_pr_passes() {
        let v = verdict(&Attention::default(), "", &[], "jimhillr-agt", &[]);
        assert!(v.pass && !v.needs_human, "{v:?}");
    }

    #[test]
    fn amber_and_threat_tags_alone_pass() {
        let diff = Attention {
            other: vec!["COOK-003".to_string()],
            risk: vec!["pane/docs/threat.toml".to_string()],
            ..Attention::default()
        };
        let v = verdict(&diff, "", &[], "jimhillr-agt", &[]);
        assert!(v.pass && !v.needs_human, "{v:?}");
    }

    #[test]
    fn a_cost_change_waits_for_jim() {
        let v = verdict(&cost_diff(), "", &[], "jimhillr-agt", &[]);
        assert!(!v.pass && v.needs_human, "{v:?}");
        assert!(
            v.line
                .starts_with("wait: needs a human's approval of abcdef1"),
            "{}",
            v.line
        );
        assert!(v.line.contains("pane/docs/cost.toml"), "{}", v.line);
        assert_eq!(v.state, State::Pending);
        assert_eq!(
            v.description,
            "Waiting for a human to approve abcdef1 (cost-changed)"
        );
    }

    #[test]
    fn the_status_is_pending_while_waiting_and_green_once_approved() {
        let diff = cost_diff();
        let state = |reviews: &[Review]| verdict(&diff, "", &[], "jimhillr-agt", reviews).state;
        assert_eq!(state(&[]), State::Pending);
        assert_eq!(
            state(&[review("jimhillr-reviewer", "APPROVED", HEAD)]),
            State::Pending
        );
        assert_eq!(
            state(&[review("jimhillr", "APPROVED", HEAD)]),
            State::Success
        );
        assert_eq!(
            state(&[review("jimhillr", "CHANGES_REQUESTED", HEAD)]),
            State::Failure
        );
        assert_eq!(
            verdict(&Attention::default(), "", &[], "jimhillr-agt", &[]).state,
            State::Success
        );
    }

    #[test]
    fn descriptions_fit_a_commit_status() {
        let diff = Attention {
            must: vec!["X".repeat(300)],
            cost: vec!["Y".repeat(300)],
            ..Attention::default()
        };
        let body = "## Attention\n\n- 🔴 must-requirement-changed\n- 🟠 cost-changed\n";
        let v = verdict(&diff, body, &["cost"], "jimhillr-agt", &[]);
        assert!(
            v.description.chars().count() <= DESCRIPTION_CHARS,
            "{}",
            v.description
        );
        assert_eq!(
            v.description,
            "Waiting for a human to approve abcdef1 (must-requirement-changed, cost-changed)"
        );
        assert_eq!(clip(&"z".repeat(200), 10).chars().count(), 10);
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
    fn only_a_humans_approval_of_the_head_passes() {
        let diff = cost_diff();
        let ok = |reviews: &[Review]| verdict(&diff, "", &[], "jimhillr-agt", reviews).pass;
        assert!(ok(&[review("jimhillr", "APPROVED", HEAD)]));
        assert!(ok(&[review("someone", "APPROVED", HEAD)]));
        assert!(!ok(&[review("jimhillr-reviewer", "APPROVED", HEAD)]));
        assert!(!ok(&[review("jimhillr-agt", "APPROVED", HEAD)]));
        assert!(!ok(&[review("other-agt", "APPROVED", HEAD)]));
        assert!(!ok(&[review("dependabot[bot]", "APPROVED", HEAD)]));
        assert!(!ok(&[review("Copilot", "APPROVED", HEAD)]));
        assert!(!ok(&[review("jimhillr", "APPROVED", "0000000")]));
        assert!(!ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr", "CHANGES_REQUESTED", HEAD)
        ]));
        assert!(!ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("someone", "CHANGES_REQUESTED", HEAD)
        ]));
        assert!(!ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr", "DISMISSED", HEAD)
        ]));
        assert!(ok(&[
            review("jimhillr", "CHANGES_REQUESTED", "0000000"),
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr", "COMMENTED", HEAD)
        ]));
        // An agent asking for changes doesn't hold it either way.
        assert!(ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr-reviewer", "CHANGES_REQUESTED", HEAD)
        ]));
    }

    #[test]
    fn only_collaborators_count() {
        let diff = cost_diff();
        let ok = |reviews: &[Review]| verdict(&diff, "", &[], "jimhillr-agt", reviews).pass;
        assert!(ok(&[review_as("jimhillr", "APPROVED", HEAD, "OWNER")]));
        assert!(ok(&[review_as("friend", "APPROVED", HEAD, "MEMBER")]));
        assert!(!ok(&[review_as("stranger", "APPROVED", HEAD, "NONE")]));
        assert!(!ok(&[review_as(
            "stranger",
            "APPROVED",
            HEAD,
            "CONTRIBUTOR"
        )]));
        // A stranger asking for changes doesn't hold it.
        assert!(ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review_as("stranger", "CHANGES_REQUESTED", HEAD, "NONE")
        ]));
        let outsider = decide(&Input {
            diff: &diff,
            body: "",
            labels: &[],
            author: "stranger",
            author_association: "CONTRIBUTOR",
            head: HEAD,
            reviews: &[],
        });
        assert!(!outsider.pass, "{outsider:?}");
    }

    #[test]
    fn a_human_authors_pr_passes_and_an_agents_does_not() {
        let v = verdict(&cost_diff(), "", &[], "jimhillr", &[]);
        assert!(v.pass && v.needs_human, "{v:?}");
        for author in ["jimhillr-agt", "jimhillr-reviewer", "dependabot[bot]"] {
            let v = verdict(&cost_diff(), "", &[], author, &[]);
            assert!(!v.pass, "{author}: {v:?}");
        }
    }

    #[test]
    fn body_tags_skip_unknown_and_other_sections() {
        let body = "## Attention\n\n- 🟠 cost-changed: x\n- 🟣 made-up\n\n## Details\n\n- 🔴 must-requirement-changed\n";
        assert_eq!(body_tags(body), [Tag::CostChanged]);
        assert_eq!(body_tags("no sections"), []);
    }
}
