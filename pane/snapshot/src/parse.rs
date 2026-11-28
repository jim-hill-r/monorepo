//! Pure parsers over file contents. Nothing here touches the file system, so
//! each one is tested against strings and fixture files.

use pane_model::{
    Deliverable, DeliverableKind, HumanIssue, Link, ProjectStatus, REPO_URL, Retro, RetroAction,
    Review, ScorecardRow, WorkItem, WorkKind, WorkSource,
};

/// The project paths in the root `mise.toml`'s `[monorepo].config_roots`.
pub fn config_roots(root_mise: &str) -> Result<Vec<String>, String> {
    let value: toml::Table = toml::from_str(root_mise).map_err(|e| e.to_string())?;
    let roots = value
        .get("monorepo")
        .and_then(|m| m.get("config_roots"))
        .and_then(|r| r.as_array())
        .ok_or("root mise.toml has no [monorepo].config_roots")?;
    roots
        .iter()
        .map(|r| {
            r.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "config_roots holds a value that is not a string".to_owned())
        })
        .collect()
}

/// The project type: `<type>` from its `standards/tasks/<type>.toml` include.
pub fn project_kind(project_mise: &str) -> Option<String> {
    const MARKER: &str = "standards/tasks/";
    let start = project_mise.find(MARKER)? + MARKER.len();
    let rest = &project_mise[start..];
    let end = rest.find(".toml")?;
    let kind = &rest[..end];
    let valid = !kind.is_empty()
        && kind
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    valid.then(|| kind.to_owned())
}

/// `DEPLOY_URL` from a project's `[env]` table.
pub fn deploy_url(project_mise: &str) -> Option<String> {
    let value: toml::Table = toml::from_str(project_mise).ok()?;
    value
        .get("env")?
        .get("DEPLOY_URL")?
        .as_str()
        .map(str::to_owned)
}

/// `RELEASE_TAG` from a project's `[env]` table: the GitHub Release its
/// nightly downloads go to (see "Game builds" in `standards/docs/mise.md`).
pub fn release_tag(project_mise: &str) -> Option<String> {
    let value: toml::Table = toml::from_str(project_mise).ok()?;
    value
        .get("env")?
        .get("RELEASE_TAG")?
        .as_str()
        .map(str::to_owned)
}

/// `[package].name` from a `Cargo.toml`.
pub fn package_name(cargo_toml: &str) -> Option<String> {
    let value: toml::Table = toml::from_str(cargo_toml).ok()?;
    value
        .get("package")?
        .get("name")?
        .as_str()
        .map(str::to_owned)
}

/// A project's production site as a deliverable, titled for the app it
/// deploys: `cookbook/cloudflare` shows as `cookbook`.
pub fn live_site(project: &str, url: &str) -> Deliverable {
    Deliverable {
        kind: DeliverableKind::LiveSite,
        title: project
            .strip_suffix("/cloudflare")
            .unwrap_or(project)
            .to_owned(),
        project: Some(project.to_owned()),
        path: None,
        url: Some(url.to_owned()),
        date: None,
        status: None,
        downloads: Vec::new(),
        body: None,
    }
}

/// A game's nightly GitHub Release, with its macOS and Android downloads at
/// the fixed asset URLs `nightly-downloads.yml` publishes:
/// `<repo>/releases/download/<tag>/<package>-macos.zip` and `-android.apk`.
pub fn release(project: &str, tag: &str, package: &str) -> Deliverable {
    let base = format!("{REPO_URL}/releases/download/{tag}");
    Deliverable {
        kind: DeliverableKind::Release,
        title: format!("{project} (nightly)"),
        project: Some(project.to_owned()),
        path: None,
        url: Some(format!("{REPO_URL}/releases/tag/{tag}")),
        date: None,
        status: None,
        downloads: vec![
            Link {
                label: "macOS".to_owned(),
                url: format!("{base}/{package}-macos.zip"),
            },
            Link {
                label: "Android".to_owned(),
                url: format!("{base}/{package}-android.apk"),
            },
        ],
        body: None,
    }
}

/// Where the review decks and their rotation table live.
pub const REVIEWS_DIR: &str = "docs/reviews";

/// The reviews in the `## Rotation` table of `docs/reviews/README.md`
/// (`| Date | Project | Deck | PR |`), newest first. Columns are found by
/// their header, so the table may gain columns. A row without a deck link
/// points at `<date>-<project>/deck.md`; a PR cell without a link to GitHub
/// gives no PR. Rows without a date and project are skipped.
pub fn reviews(readme: &str) -> Vec<Review> {
    let mut found = Vec::new();
    let mut in_rotation = false;
    let mut header: Option<Vec<String>> = None;
    for line in readme.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            in_rotation = h.trim() == "Rotation";
            header = None;
            continue;
        }
        let line = line.trim();
        if !in_rotation || !line.starts_with('|') {
            if header.is_some() && !line.starts_with('|') {
                // The table has ended.
                in_rotation = false;
            }
            continue;
        }
        let cells = table_cells(line);
        let Some(columns) = &header else {
            header = Some(cells.iter().map(|c| c.to_lowercase()).collect());
            continue;
        };
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
        {
            continue;
        }
        let cell = |name: &str| {
            columns
                .iter()
                .position(|c| c == name)
                .and_then(|i| cells.get(i))
                .map(String::as_str)
                .unwrap_or("")
        };
        let (date, project) = (cell("date"), cell("project").trim_matches('`'));
        if leading_date(date) != Some(date) || project.is_empty() {
            continue;
        }
        let deck = link_target(cell("deck"))
            .map(|t| t.trim_start_matches("./").to_owned())
            .unwrap_or_else(|| format!("{date}-{project}/deck.md"));
        let pr = link_target(cell("pr"))
            .filter(|t| t.starts_with(REPO_URL))
            .map(str::to_owned);
        found.push(Review {
            project: project.to_owned(),
            date: date.to_owned(),
            path: format!("{REVIEWS_DIR}/{deck}"),
            pr,
            scorecard: Vec::new(),
            signoffs: Vec::new(),
        });
    }
    // Newest first, keeping the table's order within a day.
    found.sort_by(|a, b| b.date.cmp(&a.date));
    found
}

