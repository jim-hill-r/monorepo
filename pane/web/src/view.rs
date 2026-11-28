//! Pure helpers behind the pages, kept out of the components so they are
//! unit tested natively.

use pane_model::{
    Deliverable, DeliverableKind, HumanIssue, Project, ProjectStatus, REPO_URL, Review,
    ScorecardRow, Snapshot, WorkItem, WorkKind, WorkSource,
};

/// Seconds since the Unix epoch for an RFC 3339 UTC time
/// (`YYYY-MM-DDTHH:MM:SSZ`, fractional seconds ignored).
pub fn parse_rfc3339(s: &str) -> Option<i64> {
    let num = |range: std::ops::Range<usize>| s.get(range)?.parse::<i64>().ok();
    if s.get(4..5)? != "-" || s.get(7..8)? != "-" || s.get(10..11)? != "T" {
        return None;
    }
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (hh, mm, ss) = (num(11..13)?, num(14..16)?, num(17..19)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh > 23 || mm > 59 || ss > 60 {
        return None;
    }
    Some(days_from_civil(y, m, d) * 86_400 + hh * 3600 + mm * 60 + ss)
}

/// Howard Hinnant's civil-to-days algorithm.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// How long ago `then` was, as of `now` (both Unix seconds): "just now",
/// "5 minutes ago", "3 hours ago", "2 days ago".
pub fn age(then: i64, now: i64) -> String {
    let secs = (now - then).max(0);
    let (n, unit) = match secs {
        0..60 => return "just now".to_owned(),
        60..3600 => (secs / 60, "minute"),
        3600..86_400 => (secs / 3600, "hour"),
        _ => (secs / 86_400, "day"),
    };
    format!("{n} {unit}{} ago", if n == 1 { "" } else { "s" })
}

/// The first seven characters of a commit hash.
pub fn short_commit(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

/// The footer line: "Snapshot of abc1234, taken 12 minutes ago".
pub fn footer_text(snapshot: &Snapshot, now: i64) -> String {
    let taken = parse_rfc3339(&snapshot.taken_at)
        .map(|t| age(t, now))
        .unwrap_or_else(|| snapshot.taken_at.clone());
    format!(
        "Snapshot of {}, taken {taken}",
        short_commit(&snapshot.commit)
    )
}

/// Projects in display order: priority first, then active, then on hold,
/// each by path.
pub fn sorted_projects(snapshot: &Snapshot) -> Vec<&Project> {
    let rank = |s: ProjectStatus| match s {
        ProjectStatus::Priority => 0,
        ProjectStatus::Active => 1,
        ProjectStatus::OnHold => 2,
    };
    let mut projects: Vec<&Project> = snapshot.projects.iter().collect();
    projects.sort_by(|a, b| {
        rank(a.status)
            .cmp(&rank(b.status))
            .then_with(|| a.path.cmp(&b.path))
    });
    projects
}

/// Open `TODO`s and `FIX`es that belong to `path`.
pub fn open_counts(snapshot: &Snapshot, path: &str) -> (usize, usize) {
    snapshot
        .work_items
        .iter()
        .filter(|i| i.project.as_deref() == Some(path))
        .fold((0, 0), |(todo, fix), i| match i.kind {
            WorkKind::Todo => (todo + 1, fix),
            WorkKind::Fix => (todo, fix + 1),
        })
}

/// A project by its path.
pub fn find_project<'a>(snapshot: &'a Snapshot, path: &str) -> Option<&'a Project> {
    snapshot.projects.iter().find(|p| p.path == path)
}

/// The label shown for a status.
pub fn status_label(status: ProjectStatus) -> &'static str {
    match status {
        ProjectStatus::Priority => "Priority",
        ProjectStatus::Active => "Active",
        ProjectStatus::OnHold => "On hold",
    }
}

/// The CSS class suffix for a status badge.
pub fn status_class(status: ProjectStatus) -> &'static str {
    match status {
        ProjectStatus::Priority => "priority",
        ProjectStatus::Active => "active",
        ProjectStatus::OnHold => "on-hold",
    }
}

/// A file on GitHub at the snapshot's commit, so line numbers match.
pub fn github_file_url(commit: &str, file: &str) -> String {
    format!("{REPO_URL}/blob/{commit}/{file}")
}

/// A directory on GitHub at the snapshot's commit.
pub fn github_tree_url(commit: &str, dir: &str) -> String {
    format!("{REPO_URL}/tree/{commit}/{dir}")
}

/// A line of a file on GitHub at the snapshot's commit.
pub fn github_line_url(commit: &str, file: &str, line: u32) -> String {
    format!("{}#L{line}", github_file_url(commit, file))
}

