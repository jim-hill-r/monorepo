//! Deck sign-offs (Jim, 2026-11-28): a reviewer marks a review deck as
//! reviewed, with a verdict and feedback, and each feedback item becomes an
//! issue file that agents work like any other
//! ([plan](../../docs/plans/2026-11-deck-sign-off.md)).
//!
//! A sign-off is a Markdown file next to its deck,
//! `docs/reviews/<folder>/signoffs/<date>-<reviewer>.md`:
//!
//! ```markdown
//! ---
//! reviewer: jim
//! date: 2026-11-28
//! verdict: changes
//! ---
//!
//! - Slide 4: Say which free tier is nearest its limit. ([issue](../../../cookbook/issues/review-2026-11-25-cookbook-jim-1.md))
//! ```
//!
//! [`files`] turns a reviewer's [`Form`] into the sign-off and its issue
//! files, so the browser and the Worker write exactly what [`parse`] reads
//! back. This module does no I/O.

use serde::{Deserialize, Serialize};

use crate::Review;

/// The folder under a deck's folder that holds its sign-offs.
pub const SIGNOFFS_DIR: &str = "signoffs";

/// The epic every feedback issue file belongs to.
pub const FEEDBACK_EPIC: &str = "Review feedback";

/// The longest feedback item, in characters.
pub const MAX_ITEM_CHARS: usize = 2000;

/// The most feedback items in one sign-off.
pub const MAX_ITEMS: usize = 20;

/// From a sign-off file to the repo root.
const SIGNOFF_TO_ROOT: &str = "../../../";

/// What the reviewer concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// Reviewed, nothing that must change.
    Reviewed,
    /// Reviewed, and the feedback asks for changes.
    Changes,
}

impl Verdict {
    /// The front matter value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Reviewed => "reviewed",
            Verdict::Changes => "changes",
        }
    }

    /// How pane shows it.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Reviewed => "Reviewed",
            Verdict::Changes => "Reviewed, needs changes",
        }
    }

    fn parse(s: &str) -> Option<Verdict> {
        match s {
            "reviewed" => Some(Verdict::Reviewed),
            "changes" => Some(Verdict::Changes),
            _ => None,
        }
    }
}

/// One feedback item of a sign-off.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feedback {
    /// The slide it is about, from 1.
    #[serde(default)]
    pub slide: Option<u32>,
    pub text: String,
    /// The issue file it was filed as, from the repo root.
    #[serde(default)]
    pub issue: Option<String>,
    /// Whether that issue file is still in the repo. The snapshot sets it;
    /// a deleted issue file means the item is done.
    #[serde(default)]
    pub open: bool,
}

/// A reviewer's sign-off on one deck.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignOff {
    /// The reviewer's public handle, never an email or account id.
    pub reviewer: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    pub verdict: Verdict,
    /// The file, from the repo root.
    pub path: String,
    #[serde(default)]
    pub feedback: Vec<Feedback>,
}

impl SignOff {
    /// Feedback items whose issue files are still open.
    #[must_use]
    pub fn open_items(&self) -> usize {
        self.feedback.iter().filter(|f| f.open).count()
    }
}

/// What a reviewer submits for one deck.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Form {
    pub verdict: Verdict,
    #[serde(default)]
    pub feedback: Vec<FormItem>,
    /// File the feedback as backlog rather than priority work.
    #[serde(default)]
    pub backlog: bool,
}

/// One feedback item as the reviewer typed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormItem {
    #[serde(default)]
    pub slide: Option<u32>,
    pub text: String,
}

/// A file to write: its path from the repo root, and its contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub path: String,
    pub contents: String,
}