/// The rows of a deck's Scorecard slide: each `| Area | Rating | Why |`
/// table under a heading that mentions "Scorecard" (a long scorecard may
/// continue on a second slide). Columns are found by their header, so their
/// order may change and the table may gain columns; a table without both an
/// Area and a Rating column is not a scorecard. The slide ends at the next
/// heading or `---` slide break. Backticks and `**` are dropped from the
/// cells, since pane shows them as plain text. Rows without an area or a
/// rating are skipped.
pub fn scorecard(deck: &str) -> Vec<ScorecardRow> {
    let mut found = Vec::new();
    let mut on_slide = false;
    let mut header: Option<Vec<String>> = None;
    for line in deck.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            on_slide = line.to_lowercase().contains("scorecard");
            header = None;
            continue;
        }
        if line == "---" {
            on_slide = false;
            header = None;
            continue;
        }
        if !on_slide {
            continue;
        }
        if !line.starts_with('|') {
            header = None;
            continue;
        }
        let cells: Vec<String> = table_cells(line).iter().map(|c| plain(c)).collect();
        let Some(columns) = &header else {
            header = Some(cells.iter().map(|c| c.to_lowercase()).collect());
            continue;
        };
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
        {
            continue;
        }
        let cell = |name: &str| {
            columns
                .iter()
                .position(|c| c == name)
                .and_then(|i| cells.get(i))
                .cloned()
        };
        let (Some(area), Some(rating)) = (cell("area"), cell("rating")) else {
            continue;
        };
        if area.is_empty() || rating.is_empty() {
            continue;
        }
        found.push(ScorecardRow {
            area,
            rating,
            why: cell("why").unwrap_or_default(),
        });
    }
    found
}

/// `cell` without Markdown's backticks and bold markers.
fn plain(cell: &str) -> String {
    cell.replace("**", "").replace('`', "").trim().to_owned()
}

/// The trimmed cells of a Markdown table row.
fn table_cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|');
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    inner.split('|').map(|c| c.trim().to_owned()).collect()
}

/// The target of the first `[text](target)` link in `cell`.
fn link_target(cell: &str) -> Option<&str> {
    let start = cell.find("](")? + 2;
    let rest = &cell[start..];
    let target = rest[..rest.find(')')?].trim();
    (!target.is_empty()).then_some(target)
}

/// A review as a deliverable: its slides at `/reviews/<folder>/deck.html`
/// in pane, and its PDF at `/reviews/<folder>/deck.pdf` (pane's release
/// builds both with `standards/scripts/review-decks.sh`).
pub fn review_deliverable(review: &Review) -> Deliverable {
    let base = format!("/reviews/{}", review.folder());
    Deliverable {
        kind: DeliverableKind::ReviewDeck,
        title: format!("{} deep review", review.project),
        project: Some(review.project.clone()),
        path: Some(review.path.clone()),
        url: Some(format!("{base}/deck.html")),
        date: Some(review.date.clone()),
        status: None,
        downloads: vec![Link {
            label: "PDF".to_owned(),
            url: format!("{base}/deck.pdf"),
        }],
        body: None,
    }
}

/// The first paragraph of a README that is not a heading, badge, quote,
/// list, table or code block, joined onto one line.
pub fn readme_summary(readme: &str) -> Option<String> {
    let mut paragraph: Vec<&str> = Vec::new();
    let mut in_code = false;
    for line in readme.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            in_code = !in_code;
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        if in_code {
            continue;
        }
        let skip = t.starts_with('#')
            || t.starts_with('>')
            || t.starts_with("[![")
            || t.starts_with("![")
            || t.starts_with('<')
            || t.starts_with("- ")
            || t.starts_with("* ")
            || t.starts_with('|');
        if t.is_empty() || skip {
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        paragraph.push(t);
    }
    (!paragraph.is_empty()).then(|| paragraph.join(" "))
}

/// Projects listed under `# Priority Projects` and `# On Hold Projects` in
/// the root `issues/README.md`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RootLists {
    pub priority: Vec<String>,
    pub on_hold: Vec<String>,
}

impl RootLists {
    /// A project's status. A sub-project (`pane/web`) takes the status of
    /// the top-level project it lives in unless it is listed itself. On hold
    /// wins over priority.
    pub fn status_of(&self, path: &str) -> ProjectStatus {
        let top = path.split('/').next().unwrap_or(path);
        let listed = |list: &[String]| list.iter().any(|p| p == path || p == top);
        if listed(&self.on_hold) {
            ProjectStatus::OnHold
        } else if listed(&self.priority) {
            ProjectStatus::Priority
        } else {
            ProjectStatus::Active
        }
    }
}

pub fn root_lists(root_issues: &str) -> RootLists {
    let mut lists = RootLists::default();
    let mut current: Option<&mut Vec<String>> = None;
    for line in root_issues.lines() {
        if let Some(h) = line.strip_prefix("# ") {
            current = match h.trim() {
                "Priority Projects" => Some(&mut lists.priority),
                "On Hold Projects" => Some(&mut lists.on_hold),
                _ => None,
            };
            continue;
        }
        if let (Some(list), Some(item)) = (current.as_deref_mut(), line.trim().strip_prefix("- ")) {
            let name = item
                .trim()
                .trim_matches(|c| c == '[' || c == ']' || c == '`');
            if !name.is_empty() {
                list.push(name.to_owned());
            }
        }
    }
    lists
}

/// A `TODO` or `FIX` marker at the start of `text`, with its tags and the
/// text after them: `TODO (agent-generated, agent-ignore): Do it` gives
/// `(Todo, ["agent-generated", "agent-ignore"], "Do it")`.
pub fn parse_marker(text: &str) -> Option<(WorkKind, Vec<String>, String)> {
    let (kind, rest) = if let Some(r) = text.strip_prefix("TODO") {
        (WorkKind::Todo, r)
    } else {
        let r = text.strip_prefix("FIX")?;
        (WorkKind::Fix, r)
    };
    // The marker must end there: `TODOS` or `FIXME` are other words.
    if rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let mut rest = rest.trim_start();
    let mut tags = Vec::new();
    if let Some(inner) = rest.strip_prefix('(') {
        let close = inner.find(')')?;
        tags = inner[..close]
            .split(',')
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
            .collect();
        rest = inner[close + 1..].trim_start();
    }
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();
    Some((kind, tags, rest.to_owned()))
}

