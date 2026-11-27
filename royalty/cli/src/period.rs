//! Report months and producer periods ("Periods" in docs/royalty-toml.md).
//! A report covers one calendar month in UTC; a quarterly or yearly family
//! is due in the last month of its period, for the whole period.

use std::fmt;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use std::time::{SystemTime, UNIX_EPOCH};

use royalty_model::Period;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Month {
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
}

impl Month {
    pub fn current() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let (year, month, _) = civil_from_days((seconds / 86_400) as i64);
        Month { year, month }
    }

    pub fn next(self) -> Self {
        if self.month == 12 {
            Month {
                year: self.year + 1,
                month: 1,
            }
        } else {
            Month {
                year: self.year,
                month: self.month + 1,
            }
        }
    }

    /// Whether a family with this period is due in this month.
    pub fn is_due(self, period: Period) -> bool {
        match period {
            Period::Month => true,
            Period::Quarter => self.month.is_multiple_of(3),
            Period::Year => self.month == 12,
        }
    }

    /// The first month of the producer's period that ends in this month.
    pub fn period_start(self, period: Period) -> Month {
        let first = match period {
            Period::Month => self.month,
            Period::Quarter => self.month.saturating_sub(2).max(1),
            Period::Year => 1,
        };
        Month {
            year: self.year,
            month: first,
        }
    }

    /// Seconds since the Unix epoch at the start of the month, UTC.
    pub fn start_timestamp(self) -> i64 {
        days_from_civil(self.year, self.month, 1) * 86_400
    }
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's
/// algorithm).
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The date of a day count since 1970-01-01.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
    (year, month, day)
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

impl FromStr for Month {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> Result<Self> {
        let (year, month) = text
            .split_once('-')
            .with_context(|| format!("period {text:?} is not YYYY-MM"))?;
        let year: i32 = year
            .parse()
            .with_context(|| format!("period {text:?} is not YYYY-MM"))?;
        let month: u32 = month
            .parse()
            .with_context(|| format!("period {text:?} is not YYYY-MM"))?;
        if !(1..=12).contains(&month) || year.to_string().len() != 4 {
            bail!("period {text:?} is not YYYY-MM");
        }
        Ok(Month { year, month })
    }
}

#[cfg(test)]
mod tests {
    use super::Month;
    use royalty_model::Period;

    fn m(text: &str) -> Month {
        match text.parse() {
            Ok(month) => month,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn periods_fall_due_in_their_last_month() {
        assert!(m("2026-11").is_due(Period::Month));
        assert!(!m("2026-11").is_due(Period::Quarter));
        assert!(m("2026-12").is_due(Period::Quarter));
        assert!(!m("2026-11").is_due(Period::Year));
        assert!(m("2026-12").is_due(Period::Year));
        assert_eq!(m("2026-12").period_start(Period::Quarter), m("2026-10"));
        assert_eq!(m("2026-12").period_start(Period::Year), m("2026-01"));
        assert_eq!(m("2026-12").next(), m("2027-01"));
        assert_eq!(m("2026-11").start_timestamp(), 1_793_491_200);
        assert_eq!(m("2024-03").start_timestamp(), 1_709_251_200);
        assert_eq!(super::civil_from_days(20_513), (2026, 3, 1));
        assert_eq!(super::civil_from_days(0), (1970, 1, 1));
        assert!("2026-13".parse::<Month>().is_err());
        assert!("26-01".parse::<Month>().is_err());
    }
}