/// Whether `handle` is a valid reviewer handle: 1 to 39 lowercase letters,
/// digits or single hyphens, not starting or ending with a hyphen (GitHub's
/// login rules, so a GitHub login always works).
#[must_use]
pub fn valid_handle(handle: &str) -> bool {
    !handle.is_empty()
        && handle.len() <= 39
        && !handle.starts_with('-')
        && !handle.ends_with('-')
        && !handle.contains("--")
        && handle
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Whether `date` looks like `YYYY-MM-DD`.
fn valid_date(date: &str) -> bool {
    let b = date.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
}

/// The sign-off file for `reviewer` on `date` in the deck folder `folder`.
#[must_use]
pub fn signoff_path(folder: &str, date: &str, reviewer: &str) -> String {
    format!("docs/reviews/{folder}/{SIGNOFFS_DIR}/{date}-{reviewer}.md")
}

/// Whether `path` is a sign-off file, and if so its deck folder.
#[must_use]
pub fn signoff_folder(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("docs/reviews/")?;
    let mut parts = rest.split('/');
    let folder = parts.next()?;
    let dir = parts.next()?;
    let name = parts.next()?;
    (dir == SIGNOFFS_DIR
        && parts.next().is_none()
        && name.ends_with(".md")
        && name != "README.md"
        && !folder.is_empty())
    .then_some(folder)
}

/// The `issues/` folder feedback on `review` goes to: the project's own
/// when it is one of `projects`, else the root one.
#[must_use]
pub fn issues_dir(review: &Review, projects: &[String]) -> String {
    if projects.contains(&review.project) {
        format!("{}/issues", review.project)
    } else {
        "issues".to_owned()
    }
}

/// One line of text: newlines and runs of spaces become one space.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The sign-off and one issue file per feedback item for `form`, by
/// `reviewer` on `date`. `projects` are the repo's project paths.
///
/// # Errors
///
/// When the handle or date is invalid, there are too many items, an item
/// is empty or too long, or a "needs changes" verdict has no feedback.
pub fn files(
    review: &Review,
    projects: &[String],
    reviewer: &str,
    date: &str,
    form: &Form,
) -> Result<Vec<File>, String> {
    if !valid_handle(reviewer) {
        return Err(format!("`{reviewer}` is not a reviewer handle"));
    }
    if !valid_date(date) {
        return Err(format!("`{date}` is not a YYYY-MM-DD date"));
    }
    if form.feedback.len() > MAX_ITEMS {
        return Err(format!("at most {MAX_ITEMS} feedback items"));
    }
    if form.verdict == Verdict::Changes && form.feedback.is_empty() {
        return Err("say what needs to change".to_owned());
    }
    let folder = review.folder();
    let path = signoff_path(folder, date, reviewer);
    let dir = issues_dir(review, projects);
    let up = "../".repeat(dir.split('/').count());
    let priority = if form.backlog { "backlog" } else { "priority" };

    let mut out = Vec::new();
    let mut items = Vec::new();
    for (i, item) in form.feedback.iter().enumerate() {
        let text = one_line(&item.text);
        if text.is_empty() {
            return Err(format!("feedback item {} is empty", i + 1));
        }
        if text.chars().count() > MAX_ITEM_CHARS {
            return Err(format!(
                "feedback item {} is over {MAX_ITEM_CHARS} characters",
                i + 1
            ));
        }
        let issue = format!("{dir}/review-{folder}-{reviewer}-{}.md", i + 1);
        let slide = item
            .slide
            .map(|n| format!(" (slide {n})"))
            .unwrap_or_default();
        out.push(File {
            path: issue.clone(),
            contents: format!(
                "---\nkind: todo\npriority: {priority}\nepic: {FEEDBACK_EPIC}\nadded: {date}\n---\n\n\
                 Review feedback from {reviewer} on the [{project} deep review]({up}{deck}){slide}, \
                 in their [sign-off]({up}{path}):\n\n> {text}\n",
                project = review.project,
                deck = review.path,
            ),
        });
        items.push(Feedback {
            slide: item.slide,
            text,
            issue: Some(issue),
            open: true,
        });
    }
    let signoff = SignOff {
        reviewer: reviewer.to_owned(),
        date: date.to_owned(),
        verdict: form.verdict,
        path,
        feedback: items,
    };
    out.push(File {
        path: signoff.path.clone(),
        contents: render(&signoff),
    });
    Ok(out)
}

/// The file for `signoff`.
#[must_use]
pub fn render(signoff: &SignOff) -> String {
    let mut out = format!(
        "---\nreviewer: {}\ndate: {}\nverdict: {}\n---\n\n",
        signoff.reviewer,
        signoff.date,
        signoff.verdict.as_str()
    );
    if signoff.feedback.is_empty() {
        out.push_str("No feedback.\n");
    }
    for item in &signoff.feedback {
        out.push_str("- ");
        if let Some(n) = item.slide {
            out.push_str(&format!("Slide {n}: "));
        }
        out.push_str(&one_line(&item.text));
        if let Some(issue) = &item.issue {
            out.push_str(&format!(" ([issue]({SIGNOFF_TO_ROOT}{issue}))"));
        }
        out.push('\n');
    }
    out
}

/// Read the sign-off file at `path` (from the repo root). Every item starts
/// open; the snapshot closes the ones whose issue files are gone.
///
/// # Errors
///
/// When the front matter is missing, a field is missing or invalid, or the
/// reviewer and date don't match the file name.
pub fn parse(path: &str, contents: &str) -> Result<SignOff, String> {
    let err = |msg: &str| format!("{path}: {msg}");
    let rest = contents
        .strip_prefix("---\n")
        .ok_or_else(|| err("no front matter"))?;
    let (front, body) = rest
        .split_once("\n---\n")
        .ok_or_else(|| err("front matter isn't closed"))?;
    let (mut reviewer, mut date, mut verdict) = (None, None, None);
    for line in front.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_owned();
        match key.trim() {
            "reviewer" => reviewer = Some(value),
            "date" => date = Some(value),
            "verdict" => verdict = Some(value),
            other => return Err(err(&format!("unknown field `{other}`"))),
        }
    }
    let reviewer = reviewer.ok_or_else(|| err("no reviewer"))?;
    let date = date.ok_or_else(|| err("no date"))?;
    let verdict = verdict.ok_or_else(|| err("no verdict"))?;
    if !valid_handle(&reviewer) {
        return Err(err(&format!("`{reviewer}` is not a reviewer handle")));
    }
    if !valid_date(&date) {
        return Err(err(&format!("`{date}` is not a YYYY-MM-DD date")));
    }
    let verdict = Verdict::parse(&verdict)
        .ok_or_else(|| err(&format!("verdict `{verdict}` is not reviewed or changes")))?;
    let name = path.rsplit('/').next().unwrap_or(path);
    if name != format!("{date}-{reviewer}.md") {
        return Err(err("the file name isn't <date>-<reviewer>.md"));
    }
    let feedback = body
        .lines()
        .filter_map(|l| l.strip_prefix("- "))
        .map(parse_item)
        .collect();
    Ok(SignOff {
        reviewer,
        date,
        verdict,
        path: path.to_owned(),
        feedback,
    })
}

