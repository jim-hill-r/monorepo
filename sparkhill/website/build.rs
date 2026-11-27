//! Sets `JIMHILLR_BUILD_YEAR` to the year the site is built, for the
//! footer's copyright line. The engine and the browser bundle are built
//! together by `perseus export`, so both see the same year and hydration
//! matches the exported HTML.

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    println!("cargo:rustc-env=JIMHILLR_BUILD_YEAR={}", year_of(days));
    // Cargo reruns this only when build.rs changes, so a local build cached
    // across New Year keeps the old year; CI and deploys build fresh.
    println!("cargo:rerun-if-changed=build.rs");
}

/// The Gregorian year of a day counted from 1970-01-01 (Howard Hinnant's
/// `civil_from_days`).
fn year_of(days: i64) -> i64 {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    era * 400 + yoe + i64::from(month <= 2)
}
