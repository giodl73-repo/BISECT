/// `redist validate` — validate a .rplan file for format correctness.
///
/// Dispatches to `redist_report::validate_rplan()`.
/// Exits 0 on PASS, non-zero on FAIL.
use crate::args::ValidateArgs;
use redist_report::validate_rplan_str;

/// Validate a .rplan file and print a human-readable summary.
/// Returns Ok(()) on pass, Err on validation failure.
pub fn run_validate(args: &ValidateArgs) -> anyhow::Result<()> {
    let path = &args.file;
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read '{}': {}", path.display(), e))?;

    // Parse and validate
    let result = validate_rplan_str(&content)
        .map_err(|e| anyhow::anyhow!("FAIL: {}", e))?;

    // Count assignments
    let rplan: serde_json::Value = serde_json::from_str(&content)?;
    let tract_count = rplan["assignments"]
        .as_object()
        .map(|m| m.len())
        .unwrap_or(0);
    let num_districts = rplan["metadata"]["num_districts"].as_u64().unwrap_or(0);
    let state = rplan["metadata"]["state_code"].as_str().unwrap_or("?");
    let year = rplan["metadata"]["year"].as_str().unwrap_or("?");
    let chamber = rplan["metadata"]["chamber"].as_str().unwrap_or("?");

    // Print warnings if any
    for w in &result.warnings {
        eprintln!("WARNING: {}", w);
    }

    // Exit with failure in strict mode if there are warnings
    if args.strict && !result.warnings.is_empty() {
        anyhow::bail!(
            "FAIL (strict): {} warning(s) treated as errors",
            result.warnings.len()
        );
    }

    println!(
        "PASS: valid RPLAN v0.1 ({} tracts, {} districts, {} {} {})",
        tract_count, num_districts, state, year, chamber
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — Task 6
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_valid_rplan_json() -> String {
        r#"{
            "rplan_version": "0.1",
            "metadata": {
                "label": "wa_test",
                "state_fips": "53",
                "state_code": "WA",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 10,
                "population_source": "total",
                "balance_tolerance_pct": 0.5,
                "created_at": "2026-04-26T00:00:00Z",
                "created_by": "test"
            },
            "assignments": {"53033000100": 1},
            "geometry": null
        }"#.to_string()
    }

    fn make_rplan_with_bad_geoid_json() -> String {
        r#"{
            "rplan_version": "0.1",
            "metadata": {
                "label": "wa_test",
                "state_fips": "53",
                "state_code": "WA",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 10,
                "population_source": "total",
                "balance_tolerance_pct": 0.5,
                "created_at": "2026-04-26T00:00:00Z",
                "created_by": "test"
            },
            "assignments": {"530330": 1},
            "geometry": null
        }"#.to_string()
    }

    #[test]
    fn test_validate_command_exists_in_commands_enum() {
        let _cmd = crate::args::Commands::Validate(ValidateArgs {
            file: PathBuf::from("test.rplan"),
            strict: false,
        });
    }

    #[test]
    fn test_validate_valid_file_exits_zero() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("plan.rplan");
        std::fs::write(&path, make_valid_rplan_json()).unwrap();
        let result = run_validate(&ValidateArgs { file: path, strict: false });
        assert!(result.is_ok(), "expected ok, got: {:?}", result);
    }

    #[test]
    fn test_validate_invalid_geoid_exits_nonzero() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("plan.rplan");
        std::fs::write(&path, make_rplan_with_bad_geoid_json()).unwrap();
        let result = run_validate(&ValidateArgs { file: path, strict: false });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("GEOID"), "expected GEOID in error, got: {msg}");
    }
}