fn parse_item(line: &str) -> Feedback {
    let mut text = line.trim();
    let mut issue = None;
    let marker = format!(" ([issue]({SIGNOFF_TO_ROOT}");
    if let Some(at) = text.rfind(&marker)
        && let Some(path) = text[at + marker.len()..].strip_suffix("))")
    {
        issue = Some(path.to_owned());
        text = &text[..at];
    }
    let mut slide = None;
    if let Some(rest) = text.strip_prefix("Slide ")
        && let Some((n, after)) = rest.split_once(": ")
        && let Ok(n) = n.parse()
    {
        slide = Some(n);
        text = after;
    }
    Feedback {
        slide,
        text: text.to_owned(),
        issue,
        open: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn review() -> Review {
        Review {
            project: "cookbook".into(),
            date: "2026-11-25".into(),
            path: "docs/reviews/2026-11-25-cookbook/deck.md".into(),
            pr: None,
            scorecard: vec![],
            signoffs: vec![],
        }
    }

    fn projects() -> Vec<String> {
        vec!["cookbook".into(), "sparkhill/webapp".into()]
    }

    fn form() -> Form {
        Form {
            verdict: Verdict::Changes,
            feedback: vec![
                FormItem {
                    slide: Some(4),
                    text: "Say which free tier\nis nearest its limit.".into(),
                },
                FormItem {
                    slide: None,
                    text: "Add a retro.".into(),
                },
            ],
            backlog: false,
        }
    }

    #[test]
    fn a_form_becomes_one_issue_per_item_and_a_signoff() -> Result<(), String> {
        let out = files(&review(), &projects(), "jim", "2026-11-28", &form())?;
        let paths: Vec<_> = out.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "cookbook/issues/review-2026-11-25-cookbook-jim-1.md",
                "cookbook/issues/review-2026-11-25-cookbook-jim-2.md",
                "docs/reviews/2026-11-25-cookbook/signoffs/2026-11-28-jim.md",
            ]
        );
        assert_eq!(
            out[0].contents,
            "---\nkind: todo\npriority: priority\nepic: Review feedback\nadded: 2026-11-28\n---\n\n\
             Review feedback from jim on the [cookbook deep review](../../docs/reviews/2026-11-25-cookbook/deck.md) (slide 4), \
             in their [sign-off](../../docs/reviews/2026-11-25-cookbook/signoffs/2026-11-28-jim.md):\n\n\
             > Say which free tier is nearest its limit.\n"
        );
        assert_eq!(
            out[2].contents,
            "---\nreviewer: jim\ndate: 2026-11-28\nverdict: changes\n---\n\n\
             - Slide 4: Say which free tier is nearest its limit. ([issue](../../../cookbook/issues/review-2026-11-25-cookbook-jim-1.md))\n\
             - Add a retro. ([issue](../../../cookbook/issues/review-2026-11-25-cookbook-jim-2.md))\n"
        );
        Ok(())
    }

    #[test]
    fn a_signoff_reads_back_what_files_wrote() -> Result<(), String> {
        let out = files(&review(), &projects(), "jim", "2026-11-28", &form())?;
        let s = parse(&out[2].path, &out[2].contents)?;
        assert_eq!(s.reviewer, "jim");
        assert_eq!(s.verdict, Verdict::Changes);
        assert_eq!(s.feedback.len(), 2);
        assert_eq!(s.feedback[0].slide, Some(4));
        assert_eq!(
            s.feedback[0].text,
            "Say which free tier is nearest its limit."
        );
        assert_eq!(
            s.feedback[1].issue.as_deref(),
            Some("cookbook/issues/review-2026-11-25-cookbook-jim-2.md")
        );
        assert_eq!(s.open_items(), 2);
        assert_eq!(render(&s), out[2].contents);
        Ok(())
    }

    #[test]
    fn feedback_on_an_unknown_project_goes_to_the_root_issues() -> Result<(), String> {
        let mut r = review();
        r.project = "gone".into();
        r.path = "docs/reviews/2026-11-25-gone/deck.md".into();
        let mut f = form();
        f.backlog = true;
        let out = files(&r, &projects(), "jim", "2026-11-28", &f)?;
        assert_eq!(out[0].path, "issues/review-2026-11-25-gone-jim-1.md");
        assert!(out[0].contents.contains("priority: backlog"));
        assert!(
            out[0]
                .contents
                .contains("](../docs/reviews/2026-11-25-gone/deck.md)")
        );
        Ok(())
    }

    #[test]
    fn a_nested_project_links_up_far_enough() -> Result<(), String> {
        let mut r = review();
        r.project = "sparkhill/webapp".into();
        let out = files(&r, &projects(), "jim", "2026-11-28", &form())?;
        assert!(out[0].path.starts_with("sparkhill/webapp/issues/"));
        assert!(out[0].contents.contains("](../../../docs/reviews/"));
        Ok(())
    }

    #[test]
    fn a_plain_review_needs_no_feedback() -> Result<(), String> {
        let f = Form {
            verdict: Verdict::Reviewed,
            feedback: vec![],
            backlog: false,
        };
        let out = files(&review(), &projects(), "jim", "2026-11-28", &f)?;
        assert_eq!(out.len(), 1);
        assert!(out[0].contents.ends_with("---\n\nNo feedback.\n"));
        assert!(parse(&out[0].path, &out[0].contents)?.feedback.is_empty());
        Ok(())
    }

    #[test]
    fn bad_forms_are_refused() {
        let r = review();
        let p = projects();
        assert!(files(&r, &p, "Jim@x", "2026-11-28", &form()).is_err());
        assert!(files(&r, &p, "jim", "28/11/2026", &form()).is_err());
        let mut f = form();
        f.feedback.clear();
        assert!(
            files(&r, &p, "jim", "2026-11-28", &f).is_err(),
            "changes need feedback"
        );
        let mut f = form();
        f.feedback[1].text = " \n ".into();
        assert!(
            files(&r, &p, "jim", "2026-11-28", &f).is_err(),
            "empty item"
        );
        let mut f = form();
        f.feedback[0].text = "x".repeat(MAX_ITEM_CHARS + 1);
        assert!(files(&r, &p, "jim", "2026-11-28", &f).is_err(), "too long");
        let mut f = form();
        f.feedback = vec![f.feedback[1].clone(); MAX_ITEMS + 1];
        assert!(files(&r, &p, "jim", "2026-11-28", &f).is_err(), "too many");
    }

    #[test]
    fn bad_signoff_files_are_refused() {
        let p = "docs/reviews/2026-11-25-cookbook/signoffs/2026-11-28-jim.md";
        assert!(parse(p, "no front matter").is_err());
        assert!(parse(p, "---\nreviewer: jim\ndate: 2026-11-28\n---\n").is_err());
        assert!(
            parse(
                p,
                "---\nreviewer: jim\ndate: 2026-11-28\nverdict: maybe\n---\n"
            )
            .is_err()
        );
        assert!(
            parse(
                p,
                "---\nreviewer: ann\ndate: 2026-11-28\nverdict: reviewed\n---\n"
            )
            .is_err(),
            "name must match"
        );
        assert!(
            parse(
                p,
                "---\nreviewer: jim\ndate: 2026-11-28\nverdict: reviewed\nemail: x\n---\n"
            )
            .is_err(),
            "no other fields, so no email sneaks in"
        );
    }

    #[test]
    fn handles_follow_github_login_rules() {
        assert!(valid_handle("jim"));
        assert!(valid_handle("jimhillr-agt"));
        assert!(!valid_handle(""));
        assert!(!valid_handle("-jim"));
        assert!(!valid_handle("jim-"));
        assert!(!valid_handle("j--im"));
        assert!(!valid_handle("Jim"));
        assert!(!valid_handle("jim@example.com"));
        assert!(!valid_handle(&"a".repeat(40)));
    }

    #[test]
    fn finds_signoff_files() {
        assert_eq!(
            signoff_folder("docs/reviews/2026-11-25-cookbook/signoffs/2026-11-28-jim.md"),
            Some("2026-11-25-cookbook")
        );
        assert_eq!(
            signoff_folder("docs/reviews/2026-11-25-cookbook/deck.md"),
            None
        );
        assert_eq!(signoff_folder("docs/reviews/x/signoffs/README.md"), None);
        assert_eq!(signoff_folder("docs/reviews/x/signoffs/a/b.md"), None);
        assert_eq!(signoff_folder("pane/docs/reviews/x/signoffs/a.md"), None);
    }
}
