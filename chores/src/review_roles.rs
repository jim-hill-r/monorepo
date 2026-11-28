//! Review roles (decision 0019, `standards/docs/decisions/0019-review-roles.md`):
//! who may approve what, as relationships in one file,
//! `standards/reviewers.toml`.
//!
//! A change has a **domain** (what kind of thing changed: cost, a
//! requirement) and lands in a **place** (a project, the root files, or
//! somewhere the PR doesn't say). Every top-level folder is in one **tier**.
//! A person holds a domain's role over a **scope**: the whole repo, a tier or
//! one project, and a scope covers every place inside it. A PR may merge
//! when, for every (domain, place) cell it touches, someone holding that
//! domain's role over a scope covering the place has approved its head
//! commit (or opened it), and none of them has asked for changes since.
//!
//! The file can't change the fixed rules here: agents (`-agt`), bots and
//! `jimhillr-reviewer` never hold a human role; `auto` only ever counts
//! for the `change` domain; the repo owner holds every role over the whole
//! repo, so a bad edit can't lock everyone out. `chores gate` reads the file
//! from the base branch, so a PR can't grant itself an approver.
//!
//! Phase 1 checks the `requirements` and `cost` domains; phase 2 adds the
//! rest. `chores reviewers` checks the file in the audit (REV-001, REV-002).

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::gate::{self, Review};

/// The policy file, from the repo root.
pub const POLICY: &str = "standards/reviewers.toml";

/// The member that means any human collaborator on the repo.
pub const COLLABORATORS: &str = "github:collaborators";

/// What kind of thing a change touches. Each domain has one role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    Change,
    Requirements,
    Cost,
    Threat,
    Quality,
    Credential,
    Minting,
    Docs,
}

impl Domain {
    /// The domain's role name. The `change` role is named per tier (Jim's
    /// names), so it depends on where the change lands.
    pub fn role(self, tier: Option<&str>) -> String {
        match (self, tier) {
            (Domain::Change, Some(tier)) => format!("{tier}-reviewer"),
            (Domain::Change, None) => "change-reviewer".to_string(),
            (Domain::Requirements, _) => "requirements-reviewer".to_string(),
            (Domain::Cost, _) => "cost-reviewer".to_string(),
            (Domain::Threat, _) => "threat-reviewer".to_string(),
            (Domain::Quality, _) => "quality-reviewer".to_string(),
            (Domain::Credential, _) => "credential-reviewer".to_string(),
            (Domain::Minting, _) => "minting-reviewer".to_string(),
            (Domain::Docs, _) => "docs-reviewer".to_string(),
        }
    }
}

/// The tiers the `change` domain names a role after.
const TIERS: &[&str] = &[
    "software-development-platform",
    "technical-platform",
    "product",
];

/// A role name's domain, and the tier a tier-named `change` role is
/// limited to. `None` for an unknown role.
pub fn role(name: &str) -> Option<(Domain, Option<&'static str>)> {
    let domain = match name {
        "change-reviewer" => Domain::Change,
        "requirements-reviewer" => Domain::Requirements,
        "cost-reviewer" => Domain::Cost,
        "threat-reviewer" => Domain::Threat,
        "quality-reviewer" => Domain::Quality,
        "credential-reviewer" => Domain::Credential,
        "minting-reviewer" => Domain::Minting,
        "docs-reviewer" => Domain::Docs,
        _ => {
            let tier = TIERS
                .iter()
                .find(|t| name.strip_suffix("-reviewer") == Some(**t))?;
            return Some((Domain::Change, Some(tier)));
        }
    };
    Some((domain, None))
}

