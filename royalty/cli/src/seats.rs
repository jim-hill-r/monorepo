//! Seats: the distinct people who committed to the consumer's project in a
//! period, from `git log`. Emails are read here to tell people apart and
//! are never written to any output (ROYL-005); only the count leaves this
//! module.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use crate::period::Month;

/// The number of distinct commit authors of the project in `dir` between
/// the start of `from` (inclusive) and the start of `until` (exclusive), by
/// author date in UTC, bots left out. Only commits
/// that touch `dir` count, so in a monorepo it is the people working on
/// this project. `None` when git can't tell (not a git repository).
pub fn count(dir: &Path, from: Month, until: Month) -> Option<u64> {
    // The whole history is read and filtered here by author date: `git log
    // --since` stops walking at the first older commit, which a rebase or a
    // merged old branch can put early.
    let output = Command::new("git")
        .args([
            "log",
            "--no-merges",
            "--format=%at%x09%ae%x09%an",
            "--",
            ".",
        ])
        .current_dir(dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let (start, end) = (from.start_timestamp(), until.start_timestamp());
    let text = String::from_utf8_lossy(&output.stdout);
    let people: BTreeSet<String> = text
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let when: i64 = fields.next()?.parse().ok()?;
            let email = fields.next()?;
            let name = fields.next().unwrap_or("");
            (start <= when && when < end && !is_bot(email, name))
                .then(|| email.trim().to_lowercase())
        })
        .filter(|email| !email.is_empty())
        .collect();
    Some(people.len() as u64)
}

/// Bots and automation accounts are not seats.
pub fn is_bot(email: &str, name: &str) -> bool {
    let email = email.to_lowercase();
    let name = name.to_lowercase();
    name.ends_with("[bot]")
        || email.contains("[bot]")
        || ["dependabot", "github-actions", "renovate"]
            .iter()
            .any(|bot| name.contains(bot) || email.contains(bot))
}

#[cfg(test)]
mod tests {
    use super::is_bot;

    #[test]
    fn bots_are_not_seats() {
        assert!(is_bot(
            "49699333+dependabot[bot]@users.noreply.github.com",
            "dependabot[bot]"
        ));
        assert!(is_bot("action@github.com", "github-actions"));
        assert!(!is_bot("ana@example.com", "Ana Example"));
    }
}
