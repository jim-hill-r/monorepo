//! Conflict markers: no tracked file holds a merge conflict left unresolved.
//!
//!   STD-005  A tracked text file has a line that starts with a conflict
//!            marker (`<<<<<<< `, `||||||| ` or `>>>>>>> `, or the marker
//!            alone on its line).
//!
//! Only the opening, base and closing markers count. A line of seven `=`
//! alone is also a Markdown heading underline, so it is never flagged by
//! itself; a real conflict always has an opening and a closing marker too.
//! Files that aren't UTF-8 (images, fonts) are skipped.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Serialize;

/// One conflict marker found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule: &'static str,
    pub path: String,
    /// 1-based line number.
    pub line: usize,
    pub text: String,
}

/// Whether `line` is an opening, base or closing conflict marker.
pub fn is_marker(line: &str) -> bool {
    ['<', '|', '>'].iter().any(|&c| {
        let marker = c.to_string().repeat(7);
        line.strip_prefix(&marker)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
    })
}

/// The conflict markers in one file's text.
pub fn check_text(path: &str, text: &str) -> Vec<Finding> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| is_marker(line.trim_end_matches('\r')))
        .map(|(i, line)| Finding {
            rule: "STD-005",
            path: path.to_string(),
            line: i + 1,
            text: line.trim_end().to_string(),
        })
        .collect()
}

/// Every conflict marker in the files git tracks under `root`.
pub fn check(root: &Path) -> Result<Vec<Finding>> {
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .context("running git ls-files")?;
    if !out.status.success() {
        bail!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let mut findings = Vec::new();
    for path in out.stdout.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let path = String::from_utf8_lossy(path);
        // A file deleted in the working tree, a submodule or a symlink to a
        // folder can't be read as text: there is nothing to check.
        let Ok(bytes) = fs::read(root.join(path.as_ref())) else {
            continue;
        };
        if let Ok(text) = std::str::from_utf8(&bytes) {
            findings.extend(check_text(&path, text));
        }
    }
    Ok(findings)
}

/// The findings as `error[STD-005]` lines, or an ok line.
pub fn render(findings: &[Finding]) -> String {
    let mut out = String::new();
    for f in findings {
        let _ = writeln!(
            out,
            "error[{}] {}:{}: unresolved merge conflict marker `{}`",
            f.rule, f.path, f.line, f.text
        );
    }
    if findings.is_empty() {
        out.push_str("conflict markers: ok\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(c: char) -> String {
        c.to_string().repeat(7)
    }

    #[test]
    fn finds_every_marker_of_a_conflict() {
        let text = format!(
            "a = 1\n{} HEAD\nb = 2\n{}\nb = 3\n{} origin/main\n",
            m('<'),
            m('='),
            m('>')
        );
        let found = check_text("x.toml", &text);
        let lines: Vec<usize> = found.iter().map(|f| f.line).collect();
        assert_eq!(lines, vec![2, 6]);
        assert_eq!(found[0].rule, "STD-005");
        assert_eq!(found[0].text, format!("{} HEAD", m('<')));
    }

    #[test]
    fn finds_diff3_base_marker_and_bare_markers() {
        let text = format!("{}\n{} base\n{}\r\n", m('<'), m('|'), m('>'));
        assert_eq!(check_text("x", &text).len(), 3);
    }

    #[test]
    fn ignores_heading_underlines_and_other_text() {
        let text = format!(
            "Title\n{}\n  {} indented\n{}x no space\n<<<< short\n",
            m('='),
            m('<'),
            m('>')
        );
        assert!(check_text("README.md", &text).is_empty());
    }

    #[test]
    fn render_lists_errors_or_ok() {
        assert_eq!(render(&[]), "conflict markers: ok\n");
        let f = Finding {
            rule: "STD-005",
            path: "a.md".into(),
            line: 3,
            text: m('>'),
        };
        assert_eq!(
            render(&[f]),
            format!(
                "error[STD-005] a.md:3: unresolved merge conflict marker `{}`\n",
                m('>')
            )
        );
    }

    #[test]
    fn the_repo_has_no_conflict_markers() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let found = check(&root).expect("git ls-files");
        assert!(found.is_empty(), "{}", render(&found));
    }
}