/// `standards/reviewers.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    #[serde(default)]
    pub tiers: BTreeMap<String, Tier>,
    #[serde(default)]
    pub grants: Vec<Grant>,
    #[serde(default)]
    pub auto: Option<Auto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tier {
    /// `strict`, `reviewed` or `fast`.
    pub protection: String,
    #[serde(default)]
    pub projects: Vec<String>,
    /// Whether the root files and dot folders belong to this tier.
    #[serde(default)]
    pub root: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub role: String,
    pub members: Vec<String>,
    pub scope: String,
    /// GitHub environments whose required reviewers must equal the members
    /// (checked in phase 3).
    #[serde(default)]
    pub environments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Auto {
    pub members: Vec<String>,
}

/// What a grant is held over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope<'a> {
    Repo,
    Tier(&'a str),
    Project(&'a str),
}

impl<'a> Scope<'a> {
    pub fn parse(s: &'a str) -> Option<Self> {
        if s == "repo" {
            return Some(Scope::Repo);
        }
        if let Some(t) = s.strip_prefix("tier:") {
            return (!t.is_empty()).then_some(Scope::Tier(t));
        }
        s.strip_prefix("project:")
            .filter(|p| !p.is_empty())
            .map(Scope::Project)
    }
}

/// Where a change lands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Place {
    /// A top-level folder.
    Project(String),
    /// A root file or a dot folder.
    Root,
    /// The PR says it touches the domain but not where (its description
    /// or a label). Only a grant over the whole repo covers it.
    Anywhere,
}

impl Place {
    /// The place a changed file lands in.
    pub fn of_path(path: &str) -> Self {
        match path.split_once('/') {
            Some((top, _)) if !top.starts_with('.') => Place::Project(top.to_string()),
            _ => Place::Root,
        }
    }
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Place::Project(p) => f.write_str(p),
            Place::Root => f.write_str("root files"),
            Place::Anywhere => f.write_str("the whole repo"),
        }
    }
}

/// One problem with the policy file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule: &'static str,
    pub message: String,
}

fn finding(rule: &'static str, message: String) -> Finding {
    Finding { rule, message }
}

impl Policy {
    pub fn parse(text: &str) -> Result<Self> {
        toml::from_str(text).context("parsing the review roles policy")
    }

    /// The policy when the file is missing or broken: no grants, so only
    /// the repo owner (a fixed rule) holds any role.
    pub fn owner_only() -> Self {
        Self::default()
    }

    /// The tier a place is in, if any.
    pub fn tier_of(&self, place: &Place) -> Option<&str> {
        self.tiers
            .iter()
            .find(|(_, t)| match place {
                Place::Project(p) => t.projects.iter().any(|q| q == p),
                Place::Root => t.root,
                Place::Anywhere => false,
            })
            .map(|(name, _)| name.as_str())
    }

    /// Whether a scope covers a place. Scopes contain each other: the repo
    /// covers every tier and a tier covers its projects.
    pub fn covers(&self, scope: Scope<'_>, place: &Place) -> bool {
        match scope {
            Scope::Repo => true,
            Scope::Tier(t) => self.tier_of(place) == Some(t),
            Scope::Project(p) => *place == Place::Project(p.to_string()),
        }
    }

    /// Whether a reviewer holds a domain's role over a place. `association`
    /// is their GitHub role on the repo (`OWNER`, `COLLABORATOR` ...).
    pub fn holds(&self, login: &str, association: &str, domain: Domain, place: &Place) -> bool {
        if !gate::is_human(login) || !gate::is_collaborator(Some(association)) {
            return false;
        }
        if association == "OWNER" {
            return true;
        }
        self.grants.iter().any(|g| {
            let Some((d, only_tier)) = role(&g.role) else {
                return false;
            };
            let Some(scope) = Scope::parse(&g.scope) else {
                return false;
            };
            d == domain
                && only_tier.is_none_or(|t| self.tier_of(place) == Some(t))
                && self.covers(scope, place)
                && g.members
                    .iter()
                    .any(|m| m == COLLABORATORS || m.eq_ignore_ascii_case(login))
        })
    }

    fn known_project(&self, p: &str) -> bool {
        self.tiers
            .values()
            .any(|t| t.projects.iter().any(|q| q == p))
    }

