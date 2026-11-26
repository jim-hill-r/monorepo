//! A calendar month, written `YYYY-MM` in the TOML files.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Month {
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
}

impl Month {
    pub fn new(year: i32, month: u32) -> Option<Self> {
        (1..=12).contains(&month).then_some(Self { year, month })
    }
}

impl FromStr for Month {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bad = || format!("`{s}` is not a month written YYYY-MM");
        let (y, m) = s.split_once('-').ok_or_else(bad)?;
        if y.len() != 4 || m.len() != 2 {
            return Err(bad());
        }
        let year = y.parse().map_err(|_| bad())?;
        let month = m.parse().map_err(|_| bad())?;
        Month::new(year, month).ok_or_else(bad)
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

impl<'de> Deserialize<'de> for Month {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_month_parses_and_prints() {
        assert_eq!(
            "2026-11".parse::<Month>(),
            Ok(Month {
                year: 2026,
                month: 11
            })
        );
        assert_eq!(
            Month {
                year: 2026,
                month: 3
            }
            .to_string(),
            "2026-03"
        );
    }

    #[test]
    fn test_month_rejects_bad_text() {
        for bad in ["2026-13", "2026-1", "26-11", "2026/11", "november"] {
            assert!(bad.parse::<Month>().is_err(), "{bad}");
        }
    }
}