/// Which list a work item sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bucket {
    /// An issue file with `priority: priority`.
    Priority,
    /// An issue file with `priority: backlog`.
    Backlog,
    /// An issue file with any other `priority`.
    Other,
    /// A `TODO`/`FIX` comment in code.
    Code,
}

impl Bucket {
    pub const ALL: [Bucket; 4] = [Self::Priority, Self::Backlog, Self::Other, Self::Code];

    pub fn of(item: &WorkItem) -> Self {
        if item.source == WorkSource::CodeComment {
            return Self::Code;
        }
        match item.section.as_deref() {
            Some(s) if s.starts_with("Priority") => Self::Priority,
            Some(s) if s.starts_with("Backlog") => Self::Backlog,
            _ => Self::Other,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Priority => "Priority",
            Self::Backlog => "Backlog",
            Self::Other => "Other sections",
            Self::Code => "Code comments",
        }
    }

    /// The value used in the filter's `<select>`.
    pub fn key(self) -> &'static str {
        match self {
            Self::Priority => "priority",
            Self::Backlog => "backlog",
            Self::Other => "other",
            Self::Code => "code",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.key() == key)
    }
}

/// The label for a work item's project: its path, or "Repo-wide".
pub fn project_label(project: Option<&str>) -> &str {
    project.unwrap_or(REPO_WIDE)
}

/// The project filter's name for items outside every project.
pub const REPO_WIDE: &str = "Repo-wide";

/// The Work page's filters. `None` means "any".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkFilter {
    /// A project path, or [`REPO_WIDE`].
    pub project: Option<String>,
    pub kind: Option<WorkKind>,
    pub bucket: Option<Bucket>,
    pub tag: Option<String>,
}

impl WorkFilter {
    pub fn matches(&self, item: &WorkItem) -> bool {
        self.project
            .as_deref()
            .is_none_or(|p| project_label(item.project.as_deref()) == p)
            && self.kind.is_none_or(|k| item.kind == k)
            && self.bucket.is_none_or(|b| Bucket::of(item) == b)
            && self
                .tag
                .as_deref()
                .is_none_or(|t| item.tags.iter().any(|x| x == t))
    }
}

/// The items `filter` keeps, FIXes first, then by bucket, project, file and
/// line.
pub fn filter_work<'a>(items: &'a [WorkItem], filter: &WorkFilter) -> Vec<&'a WorkItem> {
    let kind_rank = |k: WorkKind| match k {
        WorkKind::Fix => 0,
        WorkKind::Todo => 1,
    };
    let mut kept: Vec<&WorkItem> = items.iter().filter(|i| filter.matches(i)).collect();
    kept.sort_by(|a, b| {
        kind_rank(a.kind)
            .cmp(&kind_rank(b.kind))
            .then_with(|| Bucket::of(a).cmp(&Bucket::of(b)))
            .then_with(|| a.project.is_none().cmp(&b.project.is_none()))
            .then_with(|| a.project.cmp(&b.project))
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.line.cmp(&b.line))
    });
    kept
}

/// The projects and tags that appear on work items, sorted, for the
/// filter menus. Repo-wide items add [`REPO_WIDE`] at the end.
pub fn work_facets(items: &[WorkItem]) -> (Vec<String>, Vec<String>) {
    let mut projects: Vec<String> = items.iter().filter_map(|i| i.project.clone()).collect();
    projects.sort();
    projects.dedup();
    if items.iter().any(|i| i.project.is_none()) {
        projects.push(REPO_WIDE.to_owned());
    }
    let mut tags: Vec<String> = items.iter().flat_map(|i| i.tags.clone()).collect();
    tags.sort();
    tags.dedup();
    (projects, tags)
}

/// The label for a work item's kind.
pub fn kind_label(kind: WorkKind) -> &'static str {
    match kind {
        WorkKind::Todo => "TODO",
        WorkKind::Fix => "FIX",
    }
}

/// Whole days from `YYYY-MM-DD` to `now` (Unix seconds), or `None` for a
/// date that doesn't parse.
pub fn days_since(date: &str, now: i64) -> Option<i64> {
    let then = parse_rfc3339(&format!("{}T00:00:00Z", date.get(..10)?))?;
    Some((now - then).max(0) / 86_400)
}

/// "today", "1 day old", "12 days old".
pub fn days_label(days: i64) -> String {
    match days {
        0 => "today".to_owned(),
        1 => "1 day old".to_owned(),
        n => format!("{n} days old"),
    }
}

