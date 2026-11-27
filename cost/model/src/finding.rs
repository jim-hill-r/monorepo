//! One problem the check found.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// No `docs/cost.toml`.
    Cost001,
    /// The file doesn't parse, or names an unknown usage key.
    Cost002,
    /// A blip that isn't an approved vendor.
    Cost003,
    /// A paid plan on a $0 blip, or money with no accepted decision.
    Cost004,
    /// Usage over a budget.
    Cost005,
    /// Money over a blip's monthly budget.
    Cost006,
    /// The code uses a service, or more of a counted limit, than the file
    /// lists (the detectors in `standards/radar/detectors.toml`).
    Cost007,
    /// `reviewed` is old. Always a warning, even with `--strict`.
    Cost008,
    /// An outside host no detector or allow entry knows. Always a warning.
    Cost009,
    /// A `[[cost]]` whose `renews` date has passed: the file wasn't updated
    /// after the renewal. Always a warning.
    Cost010,
}

impl Code {
    pub fn as_str(self) -> &'static str {
        match self {
            Code::Cost001 => "COST-001",
            Code::Cost002 => "COST-002",
            Code::Cost003 => "COST-003",
            Code::Cost004 => "COST-004",
            Code::Cost005 => "COST-005",
            Code::Cost006 => "COST-006",
            Code::Cost007 => "COST-007",
            Code::Cost008 => "COST-008",
            Code::Cost009 => "COST-009",
            Code::Cost010 => "COST-010",
        }
    }

    /// Whether `--strict` (phase 3) turns this into a failure.
    pub fn can_fail(self) -> bool {
        !matches!(self, Code::Cost008 | Code::Cost009 | Code::Cost010)
    }
}

/// Printed as `warning[COST-003] cookbook/docs/cost.toml: ...`.
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
