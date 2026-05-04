/// `redist migrate` — copy a legacy state plan into the plans/{label}/ tree.
///
/// Source: `{base}/states/{state_name}/`
/// Destination: `{base}/plans/{label}/`
///
/// A minimal manifest.json is written to the destination.
use std::path::Path;
use redist_report::{PlanManifest, create_plan_dir, write_manifest_atomic, check_plan_collision};

/// Migrate a legacy state directory into a labeled plan directory.
///
/// - source: `{base}/states/{state_name_or_code}/`
/// - dest:   `{base}/plans/{label}/`
///
/// `state_identifier` can be:
///   - a 2-letter state code ("WA") — will look up full name via manifest
///   - a full lowercase state name ("washington")
/// `force`: if true, overwrite an existing plan directory without error.
pub fn run_migrate(base: &Path, state_identifier: &str, label: &str, force: bool) -> anyhow::Result<()> {
    // Try the identifier as given (lowercase), then try to resolve via manifest
    let candidate_lower = state_identifier.to_lowercase();
    let source_dir_direct = base.join("states").join(&candidate_lower);

    // Try to find the state directory: first try the lowercase identifier directly,
    // then try to resolve the 2-letter code to full state name via manifest.
    let source_dir = if source_dir_direct.exists() {
        source_dir_direct
    } else {
        // Try to resolve via manifest (for 2-letter codes like "WA" → "washington")
        let resolved = try_resolve_state_name(state_identifier)
            .map(|name| base.join("states").join(&name));

        match resolved {
            Some(dir) if dir.exists() => dir,
            _ => {
                anyhow::bail!(
                    "legacy state directory not found: '{}'. \
                     Run 'redist state --state {}' first to generate it.",
                    source_dir_direct.display(),
                    state_identifier.to_uppercase()
                );
            }
        }
    };

    // Check for collision before creating the target directory
    let plan_dir_candidate = base.join("plans").join(label);
    check_plan_collision(&plan_dir_candidate, force)?;

    // Create the target plan directory structure
    let plan_dir = create_plan_dir(base, label)?;

    // Copy all files from source state directory (recursively, flat copy at root)
    copy_dir_contents(&source_dir, &plan_dir)?;

    // Write a minimal manifest
    let state_dir_name = source_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| candidate_lower.clone());
    // Count districts from final_assignments.json to determine num_districts
    let assignments_path = plan_dir.join("final_assignments.json");
    let num_districts: usize = if assignments_path.exists() {
        let content = std::fs::read_to_string(&assignments_path).unwrap_or_default();
        let asgn: std::collections::HashMap<String, usize> =
            serde_json::from_str(&content).unwrap_or_default();
        // Count distinct district IDs
        let distinct: std::collections::HashSet<usize> = asgn.values().copied().collect();
        distinct.len().max(1)
    } else {
        1
    };

    let manifest = PlanManifest {
        label: label.to_string(),
        state_code: state_identifier.to_uppercase(),
        chamber: "congressional".to_string(),
        num_districts,
        population_source: "total".to_string(),
        partition_mode: "edge-weighted".to_string(),
        binary_download_url: "https://github.com/owner/redist/releases".to_string(),
        adjacency_file: format!("{}_adjacency.adj.bin", state_dir_name),
        tiger_source_url: "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/".to_string(),
        created_at: chrono_now(),
        // Legacy US plans are always single-member; set explicitly so manifest is unambiguous.
        seats_per_district: 1,
        total_seats: num_districts, // one seat per district for single-member systems
        electoral_system: "single_member".to_string(),
        ..Default::default()
    };
    write_manifest_atomic(&plan_dir, &manifest)?;

    Ok(())
}

/// Try to resolve a 2-letter state code to a full lowercase state name.
/// Returns None if the manifest cannot be read or the state is not found.
fn try_resolve_state_name(state_code: &str) -> Option<String> {
    // Try loading the manifest to get the full state name
    let manifest = crate::fetch::load_manifest().ok()?;
    let code_upper = state_code.to_uppercase();
    manifest.states.get(&code_upper).map(|s| {
        s.name.to_lowercase().replace(' ', "_")
    })
}

/// Copy all files (non-recursively for data dir) from src to dst.
fn copy_dir_contents(src: &Path, dst: &Path) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            std::fs::copy(&path, dst.join(file_name))?;
        } else if path.is_dir() {
            // Copy data/ subdirectory contents flat into root
            let sub_dst = dst.join(path.file_name().unwrap());
            std::fs::create_dir_all(&sub_dst)?;
            copy_dir_contents(&path, &sub_dst)?;
        }
    }
    Ok(())
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format as ISO 8601 (approximate — we don't want a heavy dep just for this)
    let days = secs / 86400;
    let rem = secs % 86400;
    let hours = rem / 3600;
    let minutes = (rem % 3600) / 60;
    let seconds = rem % 60;
    // epoch day → approx date (simplified; precise enough for manifest timestamps)
    format!("{}T{:02}:{:02}:{:02}Z", epoch_days_to_date(days), hours, minutes, seconds)
}

