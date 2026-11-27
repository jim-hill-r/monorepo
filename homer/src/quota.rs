//! Keep homer inside the RapidAPI free plan's monthly request quota.
//!
//! RapidAPI reports the plan's quota on every response in three headers:
//!
//! - `x-ratelimit-requests-limit`: requests allowed in the current period,
//! - `x-ratelimit-requests-remaining`: requests left in it,
//! - `x-ratelimit-requests-reset`: seconds until the period starts again.
//!
//! homer is a one-shot CLI, so it saves the last values it saw to a small JSON
//! file. Before each request it reads that file and stops if the quota is used
//! up and has not reset yet; after each response it saves the new values and
//! warns when few requests are left. The free plan must never cost money
//! (AGENTS.md), so homer stops rather than going past the quota.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

pub const LIMIT_HEADER: &str = "x-ratelimit-requests-limit";
pub const REMAINING_HEADER: &str = "x-ratelimit-requests-remaining";
pub const RESET_HEADER: &str = "x-ratelimit-requests-reset";

/// Warn when this many requests or fewer are left and the plan's limit is
/// unknown.
const DEFAULT_WARN_AT: u64 = 5;

/// The quota RapidAPI last reported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaState {
    /// Requests allowed per period, if RapidAPI sent it.
    pub limit: Option<u64>,
    /// Requests left in the current period.
    pub remaining: u64,
    /// When the period resets, in seconds since the Unix epoch, if known.
    pub resets_at: Option<u64>,
}

impl QuotaState {
    /// Read the quota from a RapidAPI response's headers, or `None` if the
    /// response doesn't say how many requests are left.
    pub fn from_headers(headers: &HeaderMap, now: u64) -> Option<Self> {
        let number =
            |name: &str| -> Option<u64> { headers.get(name)?.to_str().ok()?.trim().parse().ok() };
        Some(Self {
            limit: number(LIMIT_HEADER),
            remaining: number(REMAINING_HEADER)?,
            resets_at: number(RESET_HEADER).map(|secs| now.saturating_add(secs)),
        })
    }

    /// Stop if the quota is used up and has not reset yet.
    ///
    /// With no reset time, a used-up quota stays used up until a later
    /// response says otherwise, or the quota file is deleted by hand.
    pub fn ensure_request_allowed(&self, now: u64) -> Result<()> {
        if self.remaining > 0 {
            return Ok(());
        }
        match self.resets_at {
            Some(reset) if now >= reset => Ok(()),
            Some(reset) => anyhow::bail!(
                "The RapidAPI free quota{} is used up. homer stops here so it never goes \
                 past the free plan; try again in {}.",
                limit_suffix(self.limit),
                describe_wait(reset - now)
            ),
            None => anyhow::bail!(
                "The RapidAPI free quota{} is used up and RapidAPI did not say when it \
                 resets. homer stops here so it never goes past the free plan. Check your \
                 usage on RapidAPI, then delete the quota file to try again.",
                limit_suffix(self.limit)
            ),
        }
    }

    /// The quota after a request whose response didn't report one: one
    /// request fewer left, so the guard never runs on a stale count.
    ///
    /// Returns `None` once the saved period has reset, because the saved
    /// count no longer describes the current period.
    pub fn after_unreported_request(&self, now: u64) -> Option<Self> {
        if self.resets_at.is_some_and(|reset| now >= reset) {
            return None;
        }
        Some(Self {
            remaining: self.remaining.saturating_sub(1),
            ..self.clone()
        })
    }

    /// A warning to print when the quota is nearly used up.
    pub fn warning(&self) -> Option<String> {
        let warn_at = self
            .limit
            .map(|limit| (limit / 10).max(1))
            .unwrap_or(DEFAULT_WARN_AT);
        if self.remaining > warn_at {
            return None;
        }
        let left = match self.limit {
            Some(limit) => format!("{} of {}", self.remaining, limit),
            None => self.remaining.to_string(),
        };
        let plural = if self.remaining == 1 { "" } else { "s" };
        Some(format!(
            "warning: only {left} RapidAPI free request{plural} left this period. homer \
             will stop before going past the free quota."
        ))
    }
}

