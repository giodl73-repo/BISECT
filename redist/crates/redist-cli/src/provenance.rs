//! Provenance metadata for the running `redist` binary.
//!
//! Values are baked at compile time by `build.rs` and exposed here as
//! constants. They populate the `binary_version` / `binary_build_commit`
//! / `rustc_version` fields written into manifest.json and into output
//! provenance blocks. A special master can use these to attest that a
//! particular plan came from a particular source tree.
//!
//! See: docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md
//!      §Provenance and Reproducibility

use serde::{Deserialize, Serialize};

/// Cargo package version of `redist-cli` (from Cargo.toml `version`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Full git commit at build time (40 hex chars, with optional "-dirty" suffix).
pub const BUILD_COMMIT: &str = env!("REDIST_BUILD_COMMIT");

/// Short (12-char) form of [`BUILD_COMMIT`], with optional "-dirty" suffix.
pub const BUILD_COMMIT_SHORT: &str = env!("REDIST_BUILD_COMMIT_SHORT");

/// Build timestamp as Unix seconds since epoch (string form, UTC).
pub const BUILD_UNIX: &str = env!("REDIST_BUILD_UNIX");

/// `rustc --version` output captured at build time.
pub const RUSTC_VERSION: &str = env!("REDIST_RUSTC_VERSION");

/// Provenance block embedded in output JSON files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Cargo package version (e.g. "0.1.2").
    pub redist_version: String,
    /// Git commit at build time (40-hex-char + optional "-dirty").
    pub redist_build_commit: String,
    /// ISO-8601 UTC build timestamp.
    pub redist_build_date: String,
    /// rustc version that built this binary.
    pub rustc_version: String,
}

impl Provenance {
    /// Capture the running binary's provenance.
    pub fn current() -> Self {
        Self {
            redist_version: VERSION.to_string(),
            redist_build_commit: BUILD_COMMIT.to_string(),
            redist_build_date: format_unix_iso8601(BUILD_UNIX),
            rustc_version: RUSTC_VERSION.to_string(),
        }
    }

    /// Verify that a manifest's binary_version matches the running binary.
    /// Returns Ok(()) if compatible; Err with a description if not.
    pub fn verify_version_matches(&self, manifest_version: &str) -> Result<(), String> {
        if manifest_version == self.redist_version {
            return Ok(());
        }
        Err(format!(
            "binary version mismatch: manifest records {} but this binary is {}",
            manifest_version, self.redist_version
        ))
    }
}

