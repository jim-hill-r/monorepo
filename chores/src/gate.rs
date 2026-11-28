//! `chores gate`: the `human-review` label (PR-004, Jim, 2026-11-27: "Lets
//! make sure no prs with red attention flags get auto merged. Cost changes
//! must be reviewed by jim", then "Lets make it needs human review"), now
//! decided by the review roles in `standards/reviewers.toml` (decision 0019,
//! phase 1: the `requirements` and `cost` domains).
//!
//! A PR touches a (domain, place) cell when:
//!
//! - its diff changes a project's `docs/requirements.md` (`requirements`,
//!   any requirement, `must` or not) or a cost file (`cost`), placed in the
//!   file's top-level folder; the PR can't argue with these;
//! - its description's `## Attention` section declares a requirement or
//!   cost tag, or it has the `cost` label from PR CI's Cost summary job,
//!   and the diff found no cell in that domain: then the cell is the whole
//!   repo.
//!
//! Each cell passes once someone holding its domain's role over a scope
//! covering the place has approved the head commit (or opened the PR), and
//! no holder has asked for changes since. Agent accounts (`-agt`),
//! `jimhillr-reviewer` and bots never hold a role; the repo owner holds
//! every one. The policy comes from the base branch.
//!
//! The verdict is a label, not a CI status (Jim, 2026-11-28: a pending
//! status "is preventing claude from marking PRs as ready for review and it
//! shows CI as red"). The workflow (`pull-request-human-review.yml`) labels
//! a held PR `human-review` until every cell passes and takes the label off
//! then; jimhillr-reviewer refuses the label, so only a human's approval
//! can merge it. The rules are under "Human review" in
//! `standards/docs/pull-requests.md`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::badges::{self, Attention, Tag};
use crate::pr_body;
use crate::review_roles::{self, Cell, CellVerdict, Domain, Place, Policy, Pr, State};

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

/// What the check decided.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Verdict {
    /// The PR touches a domain whose role a human holds (today a
    /// requirement or a cost change), so a human must approve it.
    pub needs_human: bool,
    /// Every cell has its role's approval: no `human-review` label.
    pub pass: bool,
    /// Why it needs a human, one line per cell.
    pub reasons: Vec<String>,
    /// Each (domain, place) cell, the role it needs and who gave it.
    pub cells: Vec<CellVerdict>,
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

/// Everything the check reads about one PR.
pub struct Input<'a> {
    pub policy: &'a Policy,
    pub diff: &'a Attention,
    pub body: &'a str,
    pub labels: &'a [String],
    pub author: &'a str,
    /// The PR author's repo role (`author_association`).
    pub author_association: &'a str,
    pub head: &'a str,
    pub reviews: &'a [Review],
}

/// The domain a declared attention tag puts a PR in, for the domains phase
/// 1 checks.
fn tag_domain(tag: Tag) -> Option<Domain> {
    match tag {
        Tag::MustRequirementChanged | Tag::RequirementChanged => Some(Domain::Requirements),
        Tag::CostChanged => Some(Domain::Cost),
        _ => None,
    }
}

/// The (domain, place) cells a PR touches in the domains phase 1 checks:
/// `requirements` (any requirement change, `must` or not) and `cost`. The
/// diff says where; the description's tags and the `cost` label add a cell
/// for the whole repo only when the diff found none in that domain.
pub fn cells(input: &Input<'_>) -> Vec<Cell> {
    let mut out: Vec<Cell> = Vec::new();
    for f in &input.diff.requirement_files {
        out.push(Cell {
            domain: Domain::Requirements,
            place: Place::of_path(f),
            why: format!("`{f}` changed"),
        });
    }
    for f in &input.diff.cost {
        out.push(Cell {
            domain: Domain::Cost,
            place: Place::of_path(f),
            why: format!("`{f}` changed"),
        });
    }
    let mut declared: Vec<(Domain, String)> = body_tags(input.body)
        .into_iter()
        .filter_map(|t| tag_domain(t).map(|d| (d, format!("the description says {}", t.as_str()))))
        .collect();
    if input.labels.iter().any(|l| l == COST_LABEL) {
        declared.push((
            Domain::Cost,
            format!("labelled `{COST_LABEL}` by the Cost summary"),
        ));
    }
    let from_diff: Vec<Domain> = out.iter().map(|c| c.domain).collect();
    for (domain, why) in declared {
        if !from_diff.contains(&domain) {
            out.push(Cell {
                domain,
                place: Place::Anywhere,
                why,
            });
        }
    }
    out
}

fn cell_name(c: &CellVerdict) -> String {
    format!("{} ({})", c.role, c.place)
}

