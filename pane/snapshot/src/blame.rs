//! The date a line was added, from `git blame`, so pane can age a `FIX`
//! comment the way an issue file's `added` date ages a `FIX` file.

use std::path::Path;
use std::process::Command;

use crate::time;

/// Whether `root` is a shallow clone. Its oldest commit looks like a root
/// commit, so blame would date every older line to it; don't blame there.
pub fn is_shallow(root: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-shallow-repository"])
        .output()
        .is_ok_and(|out| out.status.success() && out.stdout.starts_with(b"true"))
}

/// The `YYYY-MM-DD` (UTC) of the commit that last changed line `line` of
/// `file` (a path from the repo root), or `None` when git can't say: not a
/// git checkout, or an uncommitted line. Call it only when
/// [`is_shallow`] is false.
pub fn line_date(root: &Path, file: &str, line: u32) -> Option<String> {
    let range = format!("{line},{line}");
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        // --root: the repo's first commit is a real date, not a boundary.
        .args(["blame", "--root", "--porcelain", "-L", &range, "--", file])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    porcelain_date(&String::from_utf8_lossy(&out.stdout))
}

/// The committer date in one line's `git blame --porcelain` output, as
/// `YYYY-MM-DD`. `None` for a boundary commit or the all-zero commit of a
/// line that isn't committed yet.
pub fn porcelain_date(porcelain: &str) -> Option<String> {
    let mut lines = porcelain.lines();
    let sha = lines.next()?.split_whitespace().next()?;
    if sha.bytes().all(|b| b == b'0') {
        return None;
    }
    let mut secs = None;
    for line in lines {
        if line == "boundary" {
            return None;
        }
        if let Some(t) = line.strip_prefix("committer-time ") {
            secs = t.trim().parse::<u64>().ok();
        }
        if line.starts_with('\t') {
            break;
        }
    }
    let stamp = time::rfc3339(secs?);
    stamp.get(..10).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMITTED: &str = "\
3f1c2a9d0e4b5c6a7f8e9d0c1b2a3f4e5d6c7b8a 12 12 1
author Claude
author-mail <noreply@anthropic.com>
author-time 1795000000
author-tz +0000
committer Claude
committer-mail <noreply@anthropic.com>
committer-time 1795704520
committer-tz +0000
summary pane: something
filename pane/src/x.rs
\t// FIX: broken
";

    #[test]
    fn reads_the_committer_date() {
        assert_eq!(porcelain_date(COMMITTED).as_deref(), Some("2026-11-26"));
    }

    #[test]
    fn a_boundary_commit_has_no_date() {
        let boundary = COMMITTED.replace("filename", "boundary\nfilename");
        assert_eq!(porcelain_date(&boundary), None);
    }

    #[test]
    fn an_uncommitted_line_has_no_date() {
        let uncommitted = COMMITTED.replace(
            "3f1c2a9d0e4b5c6a7f8e9d0c1b2a3f4e5d6c7b8a",
            "0000000000000000000000000000000000000000",
        );
        assert_eq!(porcelain_date(&uncommitted), None);
    }

    #[test]
    fn junk_has_no_date() {
        assert_eq!(porcelain_date(""), None);
        assert_eq!(porcelain_date("abc 1 1 1\nfilename x\n\tline\n"), None);
    }

    #[test]
    fn outside_a_git_checkout_there_is_no_date() {
        let dir = std::env::temp_dir().join(format!("pane_blame_none_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        assert_eq!(line_date(&dir, "missing.rs", 1), None);
        assert!(!is_shallow(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