    /// REV-001: the file breaks a fixed rule or names an unknown role,
    /// scope, member or protection. A project in two tiers is REV-002.
    pub fn validate(&self) -> Vec<Finding> {
        let mut out = Vec::new();
        let roots: Vec<&String> = self
            .tiers
            .iter()
            .filter(|(_, t)| t.root)
            .map(|(n, _)| n)
            .collect();
        if roots.len() != 1 {
            out.push(finding(
                "REV-001",
                format!(
                    "exactly one tier must hold the root files (`root = true`), not {}",
                    roots.len()
                ),
            ));
        }
        for (name, tier) in &self.tiers {
            if !TIERS.contains(&name.as_str()) {
                out.push(finding(
                    "REV-001",
                    format!("unknown tier `{name}` (one of {})", TIERS.join(", ")),
                ));
            }
            if !matches!(tier.protection.as_str(), "strict" | "reviewed" | "fast") {
                out.push(finding(
                    "REV-001",
                    format!(
                        "tier `{name}` has protection `{}` (strict, reviewed or fast)",
                        tier.protection
                    ),
                ));
            }
        }
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for (name, tier) in &self.tiers {
            for p in &tier.projects {
                if let Some(other) = seen.insert(p, name) {
                    out.push(finding(
                        "REV-002",
                        format!("`{p}` is in two tiers: `{other}` and `{name}`"),
                    ));
                }
            }
        }
        for g in &self.grants {
            out.extend(self.validate_grant(g));
        }
        if let Some(auto) = &self.auto {
            for m in auto.members.iter().filter(|m| gate::is_human(m)) {
                out.push(finding(
                    "REV-001",
                    format!(
                        "`auto` member `{m}` is a person; `auto` is for the reviewer account, which approves by written rules"
                    ),
                ));
            }
        }
        out
    }

    fn validate_grant(&self, g: &Grant) -> Vec<Finding> {
        let mut out = Vec::new();
        let r = &g.role;
        let Some((domain, only_tier)) = role(r) else {
            return vec![finding("REV-001", format!("unknown role `{r}`"))];
        };
        match Scope::parse(&g.scope) {
            None => out.push(finding(
                "REV-001",
                format!(
                    "`{r}` has scope `{}`: use repo, tier:<name> or project:<name>",
                    g.scope
                ),
            )),
            Some(Scope::Tier(t)) if !self.tiers.contains_key(t) => out.push(finding(
                "REV-001",
                format!("`{r}` names unknown tier `{t}`"),
            )),
            Some(Scope::Project(p)) if !self.known_project(p) => out.push(finding(
                "REV-001",
                format!("`{r}` names `{p}`, which is in no tier"),
            )),
            Some(scope) => {
                if let Some(t) = only_tier {
                    let inside = match scope {
                        Scope::Tier(s) => s == t,
                        Scope::Project(p) => {
                            self.tier_of(&Place::Project(p.to_string())) == Some(t)
                        }
                        Scope::Repo => false,
                    };
                    if !inside {
                        out.push(finding(
                            "REV-001",
                            format!("`{r}` can only be held over tier:{t} or one of its projects, not `{}`", g.scope),
                        ));
                    }
                }
            }
        }
        if g.members.is_empty() {
            out.push(finding("REV-001", format!("`{r}` has no members")));
        }
        for m in &g.members {
            if m.starts_with("github:") && m != COLLABORATORS {
                out.push(finding(
                    "REV-001",
                    format!("`{r}` has unknown member group `{m}` (only {COLLABORATORS})"),
                ));
            } else if m != COLLABORATORS && !gate::is_human(m) {
                out.push(finding(
                    "REV-001",
                    format!(
                        "`{r}` names `{m}`, but agents, bots and the reviewer account never hold a human role"
                    ),
                ));
            }
        }
        if !g.environments.is_empty() && !matches!(domain, Domain::Credential | Domain::Minting) {
            out.push(finding(
                "REV-001",
                format!("`{r}` lists environments; only credential and minting roles guard one"),
            ));
        }
        out
    }

