//! The review queue: which open pull requests are waiting on Jim's approval,
//! with each one's short summary and attention badges (from its labels), so he can approve the
//! clear ones in a row and stop on the ones that need a decision.
//!
//! The rules are in decision 0014
//! (`standards/docs/decisions/0014-faster-pr-approvals.md`). The browser
//! reads two public GitHub API responses (no token): the open pull requests,
//! which say which are drafts and which are stacked, and a search for open
//! pull requests whose checks passed and that still need a review. This
//! module turns them into a [`Queue`]; it does no I/O so it is unit tested
//! here and runs the same in the browser.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

/// The open pull requests, newest first, at most 100 (one page).
pub const PULLS_URL: &str =
    "https://api.github.com/repos/jimhillr/monorepo/pulls?state=open&per_page=100";

/// Open pull requests whose checks all passed and that still need an
/// approving review. GitHub's search index can lag a few minutes, so a draft
/// can still show up here; [`build`] drops it using [`PULLS_URL`].
pub const READY_SEARCH_URL: &str = "https://api.github.com/search/issues?q=repo%3Ajimhillr%2Fmonorepo+is%3Apr+is%3Aopen+review%3Arequired+status%3Asuccess&per_page=100";

/// The branch every pull request finally merges into.
pub const DEFAULT_BRANCH: &str = "main";

/// The most words of a PR summary pane shows. "What changed" is at most 50
/// words (`standards/docs/pull-requests.md`); older PRs had 75.
pub const SUMMARY_WORDS: usize = 75;

/// Labels that mean a person has to decide something: red.
pub const DECIDE_LABELS: &[&str] = &["needs-human", "credential-change"];

/// Labels that mean "read this one closely": orange.
pub const LOOK_LABELS: &[&str] = &["security", "cost"];

/// Authors whose PRs come from this repo's own agents, Jim or Dependabot.
/// A PR from anyone else (a fork) gets an orange `outside author` badge.
pub const KNOWN_AUTHORS: &[&str] = &["jimhillr", "jimhillr-agt", "dependabot[bot]"];

/// The badge for a PR from an author not in [`KNOWN_AUTHORS`].
pub const OUTSIDE_AUTHOR: &str = "outside author";

/// One open pull request, as `GET /repos/{owner}/{repo}/pulls` returns it
/// (only the fields pane uses).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Pull {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub labels: Vec<Label>,
    pub user: User,
    pub head: Branch,
    pub base: Branch,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Label {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct User {
    pub login: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Branch {
    #[serde(rename = "ref")]
    pub name: String,
}

/// Parse the body of [`PULLS_URL`].
///
/// # Errors
///
/// When the JSON is not a list of pull requests (for example GitHub's rate
/// limit message).
pub fn parse_pulls(json: &str) -> Result<Vec<Pull>, String> {
    serde_json::from_str(json).map_err(|e| format!("GitHub pull requests: {e}"))
}

#[derive(Deserialize)]
struct SearchResults {
    items: Vec<SearchItem>,
}

#[derive(Deserialize)]
struct SearchItem {
    number: u64,
}

/// Parse the body of [`READY_SEARCH_URL`] into the matching PR numbers.
///
/// # Errors
///
/// When the JSON is not a search result.
pub fn parse_ready(json: &str) -> Result<BTreeSet<u64>, String> {
    let results: SearchResults =
        serde_json::from_str(json).map_err(|e| format!("GitHub search: {e}"))?;
    Ok(results.items.into_iter().map(|i| i.number).collect())
}

/// How much attention a pull request needs, from its labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Attention {
    /// No red or orange label: read the summary and approve.
    Clear,
    /// Orange: `security` or `cost`. Read it closely.
    Look,
    /// Red: `needs-human` or `credential-change`. A decision is waiting.
    Decide,
}

impl Attention {
    /// The CSS class suffix and the word shown on the badge.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Attention::Clear => "clear",
            Attention::Look => "look",
            Attention::Decide => "decide",
        }
    }
}