/// Human issues oldest first; undated ones last, in file order.
pub fn sorted_human_issues(snapshot: &Snapshot) -> Vec<&HumanIssue> {
    let mut issues: Vec<&HumanIssue> = snapshot.human_issues.iter().collect();
    issues.sort_by(|a, b| match (&a.added, &b.added) {
        (Some(x), Some(y)) => x.cmp(y).then_with(|| a.line.cmp(&b.line)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.line.cmp(&b.line),
    });
    issues
}

/// The Deliverables page's groups, in order, with their headings.
pub const DELIVERABLE_GROUPS: [(DeliverableKind, &str); 6] = [
    (DeliverableKind::LiveSite, "Live sites"),
    (DeliverableKind::Release, "Downloads"),
    (DeliverableKind::ReviewDeck, "Review decks"),
    (DeliverableKind::Plan, "Plans"),
    (DeliverableKind::Decision, "Decisions"),
    (DeliverableKind::Retro, "Retros"),
];

/// Deliverables of one kind: newest first where they have dates, then by
/// project and title.
pub fn deliverables_of(snapshot: &Snapshot, kind: DeliverableKind) -> Vec<&Deliverable> {
    let mut found: Vec<&Deliverable> = snapshot
        .deliverables
        .iter()
        .filter(|d| d.kind == kind)
        .collect();
    found.sort_by(|a, b| {
        b.date
            .cmp(&a.date)
            .then_with(|| a.project.cmp(&b.project))
            .then_with(|| a.title.cmp(&b.title))
    });
    found
}

/// The test id suffix for a deliverable group.
pub fn deliverable_key(kind: DeliverableKind) -> &'static str {
    match kind {
        DeliverableKind::LiveSite => "live-sites",
        DeliverableKind::Release => "downloads",
        DeliverableKind::Plan => "plans",
        DeliverableKind::Decision => "decisions",
        DeliverableKind::Retro => "retros",
        DeliverableKind::ReviewDeck => "reviews",
        DeliverableKind::UatReport => "uat-reports",
    }
}

/// A plan, decision or retro by its repo path.
pub fn find_doc<'a>(snapshot: &'a Snapshot, path: &str) -> Option<&'a Deliverable> {
    snapshot.deliverables.iter().find(|d| {
        matches!(
            d.kind,
            DeliverableKind::Plan | DeliverableKind::Decision | DeliverableKind::Retro
        ) && d.path.as_deref() == Some(path)
    })
}

/// Every review in the rotation, newest first, then by project.
pub fn sorted_reviews(snapshot: &Snapshot) -> Vec<&Review> {
    let mut found: Vec<&Review> = snapshot.reviews.iter().collect();
    found.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.project.cmp(&b.project)));
    found
}

/// A review by its deck folder, such as `2026-11-25-cookbook`.
pub fn find_review<'a>(snapshot: &'a Snapshot, folder: &str) -> Option<&'a Review> {
    snapshot.reviews.iter().find(|r| r.folder() == folder)
}

/// The newest review of a project or one of its sub-projects.
pub fn latest_review<'a>(snapshot: &'a Snapshot, path: &str) -> Option<&'a Review> {
    sorted_reviews(snapshot)
        .into_iter()
        .find(|r| in_project(Some(&r.project), path))
}

/// The CSS class for a scorecard rating: `good`, `fair` or `poor` (the
/// deep-project-review skill's ratings, in any case), else `other`.
pub fn rating_class(rating: &str) -> &'static str {
    match rating.trim().to_lowercase().as_str() {
        "good" => "good",
        "fair" => "fair",
        "poor" => "poor",
        _ => "other",
    }
}

/// A one-line count of a scorecard's ratings, such as `2 Good · 4 Fair ·
/// 1 Poor`, leaving out ratings with no areas. Empty for no scorecard.
pub fn scorecard_summary(rows: &[ScorecardRow]) -> String {
    let count = |class: &str| {
        rows.iter()
            .filter(|r| rating_class(&r.rating) == class)
            .count()
    };
    [
        ("good", "Good"),
        ("fair", "Fair"),
        ("poor", "Poor"),
        ("other", "other"),
    ]
    .into_iter()
    .filter_map(|(class, label)| match count(class) {
        0 => None,
        n => Some(format!("{n} {label}")),
    })
    .collect::<Vec<_>>()
    .join(" · ")
}

/// Where pane's release serves a deck's slides (see `pane/web/mise.toml`'s
/// `reviews` task).
pub fn deck_html_url(folder: &str) -> String {
    format!("/reviews/{folder}/deck.html")
}