fn limit_suffix(limit: Option<u64>) -> String {
    limit
        .map(|l| format!(" ({l} requests)"))
        .unwrap_or_default()
}

/// Describe a wait in whole days, hours or minutes, rounded up.
fn describe_wait(secs: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    let (count, unit) = if secs >= DAY {
        (secs.div_ceil(DAY), "day")
    } else if secs >= HOUR {
        (secs.div_ceil(HOUR), "hour")
    } else {
        (secs.div_ceil(MINUTE).max(1), "minute")
    };
    let plural = if count == 1 { "" } else { "s" };
    format!("about {count} {unit}{plural}")
}

/// Seconds since the Unix epoch.
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Where homer keeps the last quota it saw, unless `--quota-file` says
/// otherwise: `$XDG_STATE_HOME/homer/rapidapi-quota.json`, falling back to
/// `~/.local/state/homer/rapidapi-quota.json`.
pub fn default_quota_file() -> Option<PathBuf> {
    let state_home = std::env::var_os("XDG_STATE_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|v| !v.is_empty())
                .map(|home| PathBuf::from(home).join(".local").join("state"))
        })?;
    Some(state_home.join("homer").join("rapidapi-quota.json"))
}

/// Read the saved quota. A missing file means homer hasn't seen one yet.
///
/// An unreadable file is an error rather than "no quota", so a corrupt file
/// can't switch the guard off.
pub fn load(path: &Path) -> Result<Option<QuotaState>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(e).with_context(|| {
                format!("Failed to read the RapidAPI quota file {}", path.display())
            });
        }
    };
    serde_json::from_str(&text).map(Some).with_context(|| {
        format!(
            "The RapidAPI quota file {} is not valid. Delete it to start over.",
            path.display()
        )
    })
}

/// Save the quota, creating the file's folder if needed.
///
/// The new state goes to a temporary file next to `path`, which is then
/// renamed over it, so an interrupted save leaves the old file whole instead
/// of a partial one that would block every lookup.
pub fn save(path: &Path, state: &QuotaState) -> Result<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }
    let json = serde_json::to_string_pretty(state)?;
    let temp = temp_path(path);
    fs::write(&temp, json + "\n")
        .with_context(|| format!("Failed to write the RapidAPI quota file {}", temp.display()))?;
    fs::rename(&temp, path)
        .with_context(|| format!("Failed to write the RapidAPI quota file {}", path.display()))
}

