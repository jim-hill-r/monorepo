//! One problem the check found.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// No `docs/threat.toml`.
    Thr001,
    /// The file doesn't parse, or has an unknown field, a bad value or a
    /// duplicate name.
    Thr002,
    /// A surface serves non-public data publicly, or its previews don't
    /// match its access.
    Thr003,
    /// Drift: the project's files show a surface, secret, sign-in or risky
    /// construct the file doesn't account for.
    Thr004,
    /// A secret with no scope or setting, or a workflow passing a secret
    /// the project doesn't list.
    Thr005,
    /// `[auth]` breaks the sign-in rules.
    Thr006,
    /// A public web surface without the security headers.
    Thr007,
    /// A reachable surface with no threat, or a threat with no mitigation.
    Thr008,
    /// `reviewed` is old. Always a warning, even with `--strict`.
    Thr009,
}

impl Code {
    pub fn as_str(self) -> &'static str {
        match self {
            Code::Thr001 => "THR-001",
            Code::Thr002 => "THR-002",
            Code::Thr003 => "THR-003",
            Code::Thr004 => "THR-004",
            Code::Thr005 => "THR-005",
            Code::Thr006 => "THR-006",
            Code::Thr007 => "THR-007",
            Code::Thr008 => "THR-008",
            Code::Thr009 => "THR-009",
        }
    }

    /// Whether `--strict` (phase 4) turns this into a failure.
    pub fn can_fail(self) -> bool {
        self != Code::Thr009
    }
}

/// Printed as `warning[THR-003] pane/docs/threat.toml: ...`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: Code,
    /// Repo-relative path of the file the finding is about.
    pub path: String,
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "warning[{}] {}: {}",
            self.code.as_str(),
            self.path,
            self.message
        )
    }
}