/// Where pane's release serves a deck's PDF.
pub fn deck_pdf_url(folder: &str) -> String {
    format!("/reviews/{folder}/deck.pdf")
}

/// A short label for a pull request URL: `#585` for `.../pull/585`.
pub fn pr_label(url: &str) -> String {
    match url.trim_end_matches('/').rsplit_once("/pull/") {
        Some((_, n)) if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => format!("#{n}"),
        _ => "pull request".to_owned(),
    }
}

/// Whether `item_project` is `path` or one of its sub-projects
/// (`pane/web` is in `pane`).
pub fn in_project(item_project: Option<&str>, path: &str) -> bool {
    item_project.is_some_and(|p| {
        p.strip_prefix(path)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}

/// The projects inside `path`, such as `pane/web` for `pane`, by path.
pub fn sub_projects<'a>(snapshot: &'a Snapshot, path: &str) -> Vec<&'a Project> {
    let mut found: Vec<&Project> = snapshot
        .projects
        .iter()
        .filter(|p| p.path != path && in_project(Some(&p.path), path))
        .collect();
    found.sort_by(|a, b| a.path.cmp(&b.path));
    found
}

/// The open `TODO`s and `FIX`es of a project and its sub-projects, in the
/// Work page's order (FIXes first, then priority, backlog, other, code).
pub fn project_work<'a>(snapshot: &'a Snapshot, path: &str) -> Vec<&'a WorkItem> {
    filter_work(&snapshot.work_items, &WorkFilter::default())
        .into_iter()
        .filter(|i| in_project(i.project.as_deref(), path))
        .collect()
}

/// `(TODO, FIX)` counts of a list of work items.
pub fn kind_counts(items: &[&WorkItem]) -> (usize, usize) {
    items.iter().fold((0, 0), |(todo, fix), i| match i.kind {
        WorkKind::Todo => (todo + 1, fix),
        WorkKind::Fix => (todo, fix + 1),
    })
}

/// A project's deliverables (its sub-projects' too), grouped as on the
/// Deliverables page, leaving out empty groups.
pub fn project_deliverables<'a>(
    snapshot: &'a Snapshot,
    path: &str,
) -> Vec<(DeliverableKind, &'static str, Vec<&'a Deliverable>)> {
    DELIVERABLE_GROUPS
        .iter()
        .map(|(kind, heading)| {
            let items: Vec<&Deliverable> = deliverables_of(snapshot, *kind)
                .into_iter()
                .filter(|d| in_project(d.project.as_deref(), path))
                .collect();
            (*kind, *heading, items)
        })
        .filter(|(_, _, items)| !items.is_empty())
        .collect()
}

