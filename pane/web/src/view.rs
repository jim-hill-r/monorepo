//! Pure helpers behind the pages, kept out of the components so they are
//! unit tested natively.

use pane_model::{Project, ProjectStatus, Snapshot, WorkKind};

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

#[cfg(test)]
mod tests {
    use super::*;
    use pane_model::{WorkItem, WorkSource};

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
}