/// Format a Unix-seconds string as ISO-8601 UTC.
/// We avoid pulling in chrono/time just for this — manual conversion.
fn format_unix_iso8601(unix_secs: &str) -> String {
    let secs: i64 = unix_secs.parse().unwrap_or(0);
    // Days since 1970-01-01 (Thursday)
    let days = secs.div_euclid(86_400);
    let time_of_day = secs.rem_euclid(86_400);
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Convert days since 1970-01-01 into (year, month, day). Proleptic Gregorian.
fn days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    // Compute year by stepping forward
    let mut year: i32 = 1970;
    loop {
        let leap = is_leap(year);
        let dy = if leap { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        year += 1;
    }
    // Compute month
    let mdays = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month: u32 = 1;
    for &m in &mdays {
        if days < m {
            break;
        }
        days -= m;
        month += 1;
    }
    let day = (days + 1) as u32;
    (year, month, day)
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_populated() {
        assert!(!VERSION.is_empty(), "VERSION must be set from CARGO_PKG_VERSION");
        assert!(!BUILD_COMMIT.is_empty(), "BUILD_COMMIT must be set by build.rs");
        assert!(!RUSTC_VERSION.is_empty());
    }

    #[test]
    fn test_provenance_current_round_trips_via_json() {
        let p = Provenance::current();
        let json = serde_json::to_string(&p).unwrap();
        let p2: Provenance = serde_json::from_str(&json).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn test_verify_version_matches_self() {
        let p = Provenance::current();
        p.verify_version_matches(&p.redist_version).expect("self-version must match");
    }

    #[test]
    fn test_verify_version_mismatch() {
        let p = Provenance::current();
        let err = p.verify_version_matches("999.999.999").expect_err("must reject mismatch");
        assert!(err.contains("999.999.999"));
        assert!(err.contains("mismatch"));
    }

    #[test]
    fn test_format_unix_iso8601_known_dates() {
        // Epoch
        assert_eq!(format_unix_iso8601("0"), "1970-01-01T00:00:00Z");
        // Leap-day boundary: 2024-03-01 00:00 UTC
        // Days from 1970-01-01 to 2024-01-01: 54*365 + 13 leap days = 19723
        // Plus Jan(31) + Feb(29 in leap) = 60. Total 19783 days. 19783 * 86400 = 1709251200.
        assert_eq!(format_unix_iso8601("1709251200"), "2024-03-01T00:00:00Z");
        // 2026-04-29 12:00 UTC: days from 1970 = 56*365 + 14 leap = 20454; +118 = 20572.
        // 20572 * 86400 + 43200 = 1777464000.
        assert_eq!(format_unix_iso8601("1777464000"), "2026-04-29T12:00:00Z");
    }

    #[test]
    fn test_is_leap() {
        assert!(is_leap(2000)); // century divisible by 400
        assert!(!is_leap(1900)); // century not divisible by 400
        assert!(is_leap(2024));
        assert!(!is_leap(2023));
    }

    #[test]
    fn test_build_commit_short_is_prefix_of_full() {
        // Allow for "-dirty" suffix on either
        let full = BUILD_COMMIT.trim_end_matches("-dirty");
        let short = BUILD_COMMIT_SHORT.trim_end_matches("-dirty");
        if full == "unknown" {
            assert_eq!(short, "unknown");
        } else {
            assert!(full.starts_with(short),
                "short commit must be a prefix of full: full={full} short={short}");
            assert!(short.len() <= 12);
        }
    }

    // ── Additional provenance tests ───────────────────────────────────────────

    #[test]
    fn test_format_unix_epoch_is_utc_midnight() {
        let result = format_unix_iso8601("0");
        assert!(result.ends_with("T00:00:00Z"), "epoch must be midnight UTC: {result}");
        assert!(result.starts_with("1970-01-01"), "epoch must be 1970-01-01: {result}");
    }

    #[test]
    fn test_format_unix_one_second_before_midnight() {
        // 86399 = 23:59:59 on 1970-01-01
        let result = format_unix_iso8601("86399");
        assert_eq!(result, "1970-01-01T23:59:59Z");
    }

    #[test]
    fn test_format_unix_midnight_next_day() {
        // 86400 = start of 1970-01-02
        let result = format_unix_iso8601("86400");
        assert_eq!(result, "1970-01-02T00:00:00Z");
    }

    #[test]
    fn test_format_unix_dec_31_1999() {
        // 1999-12-31 00:00 UTC. Days from 1970-01-01 to 1999-12-31:
        // 30 years (1970..1999), leap years: 72,76,80,84,88,92,96 = 7 → 30*365+7=10957
        // 1999 is NOT a leap year; Jan-Dec 31 days = day 365 - 1 = day 364 (0-indexed)
        // So 10957-1+364 = nope, let's just spot-check:
        // Unix: 946598400 = 1999-12-31T00:00:00Z (well-known)
        let result = format_unix_iso8601("946598400");
        assert_eq!(result, "1999-12-31T00:00:00Z");
    }

    #[test]
    fn test_format_unix_new_year_2000() {
        // 2000-01-01 00:00 UTC = Unix 946684800
        let result = format_unix_iso8601("946684800");
        assert_eq!(result, "2000-01-01T00:00:00Z");
    }

    #[test]
    fn test_format_unix_result_has_iso8601_format() {
        // Result must match YYYY-MM-DDTHH:MM:SSZ pattern
        let result = format_unix_iso8601("1000000000");
        assert_eq!(result.len(), 20, "ISO-8601 timestamp must be 20 chars: {result}");
        assert!(result.contains('T'), "must contain T separator: {result}");
        assert!(result.ends_with('Z'), "must end with Z (UTC): {result}");
        let date_part = &result[..10];
        let time_part = &result[11..19];
        assert!(date_part.contains('-'), "date part must have dashes: {date_part}");
        assert!(time_part.contains(':'), "time part must have colons: {time_part}");
    }

    #[test]
    fn test_is_leap_year_1600_is_leap() {
        assert!(is_leap(1600), "1600 is a leap year (divisible by 400)");
    }

    #[test]
    fn test_is_leap_year_2100_not_leap() {
        assert!(!is_leap(2100), "2100 is not a leap year (divisible by 100 but not 400)");
    }

    #[test]
    fn test_is_leap_year_2000_is_leap() {
        assert!(is_leap(2000), "2000 is a leap year");
    }

    #[test]
    fn test_is_leap_year_2001_not_leap() {
        assert!(!is_leap(2001), "2001 is not a leap year");
    }

    #[test]
    fn test_provenance_redist_version_semver_format() {
        let p = Provenance::current();
        // Version must look like x.y.z (at least two dots)
        let dot_count = p.redist_version.chars().filter(|&c| c == '.').count();
        assert!(dot_count >= 2,
            "redist_version must be semver (x.y.z); got: {}", p.redist_version);
    }

    #[test]
    fn test_provenance_build_date_looks_like_iso8601() {
        let p = Provenance::current();
        // Must be YYYY-MM-DDTHH:MM:SSZ
        assert!(p.redist_build_date.len() == 20,
            "build date must be 20-char ISO-8601; got: {}", p.redist_build_date);
        assert!(p.redist_build_date.ends_with('Z'),
            "build date must be UTC (ends with Z); got: {}", p.redist_build_date);
        assert!(p.redist_build_date.contains('T'),
            "build date must have T separator; got: {}", p.redist_build_date);
    }

    #[test]
    fn test_provenance_rustc_version_contains_rustc() {
        let p = Provenance::current();
        // rustc_version is captured from `rustc --version` output like "rustc 1.x.y (...)"
        assert!(p.rustc_version.contains("rustc"),
            "rustc_version must contain 'rustc'; got: {}", p.rustc_version);
    }

    #[test]
    fn test_format_unix_non_numeric_returns_epoch() {
        // Non-numeric input → parse fails → secs=0 → epoch
        let result = format_unix_iso8601("not_a_number");
        assert_eq!(result, "1970-01-01T00:00:00Z",
            "non-numeric unix_secs must fall back to epoch");
    }
}