/// The attention level of a set of label names: the highest one present.
#[must_use]
pub fn attention<'a>(labels: impl IntoIterator<Item = &'a str>) -> Attention {
    labels
        .into_iter()
        .map(|l| {
            if DECIDE_LABELS.contains(&l) {
                Attention::Decide
            } else if LOOK_LABELS.contains(&l) {
                Attention::Look
            } else {
                Attention::Clear
            }
        })
        .max()
        .unwrap_or(Attention::Clear)
}

/// The text after a `What changed` heading (any level, any case), if any.
fn what_changed(body: &str) -> Option<&str> {
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let t = line.trim();
        let heading = t.trim_start_matches('#');
        offset += line.len();
        if heading.len() < t.len() && heading.trim().eq_ignore_ascii_case("what changed") {
            return Some(&body[offset..]);
        }
    }
    None
}

/// The short summary of a PR body: the first paragraph of its `## What
/// changed` section, or, for older PRs without one, the first paragraph of
/// prose after the attribution lines, without a `Summary` heading or a bold
/// `**Summary.**` lead-in. Cut to [`SUMMARY_WORDS`] words. Markdown links
/// keep only their text. Empty when the body has no prose.
#[must_use]
pub fn summary(body: &str) -> String {
    let body = what_changed(body).unwrap_or(body);
    let mut paragraph: Vec<&str> = Vec::new();
    let mut in_comment = false;
    for line in body.lines() {
        let t = line.trim();
        if in_comment {
            if t.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        if t.starts_with("<!--") {
            in_comment = !t.contains("-->");
            continue;
        }
        let skip = t.starts_with("_Requested by")
            || t.starts_with('#')
            || t.starts_with("---")
            || t.starts_with("🤖");
        if t.is_empty() || skip {
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        paragraph.push(t);
    }
    let text = paragraph.join(" ");
    let text = text
        .strip_prefix("**Summary.**")
        .or_else(|| text.strip_prefix("**Summary:**"))
        .or_else(|| text.strip_prefix("Summary:"))
        .unwrap_or(&text);
    let plain = link_text(text);
    let words: Vec<&str> = plain.split_whitespace().collect();
    if words.len() > SUMMARY_WORDS {
        format!("{}…", words[..SUMMARY_WORDS].join(" "))
    } else {
        words.join(" ")
    }
}

/// Replace each Markdown link `[text](url)` with its text, and drop `**`.
fn link_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(open) = rest.find('[') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find("](") {
            Some(close) if !after[..close].contains('[') => {
                let tail = &after[close + 2..];
                match tail.find(')') {
                    Some(end) => {
                        out.push_str(&after[..close]);
                        rest = &tail[end + 1..];
                    }
                    None => {
                        out.push('[');
                        rest = after;
                    }
                }
            }
            _ => {
                out.push('[');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out.replace("**", "")
}

/// One pull request in the queue.
#[derive(Debug, Clone, PartialEq)]
pub struct QueueItem {
    pub number: u64,
    pub title: String,
    pub author: String,
    /// The PR's page.
    pub url: String,
    /// Its "Files changed" tab, where GitHub's review form is.
    pub review_url: String,
    pub summary: String,
    pub attention: Attention,
    /// The labels that set [`QueueItem::attention`], in label order, and
    /// [`OUTSIDE_AUTHOR`] for a PR from outside.
    pub badges: Vec<String>,
    pub created_at: String,
    /// For a stack's top PR: how many PRs the one approval covers (itself
    /// included). 1 for a PR on its own.
    pub covers: usize,
}

/// A stack: PRs whose base is another open PR's branch, so they can only
/// reach `main` together.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    /// The stack's lowest PR out of draft.
    pub root: u64,
    /// Every PR in the stack, from the root up.
    pub numbers: Vec<u64>,
    /// The base branch of the root.
    pub base: String,
}

/// The queue, as the Approve page shows it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Queue {
    /// Green, not a draft, needs a review, no red label: approve these.
    /// Clear ones first, then orange, each oldest first.
    pub approve: Vec<QueueItem>,
    /// Green and waiting, but labelled red: a decision is needed first.
    pub decide: Vec<QueueItem>,
    /// Stacks of two or more PRs that are not yet one PR into `main`.
    /// Decision 0014: the thread opens one stack PR into `main`, and Jim
    /// approves that once.
    pub stacks: Vec<Stack>,
    /// Out of draft but not in the ready search: checks still running or
    /// failing, or already approved.
    pub waiting: usize,
    /// Open drafts, which never need Jim.
    pub drafts: usize,
}

/// Build the queue from the open pull requests and the numbers the ready
/// search returned.
#[must_use]
pub fn build(pulls: &[Pull], ready: &BTreeSet<u64>) -> Queue {
    let by_head: BTreeMap<&str, &Pull> = pulls.iter().map(|p| (p.head.name.as_str(), p)).collect();
    let stacked = |p: &Pull| by_head.contains_key(p.base.name.as_str());

    let mut q = Queue::default();
    for p in pulls {
        if p.draft {
            q.drafts += 1;
            continue;
        }
        if stacked(p) {
            // Counted in its stack; it can't merge into main on its own.
            continue;
        }
        if !ready.contains(&p.number) {
            q.waiting += 1;
            continue;
        }
        let item = item(p);
        if item.attention == Attention::Decide {
            q.decide.push(item);
        } else {
            q.approve.push(item);
        }
    }
    let open: Vec<&Pull> = pulls.iter().filter(|p| !p.draft).collect();
    q.stacks = stacks(&open);
    let order = |a: &QueueItem, b: &QueueItem| {
        (a.attention, &a.created_at, a.number).cmp(&(b.attention, &b.created_at, b.number))
    };
    q.approve.sort_by(order);
    q.decide.sort_by(order);
    q
}

/// The attention badges and level of a PR from its labels and author: red
/// and orange labels, plus [`OUTSIDE_AUTHOR`] (orange) for an author not in
/// [`KNOWN_AUTHORS`].
#[must_use]
pub fn badges(labels: &[Label], author: &str) -> (Vec<String>, Attention) {
    let names = || labels.iter().map(|l| l.name.as_str());
    let mut badges: Vec<String> = names()
        .filter(|l| DECIDE_LABELS.contains(l) || LOOK_LABELS.contains(l))
        .map(str::to_owned)
        .collect();
    let mut level = attention(names());
    if !KNOWN_AUTHORS.contains(&author) {
        badges.push(OUTSIDE_AUTHOR.to_owned());
        level = level.max(Attention::Look);
    }
    (badges, level)
}

fn item(p: &Pull) -> QueueItem {
    let (badges, level) = badges(&p.labels, &p.user.login);
    let covers = stack_size_from_body(p.body.as_deref().unwrap_or_default());
    QueueItem {
        number: p.number,
        title: p.title.clone(),
        author: p.user.login.clone(),
        url: p.html_url.clone(),
        review_url: format!("{}/files", p.html_url.trim_end_matches('/')),
        summary: summary(p.body.as_deref().unwrap_or_default()),
        attention: level,
        badges,
        created_at: p.created_at.clone(),
        covers,
    }
}

/// A stack PR says how many PRs it covers with a `Stack: #1, #2, #3` line
/// (decision 0014). 1 when it has none.
fn stack_size_from_body(body: &str) -> usize {
    body.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("Stack:"))
        .map(|rest| rest.matches('#').count())
        .filter(|n| *n > 0)
        .unwrap_or(1)
}