    /// REV-002: every top-level folder is in exactly one tier, and every
    /// project a tier lists exists.
    pub fn check_folders(&self, folders: &[String]) -> Vec<Finding> {
        let mut out = Vec::new();
        for f in folders {
            if !self.known_project(f) {
                out.push(finding(
                    "REV-002",
                    format!("top-level folder `{f}` is in no tier of {POLICY}"),
                ));
            }
        }
        for (name, tier) in &self.tiers {
            for p in tier.projects.iter().filter(|p| !folders.contains(p)) {
                out.push(finding(
                    "REV-002",
                    format!("tier `{name}` lists `{p}`, which is not a top-level folder"),
                ));
            }
        }
        out
    }
}

/// One thing a PR touches that needs a role's approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub domain: Domain,
    pub place: Place,
    /// Why the PR touches it, such as "cookbook/docs/cost.toml changed".
    pub why: String,
}

/// Where the holders' reviews leave one cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    /// A holder opened the PR.
    Opened,
    /// A holder approved the head and no holder asks for changes.
    Approved,
    /// A holder's last word is "changes requested".
    ChangesRequested,
    /// No holder has approved the head yet.
    Waiting,
}

/// The verdict on one cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CellVerdict {
    pub domain: Domain,
    pub place: String,
    pub role: String,
    pub why: Vec<String>,
    pub state: State,
    /// Who opened, approved or asked for changes.
    pub by: Option<String>,
}

impl CellVerdict {
    pub fn passed(&self) -> bool {
        matches!(self.state, State::Opened | State::Approved)
    }
}

/// Everything about one PR the check needs besides its cells.
pub struct Pr<'a> {
    pub author: &'a str,
    pub author_association: &'a str,
    pub head: &'a str,
    pub reviews: &'a [Review],
}

/// Decide every cell a PR touches. Cells with the same domain and place
/// are merged, keeping each reason.
pub fn check(policy: &Policy, cells: &[Cell], pr: &Pr<'_>) -> Vec<CellVerdict> {
    let mut merged: BTreeMap<(Domain, Place), Vec<String>> = BTreeMap::new();
    for c in cells {
        let why = merged.entry((c.domain, c.place.clone())).or_default();
        if !why.contains(&c.why) {
            why.push(c.why.clone());
        }
    }
    merged
        .into_iter()
        .map(|((domain, place), why)| {
            let (state, by) = cell_state(policy, domain, &place, pr);
            CellVerdict {
                domain,
                role: domain.role(policy.tier_of(&place)),
                place: place.to_string(),
                why,
                state,
                by,
            }
        })
        .collect()
}

/// Each holder's last word counts, a dismissed review withdraws theirs,
/// and a comment changes nothing.
fn cell_state(
    policy: &Policy,
    domain: Domain,
    place: &Place,
    pr: &Pr<'_>,
) -> (State, Option<String>) {
    if policy.holds(pr.author, pr.author_association, domain, place) {
        return (State::Opened, Some(pr.author.to_string()));
    }
    let mut last: Vec<(&str, &Review)> = Vec::new();
    for r in pr.reviews {
        let Some(login) = r.user.as_ref().map(|u| u.login.as_str()) else {
            continue;
        };
        let association = r.author_association.as_deref().unwrap_or("NONE");
        if !matches!(
            r.state.as_str(),
            "APPROVED" | "CHANGES_REQUESTED" | "DISMISSED"
        ) || !policy.holds(login, association, domain, place)
        {
            continue;
        }
        last.retain(|(l, _)| *l != login);
        last.push((login, r));
    }
    if let Some((login, _)) = last.iter().find(|(_, r)| r.state == "CHANGES_REQUESTED") {
        return (State::ChangesRequested, Some((*login).to_string()));
    }
    if let Some((login, _)) = last
        .iter()
        .find(|(_, r)| r.state == "APPROVED" && r.commit_id.as_deref() == Some(pr.head))
    {
        return (State::Approved, Some((*login).to_string()));
    }
    (State::Waiting, None)
}

