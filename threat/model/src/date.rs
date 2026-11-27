//! A calendar date, so `reviewed` can be compared with today without a date
//! crate.

use std::fmt;
use std::str::FromStr;

/// A proleptic Gregorian date, written `YYYY-MM-DD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// `None` when the month or day is out of range.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Date> {
        let valid = (1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month);
        valid.then_some(Date { year, month, day })
    }

    /// Days since 1970-01-01 (negative before it).
    pub fn days(self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` after 1970-01-01.
    pub fn from_days(days: i64) -> Date {
        // Howard Hinnant's civil_from_days.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        Date {
            year: i32::try_from(year).unwrap_or(i32::MAX),
            month: u32::try_from(month).unwrap_or(1),
            day: u32::try_from(day).unwrap_or(1),
        }
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl FromStr for Date {
    type Err = String;

    fn from_str(s: &str) -> Result<Date, String> {
        let bad = || format!("`{s}` is not a date written YYYY-MM-DD");
        let mut parts = s.split('-');
        let (Some(y), Some(m), Some(d), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(bad());
        };
        if y.len() != 4 || m.len() != 2 || d.len() != 2 {
            return Err(bad());
        }
        let (Ok(y), Ok(m), Ok(d)) = (y.parse(), m.parse(), d.parse()) else {
            return Err(bad());
        };
        Date::new(y, m, d).ok_or_else(bad)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parses_and_prints() {
        let d: Date = "2026-11-27".parse().expect("date");
        assert_eq!(
            d,
            Date {
                year: 2026,
                month: 11,
                day: 27
            }
        );
        assert_eq!(d.to_string(), "2026-11-27");
    }

    #[test]
    fn test_rejects_bad_dates() {
        for s in [
            "2026-13-01",
            "2026-02-29",
            "27 Nov 2026",
            "2026-1-01",
            "2026-11-27-1",
            "",
        ] {
            assert!(s.parse::<Date>().is_err(), "{s}");
        }
        assert!("2024-02-29".parse::<Date>().is_ok());
    }

    #[test]
    fn test_days_round_trip() {
        assert_eq!(Date::new(1970, 1, 1).map(Date::days), Some(0));
        let d = Date::new(2026, 11, 27).expect("date");
        assert_eq!(Date::from_days(d.days()), d);
        let leap = Date::new(2024, 2, 29).expect("date");
        assert_eq!(
            Date::from_days(leap.days() + 1),
            Date::new(2024, 3, 1).expect("date")
        );
        let a = Date::new(2026, 5, 31).expect("date");
        assert_eq!(d.days() - a.days(), 180);
    }
}