/// Stacks among the PRs out of draft. A stack whose lower PRs went back to
/// draft after its top PR was retargeted to `main` is no longer listed.
fn stacks(pulls: &[&Pull]) -> Vec<Stack> {
    let heads: BTreeSet<&str> = pulls.iter().map(|p| p.head.name.as_str()).collect();
    // children[base branch] = PRs on top of it.
    let mut children: BTreeMap<&str, Vec<&Pull>> = BTreeMap::new();
    for p in pulls {
        children.entry(p.base.name.as_str()).or_default().push(p);
    }
    let mut out = Vec::new();
    for root in pulls
        .iter()
        .filter(|p| !heads.contains(p.base.name.as_str()))
    {
        let mut numbers = vec![root.number];
        let mut frontier = vec![root.head.name.as_str()];
        while let Some(branch) = frontier.pop() {
            let mut kids = children.get(branch).cloned().unwrap_or_default();
            kids.sort_by_key(|p| p.number);
            for k in kids {
                if !numbers.contains(&k.number) {
                    numbers.push(k.number);
                    frontier.push(k.head.name.as_str());
                }
            }
        }
        if numbers.len() > 1 {
            out.push(Stack {
                root: root.number,
                numbers,
                base: root.base.name.clone(),
            });
        }
    }
    out.sort_by_key(|s| s.root);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pull(number: u64, head: &str, base: &str, labels: &[&str], draft: bool) -> Pull {
        Pull {
            number,
            title: format!("PR {number}"),
            html_url: format!("https://github.com/jimhillr/monorepo/pull/{number}"),
            draft,
            body: Some(format!(
                "**Summary.** Change {number}. Details in [the plan](docs/plan.md)."
            )),
            labels: labels
                .iter()
                .map(|l| Label {
                    name: (*l).to_owned(),
                })
                .collect(),
            user: User {
                login: "jimhillr-agt".to_owned(),
            },
            head: Branch {
                name: head.to_owned(),
            },
            base: Branch {
                name: base.to_owned(),
            },
            created_at: format!("2026-11-{:02}T00:00:00Z", 10 + number % 10),
        }
    }

    #[test]
    fn attention_takes_the_highest_label() {
        assert_eq!(attention([]), Attention::Clear);
        assert_eq!(attention(["rust", "dependencies"]), Attention::Clear);
        assert_eq!(attention(["cost"]), Attention::Look);
        assert_eq!(attention(["cost", "needs-human"]), Attention::Decide);
        assert_eq!(attention(["credential-change"]), Attention::Decide);
    }

    #[test]
    fn summary_skips_attribution_and_keeps_link_text() {
        let body = "<!-- claude-projects-attribution: {\"github_login\":\"jimhillr-agt\"} -->\n\
            _Requested by **Jim** · [project thread](https://claude.ai/x)_\n\n\
            **Summary.** Adds the **queue**. Details in [decision 0014](standards/docs/decisions/0014.md).\n\
            Second line of the same paragraph.\n\n## Testing\n\nRan it.";
        assert_eq!(
            summary(body),
            "Adds the queue. Details in decision 0014. Second line of the same paragraph."
        );
    }

    #[test]
    fn summary_after_a_heading_and_a_multiline_comment() {
        let body = "<!--\nTemplate notes\n-->\n## Summary\n\nDoes a thing.\n\n## Testing";
        assert_eq!(summary(body), "Does a thing.");
        assert_eq!(summary(""), "");
    }

    #[test]
    fn summary_reads_the_what_changed_section() {
        let body = "_Requested by **Jim**_\n\n## Attention\n\n- 🟢 requirements-unchanged\n\n\
            ## What changed\n\nSplits PR descriptions into [sections](a.md).\n\n## Why\n\nJim asked.";
        assert_eq!(summary(body), "Splits PR descriptions into sections.");
    }

    #[test]
    fn summary_is_cut_to_75_words() {
        let long = vec!["word"; 100].join(" ");
        let s = summary(&long);
        assert_eq!(s.split_whitespace().count(), SUMMARY_WORDS);
        assert!(s.ends_with('…'));
    }

    #[test]
    fn builds_the_queue_from_pulls_and_the_ready_search() {
        let pulls = vec![
            pull(1, "a", "main", &[], false),
            pull(2, "b", "main", &["cost"], false),
            pull(3, "c", "main", &["needs-human"], false),
            pull(4, "d", "main", &[], true),
            pull(5, "e", "main", &[], false),
        ];
        // 4 is a draft the search index still lists; 5 isn't green yet.
        let ready: BTreeSet<u64> = [1, 2, 3, 4].into();
        let q = build(&pulls, &ready);
        let approve: Vec<u64> = q.approve.iter().map(|i| i.number).collect();
        assert_eq!(approve, vec![1, 2]);
        assert_eq!(q.approve[1].badges, vec!["cost".to_owned()]);
        assert_eq!(
            q.approve[0].review_url,
            "https://github.com/jimhillr/monorepo/pull/1/files"
        );
        assert_eq!(q.approve[0].summary, "Change 1. Details in the plan.");
        assert_eq!(
            q.decide.iter().map(|i| i.number).collect::<Vec<_>>(),
            vec![3]
        );
        assert_eq!((q.drafts, q.waiting), (1, 1));
        assert!(q.stacks.is_empty());
    }

    #[test]
    fn stacked_prs_group_into_one_stack() {
        let pulls = vec![
            pull(10, "s1", "main", &[], false),
            pull(11, "s2", "s1", &[], false),
            pull(12, "s3", "s2", &[], false),
            pull(13, "solo", "main", &[], false),
        ];
        let ready: BTreeSet<u64> = [10, 11, 12, 13].into();
        let q = build(&pulls, &ready);
        assert_eq!(q.stacks.len(), 1);
        assert_eq!(q.stacks[0].numbers, vec![10, 11, 12]);
        assert_eq!(q.stacks[0].base, "main");
        // Only the root of a stack can merge into main, so only it is listed.
        let approve: Vec<u64> = q.approve.iter().map(|i| i.number).collect();
        assert_eq!(approve, vec![10, 13]);
    }

    #[test]
    fn a_retargeted_stack_is_one_pr() {
        // Decision 0014: the top PR goes to main, the rest back to draft.
        let pulls = vec![
            pull(10, "s1", "main", &[], true),
            pull(11, "s2", "s1", &[], true),
            pull(12, "s3", "main", &[], false),
        ];
        let q = build(&pulls, &[12].into());
        assert!(q.stacks.is_empty());
        assert_eq!(
            q.approve.iter().map(|i| i.number).collect::<Vec<_>>(),
            vec![12]
        );
    }

    #[test]
    fn a_stack_on_a_draft_starts_at_its_first_open_pr() {
        let pulls = vec![
            pull(10, "s1", "main", &[], true),
            pull(11, "s2", "s1", &[], false),
            pull(12, "s3", "s2", &[], false),
        ];
        let q = build(&pulls, &[11, 12].into());
        assert_eq!(q.stacks[0].numbers, vec![11, 12]);
        assert_eq!(q.stacks[0].base, "s1");
        assert!(q.approve.is_empty());
    }

    #[test]
    fn a_pr_from_outside_is_orange() {
        let mut p = pull(30, "fork", "main", &[], false);
        p.user.login = "someone".to_owned();
        let q = build(&[p], &[30].into());
        assert_eq!(q.approve[0].attention, Attention::Look);
        assert_eq!(q.approve[0].badges, vec![OUTSIDE_AUTHOR.to_owned()]);
    }

    #[test]
    fn a_stack_pr_says_how_many_it_covers() {
        let mut p = pull(20, "top", "main", &[], false);
        p.body = Some("**Summary.** The whole stack.\n\nStack: #14, #15, #16, #20".to_owned());
        let q = build(&[p], &[20].into());
        assert_eq!(q.approve[0].covers, 4);
    }

    #[test]
    fn parses_github_responses() -> Result<(), String> {
        let pulls = r#"[{"number":7,"title":"t","html_url":"https://github.com/jimhillr/monorepo/pull/7",
            "draft":false,"body":null,"labels":[{"name":"cost","id":1}],"user":{"login":"x"},
            "head":{"ref":"h","sha":"1"},"base":{"ref":"main"},"created_at":"2026-11-27T00:00:00Z"}]"#;
        let parsed = parse_pulls(pulls)?;
        assert_eq!(parsed[0].head.name, "h");
        assert_eq!(parsed[0].body, None);
        let ready = parse_ready(r#"{"total_count":2,"items":[{"number":7},{"number":9}]}"#)?;
        assert_eq!(ready, [7, 9].into());
        assert!(parse_pulls(r#"{"message":"API rate limit exceeded"}"#).is_err());
        Ok(())
    }
}