fn git_show(root: &Path, base: &str) -> Option<String> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", &format!("{base}:{POLICY}")])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// The policy `chores gate` judges a PR by: the base branch's copy, so a
/// PR can't grant itself an approver. Until the file first reaches the base
/// branch, the working tree's copy. A missing, unreadable or invalid file
/// gives the owner-only policy, with a warning to print.
pub fn load_for_gate(root: &Path, base: &str) -> (Policy, Option<String>) {
    let text = git_show(root, base).or_else(|| std::fs::read_to_string(root.join(POLICY)).ok());
    let Some(text) = text else {
        return (
            Policy::owner_only(),
            Some(format!(
                "warning: no {POLICY}; only the repo owner can approve"
            )),
        );
    };
    match Policy::parse(&text) {
        Ok(p) => {
            let broken: Vec<String> = p
                .validate()
                .into_iter()
                .filter(|f| f.rule == "REV-001")
                .map(|f| f.message)
                .collect();
            if broken.is_empty() {
                (p, None)
            } else {
                (
                    Policy::owner_only(),
                    Some(format!(
                        "warning: {POLICY} breaks REV-001 ({}); only the repo owner can approve",
                        broken.join("; ")
                    )),
                )
            }
        }
        Err(e) => (
            Policy::owner_only(),
            Some(format!("warning: {e:#}; only the repo owner can approve")),
        ),
    }
}

/// The top-level folders git knows about, dot folders left out.
fn top_level_folders(root: &Path) -> Result<Vec<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .output()
        .context("running git ls-files")?;
    if !out.status.success() {
        bail!("git ls-files failed");
    }
    let mut folders: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter_map(|l| l.split_once('/').map(|(top, _)| top.to_string()))
        .filter(|top| !top.starts_with('.'))
        .collect();
    folders.sort();
    folders.dedup();
    Ok(folders)
}

