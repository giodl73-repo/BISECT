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
            "ERROR: Plan '{}' already exists (created {}). \
             Use --force to overwrite or choose a different --label.",
            label,
            created_at,
        );
    }
    Ok(())
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
        assert!(msg.contains("already exists"));
        assert!(msg.contains("2026-04-26"), "Error must include existing plan timestamp");
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
}
