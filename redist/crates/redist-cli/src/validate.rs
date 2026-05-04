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

    // ── Additional L0 tests ───────────────────────────────────────────────────

    #[test]
    fn test_validate_missing_file_errors() {
        let path = PathBuf::from("/nonexistent/path/plan.rplan");
        let result = run_validate(&ValidateArgs { file: path, strict: false });
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("cannot read"), "error must say 'cannot read': {msg}");
    }

    #[test]
    fn test_validate_not_json_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.rplan");
        std::fs::write(&path, b"this is not json at all").unwrap();
        let result = run_validate(&ValidateArgs { file: path, strict: false });
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_file_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("empty.rplan");
        std::fs::write(&path, b"").unwrap();
        let result = run_validate(&ValidateArgs { file: path, strict: false });
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_object_but_wrong_schema_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("wrong_schema.rplan");
        // Valid JSON object but missing rplan_version/metadata/assignments
        std::fs::write(&path, br#"{"foo": "bar", "baz": 42}"#).unwrap();
        let result = run_validate(&ValidateArgs { file: path, strict: false });
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_strict_mode_flag_accessible() {
        let args = ValidateArgs {
            file: PathBuf::from("test.rplan"),
            strict: true,
        };
        assert!(args.strict);
    }

    #[test]
    fn test_validate_strict_false_flag_accessible() {
        let args = ValidateArgs {
            file: PathBuf::from("test.rplan"),
            strict: false,
        };
        assert!(!args.strict);
    }

    #[test]
    fn test_validate_valid_rplan_str_directly() {
        // Exercise validate_rplan_str directly — bypasses file I/O
        let json = make_valid_rplan_json();
        let result = redist_report::validate_rplan_str(&json);
        assert!(result.is_ok(), "valid rplan must pass: {:?}", result);
    }

    #[test]
    fn test_validate_bad_geoid_rplan_str_directly() {
        let json = make_rplan_with_bad_geoid_json();
        let result = redist_report::validate_rplan_str(&json);
        assert!(result.is_err(), "bad GEOID rplan must fail validation");
    }

    #[test]
    fn test_validate_args_file_path_stored() {
        let path = PathBuf::from("/some/path/plan.rplan");
        let args = ValidateArgs { file: path.clone(), strict: false };
        assert_eq!(args.file, path);
    }

    #[test]
    fn test_validate_args_in_commands_enum_with_strict() {
        let _cmd = crate::args::Commands::Validate(ValidateArgs {
            file: PathBuf::from("strict_test.rplan"),
            strict: true,
        });
    }

    fn make_rplan_missing_version_json() -> String {
        r#"{
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

    #[test]
    fn test_validate_missing_rplan_version_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("no_version.rplan");
        std::fs::write(&path, make_rplan_missing_version_json()).unwrap();
        let result = run_validate(&ValidateArgs { file: path, strict: false });
        assert!(result.is_err(), "rplan without rplan_version field must fail");
    }

    fn make_rplan_missing_assignments_json() -> String {
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
            "geometry": null
        }"#.to_string()
    }

    #[test]
    fn test_validate_missing_assignments_field_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("no_assignments.rplan");
        std::fs::write(&path, make_rplan_missing_assignments_json()).unwrap();
        let result = run_validate(&ValidateArgs { file: path, strict: false });
        assert!(result.is_err(), "rplan without assignments field must fail");
    }

    #[test]
    fn test_validate_valid_rplan_metadata_fields_parsed() {
        // validate_rplan_str on a valid JSON lets us parse the state, year, chamber
        let json = make_valid_rplan_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["metadata"]["state_code"].as_str().unwrap(), "WA");
        assert_eq!(v["metadata"]["year"].as_str().unwrap(), "2020");
        assert_eq!(v["metadata"]["chamber"].as_str().unwrap(), "congressional");
        assert_eq!(v["metadata"]["num_districts"].as_u64().unwrap(), 10);
    }
}
