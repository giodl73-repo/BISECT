/// `redist migrate` — copy a legacy state plan into the plans/{label}/ tree.
///
/// Source: `{base}/states/{state_name}/`
/// Destination: `{base}/plans/{label}/`
///
/// A minimal manifest.json is written to the destination.
use std::path::Path;
use redist_report::{PlanManifest, create_plan_dir, write_manifest_atomic};

/// Migrate a legacy state directory into a labeled plan directory.
///
/// - source: `{base}/states/{state_name_or_code}/`
/// - dest:   `{base}/plans/{label}/`
///
/// `state_identifier` can be:
///   - a 2-letter state code ("WA") — will look up full name via manifest
///   - a full lowercase state name ("washington")
pub fn run_migrate(base: &Path, state_identifier: &str, label: &str) -> anyhow::Result<()> {
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

    // Create the target plan directory structure
    let plan_dir = create_plan_dir(base, label)?;

    // Copy all files from source state directory (recursively, flat copy at root)
    copy_dir_contents(&source_dir, &plan_dir)?;

    // Write a minimal manifest
    let state_dir_name = source_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| candidate_lower.clone());
    let manifest = PlanManifest {
        label: label.to_string(),
        state_code: state_identifier.to_uppercase(),
        chamber: "congressional".to_string(),
        population_source: "total".to_string(),
        partition_mode: "edge-weighted".to_string(),
        binary_download_url: "https://github.com/owner/redist/releases".to_string(),
        adjacency_file: format!("{}_adjacency.adj.bin", state_dir_name),
        tiger_source_url: "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/".to_string(),
        created_at: chrono_now(),
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
        run_migrate(tmp.path(), "washington", "wa_congressional_2020").unwrap();
        // Verify plan dir exists with assignments
        let plan_dir = tmp.path().join("plans").join("wa_congressional_2020");
        assert!(plan_dir.join("final_assignments.json").exists());
    }

    #[test]
    fn test_migrate_creates_minimal_manifest() {
        let tmp = TempDir::new().unwrap();
        setup_legacy_state_dir(tmp.path(), "washington");
        run_migrate(tmp.path(), "washington", "wa_congressional_2020").unwrap();
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
        let result = run_migrate(tmp.path(), "zzznotastate", "zz_plan");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