/// What kind of issue file a repo path is, if any: a file in an `issues/`
/// folder (`issues/<slug>.md` or `<project>/issues/<slug>.md`), or a human
/// task in the root `issues/human/`. Each folder's `README.md` is notes, not
/// an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueFile {
    Work,
    Human,
}

pub fn issue_file_kind(rel: &str) -> Option<IssueFile> {
    let (dir, name) = rel.rsplit_once('/')?;
    if name == "README.md" || !name.ends_with(".md") {
        return None;
    }
    if dir == "issues/human" {
        Some(IssueFile::Human)
    } else if dir == "issues" || dir.ends_with("/issues") {
        Some(IssueFile::Work)
    } else {
        None
    }
}

/// An issue file split into its front matter and body.
#[derive(Debug, PartialEq, Eq)]
pub struct FrontMatter {
    /// The `key: value` lines, with quotes taken off quoted values.
    pub fields: Vec<(String, String)>,
    /// The text after the front matter, whitespace collapsed to single spaces.
    pub body: String,
    /// The 1-based line the body starts on.
    pub line: u32,
}

/// Split an issue file into the `key: value` lines between a `---` first
/// line and the next `---`, and the body after them. `None` when the file
/// has no front matter.
pub fn front_matter(contents: &str) -> Option<FrontMatter> {
    let mut lines = contents.lines().enumerate();
    if lines.next()?.1.trim_end() != "---" {
        return None;
    }
    let mut fields = Vec::new();
    let mut body_start = None;
    for (i, line) in lines.by_ref() {
        if line.trim_end() == "---" {
            body_start = Some(i + 1);
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            fields.push((key.trim().to_owned(), unquote(value.trim())));
        }
    }
    let body_start = body_start?;
    let mut body_line = None;
    let mut words = Vec::new();
    for (i, line) in lines {
        if body_line.is_none() && !line.trim().is_empty() {
            body_line = Some(i + 1);
        }
        words.extend(line.split_whitespace());
    }
    let line = body_line.unwrap_or(body_start + 1);
    Some(FrontMatter {
        fields,
        body: words.join(" "),
        line: u32::try_from(line).unwrap_or(u32::MAX),
    })
}

fn unquote(value: &str) -> String {
    match value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        Some(inner) => inner.replace("\\\"", "\"").replace("\\\\", "\\"),
        None => value.to_owned(),
    }
}

fn field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .filter(|v| !v.is_empty())
}

/// One agent issue file (`<project>/issues/<slug>.md`). Its front matter
/// gives the kind (`todo` or `fix`), the section (`priority` is "Priority
/// Issues", `backlog` is "Backlog"), the epic and the tags (`[a, b]`); the
/// body is the text. `None` when the front matter is missing or its kind is
/// not `todo` or `fix`.
pub fn issue_file(file: &str, project: Option<&str>, contents: &str) -> Option<WorkItem> {
    let FrontMatter {
        fields,
        body: text,
        line,
    } = front_matter(contents)?;
    let kind = match field(&fields, "kind")? {
        "todo" => WorkKind::Todo,
        "fix" => WorkKind::Fix,
        _ => return None,
    };
    let section = field(&fields, "priority").map(|p| match p {
        "priority" => "Priority Issues".to_owned(),
        "backlog" => "Backlog".to_owned(),
        other => other.to_owned(),
    });
    let tags = field(&fields, "tags")
        .map(|t| {
            t.trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|t| t.trim().to_owned())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Some(WorkItem {
        kind,
        text,
        project: project.map(str::to_owned),
        file: file.to_owned(),
        line,
        source: WorkSource::IssuesFile,
        section,
        epic: field(&fields, "epic").map(str::to_owned),
        tags,
        added: field(&fields, "added")
            .and_then(leading_date)
            .map(str::to_owned),
    })
}

/// The project a repo path belongs to: the longest config root that
/// contains it, or `None` for files outside every project.
pub fn owning_project<'a>(path: &str, roots: &'a [String]) -> Option<&'a str> {
    roots
        .iter()
        .filter(|r| path == r.as_str() || path.starts_with(&format!("{r}/")))
        .max_by_key(|r| r.len())
        .map(String::as_str)
}

/// One human task file (`issues/human/<slug>.md`): the front matter's
/// `added` (a `YYYY-MM-DD` date), `unblocks` and `runbook` give its age,
/// what it unblocks and its runbook. The text is the body's
/// `## Requirement` section, and `why` its `## Why it can't be automated`
/// section (ISS-004 in `standards/scripts/issues.sh`); a body without them
/// is all text. `None` without front matter.
pub fn human_issue_file(file: &str, contents: &str) -> Option<HumanIssue> {
    let FrontMatter { fields, body, line } = front_matter(contents)?;
    let (text, why) = human_sections(&body);
    Some(HumanIssue {
        text,
        file: file.to_owned(),
        line,
        added: field(&fields, "added")
            .and_then(leading_date)
            .map(str::to_owned),
        unblocks: field(&fields, "unblocks").map(|u| u.trim_end_matches('.').to_owned()),
        why,
        runbook: field(&fields, "runbook").map(str::to_owned),
    })
}

/// Splits a human task's body (words joined by spaces) into its
/// requirement and why sections; the `## Runbook` section is dropped, since
/// the `runbook` field gives the link.
fn human_sections(body: &str) -> (String, Option<String>) {
    const REQUIREMENT: &str = "## Requirement";
    const WHY: &str = "## Why it can't be automated";
    const RUNBOOK: &str = "## Runbook";
    let Some(rest) = body.strip_prefix(REQUIREMENT) else {
        return (body.to_owned(), None);
    };
    let (rest, _) = rest.split_once(RUNBOOK).unwrap_or((rest, ""));
    let (text, why) = match rest.split_once(WHY) {
        Some((text, why)) => (text, Some(why.trim().to_owned())),
        None => (rest, None),
    };
    (text.trim().to_owned(), why.filter(|w| !w.is_empty()))
}