/// The temporary file `save` writes before renaming it over `path`.
fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_static(value));
        }
        map
    }

    fn state(limit: Option<u64>, remaining: u64, resets_at: Option<u64>) -> QuotaState {
        QuotaState {
            limit,
            remaining,
            resets_at,
        }
    }

    #[test]
    fn from_headers_reads_all_three() {
        let map = headers(&[
            (LIMIT_HEADER, "100"),
            (REMAINING_HEADER, "42"),
            (RESET_HEADER, "3600"),
        ]);
        assert_eq!(
            QuotaState::from_headers(&map, 1_000),
            Some(state(Some(100), 42, Some(4_600)))
        );
    }

    #[test]
    fn from_headers_needs_remaining() {
        let map = headers(&[(LIMIT_HEADER, "100"), (RESET_HEADER, "3600")]);
        assert_eq!(QuotaState::from_headers(&map, 0), None);
        let map = headers(&[(REMAINING_HEADER, "lots")]);
        assert_eq!(QuotaState::from_headers(&map, 0), None);
    }

    #[test]
    fn from_headers_without_limit_or_reset() {
        let map = headers(&[(REMAINING_HEADER, " 7 ")]);
        assert_eq!(
            QuotaState::from_headers(&map, 0),
            Some(state(None, 7, None))
        );
    }

    #[test]
    fn requests_allowed_while_quota_left() {
        assert!(state(Some(100), 1, None).ensure_request_allowed(0).is_ok());
    }

    #[test]
    fn used_up_quota_stops_until_reset() {
        let s = state(Some(100), 0, Some(10_000));
        let err = s
            .ensure_request_allowed(10_000 - 2 * 3600)
            .expect_err("used-up quota must stop");
        let message = err.to_string();
        assert!(message.contains("used up"), "{message}");
        assert!(message.contains("(100 requests)"), "{message}");
        assert!(message.contains("about 2 hours"), "{message}");
        assert!(s.ensure_request_allowed(10_000).is_ok());
    }

    #[test]
    fn used_up_quota_without_reset_stops() {
        let err = state(None, 0, None)
            .ensure_request_allowed(u64::MAX)
            .expect_err("used-up quota with no reset time must stop");
        assert!(err.to_string().contains("delete the quota file"), "{err}");
    }

    #[test]
    fn warning_at_ten_percent_of_limit() {
        assert_eq!(state(Some(100), 11, None).warning(), None);
        let warning = state(Some(100), 10, None).warning().expect("warn at 10%");
        assert!(warning.contains("10 of 100"), "{warning}");
    }

    #[test]
    fn warning_with_small_or_unknown_limit() {
        assert_eq!(state(Some(5), 2, None).warning(), None);
        let warning = state(Some(5), 1, None).warning().expect("warn at last one");
        assert!(
            warning.contains("1 of 5 RapidAPI free request left"),
            "{warning}"
        );
        assert_eq!(state(None, 6, None).warning(), None);
        assert!(state(None, 5, None).warning().is_some());
    }

    #[test]
    fn describe_wait_rounds_up() {
        assert_eq!(describe_wait(0), "about 1 minute");
        assert_eq!(describe_wait(61), "about 2 minutes");
        assert_eq!(describe_wait(3600), "about 1 hour");
        assert_eq!(describe_wait(3 * 86_400 + 1), "about 4 days");
    }

    #[test]
    fn unreported_request_lowers_remaining_by_one() {
        let s = state(Some(100), 3, Some(500));
        assert_eq!(
            s.after_unreported_request(100),
            Some(state(Some(100), 2, Some(500)))
        );
        assert_eq!(
            state(None, 0, None).after_unreported_request(100),
            Some(state(None, 0, None)),
            "never below zero"
        );
    }

    #[test]
    fn unreported_request_after_reset_changes_nothing() {
        // The period has restarted, so the saved count is no longer the
        // current one; leave it for the next response to correct.
        assert_eq!(
            state(Some(100), 0, Some(500)).after_unreported_request(500),
            None
        );
    }

    #[test]
    fn save_replaces_an_existing_file_and_ignores_leftover_temp_files() {
        let dir = std::env::temp_dir().join(format!("homer-quota-atomic-{}", std::process::id()));
        fs::remove_dir_all(&dir).ok();
        let path = dir.join("quota.json");
        save(&path, &state(Some(100), 9, None)).expect("first save");
        save(&path, &state(Some(100), 8, None)).expect("second save");
        assert_eq!(load(&path).expect("load"), Some(state(Some(100), 8, None)));
        // A temp file left by an interrupted save doesn't affect the quota.
        fs::write(temp_path(&path), "{ partial").expect("write leftover");
        assert_eq!(load(&path).expect("load"), Some(state(Some(100), 8, None)));
        save(&path, &state(Some(100), 7, None)).expect("save over leftover");
        assert_eq!(load(&path).expect("load"), Some(state(Some(100), 7, None)));
        let names: Vec<_> = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(|e| e.ok().map(|e| e.file_name()))
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from("quota.json")]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = std::env::temp_dir().join(format!("homer-quota-test-{}", std::process::id()));
        let path = dir.join("nested").join("quota.json");
        let s = state(Some(100), 3, Some(123));
        save(&path, &s).expect("save quota");
        assert_eq!(load(&path).expect("load quota"), Some(s));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_missing_file_is_none_and_corrupt_file_is_error() {
        let dir = std::env::temp_dir().join(format!("homer-quota-bad-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        assert_eq!(
            load(&dir.join("missing.json")).expect("missing is ok"),
            None
        );
        let bad = dir.join("bad.json");
        fs::write(&bad, "not json").expect("write bad file");
        let err = load(&bad).expect_err("corrupt file must be an error");
        assert!(err.to_string().contains("Delete it"), "{err}");
        fs::remove_dir_all(&dir).ok();
    }
}