/// The summary line for verdicts that don't all pass.
fn hold_line(verdicts: &[CellVerdict], head: &str, reasons: &[String]) -> String {
    let mut parts = Vec::new();
    let asked: Vec<String> = verdicts
        .iter()
        .filter(|c| c.state == State::ChangesRequested)
        .map(|c| {
            format!(
                "{} asked for changes as {}",
                c.by.as_deref().unwrap_or("?"),
                cell_name(c)
            )
        })
        .collect();
    if !asked.is_empty() {
        parts.push(asked.join("; "));
    }
    let waiting: Vec<String> = verdicts
        .iter()
        .filter(|c| c.state == State::Waiting)
        .map(cell_name)
        .collect();
    if !waiting.is_empty() {
        parts.push(format!(
            "needs approval of {head} from {}",
            waiting.join(", ")
        ));
    }
    format!("hold: {} ({})", parts.join("; "), reasons.join("; "))
}

/// Decide the check for one PR.
pub fn decide(input: &Input<'_>) -> Verdict {
    let verdicts = review_roles::check(
        input.policy,
        &cells(input),
        &Pr {
            author: input.author,
            author_association: input.author_association,
            head: input.head,
            reviews: input.reviews,
        },
    );
    let reasons: Vec<String> = verdicts
        .iter()
        .map(|c| format!("{}: {}", cell_name(c), c.why.join(", ")))
        .collect();
    let head: String = input.head.chars().take(7).collect();
    let pass = verdicts.iter().all(CellVerdict::passed);
    let line = if verdicts.is_empty() {
        "pass: no requirement or cost change".to_string()
    } else if pass {
        let given: Vec<String> = verdicts
            .iter()
            .map(|c| {
                let by = c.by.as_deref().unwrap_or("?");
                let how = if c.state == State::Opened {
                    "opened"
                } else {
                    "approved"
                };
                format!("{} {how} by {by}", cell_name(c))
            })
            .collect();
        format!("pass: every role approved {head}: {}", given.join("; "))
    } else {
        hold_line(&verdicts, &head, &reasons)
    };
    Verdict {
        needs_human: !verdicts.is_empty(),
        pass,
        reasons,
        cells: verdicts,
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
    let (policy, warning) = review_roles::load_for_gate(root, &args.base);
    if let Some(w) = warning {
        println!("{w}");
    }
    let verdict = decide(&Input {
        policy: &policy,
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

    fn policy() -> Policy {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(review_roles::POLICY);
        Policy::parse(&std::fs::read_to_string(path).unwrap_or_default()).unwrap_or_default()
    }

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
        let policy = policy();
        decide(&Input {
            policy: &policy,
            diff,
            body,
            labels: &labels,
            author,
            author_association: "COLLABORATOR",
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
            must: vec!["COOK-001".to_string()],
            requirement_files: vec!["cookbook/docs/requirements.md".to_string()],
            ..Attention::default()
        }
    }

    #[test]
    fn a_quiet_pr_passes() {
        let v = verdict(&Attention::default(), "", &[], "jimhillr-agt", &[]);
        assert!(v.pass && !v.needs_human && v.cells.is_empty(), "{v:?}");
        assert_eq!(v.line, "pass: no requirement or cost change");
    }

    #[test]
    fn a_threat_change_alone_passes_until_phase_2() {
        let diff = Attention {
            risk: vec!["pane/docs/threat.toml".to_string()],
            ..Attention::default()
        };
        let v = verdict(&diff, "", &[], "jimhillr-agt", &[]);
        assert!(v.pass && !v.needs_human, "{v:?}");
    }

    #[test]
    fn a_cost_change_waits_for_a_cost_reviewer() {
        let v = verdict(&cost_diff(), "", &[], "jimhillr-agt", &[]);
        assert!(!v.pass && v.needs_human, "{v:?}");
        assert_eq!(
            v.line,
            "hold: needs approval of abcdef1 from cost-reviewer (pane) (cost-reviewer (pane): `pane/docs/cost.toml` changed)"
        );
    }

    #[test]
    fn any_requirement_change_waits_for_a_requirements_reviewer() {
        // Stricter than before phase 1: a `should` change holds too.
        let diff = Attention {
            other: vec!["COOK-003".to_string()],
            requirement_files: vec!["cookbook/docs/requirements.md".to_string()],
            ..Attention::default()
        };
        for d in [diff, must_diff()] {
            let v = verdict(&d, "", &[], "jimhillr-agt", &[]);
            assert!(!v.pass, "{v:?}");
            assert!(
                v.line.contains("requirements-reviewer (cookbook)"),
                "{}",
                v.line
            );
        }
    }

    #[test]
    fn changes_requested_holds_with_its_own_line() {
        let v = verdict(
            &cost_diff(),
            "",
            &[],
            "jimhillr-agt",
            &[review("jimhillr", "CHANGES_REQUESTED", HEAD)],
        );
        assert!(!v.pass, "{v:?}");
        assert!(
            v.line
                .starts_with("hold: jimhillr asked for changes as cost-reviewer (pane)"),
            "{}",
            v.line
        );
    }

    #[test]
    fn the_description_and_the_cost_label_count_for_the_whole_repo() {
        let body = "## Attention\n\n- 🔴 must-requirement-changed: COOK-001\n- 🟢 cost-unchanged\n\n## What changed\n\nx\n";
        let v = verdict(&Attention::default(), body, &[], "jimhillr-agt", &[]);
        assert!(!v.pass, "{v:?}");
        assert_eq!(
            v.reasons,
            [
                "requirements-reviewer (the whole repo): the description says must-requirement-changed"
            ]
        );
        let v = verdict(&Attention::default(), "", &["cost"], "jimhillr-agt", &[]);
        assert!(!v.pass && v.reasons[0].contains("`cost`"), "{v:?}");
        // When the diff already places the domain, the label adds no cell.
        let v = verdict(&cost_diff(), "", &["cost"], "jimhillr-agt", &[]);
        assert_eq!(v.cells.len(), 1, "{v:?}");
        assert_eq!(v.cells[0].place, "pane");
    }

    #[test]
    fn one_approval_from_jim_passes_every_cell() {
        let diff = Attention {
            cost: vec!["pane/docs/cost.toml".to_string(), "mise.toml".to_string()],
            ..must_diff()
        };
        let v = verdict(
            &diff,
            "",
            &["cost"],
            "jimhillr-agt",
            &[review("jimhillr", "APPROVED", HEAD)],
        );
        assert!(v.pass && v.needs_human, "{v:?}");
        assert_eq!(v.cells.len(), 3, "{v:?}");
        assert!(
            v.line.starts_with("pass: every role approved abcdef1: "),
            "{}",
            v.line
        );
        assert!(
            v.line
                .contains("cost-reviewer (root files) approved by jimhillr"),
            "{}",
            v.line
        );
    }

    #[test]
    fn only_a_role_holders_approval_of_the_head_passes() {
        let diff = cost_diff();
        let ok = |reviews: &[Review]| verdict(&diff, "", &[], "jimhillr-agt", reviews).pass;
        assert!(ok(&[review("jimhillr", "APPROVED", HEAD)]));
        assert!(ok(&[review_as("jimhillr", "APPROVED", HEAD, "OWNER")]));
        // Since phase 1, a collaborator who isn't a cost-reviewer doesn't count.
        assert!(!ok(&[review("someone", "APPROVED", HEAD)]));
        assert!(!ok(&[review("jimhillr-reviewer", "APPROVED", HEAD)]));
        assert!(!ok(&[review("jimhillr-agt", "APPROVED", HEAD)]));
        assert!(!ok(&[review("dependabot[bot]", "APPROVED", HEAD)]));
        assert!(!ok(&[review("Copilot", "APPROVED", HEAD)]));
        assert!(!ok(&[review("jimhillr", "APPROVED", "0000000")]));
        assert!(!ok(&[review_as("jimhillr", "APPROVED", HEAD, "NONE")]));
        assert!(!ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr", "DISMISSED", HEAD)
        ]));
        // An agent asking for changes doesn't hold it.
        assert!(ok(&[
            review("jimhillr", "APPROVED", HEAD),
            review("jimhillr-reviewer", "CHANGES_REQUESTED", HEAD)
        ]));
    }

    #[test]
    fn a_role_holders_own_pr_passes_and_an_agents_does_not() {
        let v = verdict(&cost_diff(), "", &[], "jimhillr", &[]);
        assert!(v.pass && v.needs_human, "{v:?}");
        assert!(v.line.contains("opened by jimhillr"), "{}", v.line);
        for author in [
            "jimhillr-agt",
            "jimhillr-reviewer",
            "dependabot[bot]",
            "someone",
        ] {
            let v = verdict(&cost_diff(), "", &[], author, &[]);
            assert!(!v.pass, "{author}: {v:?}");
        }
    }

    #[test]
    fn a_missing_policy_leaves_only_the_owner() {
        let none = Policy::owner_only();
        let diff = cost_diff();
        let run = |association: &str| {
            decide(&Input {
                policy: &none,
                diff: &diff,
                body: "",
                labels: &[],
                author: "jimhillr-agt",
                author_association: "NONE",
                head: HEAD,
                reviews: &[review_as("jimhillr", "APPROVED", HEAD, association)],
            })
            .pass
        };
        assert!(run("OWNER"));
        assert!(!run("COLLABORATOR"));
    }

    #[test]
    fn body_tags_skip_unknown_and_other_sections() {
        let body = "## Attention\n\n- 🟠 cost-changed: x\n- 🟣 made-up\n\n## Details\n\n- 🔴 must-requirement-changed\n";
        assert_eq!(body_tags(body), [Tag::CostChanged]);
        assert_eq!(body_tags("no sections"), []);
    }
}