/// A `YYYY-MM-DD` date at the start of `s`.
fn leading_date(s: &str) -> Option<&str> {
    let date = s.get(..10)?;
    let ok = date.char_indices().all(|(i, c)| match i {
        4 | 7 => c == '-',
        _ => c.is_ascii_digit(),
    });
    ok.then_some(date)
}

/// A `YYYY-MM-DD` or `YYYY-MM` date at the start of `s`.
fn leading_date_or_month(s: &str) -> Option<&str> {
    leading_date(s).or_else(|| {
        let month = s.get(..7)?;
        let ok = month.char_indices().all(|(i, c)| match i {
            4 => c == '-',
            _ => c.is_ascii_digit(),
        }) && !s[7..].starts_with(|c: char| c.is_ascii_digit());
        ok.then_some(month)
    })
}

/// File extensions scanned for `TODO`/`FIX` comments.
pub const CODE_EXTENSIONS: &[&str] = &[
    "rs", "toml", "sh", "bash", "yml", "yaml", "ts", "tsx", "js", "mjs", "py", "css", "scss",
    "html", "sql",
];

/// Comment openers a `TODO`/`FIX` may follow.
const COMMENT_MARKERS: &[&str] = &["//", "#", "/*", "<!--", "--", ";", "*"];

/// Every `TODO`/`FIX` that follows a comment opener on a line of code, such
/// as `// TODO (agent-generated): ...` or `x = 1 # FIX: ...`.
pub fn code_comments(file: &str, project: Option<&str>, contents: &str) -> Vec<WorkItem> {
    let mut items = Vec::new();
    for (i, line) in contents.lines().enumerate() {
        if let Some((kind, tags, text)) = comment_marker(line) {
            items.push(WorkItem {
                kind,
                text,
                project: project.map(str::to_owned),
                file: file.to_owned(),
                line: u32::try_from(i + 1).unwrap_or(u32::MAX),
                source: WorkSource::CodeComment,
                section: None,
                epic: None,
                tags,
                // repo::build fills a FIX's date in from git blame.
                added: None,
            });
        }
    }
    items
}

fn comment_marker(line: &str) -> Option<(WorkKind, Vec<String>, String)> {
    // Openers inside a "string" or `code span` don't start a comment.
    let mut in_string = false;
    let mut in_code = false;
    let mut escaped = false;
    for (at, c) in line.char_indices() {
        if in_string {
            match (escaped, c) {
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => escaped = false,
            }
            continue;
        }
        match c {
            '"' if !in_code => {
                in_string = true;
                continue;
            }
            '`' => {
                in_code = !in_code;
                continue;
            }
            _ if in_code => continue,
            _ => {}
        }
        let rest = &line[at..];
        let Some(marker) = COMMENT_MARKERS.iter().find(|m| rest.starts_with(**m)) else {
            continue;
        };
        let after = rest[marker.len()..]
            .trim_start_matches(['/', '!', '*', '#', '-'])
            .trim_start();
        if let Some((kind, tags, text)) = parse_marker(after) {
            let text = text.trim_end_matches("*/").trim_end_matches("-->").trim();
            return Some((kind, tags, text.to_owned()));
        }
    }
    None
}

/// A plan, decision or retro file: its kind comes from the folder it is in
/// (`docs/plans`, `docs/decisions`, `docs/retros`).
pub fn doc_kind(path: &str) -> Option<DeliverableKind> {
    let mut parts = path.rsplit('/');
    let name = parts.next()?;
    let folder = parts.next()?;
    let docs = parts.next()?;
    if docs != "docs" || !name.ends_with(".md") || name == "README.md" {
        return None;
    }
    match folder {
        "plans" => Some(DeliverableKind::Plan),
        "decisions" => Some(DeliverableKind::Decision),
        "retros" => Some(DeliverableKind::Retro),
        _ => None,
    }
}

/// A plan, decision or retro as a deliverable: title from its first `# `
/// heading (else the file name), status from a `- Status:` line, and date
/// from a `- Date:` line, a `Started <date>` line, or the file name.
pub fn doc_deliverable(
    kind: DeliverableKind,
    path: &str,
    project: Option<&str>,
    contents: &str,
) -> Deliverable {
    let name = path.rsplit('/').next().unwrap_or(path);
    let title = contents
        .lines()
        .find_map(|l| l.strip_prefix("# "))
        .map(|t| t.trim().to_owned())
        .unwrap_or_else(|| name.trim_end_matches(".md").to_owned());
    let field = |key: &str| {
        contents.lines().find_map(|l| {
            l.trim_start()
                .strip_prefix("- ")
                .and_then(|l| l.strip_prefix(key))
                .map(|v| v.trim().to_owned())
        })
    };
    let date = field("Date:")
        .and_then(|d| leading_date(&d).map(str::to_owned))
        .or_else(|| {
            contents.lines().find_map(|l| {
                l.strip_prefix("Started ")
                    .and_then(leading_date)
                    .map(str::to_owned)
            })
        })
        .or_else(|| leading_date_or_month(name).map(str::to_owned));
    Deliverable {
        kind,
        title,
        project: project.map(str::to_owned),
        path: Some(path.to_owned()),
        url: None,
        date,
        status: field("Status:"),
        downloads: Vec::new(),
        body: Some(contents.to_owned()),
    }
}

/// A retro's actions: the `- [ ]` and `- [x]` items under `## Actions`.
pub fn retro(path: &str, project: Option<&str>, contents: &str) -> Retro {
    let doc = doc_deliverable(DeliverableKind::Retro, path, project, contents);
    let mut actions = Vec::new();
    let mut in_actions = false;
    for line in contents.lines() {
        if line.starts_with('#') {
            in_actions = line.trim_start_matches('#').trim() == "Actions";
            continue;
        }
        if !in_actions {
            continue;
        }
        let item = line.trim_start();
        let (done, text) = if let Some(t) = item.strip_prefix("- [ ]") {
            (false, t)
        } else if let Some(t) = item
            .strip_prefix("- [x]")
            .or_else(|| item.strip_prefix("- [X]"))
        {
            (true, t)
        } else {
            continue;
        };
        actions.push(RetroAction {
            text: text.trim().to_owned(),
            done,
        });
    }
    Retro {
        project: doc.project,
        title: doc.title,
        date: doc.date.unwrap_or_default(),
        path: path.to_owned(),
        actions,
    }
}