fn epoch_days_to_date(days: u64) -> String {
    // Approximate gregorian date from unix days (accurate for 1970-2100)
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
// Tests — Task 9
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_legacy_state_dir(base: &Path, state_name: &str) {
        let state_dir = base.join("states").join(state_name);
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::write(
            state_dir.join("final_assignments.json"),
            r#"{"53033000100":1}"#,
        )
        .unwrap();
    }

    #[test]
    fn test_migrate_copies_final_assignments() {
        let tmp = TempDir::new().unwrap();
        // Set up legacy state dir using the full lowercase name (as CLI produces it)
        let state_dir = tmp.path().join("states").join("washington");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::write(
            state_dir.join("final_assignments.json"),
            r#"{"53033000100":1}"#,
        )
        .unwrap();
        // Run migrate using the full name (no manifest needed in test env)
        run_migrate(tmp.path(), "washington", "wa_congressional_2020", false).unwrap();
        // Verify plan dir exists with assignments
        let plan_dir = tmp.path().join("plans").join("wa_congressional_2020");
        assert!(plan_dir.join("final_assignments.json").exists());
    }

    #[test]
    fn test_migrate_creates_minimal_manifest() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "washington");
        run_migrate(tmp.path(), "washington", "wa_congressional_2020", false).unwrap();
        let manifest_path = tmp
            .path()
            .join("plans")
            .join("wa_congressional_2020")
            .join("manifest.json");
        assert!(manifest_path.exists());
        let manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
        assert_eq!(
            manifest["label"].as_str().unwrap(),
            "wa_congressional_2020"
        );
    }

    #[test]
    fn test_migrate_nonexistent_state_errors() {
        let tmp = TempDir::new().unwrap();
        let result = run_migrate(tmp.path(), "zzznotastate", "zz_plan", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_migrate_collision_without_force_errors() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "washington");
        // First migrate succeeds
        run_migrate(tmp.path(), "washington", "wa_congressional_2020", false).unwrap();
        // Second without --force must fail with "exists" in the message
        let result = run_migrate(tmp.path(), "washington", "wa_congressional_2020", false);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string().to_lowercase();
        assert!(msg.contains("exists"), "expected 'exists' in: {msg}");
    }

    #[test]
    fn test_migrate_force_allows_overwrite() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "washington");
        run_migrate(tmp.path(), "washington", "wa_congressional_2020", false).unwrap();
        // Second with --force must succeed
        run_migrate(tmp.path(), "washington", "wa_congressional_2020", true).unwrap();
    }

    /// Scenario 22: Migrated manifest must have seats_per_district, total_seats,
    /// and electoral_system explicitly set for legacy US plans.
    #[test]
    fn test_migrate_manifest_has_seats_fields() {
        let tmp = TempDir::new().unwrap();
        // Setup legacy state dir with 3 districts in assignments
        let state_dir = tmp.path().join("states").join("washington");
        std::fs::create_dir_all(&state_dir).unwrap();
        // Write assignments with 3 distinct district IDs
        std::fs::write(
            state_dir.join("final_assignments.json"),
            r#"{"53033000100":1,"53033000200":2,"53033000300":3}"#,
        ).unwrap();

        run_migrate(tmp.path(), "washington", "wa_congressional_2020", false).unwrap();

        let manifest_path = tmp
            .path()
            .join("plans")
            .join("wa_congressional_2020")
            .join("manifest.json");
        assert!(manifest_path.exists());
        let content = std::fs::read_to_string(&manifest_path).unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&content).unwrap();

        // seats_per_district must be explicitly 1 (single-member)
        assert_eq!(
            manifest["seats_per_district"].as_u64().unwrap_or(0),
            1,
            "seats_per_district must be 1 for legacy US plans"
        );
        // total_seats must equal num_districts (one seat per district)
        let num_districts = manifest["num_districts"].as_u64().unwrap_or(0);
        assert_eq!(
            manifest["total_seats"].as_u64().unwrap_or(0),
            num_districts,
            "total_seats must equal num_districts for single-member plans"
        );
        // electoral_system must be "single_member"
        assert_eq!(
            manifest["electoral_system"].as_str().unwrap_or(""),
            "single_member",
            "electoral_system must be 'single_member' for legacy US plans"
        );
    }

    // ── Additional L0 tests ───────────────────────────────────────────────────

    #[test]
    fn test_migrate_manifest_label_matches_arg() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "vermont");
        run_migrate(tmp.path(), "vermont", "vt_congressional_2020", false).unwrap();
        let manifest_path = tmp.path().join("plans").join("vt_congressional_2020").join("manifest.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
        assert_eq!(v["label"].as_str().unwrap(), "vt_congressional_2020");
    }

    #[test]
    fn test_migrate_state_code_uppercased_in_manifest() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "vermont");
        run_migrate(tmp.path(), "vermont", "vt_plan", false).unwrap();
        let manifest_path = tmp.path().join("plans").join("vt_plan").join("manifest.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
        // state_code is set from state_identifier.to_uppercase()
        assert_eq!(v["state_code"].as_str().unwrap(), "VERMONT");
    }

    #[test]
    fn test_migrate_creates_analysis_subdir() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "delaware");
        run_migrate(tmp.path(), "delaware", "de_plan", false).unwrap();
        let analysis_dir = tmp.path().join("plans").join("de_plan").join("analysis");
        assert!(analysis_dir.exists(), "migrate must create analysis/ subdirectory");
    }

    #[test]
    fn test_migrate_creates_maps_subdir() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "delaware");
        run_migrate(tmp.path(), "delaware", "de_plan2", false).unwrap();
        let maps_dir = tmp.path().join("plans").join("de_plan2").join("maps");
        assert!(maps_dir.exists(), "migrate must create maps/ subdirectory");
    }

    #[test]
    fn test_migrate_creates_intermediate_subdir() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "delaware");
        run_migrate(tmp.path(), "delaware", "de_plan3", false).unwrap();
        let int_dir = tmp.path().join("plans").join("de_plan3").join("intermediate");
        assert!(int_dir.exists(), "migrate must create intermediate/ subdirectory");
    }

    #[test]
    fn test_migrate_num_districts_counted_correctly() {
        let tmp = TempDir::new().unwrap();
        // 5 distinct districts
        let state_dir = tmp.path().join("states").join("oregon");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::write(
            state_dir.join("final_assignments.json"),
            r#"{"41000001":1,"41000002":2,"41000003":3,"41000004":4,"41000005":5}"#,
        ).unwrap();
        run_migrate(tmp.path(), "oregon", "or_plan", false).unwrap();
        let manifest_path = tmp.path().join("plans").join("or_plan").join("manifest.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
        assert_eq!(v["num_districts"].as_u64().unwrap(), 5);
    }

    #[test]
    fn test_migrate_single_assignment_gives_one_district() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "alaska"); // only 1 assignment → 1 district
        run_migrate(tmp.path(), "alaska", "ak_plan", false).unwrap();
        let manifest_path = tmp.path().join("plans").join("ak_plan").join("manifest.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
        assert_eq!(v["num_districts"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_migrate_no_assignments_file_gives_one_district() {
        let tmp = TempDir::new().unwrap();
        // State dir exists but no final_assignments.json
        let state_dir = tmp.path().join("states").join("wyoming");
        std::fs::create_dir_all(&state_dir).unwrap();
        run_migrate(tmp.path(), "wyoming", "wy_plan", false).unwrap();
        let manifest_path = tmp.path().join("plans").join("wy_plan").join("manifest.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
        assert_eq!(v["num_districts"].as_u64().unwrap(), 1,
            "missing assignments file must default to 1 district");
    }

    #[test]
    fn test_migrate_copies_subdirectory_recursively() {
        let tmp = TempDir::new().unwrap();
        // State dir with a data/ subdirectory
        let state_dir = tmp.path().join("states").join("nevada");
        let data_dir = state_dir.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(state_dir.join("final_assignments.json"), r#"{"32001000100":1}"#).unwrap();
        std::fs::write(data_dir.join("summary.json"), r#"{"districts":[]}"#).unwrap();

        run_migrate(tmp.path(), "nevada", "nv_plan", false).unwrap();

        // data/summary.json must be copied under the plan dir
        let copied = tmp.path().join("plans").join("nv_plan").join("data").join("summary.json");
        assert!(copied.exists(), "subdirectory files must be copied recursively");
    }

    #[test]
    fn test_chrono_now_format() {
        let ts = chrono_now();
        // Must look like 2026-05-02T12:00:00Z
        assert!(ts.contains('T'), "timestamp must contain T separator");
        assert!(ts.ends_with('Z'), "timestamp must end with Z");
        assert_eq!(ts.len(), 20, "timestamp must be exactly 20 chars (YYYY-MM-DDTHH:MM:SSZ)");
    }

    #[test]
    fn test_epoch_days_to_date_unix_epoch() {
        // Day 0 = 1970-01-01
        let date = epoch_days_to_date(0);
        assert_eq!(date, "1970-01-01");
    }

    #[test]
    fn test_epoch_days_to_date_known_date() {
        // 2026-05-02 = 20574 days since epoch (approx)
        // Days: 56 * 365 + leap years (14) + 31 (Jan) + 28 (Feb) + 31 (Mar) + 30 (Apr) + 2 = ?
        // Let's verify 2000-01-01 = 10957 days
        let date = epoch_days_to_date(10957);
        assert_eq!(date, "2000-01-01", "epoch day 10957 must be 2000-01-01");
    }

    #[test]
    fn test_migrate_adjacency_file_name_in_manifest() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "utah");
        run_migrate(tmp.path(), "utah", "ut_plan", false).unwrap();
        let manifest_path = tmp.path().join("plans").join("ut_plan").join("manifest.json");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
        let adj_file = v["adjacency_file"].as_str().unwrap_or("");
        assert!(adj_file.contains("adjacency"), "adjacency_file must contain 'adjacency'");
        assert!(adj_file.ends_with(".adj.bin"), "adjacency_file must end with .adj.bin");
    }
}