/// The newest plan, decision and retro of a project (or its sub-projects),
/// with a label for each, skipping kinds it has none of.
pub fn latest_docs<'a>(snapshot: &'a Snapshot, path: &str) -> Vec<(&'static str, &'a Deliverable)> {
    [
        (DeliverableKind::Plan, "Latest plan"),
        (DeliverableKind::Decision, "Latest decision"),
        (DeliverableKind::Retro, "Latest retro"),
    ]
    .into_iter()
    .filter_map(|(kind, label)| {
        deliverables_of(snapshot, kind)
            .into_iter()
            .find(|d| d.path.is_some() && in_project(d.project.as_deref(), path))
            .map(|d| (label, d))
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(path: &str, status: ProjectStatus) -> Project {
        Project {
            path: path.to_owned(),
            kind: "rust".to_owned(),
            deploy_url: None,
            summary: None,
            status,
        }
    }

    fn item(project: &str, kind: WorkKind) -> WorkItem {
        WorkItem {
            kind,
            text: "x".to_owned(),
            project: Some(project.to_owned()),
            file: format!("{project}/issues/x.md"),
            line: 1,
            source: WorkSource::IssuesFile,
            section: None,
            epic: None,
            tags: vec![],
            added: None,
        }
    }

    #[test]
    fn parses_rfc3339() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_rfc3339("2000-02-29T00:00:00Z"), Some(951_782_400));
        assert_eq!(parse_rfc3339("2026-11-26T14:48:40Z"), Some(1_795_704_520));
        assert_eq!(
            parse_rfc3339("2026-11-26T14:48:40.123Z"),
            Some(1_795_704_520)
        );
        assert_eq!(parse_rfc3339("2026-13-01T00:00:00Z"), None);
        assert_eq!(parse_rfc3339("yesterday"), None);
        assert_eq!(parse_rfc3339(""), None);
    }

    #[test]
    fn formats_ages() {
        assert_eq!(age(100, 100), "just now");
        assert_eq!(age(100, 159), "just now");
        assert_eq!(age(0, 60), "1 minute ago");
        assert_eq!(age(0, 12 * 60), "12 minutes ago");
        assert_eq!(age(0, 3600), "1 hour ago");
        assert_eq!(age(0, 5 * 3600 + 59), "5 hours ago");
        assert_eq!(age(0, 86_400), "1 day ago");
        assert_eq!(age(0, 3 * 86_400), "3 days ago");
        assert_eq!(age(100, 0), "just now", "a clock behind the snapshot");
    }

    #[test]
    fn footer_shows_short_commit_and_age() {
        let s = Snapshot::new("abc1234def", "2026-11-26T12:00:00Z");
        let now = parse_rfc3339("2026-11-26T12:12:00Z").unwrap_or_default();
        assert_eq!(
            footer_text(&s, now),
            "Snapshot of abc1234, taken 12 minutes ago"
        );
        let s = Snapshot::new("abc", "not a time");
        assert_eq!(footer_text(&s, 0), "Snapshot of abc, taken not a time");
    }

    #[test]
    fn sorts_priority_then_active_then_on_hold() {
        let mut s = Snapshot::new("a", "t");
        s.projects = vec![
            project("zeta", ProjectStatus::Active),
            project("held", ProjectStatus::OnHold),
            project("pane/web", ProjectStatus::Priority),
            project("alpha", ProjectStatus::Active),
            project("pane", ProjectStatus::Priority),
        ];
        let order: Vec<_> = sorted_projects(&s)
            .iter()
            .map(|p| p.path.as_str())
            .collect();
        assert_eq!(order, ["pane", "pane/web", "alpha", "zeta", "held"]);
    }

    #[test]
    fn counts_open_items_per_project() {
        let mut s = Snapshot::new("a", "t");
        s.work_items = vec![
            item("pane", WorkKind::Todo),
            item("pane", WorkKind::Todo),
            item("pane", WorkKind::Fix),
            item("pane/web", WorkKind::Todo),
        ];
        assert_eq!(open_counts(&s, "pane"), (2, 1));
        assert_eq!(open_counts(&s, "pane/web"), (1, 0));
        assert_eq!(open_counts(&s, "other"), (0, 0));
    }

    #[test]
    fn finds_projects_and_labels_status() {
        let mut s = Snapshot::new("a", "t");
        s.projects = vec![project("pane/web", ProjectStatus::OnHold)];
        assert!(find_project(&s, "pane/web").is_some());
        assert!(find_project(&s, "pane").is_none());
        assert_eq!(status_label(ProjectStatus::OnHold), "On hold");
        assert_eq!(status_class(ProjectStatus::OnHold), "on-hold");
        assert_eq!(short_commit("abc"), "abc");
    }

    fn work(
        project: Option<&str>,
        kind: WorkKind,
        section: Option<&str>,
        tags: &[&str],
    ) -> WorkItem {
        WorkItem {
            kind,
            text: "x".to_owned(),
            project: project.map(str::to_owned),
            file: format!("{}/issues/x.md", project.unwrap_or(".")),
            line: 1,
            source: WorkSource::IssuesFile,
            section: section.map(str::to_owned),
            epic: None,
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            added: None,
        }
    }

    #[test]
    fn links_lines_on_github_at_the_commit() {
        assert_eq!(
            github_line_url("abc", "pane/docs/issues/x.md", 7),
            "https://github.com/jimhillr/monorepo/blob/abc/pane/docs/issues/x.md#L7"
        );
    }

    #[test]
    fn buckets_items_by_section_and_source() {
        let mut code = work(Some("a"), WorkKind::Todo, None, &[]);
        code.source = WorkSource::CodeComment;
        assert_eq!(Bucket::of(&code), Bucket::Code);
        assert_eq!(
            Bucket::of(&work(None, WorkKind::Todo, Some("Priority Issues"), &[])),
            Bucket::Priority
        );
        assert_eq!(
            Bucket::of(&work(None, WorkKind::Todo, Some("Backlog"), &[])),
            Bucket::Backlog
        );
        assert_eq!(
            Bucket::of(&work(None, WorkKind::Todo, Some("Done"), &[])),
            Bucket::Other
        );
        assert_eq!(
            Bucket::of(&work(None, WorkKind::Todo, None, &[])),
            Bucket::Other
        );
        for b in Bucket::ALL {
            assert_eq!(Bucket::from_key(b.key()), Some(b));
        }
        assert_eq!(Bucket::from_key(""), None);
    }

    #[test]
    fn filters_work_by_each_field() {
        let items = vec![
            work(
                Some("a"),
                WorkKind::Todo,
                Some("Priority Issues"),
                &["agent-generated"],
            ),
            work(Some("a"), WorkKind::Fix, Some("Backlog"), &[]),
            work(
                Some("b"),
                WorkKind::Todo,
                Some("Backlog"),
                &["agent-ignore"],
            ),
            work(None, WorkKind::Todo, Some("Priority Issues"), &[]),
        ];
        let count = |f: WorkFilter| filter_work(&items, &f).len();
        assert_eq!(count(WorkFilter::default()), 4);
        assert_eq!(
            count(WorkFilter {
                project: Some("a".into()),
                ..WorkFilter::default()
            }),
            2
        );
        assert_eq!(
            count(WorkFilter {
                project: Some(REPO_WIDE.into()),
                ..WorkFilter::default()
            }),
            1
        );
        assert_eq!(
            count(WorkFilter {
                kind: Some(WorkKind::Fix),
                ..WorkFilter::default()
            }),
            1
        );
        assert_eq!(
            count(WorkFilter {
                bucket: Some(Bucket::Backlog),
                ..WorkFilter::default()
            }),
            2
        );
        assert_eq!(
            count(WorkFilter {
                tag: Some("agent-ignore".into()),
                ..WorkFilter::default()
            }),
            1
        );
        assert_eq!(
            count(WorkFilter {
                project: Some("a".into()),
                kind: Some(WorkKind::Todo),
                bucket: Some(Bucket::Backlog),
                tag: None,
            }),
            0
        );
    }

    #[test]
    fn sorts_fixes_first_then_priority_then_projects_then_repo_wide() {
        let items = vec![
            work(None, WorkKind::Todo, Some("Priority Issues"), &[]),
            work(Some("b"), WorkKind::Todo, Some("Backlog"), &[]),
            work(Some("b"), WorkKind::Todo, Some("Priority Issues"), &[]),
            work(Some("a"), WorkKind::Todo, Some("Priority Issues"), &[]),
            work(Some("z"), WorkKind::Fix, Some("Backlog"), &[]),
        ];
        let order: Vec<_> = filter_work(&items, &WorkFilter::default())
            .iter()
            .map(|i| {
                (
                    kind_label(i.kind),
                    project_label(i.project.as_deref()),
                    Bucket::of(i),
                )
            })
            .collect();
        assert_eq!(
            order,
            [
                ("FIX", "z", Bucket::Backlog),
                ("TODO", "a", Bucket::Priority),
                ("TODO", "b", Bucket::Priority),
                ("TODO", REPO_WIDE, Bucket::Priority),
                ("TODO", "b", Bucket::Backlog),
            ]
        );
    }

    #[test]
    fn lists_facets_with_repo_wide_last() {
        let items = vec![
            work(Some("b"), WorkKind::Todo, None, &["x", "agent-generated"]),
            work(None, WorkKind::Todo, None, &["x"]),
            work(Some("a"), WorkKind::Todo, None, &[]),
        ];
        let (projects, tags) = work_facets(&items);
        assert_eq!(projects, ["a", "b", REPO_WIDE]);
        assert_eq!(tags, ["agent-generated", "x"]);
        let (projects, _) = work_facets(&items[2..]);
        assert_eq!(projects, ["a"]);
    }

    #[test]
    fn counts_days_since_a_date() {
        let now = parse_rfc3339("2026-11-26T18:00:00Z").unwrap_or_default();
        assert_eq!(days_since("2026-11-26", now), Some(0));
        assert_eq!(days_since("2026-11-25", now), Some(1));
        assert_eq!(days_since("2026-11-01", now), Some(25));
        assert_eq!(days_since("2026-12-01", now), Some(0), "a future date");
        assert_eq!(days_since("soon", now), None);
        assert_eq!(days_label(0), "today");
        assert_eq!(days_label(1), "1 day old");
        assert_eq!(days_label(9), "9 days old");
    }

    #[test]
    fn sorts_human_issues_oldest_first_undated_last() {
        let issue = |line: u32, added: Option<&str>| HumanIssue {
            text: format!("line {line}"),
            file: "standards/docs/issues/human/x.md".into(),
            line,
            added: added.map(str::to_owned),
            unblocks: None,
            why: None,
            runbook: None,
            project: None,
        };
        let mut s = Snapshot::new("a", "t");
        s.human_issues = vec![
            issue(1, Some("2026-11-26")),
            issue(2, None),
            issue(3, Some("2026-11-20")),
            issue(4, Some("2026-11-26")),
            issue(5, None),
        ];
        let order: Vec<_> = sorted_human_issues(&s).iter().map(|i| i.line).collect();
        assert_eq!(order, [3, 1, 4, 2, 5]);
    }

    #[test]
    fn groups_deliverables_newest_first() {
        let doc = |kind, title: &str, date: Option<&str>| Deliverable {
            kind,
            title: title.into(),
            project: None,
            path: Some(format!("docs/{title}.md")),
            url: None,
            date: date.map(str::to_owned),
            status: None,
            downloads: vec![],
            body: None,
        };
        let mut s = Snapshot::new("a", "t");
        s.deliverables = vec![
            doc(DeliverableKind::Plan, "old", Some("2026-10")),
            doc(DeliverableKind::Decision, "d", None),
            doc(DeliverableKind::Plan, "undated", None),
            doc(DeliverableKind::Plan, "new", Some("2026-11-20")),
        ];
        let plans: Vec<_> = deliverables_of(&s, DeliverableKind::Plan)
            .iter()
            .map(|d| d.title.as_str())
            .collect();
        assert_eq!(plans, ["new", "old", "undated"]);
        assert_eq!(deliverables_of(&s, DeliverableKind::Retro).len(), 0);
        assert!(find_doc(&s, "docs/d.md").is_some());
        assert!(find_doc(&s, "docs/nope.md").is_none());
    }

    fn review(project: &str, date: &str, pr: Option<&str>) -> Review {
        Review {
            project: project.into(),
            date: date.into(),
            path: format!("docs/reviews/{date}-{project}/deck.md"),
            pr: pr.map(str::to_owned),
            scorecard: vec![],
        }
    }

    fn row(area: &str, rating: &str) -> ScorecardRow {
        ScorecardRow {
            area: area.into(),
            rating: rating.into(),
            why: String::new(),
        }
    }

    #[test]
    fn classes_ratings_in_any_case() {
        assert_eq!(rating_class("Good"), "good");
        assert_eq!(rating_class(" FAIR "), "fair");
        assert_eq!(rating_class("poor"), "poor");
        assert_eq!(rating_class("Excellent"), "other");
        assert_eq!(rating_class(""), "other");
    }

    #[test]
    fn summarises_a_scorecard() {
        let rows = [
            row("Goals", "Poor"),
            row("UX", "Fair"),
            row("Security", "Good"),
            row("Quality", "fair"),
        ];
        assert_eq!(scorecard_summary(&rows), "1 Good · 2 Fair · 1 Poor");
        assert_eq!(scorecard_summary(&rows[1..2]), "1 Fair");
        assert_eq!(
            scorecard_summary(&[row("A", "Good"), row("B", "Great")]),
            "1 Good · 1 other"
        );
        assert_eq!(scorecard_summary(&[]), "");
    }

    #[test]
    fn lists_reviews_newest_first_and_finds_them() {
        let mut s = Snapshot::new("a", "t");
        s.reviews = vec![
            review("cookbook", "2026-11-11", None),
            review(
                "pane",
                "2026-11-25",
                Some("https://github.com/jimhillr/monorepo/pull/9"),
            ),
            review("homer", "2026-11-25", None),
        ];
        let order: Vec<_> = sorted_reviews(&s)
            .iter()
            .map(|r| r.project.as_str())
            .collect();
        assert_eq!(order, ["homer", "pane", "cookbook"]);
        assert_eq!(
            find_review(&s, "2026-11-11-cookbook").map(|r| r.project.as_str()),
            Some("cookbook")
        );
        assert!(find_review(&s, "2026-11-11-nope").is_none());
    }

    #[test]
    fn a_project_shows_its_newest_review() {
        let mut s = Snapshot::new("a", "t");
        s.reviews = vec![
            review("pane", "2026-11-01", None),
            review("pane/web", "2026-11-20", None),
            review("panel", "2026-11-30", None),
        ];
        assert_eq!(
            latest_review(&s, "pane").map(|r| r.date.as_str()),
            Some("2026-11-20")
        );
        assert_eq!(
            latest_review(&s, "pane/web").map(|r| r.date.as_str()),
            Some("2026-11-20")
        );
        assert!(latest_review(&s, "cookbook").is_none());
    }

    #[test]
    fn links_deck_slides_pdf_and_pr() {
        assert_eq!(
            deck_html_url("2026-11-25-cookbook"),
            "/reviews/2026-11-25-cookbook/deck.html"
        );
        assert_eq!(
            deck_pdf_url("2026-11-25-cookbook"),
            "/reviews/2026-11-25-cookbook/deck.pdf"
        );
        assert_eq!(
            pr_label("https://github.com/jimhillr/monorepo/pull/585"),
            "#585"
        );
        assert_eq!(
            pr_label("https://github.com/jimhillr/monorepo/pull/585/"),
            "#585"
        );
        assert_eq!(
            pr_label("https://github.com/jimhillr/monorepo/pulls"),
            "pull request"
        );
    }

    #[test]
    fn a_review_deck_is_not_a_document() {
        let mut s = Snapshot::new("a", "t");
        s.deliverables = vec![doc_of(
            DeliverableKind::ReviewDeck,
            Some("pane"),
            "docs/reviews/2026-11-25-pane/deck.md",
            Some("2026-11-25"),
        )];
        assert!(find_doc(&s, "docs/reviews/2026-11-25-pane/deck.md").is_none());
    }

    fn doc_of(
        kind: DeliverableKind,
        project: Option<&str>,
        path: &str,
        date: Option<&str>,
    ) -> Deliverable {
        Deliverable {
            kind,
            title: path.to_owned(),
            project: project.map(str::to_owned),
            path: Some(path.to_owned()),
            url: None,
            date: date.map(str::to_owned),
            status: None,
            downloads: vec![],
            body: None,
        }
    }

    #[test]
    fn a_project_holds_its_sub_projects_but_not_lookalikes() {
        assert!(in_project(Some("pane"), "pane"));
        assert!(in_project(Some("pane/web"), "pane"));
        assert!(!in_project(Some("pane_old"), "pane"));
        assert!(!in_project(Some("pan"), "pane"));
        assert!(!in_project(None, "pane"));
        let mut s = Snapshot::new("a", "t");
        s.projects = vec![
            project("pane/web", ProjectStatus::Active),
            project("pane", ProjectStatus::Active),
            project("pane/cloudflare", ProjectStatus::Active),
            project("panel", ProjectStatus::Active),
        ];
        let subs: Vec<_> = sub_projects(&s, "pane")
            .iter()
            .map(|p| p.path.as_str())
            .collect();
        assert_eq!(subs, ["pane/cloudflare", "pane/web"]);
        assert!(sub_projects(&s, "pane/web").is_empty());
    }

    #[test]
    fn project_work_includes_sub_projects_fixes_first() {
        let mut s = Snapshot::new("a", "t");
        s.work_items = vec![
            item("pane", WorkKind::Todo),
            item("pane/web", WorkKind::Fix),
            item("panel", WorkKind::Fix),
            item("other", WorkKind::Todo),
        ];
        let work = project_work(&s, "pane");
        let order: Vec<_> = work.iter().map(|i| i.project.as_deref()).collect();
        assert_eq!(order, [Some("pane/web"), Some("pane")]);
        assert_eq!(kind_counts(&work), (1, 1));
        assert_eq!(kind_counts(&project_work(&s, "pane/web")), (0, 1));
    }

    #[test]
    fn project_deliverables_skip_empty_groups_and_other_projects() {
        let mut s = Snapshot::new("a", "t");
        s.deliverables = vec![
            doc_of(
                DeliverableKind::Plan,
                Some("pane"),
                "pane/docs/plans/a.md",
                Some("2026-11-01"),
            ),
            doc_of(
                DeliverableKind::Plan,
                Some("pane"),
                "pane/docs/plans/b.md",
                Some("2026-11-20"),
            ),
            doc_of(
                DeliverableKind::Retro,
                Some("pane/web"),
                "pane/web/docs/retros/r.md",
                Some("2026-11-21"),
            ),
            doc_of(
                DeliverableKind::Decision,
                Some("cookbook"),
                "cookbook/docs/decisions/d.md",
                None,
            ),
            doc_of(
                DeliverableKind::Decision,
                None,
                "standards/docs/decisions/0001.md",
                None,
            ),
        ];
        let groups: Vec<_> = project_deliverables(&s, "pane")
            .iter()
            .map(|(_, heading, items)| (*heading, items.len()))
            .collect();
        assert_eq!(groups, [("Plans", 2), ("Retros", 1)]);
        let latest: Vec<_> = latest_docs(&s, "pane")
            .iter()
            .map(|(label, d)| (*label, d.path.as_deref().unwrap_or_default()))
            .collect();
        assert_eq!(
            latest,
            [
                ("Latest plan", "pane/docs/plans/b.md"),
                ("Latest retro", "pane/web/docs/retros/r.md"),
            ]
        );
        assert!(latest_docs(&s, "nothing").is_empty());
        assert_eq!(
            github_tree_url("abc", "pane/web"),
            "https://github.com/jimhillr/monorepo/tree/abc/pane/web"
        );
    }
}
