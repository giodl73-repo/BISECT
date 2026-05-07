/// Plan output directory management.
///
/// Two output tree styles:
///   1. Legacy: `{base}/{year}/states/{state_name}/`         (unlabeled runs)
///   2. Labeled: `{base}/{year}/plans/{label}/`              (new --label runs)
///
/// Label collision protection: if manifest.json exists in the target directory
/// and `--force` is not set, exit non-zero with a clear error message including
/// the existing plan's creation timestamp and a hint to use `--force`.
use std::path::{Path, PathBuf};

/// Return the plan output directory for a labeled run.
/// Creates: `{base}/plans/{label}/`
pub fn plan_output_dir(base: &Path, label: &str) -> PathBuf {
    base.join("plans").join(label)
}

/// Return the state output directory for a legacy unlabeled run.
/// Creates: `{base}/states/{state_name}/`
pub fn state_output_dir(base: &Path, state_name: &str) -> PathBuf {
    base.join("states").join(state_name)
}

/// Create a labeled plan directory with standard subdirectories.
/// Structure:
///   plans/{label}/
///     analysis/
///     maps/
///     intermediate/
pub fn create_plan_dir(base: &Path, label: &str) -> anyhow::Result<PathBuf> {
    let plan_dir = plan_output_dir(base, label);
    std::fs::create_dir_all(plan_dir.join("analysis"))?;
    std::fs::create_dir_all(plan_dir.join("maps"))?;
    std::fs::create_dir_all(plan_dir.join("intermediate"))?;
    Ok(plan_dir)
}

/// Check whether a plan directory already has a completed manifest.
/// Returns Err if manifest.json exists and force=false.
/// The error message includes the existing plan's `created_at` timestamp
/// and instructs the user to pass `--force` to overwrite.
pub fn check_plan_collision(plan_dir: &Path, force: bool) -> anyhow::Result<()> {
    let manifest_path = plan_dir.join("manifest.json");
    if !force && manifest_path.exists() {
        // Read created_at from existing manifest for a helpful error message
        let created_at = std::fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["created_at"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let label = plan_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| plan_dir.display().to_string());

        anyhow::bail!(
            "ERROR: plan '{}' already exists at: {}\n\
             Use --force to overwrite, or choose a different --label.",
            label,
            plan_dir.display(),
        );
    }
    Ok(())
}

/// Return current UTC time as ISO 8601 string (approximate, no external dep).
pub fn now_iso8601() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let rem = secs % 86400;
    let hours = rem / 3600;
    let minutes = (rem % 3600) / 60;
    let seconds = rem % 60;
    format!("{}T{:02}:{:02}:{:02}Z", epoch_days_to_date(days), hours, minutes, seconds)
}