/// `chores reviewers`: check the working tree's policy file (REV-001,
/// REV-002). Returns the findings; any finding fails the audit.
pub fn audit(root: &Path) -> Result<Vec<Finding>> {
    let path = root.join(POLICY);
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let policy = match Policy::parse(&text) {
        Ok(p) => p,
        Err(e) => return Ok(vec![finding("REV-001", format!("{e:#}"))]),
    };
    let mut out = policy.validate();
    out.extend(policy.check_folders(&top_level_folders(root)?));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::User;

    const HEAD: &str = "abcdef1234";

    fn real() -> Policy {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(POLICY);
        Policy::parse(&std::fs::read_to_string(path).unwrap_or_default()).unwrap_or_default()
    }

    fn review(login: &str, state: &str, commit: &str, association: &str) -> Review {
        Review {
            user: Some(User {
                login: login.to_string(),
            }),
            state: state.to_string(),
            commit_id: Some(commit.to_string()),
            author_association: Some(association.to_string()),
        }
    }

    fn project(p: &str) -> Place {
        Place::Project(p.to_string())
    }

    #[test]
    fn the_real_file_is_valid() {
        let p = real();
        assert!(!p.tiers.is_empty() && !p.grants.is_empty());
        assert_eq!(p.validate(), []);
    }

    #[test]
    fn the_real_file_covers_every_top_level_folder() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let Ok(folders) = top_level_folders(&root) else {
            return; // not a git checkout
        };
        assert_eq!(real().check_folders(&folders), []);
    }

    #[test]
    fn the_next_task_fallback_matches_the_product_tier() {
        let p = real();
        let mut products = p.tiers["product"].projects.clone();
        products.sort();
        let mut fallback: Vec<String> = crate::tiers::FALLBACK_PRODUCTS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        fallback.sort();
        assert_eq!(products, fallback);
    }

    #[test]
    fn roles_map_to_one_domain() {
        assert_eq!(role("cost-reviewer"), Some((Domain::Cost, None)));
        assert_eq!(
            role("product-reviewer"),
            Some((Domain::Change, Some("product")))
        );
        assert_eq!(role("critical-reviewer"), None);
        assert_eq!(role("-reviewer"), None);
        assert_eq!(Domain::Change.role(Some("product")), "product-reviewer");
        assert_eq!(Domain::Cost.role(Some("product")), "cost-reviewer");
    }

    #[test]
    fn places_and_tiers() {
        let p = real();
        assert_eq!(
            Place::of_path("cookbook/docs/cost.toml"),
            project("cookbook")
        );
        assert_eq!(Place::of_path("AGENTS.md"), Place::Root);
        assert_eq!(Place::of_path(".github/workflows/x.yml"), Place::Root);
        assert_eq!(p.tier_of(&project("cookbook")), Some("product"));
        assert_eq!(p.tier_of(&project("royalty")), Some("technical-platform"));
        assert_eq!(
            p.tier_of(&Place::Root),
            Some("software-development-platform")
        );
        assert_eq!(p.tier_of(&Place::Anywhere), None);
    }

    #[test]
    fn scopes_contain_each_other() {
        let p = real();
        let cook = project("cookbook");
        assert!(p.covers(Scope::Repo, &cook));
        assert!(p.covers(Scope::Repo, &Place::Anywhere));
        assert!(p.covers(Scope::Tier("product"), &cook));
        assert!(!p.covers(Scope::Tier("product"), &project("pane")));
        assert!(!p.covers(Scope::Tier("product"), &Place::Anywhere));
        assert!(p.covers(Scope::Project("cookbook"), &cook));
        assert!(!p.covers(Scope::Project("cookbook"), &project("homer")));
        assert_eq!(Scope::parse("tier:"), None);
        assert_eq!(Scope::parse("team:x"), None);
    }

    #[test]
    fn who_holds_what_today() {
        let p = real();
        let cook = project("cookbook");
        assert!(p.holds("jimhillr", "COLLABORATOR", Domain::Cost, &cook));
        assert!(p.holds("jimhillr", "OWNER", Domain::Requirements, &Place::Anywhere));
        // Any collaborator is a product-reviewer, but not a cost-reviewer.
        assert!(p.holds("friend", "COLLABORATOR", Domain::Change, &cook));
        assert!(!p.holds("friend", "COLLABORATOR", Domain::Change, &project("pane")));
        assert!(!p.holds("friend", "COLLABORATOR", Domain::Cost, &cook));
        // A named member must still be a collaborator.
        assert!(!p.holds("jimhillr", "NONE", Domain::Cost, &cook));
    }

    #[test]
    fn fixed_rules_beat_the_file() {
        let p = Policy::parse(
            r#"
[tiers.product]
protection = "fast"
projects = ["cookbook"]
root = true
[[grants]]
role = "cost-reviewer"
members = ["jimhillr-agt", "jimhillr-reviewer", "dependabot[bot]", "github:collaborators"]
scope = "repo"
"#,
        )
        .unwrap_or_default();
        let cook = project("cookbook");
        for bot in ["jimhillr-agt", "jimhillr-reviewer", "dependabot[bot]"] {
            assert!(!p.holds(bot, "OWNER", Domain::Cost, &cook), "{bot}");
        }
        let rules: Vec<&str> = p.validate().iter().map(|f| f.rule).collect();
        assert_eq!(rules, ["REV-001"; 3]);
        // The owner holds every role even with no grants at all.
        let none = Policy::owner_only();
        assert!(none.holds("jimhillr", "OWNER", Domain::Minting, &Place::Anywhere));
        assert!(!none.holds("friend", "COLLABORATOR", Domain::Change, &cook));
    }

    #[test]
    fn validate_reports_each_mistake() {
        let p = Policy::parse(
            r#"
[tiers.product]
protection = "quick"
projects = ["cookbook", "homer"]
[tiers.technical-platform]
protection = "reviewed"
projects = ["homer"]
[tiers.made-up]
protection = "strict"
[[grants]]
role = "critical-reviewer"
members = ["jimhillr"]
scope = "repo"
[[grants]]
role = "cost-reviewer"
members = []
scope = "tier:nowhere"
[[grants]]
role = "product-reviewer"
members = ["github:everyone"]
scope = "repo"
[[grants]]
role = "docs-reviewer"
members = ["jimhillr"]
scope = "project:pane"
environments = ["infra-mint"]
[auto]
members = ["jimhillr"]
"#,
        )
        .unwrap_or_default();
        let messages: Vec<String> = p
            .validate()
            .into_iter()
            .map(|f| format!("{} {}", f.rule, f.message))
            .collect();
        let want = [
            "REV-001 exactly one tier must hold the root files",
            "REV-001 unknown tier `made-up`",
            "REV-001 tier `product` has protection `quick`",
            "REV-002 `homer` is in two tiers",
            "REV-001 unknown role `critical-reviewer`",
            "REV-001 `cost-reviewer` names unknown tier `nowhere`",
            "REV-001 `cost-reviewer` has no members",
            "REV-001 `product-reviewer` can only be held over tier:product",
            "REV-001 `product-reviewer` has unknown member group `github:everyone`",
            "REV-001 `docs-reviewer` names `pane`, which is in no tier",
            "REV-001 `docs-reviewer` lists environments",
            "REV-001 `auto` member `jimhillr` is a person",
        ];
        assert_eq!(messages.len(), want.len(), "{messages:#?}");
        for w in want {
            assert!(
                messages.iter().any(|m| m.starts_with(w)),
                "missing {w}: {messages:#?}"
            );
        }
    }

    #[test]
    fn folders_in_no_tier_or_missing() {
        let p = real();
        let mut folders: Vec<String> = p
            .tiers
            .values()
            .flat_map(|t| t.projects.clone())
            .filter(|f| f != "bogan")
            .collect();
        folders.push("new_thing".to_string());
        let found = p.check_folders(&folders);
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found.iter().all(|f| f.rule == "REV-002"));
        assert!(found[0].message.contains("`new_thing` is in no tier"));
        assert!(found[1].message.contains("lists `bogan`"));
    }

    fn cells() -> Vec<Cell> {
        vec![
            Cell {
                domain: Domain::Cost,
                place: project("pane"),
                why: "pane/docs/cost.toml changed".to_string(),
            },
            Cell {
                domain: Domain::Cost,
                place: project("pane"),
                why: "pane/wrangler.toml changed".to_string(),
            },
            Cell {
                domain: Domain::Requirements,
                place: project("cookbook"),
                why: "cookbook/docs/requirements.md changed".to_string(),
            },
        ]
    }

    fn pr<'a>(author: &'a str, reviews: &'a [Review]) -> Pr<'a> {
        Pr {
            author,
            author_association: "COLLABORATOR",
            head: HEAD,
            reviews,
        }
    }

    #[test]
    fn cells_merge_and_wait_for_their_role() {
        let v = check(&real(), &cells(), &pr("jimhillr-agt", &[]));
        assert_eq!(v.len(), 2, "{v:?}");
        assert_eq!(v[0].domain, Domain::Requirements);
        assert_eq!(v[1].role, "cost-reviewer");
        assert_eq!(v[1].place, "pane");
        assert_eq!(v[1].why.len(), 2);
        assert!(v.iter().all(|c| c.state == State::Waiting));
    }

    #[test]
    fn one_approval_counts_for_every_role_its_author_holds() {
        let reviews = [review("jimhillr", "APPROVED", HEAD, "OWNER")];
        let v = check(&real(), &cells(), &pr("jimhillr-agt", &reviews));
        assert!(v.iter().all(CellVerdict::passed), "{v:?}");
        assert_eq!(v[0].by.as_deref(), Some("jimhillr"));
    }

    #[test]
    fn only_a_holders_last_word_on_the_head_counts() {
        let p = real();
        let ok = |reviews: &[Review]| {
            check(&p, &cells(), &pr("jimhillr-agt", reviews))
                .iter()
                .all(CellVerdict::passed)
        };
        let c = "COLLABORATOR";
        assert!(ok(&[review("jimhillr", "APPROVED", HEAD, c)]));
        assert!(!ok(&[review("friend", "APPROVED", HEAD, c)]));
        assert!(!ok(&[review("jimhillr-reviewer", "APPROVED", HEAD, c)]));
        assert!(!ok(&[review("jimhillr", "APPROVED", "0000000", c)]));
        assert!(!ok(&[review("jimhillr", "APPROVED", HEAD, "NONE")]));
        assert!(!ok(&[
            review("jimhillr", "APPROVED", HEAD, c),
            review("jimhillr", "DISMISSED", HEAD, c)
        ]));
        assert!(ok(&[
            review("jimhillr", "CHANGES_REQUESTED", "0000000", c),
            review("jimhillr", "APPROVED", HEAD, c),
            review("jimhillr", "COMMENTED", HEAD, c)
        ]));
        // Someone without the role asking for changes doesn't hold it.
        assert!(ok(&[
            review("jimhillr", "APPROVED", HEAD, c),
            review("friend", "CHANGES_REQUESTED", HEAD, c)
        ]));
        let held = check(
            &p,
            &cells(),
            &pr(
                "jimhillr-agt",
                &[
                    review("jimhillr", "APPROVED", HEAD, c),
                    review("jimhillr", "CHANGES_REQUESTED", HEAD, c),
                ],
            ),
        );
        assert_eq!(held[0].state, State::ChangesRequested);
        assert_eq!(held[0].by.as_deref(), Some("jimhillr"));
    }

    #[test]
    fn a_holder_opening_the_pr_satisfies_it() {
        let v = check(&real(), &cells(), &pr("jimhillr", &[]));
        assert!(v.iter().all(|c| c.state == State::Opened), "{v:?}");
        let v = check(&real(), &cells(), &pr("friend", &[]));
        assert!(v.iter().all(|c| c.state == State::Waiting), "{v:?}");
    }

    #[test]
    fn a_project_scoped_grant_covers_only_its_project() {
        let mut p = real();
        p.grants.push(Grant {
            role: "cost-reviewer".to_string(),
            members: vec!["sam".to_string()],
            scope: "project:pane".to_string(),
            environments: Vec::new(),
        });
        assert_eq!(p.validate(), []);
        let reviews = [review("sam", "APPROVED", HEAD, "COLLABORATOR")];
        let v = check(&p, &cells(), &pr("jimhillr-agt", &reviews));
        assert_eq!(v[1].state, State::Approved, "{v:?}");
        assert_eq!(v[0].state, State::Waiting, "{v:?}");
        let anywhere = [Cell {
            domain: Domain::Cost,
            place: Place::Anywhere,
            why: "labelled cost".to_string(),
        }];
        let v = check(&p, &anywhere, &pr("jimhillr-agt", &reviews));
        assert_eq!(v[0].state, State::Waiting, "{v:?}");
    }

    #[test]
    fn a_broken_file_falls_back_to_the_owner() {
        let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let (p, warning) = load_for_gate(dir.path(), "origin/main");
        assert!(p.grants.is_empty());
        assert!(warning.is_some_and(|w| w.contains("only the repo owner")));
        std::fs::create_dir_all(dir.path().join("standards")).unwrap_or_else(|e| panic!("{e}"));
        std::fs::write(
            dir.path().join(POLICY),
            "[[grants]]\nrole = \"made-up\"\nmembers = [\"x\"]\nscope = \"repo\"\n",
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let (p, warning) = load_for_gate(dir.path(), "origin/main");
        assert!(p.grants.is_empty());
        assert!(warning.is_some_and(|w| w.contains("REV-001")));
    }
}