/// Parse `pane/health.toml` into the health rules' thresholds. Missing keys
/// take the plan's starting values; an unknown key is an error.
pub fn health_thresholds(health_toml: &str) -> Result<pane_model::health::Thresholds, String> {
    toml::from_str(health_toml).map_err(|e| format!("pane/health.toml: {e}"))
}

/// The 1-based line of every `[[table]]` header in a TOML file, in order,
/// so each array entry can link to its source line.
pub fn table_lines(text: &str, table: &str) -> Vec<u32> {
    let header = format!("[[{table}]]");
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            let line = line.trim();
            line == header
                || line
                    .strip_prefix(&header)
                    .is_some_and(|rest| rest.trim_start().starts_with('#'))
        })
        .filter_map(|(i, _)| u32::try_from(i + 1).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT_MISE: &str = include_str!("../tests/fixtures/repo/mise.toml.fixture");
    const ROOT_ISSUES: &str = include_str!("../tests/fixtures/repo/issues/README.md.fixture");
    const APP_MISE: &str = include_str!("../tests/fixtures/repo/app/mise.toml.fixture");
    const APP_WEB_MISE: &str = include_str!("../tests/fixtures/repo/app/web/mise.toml.fixture");
    const APP_README: &str = include_str!("../tests/fixtures/repo/app/README.md.fixture");
    const FLAT: &str = include_str!("../tests/fixtures/repo/app/issues/a-flat-item.md.fixture");
    const EPIC: &str = include_str!("../tests/fixtures/repo/app/issues/b-epic-item.md.fixture");
    const PHASE: &str = include_str!("../tests/fixtures/repo/app/issues/c-phase-item.md.fixture");
    const LATER: &str = include_str!("../tests/fixtures/repo/app/issues/d-later.md.fixture");
    const NO_FRONT: &str =
        include_str!("../tests/fixtures/repo/app/issues/e-no-front-matter.md.fixture");

    const HUMAN: &str =
        include_str!("../tests/fixtures/repo/issues/human/do-the-dashboard-thing.md.fixture");
    const HUMAN_NO_NOTE: &str =
        include_str!("../tests/fixtures/repo/issues/human/no-note.md.fixture");
    const CODE: &str = include_str!("../tests/fixtures/repo/app/src/main.rs.fixture");
    const PLAN: &str =
        include_str!("../tests/fixtures/repo/app/docs/plans/2026-11-big-plan.md.fixture");
    const DECISION: &str =
        include_str!("../tests/fixtures/repo/app/docs/decisions/0001-pick-a-thing.md.fixture");
    const RETRO: &str =
        include_str!("../tests/fixtures/repo/app/docs/retros/2026-11-20-launch.md.fixture");

    const REVIEWS: &str = include_str!("../tests/fixtures/repo/docs/reviews/README.md.fixture");

    #[test]
    fn reads_the_review_rotation_newest_first() {
        let found = reviews(REVIEWS);
        let rows: Vec<_> = found
            .iter()
            .map(|r| {
                (
                    r.date.as_str(),
                    r.project.as_str(),
                    r.path.as_str(),
                    r.pr.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                (
                    "2026-11-25",
                    "app",
                    "docs/reviews/2026-11-25-app/deck.md",
                    Some("https://github.com/jimhillr/monorepo/pull/12"),
                ),
                (
                    "2026-11-18",
                    "tool",
                    "docs/reviews/2026-11-18-tool/deck.md",
                    None,
                ),
                (
                    "2026-11-11",
                    "app/web",
                    "docs/reviews/2026-11-11-web/deck.md",
                    None,
                ),
            ]
        );
    }

    #[test]
    fn review_rotation_ignores_other_tables_and_bad_rows() {
        let readme = "## Other\n\n| Date | Project |\n| --- | --- |\n| 2026-01-01 | x |\n\n## Rotation\n\n| Project | Date |\n| --- | --- |\n| y | 2026-02-02 |\n| z | soon |\n| | 2026-02-03 |\n\nAfter.\n\n| Date | Project |\n| 2026-03-03 | w |\n";
        let found = reviews(readme);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].project, "y");
        assert_eq!(found[0].path, "docs/reviews/2026-02-02-y/deck.md");
        assert!(reviews("").is_empty());
    }

    const DECK: &str =
        include_str!("../tests/fixtures/repo/docs/reviews/2026-11-25-app/deck.md.fixture");

    #[test]
    fn reads_the_scorecard_slides_of_a_deck() {
        let rows: Vec<_> = scorecard(DECK)
            .into_iter()
            .map(|r| (r.area, r.rating, r.why))
            .collect();
        let want = [
            ("Goals", "Poor", "The core feature is not started"),
            ("User experience", "Fair", "Fast, but main.rs is huge"),
            ("Security", "Good", "Static site"),
            ("Quality", "Good", "CI green"),
            ("Operations", "Fair", ""),
            ("Tech selection", "Fair", ""),
        ]
        .map(|(a, r, w)| (a.to_owned(), r.to_owned(), w.to_owned()));
        assert_eq!(rows, want);
    }

    #[test]
    fn a_deck_without_a_scorecard_has_none() {
        assert!(scorecard("").is_empty());
        assert!(
            scorecard("# Summary\n\n| Area | Rating |\n| --- | --- |\n| A | Good |\n").is_empty()
        );
        // A Scorecard heading over a table without Area and Rating columns.
        assert!(
            scorecard("# Scorecard\n\n| Thing | Score |\n| --- | --- |\n| A | 3 |\n").is_empty()
        );
        // A table that continues past a slide break is not on the slide.
        assert!(
            scorecard("# Scorecard\n\n---\n\n| Area | Rating |\n| --- | --- |\n| A | Good |\n")
                .is_empty()
        );
    }

    #[test]
    fn a_review_deliverable_links_its_slides_and_pdf() {
        let r = Review {
            project: "app".into(),
            date: "2026-11-25".into(),
            path: "docs/reviews/2026-11-25-app/deck.md".into(),
            pr: None,
            scorecard: vec![],
            signoffs: vec![],
        };
        let d = review_deliverable(&r);
        assert_eq!(d.kind, DeliverableKind::ReviewDeck);
        assert_eq!(d.title, "app deep review");
        assert_eq!(d.project.as_deref(), Some("app"));
        assert_eq!(d.date.as_deref(), Some("2026-11-25"));
        assert_eq!(d.url.as_deref(), Some("/reviews/2026-11-25-app/deck.html"));
        assert_eq!(d.downloads[0].url, "/reviews/2026-11-25-app/deck.pdf");
    }

    #[test]
    fn reads_human_issue_files() -> Result<(), String> {
        let issue = human_issue_file("issues/human/d.md", HUMAN).ok_or("no issue")?;
        assert_eq!(issue.text, "Do the dashboard thing.");
        assert_eq!(issue.added.as_deref(), Some("2026-11-20"));
        assert_eq!(issue.unblocks.as_deref(), Some("the app deploy: step 2"));
        assert_eq!(issue.line, 6);
        assert_eq!(issue.file, "issues/human/d.md");

        let bare = human_issue_file("issues/human/n.md", HUMAN_NO_NOTE).ok_or("no issue")?;
        assert_eq!(bare.text, "No note on this one.");
        assert_eq!(bare.added, None);
        assert_eq!(bare.unblocks, None);

        assert_eq!(bare.why, None);
        assert_eq!(bare.runbook, None);

        assert_eq!(
            human_issue_file("issues/human/x.md", "No front matter.\n"),
            None
        );
        Ok(())
    }

    #[test]
    fn reads_a_human_issue_with_its_runbook() -> Result<(), String> {
        let contents = "---\nadded: 2026-11-27\nrunbook: standards/docs/runbooks/x.md\n---\n\n## Requirement\n\nMake the\ntoken.\n\n## Why it can't be automated\n\nNo API.\n\n## Runbook\n\n[Make the token](../../standards/docs/runbooks/x.md)\n";
        let issue = human_issue_file("issues/human/x.md", contents).ok_or("no issue")?;
        assert_eq!(issue.text, "Make the token.");
        assert_eq!(issue.why.as_deref(), Some("No API."));
        assert_eq!(
            issue.runbook.as_deref(),
            Some("standards/docs/runbooks/x.md")
        );
        Ok(())
    }

    #[test]
    fn reads_release_tag_and_package_name() {
        let mise = "[env]\nDEPLOY_URL = \"https://g.pages.dev\"\nRELEASE_TAG = \"g-nightly\"\n";
        assert_eq!(release_tag(mise).as_deref(), Some("g-nightly"));
        assert_eq!(release_tag("[env]\n"), None);
        assert_eq!(release_tag("not toml ["), None);
        let cargo = "[package]\nname = \"my_game\"\nversion = \"0.1.0\"\n";
        assert_eq!(package_name(cargo).as_deref(), Some("my_game"));
        assert_eq!(package_name("[workspace]\n"), None);
    }

    #[test]
    fn builds_release_download_links() {
        let r = release("games/g", "g-nightly", "my_game");
        assert_eq!(r.kind, DeliverableKind::Release);
        assert_eq!(r.title, "games/g (nightly)");
        assert_eq!(
            r.url.as_deref(),
            Some("https://github.com/jimhillr/monorepo/releases/tag/g-nightly")
        );
        let urls: Vec<_> = r
            .downloads
            .iter()
            .map(|l| (l.label.as_str(), l.url.as_str()))
            .collect();
        assert_eq!(
            urls,
            [
                (
                    "macOS",
                    "https://github.com/jimhillr/monorepo/releases/download/g-nightly/my_game-macos.zip"
                ),
                (
                    "Android",
                    "https://github.com/jimhillr/monorepo/releases/download/g-nightly/my_game-android.apk"
                ),
            ]
        );
        let site = live_site("app/web", "https://app.pages.dev");
        assert_eq!(site.kind, DeliverableKind::LiveSite);
        assert_eq!(site.url.as_deref(), Some("https://app.pages.dev"));
        assert_eq!(site.title, "app/web");
        let site = live_site("cookbook/cloudflare", "https://c.pages.dev");
        assert_eq!(site.title, "cookbook");
        assert_eq!(site.project.as_deref(), Some("cookbook/cloudflare"));
    }

    #[test]
    fn classifies_issue_files() {
        assert_eq!(issue_file_kind("issues/a.md"), Some(IssueFile::Work));
        assert_eq!(
            issue_file_kind("app/web/issues/a.md"),
            Some(IssueFile::Work)
        );
        assert_eq!(issue_file_kind("issues/human/a.md"), Some(IssueFile::Human));
        assert_eq!(issue_file_kind("issues/README.md"), None);
        assert_eq!(issue_file_kind("issues/human/README.md"), None);
        assert_eq!(issue_file_kind("app/issues/notes.txt"), None);
        assert_eq!(issue_file_kind("app/myissues/a.md"), None);
        assert_eq!(issue_file_kind("app/issues/deeper/a.md"), None);
        assert_eq!(issue_file_kind("ISSUES.md"), None);
    }

    #[test]
    fn unquotes_front_matter_values() -> Result<(), String> {
        let FrontMatter { fields, body, line } =
            front_matter("---\nepic: \"A / B: \\\"c\\\"\"\nkey: plain\n---\nBody\n")
                .ok_or("none")?;
        assert_eq!(
            fields,
            vec![
                ("epic".to_owned(), "A / B: \"c\"".to_owned()),
                ("key".to_owned(), "plain".to_owned())
            ]
        );
        assert_eq!((body.as_str(), line), ("Body", 5));
        assert_eq!(front_matter("---\nkind: todo\n"), None, "unclosed");
        Ok(())
    }

    #[test]
    fn reads_code_comments() {
        let items = code_comments("app/src/main.rs", Some("app"), CODE);
        let got: Vec<_> = items
            .iter()
            .map(|i| (i.line, i.kind, i.text.as_str(), i.tags.clone()))
            .collect();
        assert_eq!(
            got,
            vec![
                (
                    1,
                    WorkKind::Todo,
                    "Line comment.",
                    vec!["agent-ignore".to_owned()]
                ),
                (
                    4,
                    WorkKind::Fix,
                    "Trailing comment.",
                    vec!["agent-ignore".to_owned()]
                ),
                (
                    5,
                    WorkKind::Todo,
                    "Block comment.",
                    vec!["agent-ignore".to_owned()]
                ),
                (
                    6,
                    WorkKind::Todo,
                    "After commented-out code.",
                    vec!["agent-ignore".to_owned()]
                ),
            ]
        );
        assert!(items.iter().all(|i| i.source == WorkSource::CodeComment));
    }

    #[test]
    fn code_comments_in_other_languages() {
        let items = code_comments("x.sh", None, "# TODO (agent-ignore): Shell.\necho hi\n");
        assert_eq!(items[0].text, "Shell.");
        let items = code_comments("x.html", None, "<!-- FIX (agent-ignore): Page. -->\n");
        assert_eq!(items[0].text, "Page.");
        assert!(code_comments("x.rs", None, "#[derive(Debug)]\nlet todo = 1;\n").is_empty());
        assert!(code_comments("x.rs", None, "let s = \"// TODO: in a string\";\n").is_empty());
        assert!(code_comments("x.rs", None, "/// Like `// TODO: example` in docs.\n").is_empty());
        let items = code_comments(
            "x.rs",
            None,
            "f(\"a\\\"b\"); // TODO (agent-ignore): After.\n",
        );
        assert_eq!(items[0].text, "After.");
    }

    #[test]
    fn classifies_doc_folders() {
        assert_eq!(doc_kind("app/docs/plans/x.md"), Some(DeliverableKind::Plan));
        assert_eq!(
            doc_kind("standards/docs/decisions/0001-x.md"),
            Some(DeliverableKind::Decision)
        );
        assert_eq!(
            doc_kind("a/docs/retros/2026-11-01-x.md"),
            Some(DeliverableKind::Retro)
        );
        assert_eq!(doc_kind("a/docs/decisions/README.md"), None);
        assert_eq!(doc_kind("a/plans/x.md"), None);
        assert_eq!(doc_kind("a/docs/plans/x.txt"), None);
        assert_eq!(doc_kind("x.md"), None);
    }

    #[test]
    fn reads_plan_and_decision() {
        let plan = doc_deliverable(
            DeliverableKind::Plan,
            "app/docs/plans/2026-11-big-plan.md",
            Some("app"),
            PLAN,
        );
        assert_eq!(plan.title, "Plan: a big plan");
        assert_eq!(plan.date.as_deref(), Some("2026-11-18"));
        assert_eq!(plan.status, None);
        let decision = doc_deliverable(
            DeliverableKind::Decision,
            "app/docs/decisions/0001-pick-a-thing.md",
            Some("app"),
            DECISION,
        );
        assert_eq!(decision.title, "0001. Pick a thing");
        assert_eq!(decision.date.as_deref(), Some("2026-11-19"));
        assert_eq!(decision.status.as_deref(), Some("Accepted"));
    }

    #[test]
    fn doc_date_falls_back_to_the_file_name() {
        let d = doc_deliverable(DeliverableKind::Plan, "a/docs/plans/2026-10-x.md", None, "");
        assert_eq!(d.date.as_deref(), Some("2026-10"));
        assert_eq!(d.title, "2026-10-x");
        let d = doc_deliverable(DeliverableKind::Plan, "a/docs/plans/x.md", None, "# X\n");
        assert_eq!(d.date, None);
    }

    #[test]
    fn reads_retro_actions() {
        let r = retro("app/docs/retros/2026-11-20-launch.md", Some("app"), RETRO);
        assert_eq!(r.title, "Retro: launch");
        assert_eq!(r.date, "2026-11-20");
        assert_eq!(
            r.actions,
            vec![
                RetroAction {
                    text: "Write the runbook.".to_owned(),
                    done: true
                },
                RetroAction {
                    text: "Add a smoke test.".to_owned(),
                    done: false
                },
            ]
        );
    }

    #[test]
    fn reads_config_roots() {
        assert_eq!(
            config_roots(ROOT_MISE),
            Ok(vec![
                "app".to_owned(),
                "app/web".to_owned(),
                "tool".to_owned()
            ])
        );
    }

    #[test]
    fn config_roots_errors_without_the_table() {
        assert!(config_roots("[tools]\nnode = \"24\"\n").is_err());
        assert!(config_roots("not = [toml").is_err());
        assert!(config_roots("[monorepo]\nconfig_roots = [1]\n").is_err());
    }

    #[test]
    fn reads_project_kind() {
        assert_eq!(project_kind(APP_MISE), Some("rust".to_owned()));
        assert_eq!(project_kind(APP_WEB_MISE), Some("dioxus-web".to_owned()));
        assert_eq!(project_kind("[tools]\n"), None);
        assert_eq!(project_kind("includes = [\"standards/tasks/.toml\"]"), None);
    }

    #[test]
    fn reads_deploy_url() {
        assert_eq!(
            deploy_url(APP_WEB_MISE),
            Some("https://app.pages.dev".to_owned())
        );
        assert_eq!(deploy_url(APP_MISE), None);
        assert_eq!(deploy_url("not toml ["), None);
    }

    #[test]
    fn readme_summary_skips_headings_badges_and_code() {
        assert_eq!(
            readme_summary(APP_README),
            Some("App does one thing well. It is named for what it is.".to_owned())
        );
        assert_eq!(readme_summary("# Title\n\n## Only headings\n"), None);
        assert_eq!(
            readme_summary("# T\n\n- a list\n- item\n\n| a | b |\n\nProse here.\n"),
            Some("Prose here.".to_owned())
        );
        assert_eq!(
            readme_summary("```\ncode\n```\n\nText.\n"),
            Some("Text.".to_owned())
        );
    }

    #[test]
    fn reads_root_lists_and_status() {
        let lists = root_lists(ROOT_ISSUES);
        assert_eq!(lists.priority, vec!["app".to_owned()]);
        assert_eq!(lists.on_hold, vec!["tool".to_owned()]);
        assert_eq!(lists.status_of("app"), ProjectStatus::Priority);
        assert_eq!(lists.status_of("app/web"), ProjectStatus::Priority);
        assert_eq!(lists.status_of("tool"), ProjectStatus::OnHold);
        assert_eq!(lists.status_of("other"), ProjectStatus::Active);
    }

    #[test]
    fn on_hold_wins_over_priority() {
        let lists = root_lists("# Priority Projects\n- a\n\n# On Hold Projects\n- a\n");
        assert_eq!(lists.status_of("a"), ProjectStatus::OnHold);
    }

    #[test]
    fn parses_markers() {
        assert_eq!(
            parse_marker("TODO (agent-generated, agent-ignore): Do it."),
            Some((
                WorkKind::Todo,
                vec!["agent-generated".to_owned(), "agent-ignore".to_owned()],
                "Do it.".to_owned()
            ))
        );
        assert_eq!(
            parse_marker("FIX: Broken."),
            Some((WorkKind::Fix, vec![], "Broken.".to_owned()))
        );
        assert_eq!(
            parse_marker("TODO Plain text"),
            Some((WorkKind::Todo, vec![], "Plain text".to_owned()))
        );
        assert_eq!(parse_marker("TODOS are fine"), None);
        assert_eq!(parse_marker("FIXME later"), None);
        assert_eq!(parse_marker("Not a marker"), None);
        assert_eq!(parse_marker("TODO (unclosed: text"), None);
    }

    #[test]
    fn issue_file_keeps_its_added_date() -> Result<(), String> {
        let with = "---\nkind: fix\npriority: priority\nadded: 2026-11-20\n---\n\nBroken.\n";
        let item = issue_file("app/issues/x.md", Some("app"), with).ok_or("no item")?;
        assert_eq!(item.added.as_deref(), Some("2026-11-20"));
        let odd = with.replace("2026-11-20", "last week");
        let item = issue_file("app/issues/x.md", Some("app"), &odd).ok_or("no item")?;
        assert_eq!(item.added, None);
        Ok(())
    }

    #[test]
    fn reads_issue_files_with_sections_and_epics() {
        let items: Vec<_> = [FLAT, EPIC, PHASE, LATER]
            .into_iter()
            .filter_map(|c| issue_file("app/issues/x.md", Some("app"), c))
            .collect();
        let summary: Vec<_> = items
            .iter()
            .map(|i| {
                (
                    i.kind,
                    i.line,
                    i.section.as_deref(),
                    i.epic.as_deref(),
                    i.text.as_str(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            vec![
                (
                    WorkKind::Todo,
                    8,
                    Some("Priority Issues"),
                    None,
                    "Flat item."
                ),
                (
                    WorkKind::Todo,
                    10,
                    Some("Priority Issues"),
                    Some("Big epic"),
                    "Epic item."
                ),
                (
                    WorkKind::Fix,
                    10,
                    Some("Priority Issues"),
                    Some("Big epic / Phase 1: start"),
                    "Phase item. ```markdown - TODO (agent-ignore): Inside a code block, part of the text. ```"
                ),
                (WorkKind::Todo, 8, Some("Backlog"), None, "Later."),
            ]
        );
        assert_eq!(
            items[0].tags,
            vec!["agent-generated".to_owned(), "agent-ignore".to_owned()]
        );
        assert_eq!(items[2].tags, vec!["agent-ignore".to_owned()]);
        assert!(items.iter().all(|i| i.project.as_deref() == Some("app")
            && i.file == "app/issues/x.md"
            && i.source == WorkSource::IssuesFile));
        assert_eq!(issue_file("app/issues/e.md", Some("app"), NO_FRONT), None);
        assert_eq!(
            issue_file("a/issues/k.md", None, "---\nkind: idea\n---\nText\n"),
            None,
            "unknown kind"
        );
    }

    #[test]
    fn owning_project_picks_the_longest_root() {
        let roots = vec!["app".to_owned(), "app/web".to_owned()];
        assert_eq!(
            owning_project("app/web/issues/a.md", &roots),
            Some("app/web")
        );
        assert_eq!(owning_project("app/issues/a.md", &roots), Some("app"));
        assert_eq!(owning_project("application/issues/a.md", &roots), None);
        assert_eq!(owning_project("issues/a.md", &roots), None);
    }

    #[test]
    fn pane_health_toml_parses_and_sets_every_key() -> Result<(), String> {
        let text = include_str!("../../health.toml");
        let parsed = health_thresholds(text)?;
        // Every key is written out, so the file is the full list Jim tunes.
        let table: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        let defaults = toml::Table::try_from(pane_model::health::Thresholds::default())
            .map_err(|e| e.to_string())?;
        for (section, keys) in &defaults {
            let (Some(want), Some(got)) = (
                keys.as_table(),
                table.get(section).and_then(|v| v.as_table()),
            ) else {
                return Err(format!("health.toml is missing [{section}]"));
            };
            for key in want.keys() {
                if !got.contains_key(key) {
                    return Err(format!("health.toml is missing {section}.{key}"));
                }
            }
        }
        assert!(parsed.build.stale_after_days > 0);
        Ok(())
    }

    #[test]
    fn health_thresholds_reject_typos_and_default_missing() -> Result<(), String> {
        assert!(health_thresholds("[build]\nstale_days = 3\n").is_err());
        let t = health_thresholds("[build]\nstale_after_days = 3\n")?;
        assert_eq!(t.build.stale_after_days, 3);
        assert_eq!(t.review, pane_model::health::ReviewThresholds::default());
        Ok(())
    }

    #[test]
    fn finds_table_header_lines() {
        let text = "a = 1\n[[cost]]\nx = 1\n  [[cost]] # second\n[[revenue]]\n[cost]\n[[costs]]\n";
        assert_eq!(table_lines(text, "cost"), [2, 4]);
        assert_eq!(table_lines(text, "revenue"), [5]);
        assert!(table_lines(text, "service").is_empty());
    }
}