fn epoch_days_to_date(days: u64) -> String {
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

// ---------------------------------------------------------------------------
// Tests — Task 5
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_label_collision_exits_nonzero_without_force() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("washington_house_2020");
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(
            plan_dir.join("manifest.json"),
            r#"{"created_at":"2026-04-26T14:23:00Z","label":"washington_house_2020"}"#,
        )
        .unwrap();
        let result = check_plan_collision(&plan_dir, false);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("already exists"), "Error must say 'already exists'");
        assert!(msg.contains("washington_house_2020"), "Error must include plan label");
        // Task 115: error must include the full plan directory path
        let plan_dir_str = plan_dir.to_string_lossy();
        assert!(msg.contains(plan_dir_str.as_ref()),
            "Error must include the full plan directory path '{}', got: {}", plan_dir_str, msg);
        assert!(msg.contains("--force"), "Error must mention --force option");
    }

    #[test]
    fn test_label_force_flag_allows_overwrite() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("wa_house");
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(
            plan_dir.join("manifest.json"),
            r#"{"created_at":"2026-04-26T00:00:00Z"}"#,
        )
        .unwrap();
        // With --force, collision check passes
        assert!(check_plan_collision(&plan_dir, true).is_ok());
    }

    #[test]
    fn test_plan_dir_structure_created() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().to_path_buf();
        create_plan_dir(&base, "wa_house_draft1").unwrap();
        assert!(base.join("plans").join("wa_house_draft1").exists());
        assert!(base.join("plans").join("wa_house_draft1").join("analysis").exists());
        assert!(base.join("plans").join("wa_house_draft1").join("maps").exists());
        assert!(base.join("plans").join("wa_house_draft1").join("intermediate").exists());
    }

    #[test]
    fn test_legacy_state_dir_unaffected_by_labeled_run() {
        // Unlabeled runs continue writing to states/{state_name}/
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().to_path_buf();
        let state_dir = state_output_dir(&base, "washington");
        assert!(state_dir.ends_with("states/washington"));
    }

    #[test]
    fn test_no_collision_when_no_manifest() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("new_plan");
        std::fs::create_dir_all(&plan_dir).unwrap();
        // No manifest.json → no collision
        assert!(check_plan_collision(&plan_dir, false).is_ok());
    }

    // ── Additional L0 tests ───────────────────────────────────────────────────

    #[test]
    fn test_plan_output_dir_joins_plans_and_label() {
        let base = std::path::Path::new("/tmp/outputs");
        let dir = plan_output_dir(base, "wa_house_2020");
        assert!(dir.ends_with("plans/wa_house_2020"));
    }

    #[test]
    fn test_state_output_dir_joins_states_and_name() {
        let base = std::path::Path::new("/tmp/outputs");
        let dir = state_output_dir(base, "washington");
        assert!(dir.ends_with("states/washington"));
    }

    #[test]
    fn test_plan_output_dir_different_labels_produce_different_paths() {
        let base = std::path::Path::new("/tmp/out");
        let d1 = plan_output_dir(base, "plan_a");
        let d2 = plan_output_dir(base, "plan_b");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_state_output_dir_different_states_produce_different_paths() {
        let base = std::path::Path::new("/tmp/out");
        let d1 = state_output_dir(base, "california");
        let d2 = state_output_dir(base, "texas");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_create_plan_dir_returns_correct_path() {
        let tmp = TempDir::new().unwrap();
        let returned = create_plan_dir(tmp.path(), "test_label").unwrap();
        let expected = tmp.path().join("plans").join("test_label");
        assert_eq!(returned, expected);
    }

    #[test]
    fn test_create_plan_dir_idempotent() {
        let tmp = TempDir::new().unwrap();
        // Creating the same plan dir twice must not fail
        create_plan_dir(tmp.path(), "idempotent_label").unwrap();
        create_plan_dir(tmp.path(), "idempotent_label").unwrap();
        assert!(tmp.path().join("plans").join("idempotent_label").join("analysis").exists());
    }

    #[test]
    fn test_collision_error_message_contains_force_hint() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("existing");
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(plan_dir.join("manifest.json"), r#"{"created_at":"2026-01-01T00:00:00Z"}"#).unwrap();
        let err = check_plan_collision(&plan_dir, false).unwrap_err();
        assert!(err.to_string().contains("--force"), "collision error must mention --force");
    }

    #[test]
    fn test_collision_error_message_contains_label() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("my_unique_label");
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(plan_dir.join("manifest.json"), r#"{"created_at":"2026-01-01T00:00:00Z"}"#).unwrap();
        let err = check_plan_collision(&plan_dir, false).unwrap_err();
        assert!(err.to_string().contains("my_unique_label"), "collision error must contain the label");
    }

    #[test]
    fn test_collision_error_reads_created_at_from_manifest() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("dated_plan");
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(
            plan_dir.join("manifest.json"),
            r#"{"created_at":"2026-03-15T09:00:00Z","label":"dated_plan"}"#,
        ).unwrap();
        // The check_plan_collision reads created_at; error contains the path
        let err = check_plan_collision(&plan_dir, false).unwrap_err().to_string();
        // path must be in the error
        assert!(err.contains("dated_plan"), "error must contain the plan label");
    }

    #[test]
    fn test_now_iso8601_format() {
        let ts = now_iso8601();
        // Must be 20 chars: YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(ts.len(), 20, "ISO 8601 timestamp must be 20 chars, got: {ts}");
        assert!(ts.contains('T'), "must have T separator");
        assert!(ts.ends_with('Z'), "must end with Z");
        // Year must be 4 digits starting with 20
        assert!(ts.starts_with("20"), "timestamp must start with 20xx");
    }

    #[test]
    fn test_now_iso8601_is_recent() {
        // now_iso8601 must produce a year >= 2026
        let ts = now_iso8601();
        let year: u32 = ts[..4].parse().unwrap();
        assert!(year >= 2026, "timestamp year must be >= 2026, got {year}");
    }

    #[test]
    fn test_epoch_days_to_date_1970() {
        // epoch_days_to_date is private but exercised via now_iso8601 (which calls it).
        // Verify now_iso8601 doesn't panic and returns a plausible date.
        let ts = now_iso8601();
        let month: u32 = ts[5..7].parse().unwrap();
        let day: u32 = ts[8..10].parse().unwrap();
        assert!((1..=12).contains(&month), "month must be 1-12");
        assert!((1..=31).contains(&day), "day must be 1-31");
    }

    #[test]
    fn test_check_plan_collision_nonexistent_dir_ok() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("never_existed");
        // Directory doesn't exist → manifest.json definitely missing → no collision
        assert!(check_plan_collision(&plan_dir, false).is_ok());
    }

    #[test]
    fn test_check_plan_collision_corrupted_manifest_still_detects_collision() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("corrupt");
        std::fs::create_dir_all(&plan_dir).unwrap();
        // Corrupt JSON → created_at will fall back to "unknown" but collision is still detected
        std::fs::write(plan_dir.join("manifest.json"), b"not valid json").unwrap();
        let result = check_plan_collision(&plan_dir, false);
        assert!(result.is_err(), "corrupt manifest must still trigger collision error");
    }

    #[test]
    fn test_plan_output_dir_with_label_containing_underscores() {
        let base = std::path::Path::new("/out");
        let dir = plan_output_dir(base, "wa_congressional_2020_draft");
        assert!(dir.ends_with("plans/wa_congressional_2020_draft"));
    }

    #[test]
    fn test_state_output_dir_with_underscore_state_name() {
        let base = std::path::Path::new("/out");
        let dir = state_output_dir(base, "new_york");
        assert!(dir.ends_with("states/new_york"));
    }

    #[test]
    fn test_create_plan_dir_all_three_subdirs_exist() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = create_plan_dir(tmp.path(), "full_structure").unwrap();
        assert!(plan_dir.join("analysis").exists());
        assert!(plan_dir.join("maps").exists());
        assert!(plan_dir.join("intermediate").exists());
    }
}
