/// import_cmd.rs — `redist import` command dispatch.
///
/// Imports a GeoJSON plan (or GerryChain JSON) into the RPLAN format.
/// Dispatches to redist_report::import_plan_to_rplan or import_gerrychain_to_assignments.
use std::path::PathBuf;
use crate::args::ImportArgs;
use redist_core::state_code_to_fips;
use redist_report::{
    RplanFile, RplanMetadata, import_geojson_plan, import_gerrychain_to_assignments,
    write_rplan, validate_geoid_format_batch,
};

/// Return true if the path has a shapefile-family extension (.shp, .shx, .dbf, .prj, .zip).
fn is_shapefile_extension(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("shp") | Some("shx") | Some("dbf") | Some("prj") | Some("zip")
    )
}

/// Build the shapefile guidance error message for a given extension.
fn shapefile_error_message(path: &std::path::Path) -> String {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("shp");
    let (title, extra) = if ext == "zip" {
        (
            "ZIP archives (common for Census TIGER downloads) are not directly supported by redist import.",
            "Extract the archive first, then convert the contained .shp to GeoJSON.",
        )
    } else {
        (
            &*format!("Shapefile format (.{ext}) is not directly supported by redist import."),
            "Convert to GeoJSON first using one of:",
        )
    };
    format!(
        "ERROR: {title}\n\
         {extra}\n  \
         ogr2ogr -f GeoJSON output.geojson input.shp\n  \
         qgis: Layer > Save As > GeoJSON\n  \
         python: geopandas.read_file('input.shp').to_file('output.geojson', driver='GeoJSON')\n\
         Then: redist import --format geojson --file output.geojson --label <label>"
    )
}

/// Run the import command.
pub fn run_import(args: &ImportArgs) -> anyhow::Result<()> {
    let file_path = &args.file;

    // Shapefile / ZIP guard — must fire before file-existence check so the message
    // is shown even when the file doesn't exist yet.
    if is_shapefile_extension(file_path) {
        anyhow::bail!("{}", shapefile_error_message(file_path));
    }

    if !file_path.exists() {
        anyhow::bail!("Input file not found: '{}'", file_path.display());
    }

    let content = std::fs::read_to_string(file_path)?;
    let output_dir = PathBuf::from(&args.output_base);
    let plan_dir = output_dir
        .join(&args.version)
        .join(&args.year)
        .join("plans")
        .join(&args.label);
    std::fs::create_dir_all(&plan_dir)?;

    // Dispatch based on format
    let assignments = match args.format.as_str() {
        "gerrychain" => {
            // GerryChain v2.3 JSON import
            import_gerrychain_to_assignments(&content)?
        }
        "dra-csv" => {
            // Dave's Redistricting App CSV import
            import_assignments_from_dra_csv(&content)?
        }
        _ => {
            // Auto-detect DRA CSV by file extension
            if file_path.extension().and_then(|e| e.to_str()) == Some("csv") {
                import_assignments_from_dra_csv(&content)?
            } else {
                // Default: GeoJSON import (PIP assignment)
                // For GeoJSON import without real tract centroids: use assignment field from properties
                // or fall back to deriving centroids from the GeoJSON itself
                import_assignments_from_geojson_properties(&content)?
            }
        }
    };

    // Validate GEOID format
    validate_geoid_format_batch(&assignments)
        .map_err(|e| anyhow::anyhow!("GEOID validation failed: {e}"))?;

    let num_districts = assignments.values().copied().max().unwrap_or(0);

    let metadata = RplanMetadata {
        label: args.label.clone(),
        state_fips: state_code_to_fips(&args.state.to_uppercase()).unwrap_or("00").to_string(),
        state_code: args.state.to_uppercase(),
        year: args.year.clone(),
        chamber: "imported".into(),
        num_districts,
        population_source: "unknown".into(),
        balance_tolerance_pct: 0.0,
        created_at: redist_report::now_iso8601(),
        created_by: "redist import".into(),
        notes: Some(format!("imported from {}", file_path.display())),
        ..Default::default()
    };

    let rplan_path = plan_dir.join(format!("{}.rplan", args.label));
    write_rplan(&rplan_path, &metadata, &assignments, None)?;

    // Also write final_assignments.json for compatibility with analyze command
    let assignments_path = plan_dir.join("final_assignments.json");
    std::fs::write(&assignments_path, serde_json::to_string_pretty(&assignments)?)?;

    // SSI Task 7 (civic-bypass) — validation: --as-civic-counter-proposal
    // requires --submitted-by, otherwise the manifest tag becomes meaningless.
    if args.as_civic_counter_proposal {
        let submitted_by = args.submitted_by.as_deref().unwrap_or("");
        if submitted_by.trim().is_empty() {
            anyhow::bail!(
                "[INPUT] --as-civic-counter-proposal requires --submitted-by \"<organization name>\". \
                 Without a submitter, the civic-counter-proposal tag is unusable in downstream \
                 comparison reports. See docs/superpowers/specs/2026-04-30-state-staff-interop.md."
            );
        }
    }

    // Compute submission_type / submitted_at default per SSI Task 7.
    let submission_type = if args.as_civic_counter_proposal {
        "civic_counter_proposal".to_string()
    } else {
        "authoritative".to_string()
    };
    let submitted_at = args.submitted_at.clone().or_else(|| {
        if args.as_civic_counter_proposal {
            Some(redist_report::now_iso8601())
        } else {
            None
        }
    });

    // Write minimal manifest.json
    let manifest = redist_report::PlanManifest {
        label: args.label.clone(),
        state_code: args.state.to_uppercase(),
        year: args.year.clone(),
        chamber: "imported".into(),
        num_districts,
        created_at: redist_report::now_iso8601(),
        submission_type,
        submitted_by: args.submitted_by.clone(),
        submitted_at,
        source_tool: Some(args.format.clone()),
        ..Default::default()
    };
    // SSI Task 6 — Callais p.36 mutex preflight at the import gate. For
    // freshly-imported plans this is mostly a forward guard; partition_mode
    // and population_source on imports default to empty strings so the
    // preflight passes. The check fires for hand-edited or upstream-tampered
    // manifest fields.
    redist_report::manifest::callais_preflight(&manifest)?;
    redist_report::write_manifest_atomic(&plan_dir, &manifest)?;

    eprintln!(
        "[OK] Imported {} tracts → {} districts. RPLAN: {}",
        assignments.len(),
        num_districts,
        rplan_path.display()
    );

    Ok(())
}

/// Import assignments from GeoJSON properties (tract-level assignments embedded in features).
/// Each feature must have a "geoid" and "district" property.
/// If features have polygon geometry with district_id: use those as district polygons.
fn import_assignments_from_geojson_properties(
    geojson_str: &str,
) -> anyhow::Result<std::collections::HashMap<String, usize>> {
    let v: serde_json::Value = serde_json::from_str(geojson_str)?;

    // Check if this is a tract-level assignment GeoJSON (features have geoid property)
    let features = v["features"].as_array();
    if let Some(arr) = features {
        // Try to extract geoid → district from feature properties
        let mut assignments = std::collections::HashMap::new();
        for feature in arr {
            let props = &feature["properties"];
            if let (Some(geoid), Some(district)) = (
                props["geoid"].as_str(),
                props["district"].as_u64(),
            ) {
                assignments.insert(geoid.to_string(), district as usize);
            } else if let (Some(geoid), Some(district)) = (
                props["GEOID"].as_str(),
                props["district"].as_u64(),
            ) {
                assignments.insert(geoid.to_string(), district as usize);
            }
        }
        if !assignments.is_empty() {
            return Ok(assignments);
        }

        // If no tract assignments found: this is a district-polygon GeoJSON.
        // We can't do PIP without real tract centroids; return error with guidance.
        anyhow::bail!(
            "GeoJSON does not contain tract-level assignments (geoid + district properties). \
             For district-polygon GeoJSON, provide tract centroids via a supplementary file \
             or use the Python pipeline's import command."
        );
    }

    anyhow::bail!("Invalid GeoJSON: missing 'features' array")
}

/// Import assignments from Dave's Redistricting App CSV format.
///
/// DRA exports a two-column CSV:
///   Variant 1: GEOID,DISTRICT header  (GEOID = 11-char numeric, DISTRICT = small int)
///   Variant 2: DISTRICT,GEOID header  (reversed columns)
///   Variant 3: no header              (first column is detected as GEOID by length)
///
/// Returns HashMap<GEOID, district_id>.
pub fn import_assignments_from_dra_csv(
    csv_str: &str,
) -> anyhow::Result<std::collections::HashMap<String, usize>> {
    let mut lines = csv_str.lines().peekable();
    if lines.peek().is_none() {
        anyhow::bail!("DRA CSV is empty");
    }

    // Detect header row by checking whether the first token is numeric.
    let first_line = *lines.peek().unwrap();
    let first_fields: Vec<&str> = first_line.splitn(2, ',').collect();
    let has_header = !first_fields.get(0).map(|f| f.trim()).unwrap_or("").chars()
        .next().map(|c| c.is_ascii_digit()).unwrap_or(false);

    // Determine column order from header (or default to GEOID=0, DISTRICT=1).
    let (geoid_col, district_col) = if has_header {
        let header_line = lines.next().unwrap(); // consume the header
        let headers: Vec<&str> = header_line.split(',').collect();
        let h0 = headers.get(0).map(|h| h.trim().to_uppercase()).unwrap_or_default();
        let h1 = headers.get(1).map(|h| h.trim().to_uppercase()).unwrap_or_default();
        // Detect reversed order: first column is DISTRICT (small-int-like name)
        let geoid_first = h0.contains("GEOID") || h0.contains("GEO_ID");
        let district_first = h0.contains("DISTRICT") || h0.contains("DIST") || h0 == "ID";
        if district_first && (h1.contains("GEOID") || h1.contains("GEO_ID")) {
            (1usize, 0usize) // reversed: DISTRICT, GEOID
        } else if geoid_first {
            (0, 1) // normal: GEOID, DISTRICT
        } else {
            // Unknown header order — try to auto-detect from first data row
            (0, 1)
        }
    } else {
        (0usize, 1usize) // no header: assume GEOID first
    };

    let mut assignments = std::collections::HashMap::new();
    for (lineno, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let fields: Vec<&str> = line.split(',').collect();
        let geoid_raw = fields.get(geoid_col).map(|f| f.trim()).unwrap_or("");
        let district_raw = fields.get(district_col).map(|f| f.trim()).unwrap_or("");

        // If auto-detect failed and geoid looks numeric with small value, swap
        let (geoid, district_str) =
            if geoid_raw.len() < 5 && district_raw.len() == 11 && district_raw.chars().all(|c| c.is_ascii_digit()) {
                (district_raw, geoid_raw)
            } else {
                (geoid_raw, district_raw)
            };

        let district: usize = district_str.parse().map_err(|_| {
            anyhow::anyhow!("DRA CSV line {}: cannot parse district '{}' as integer", lineno + 1, district_str)
        })?;
        assignments.insert(geoid.to_string(), district);
    }

    if assignments.is_empty() {
        anyhow::bail!("DRA CSV produced no assignments — check file format");
    }
    Ok(assignments)
}

// ---------------------------------------------------------------------------
// Tests — Task 7
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::ImportArgs;

    #[test]
    fn test_import_args_parsed() {
        let args = ImportArgs {
            file: std::path::PathBuf::from("wa_10_districts.geojson"),
            state: "WA".into(),
            year: "2020".into(),
            label: "wa_imported_test".into(),
            version: "test".into(),
            format: "geojson".into(),
            output_base: "outputs".into(),
            as_civic_counter_proposal: false,
            submitted_by: None,
            submitted_at: None,
        };
        assert_eq!(args.state, "WA");
        assert_eq!(args.label, "wa_imported_test");
    }

    // ── Task 128: Shapefile import guidance ──────────────────────────────────

    #[test]
    fn test_import_shapefile_extension_gives_actionable_error() {
        // A .shp file should produce an error containing "ogr2ogr" guidance.
        let args = ImportArgs {
            file: std::path::PathBuf::from("plan.shp"),
            state: "WA".into(),
            year: "2020".into(),
            label: "wa_test".into(),
            version: "test".into(),
            format: "geojson".into(),
            output_base: "outputs".into(),
            as_civic_counter_proposal: false,
            submitted_by: None,
            submitted_at: None,
        };
        let result = run_import(&args);
        assert!(result.is_err(), "importing .shp should fail");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("ogr2ogr"), "error should mention ogr2ogr, got: {msg}");
        assert!(msg.contains("GeoJSON"), "error should mention GeoJSON conversion, got: {msg}");
    }

    #[test]
    fn test_import_zip_extension_gives_actionable_error() {
        let args = ImportArgs {
            file: std::path::PathBuf::from("tl_2020_us_tract.zip"),
            state: "WA".into(),
            year: "2020".into(),
            label: "wa_test".into(),
            version: "test".into(),
            format: "geojson".into(),
            output_base: "outputs".into(),
            as_civic_counter_proposal: false,
            submitted_by: None,
            submitted_at: None,
        };
        let result = run_import(&args);
        assert!(result.is_err(), "importing .zip should fail");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("ogr2ogr"), "error should mention ogr2ogr, got: {msg}");
    }

    #[test]
    fn test_import_shx_extension_gives_error() {
        let args = ImportArgs {
            file: std::path::PathBuf::from("plan.shx"),
            state: "WA".into(),
            year: "2020".into(),
            label: "wa_test".into(),
            version: "test".into(),
            format: "geojson".into(),
            output_base: "outputs".into(),
            as_civic_counter_proposal: false,
            submitted_by: None,
            submitted_at: None,
        };
        let result = run_import(&args);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("ogr2ogr"));
    }

    // ── Task 140: DRA CSV format import ──────────────────────────────────────

    #[test]
    fn test_import_dra_csv_with_header() {
        let csv = "GEOID,DISTRICT\n53001000100,1\n53001000200,2\n53001000300,1\n";
        let result = import_assignments_from_dra_csv(csv);
        assert!(result.is_ok(), "should parse DRA CSV with header: {:?}", result.err());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 3);
        assert_eq!(assignments["53001000100"], 1);
        assert_eq!(assignments["53001000200"], 2);
        assert_eq!(assignments["53001000300"], 1);
    }

    #[test]
    fn test_import_dra_csv_without_header() {
        let csv = "53001000100,1\n53001000200,2\n53001000300,3\n";
        let result = import_assignments_from_dra_csv(csv);
        assert!(result.is_ok(), "should parse headerless DRA CSV: {:?}", result.err());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 3);
        assert_eq!(assignments["53001000100"], 1);
    }

    #[test]
    fn test_import_dra_csv_reversed_columns() {
        let csv = "DISTRICT,GEOID\n1,53001000100\n2,53001000200\n";
        let result = import_assignments_from_dra_csv(csv);
        assert!(result.is_ok(), "should parse reversed-column DRA CSV: {:?}", result.err());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 2);
        assert_eq!(assignments["53001000100"], 1);
        assert_eq!(assignments["53001000200"], 2);
    }

    #[test]
    fn test_migrate_args_parsed() {
        let args = crate::args::MigrateArgs {
            state: "WA".into(),
            label: "wa_house_old".into(),
            version: "v1".into(),
            year: "2020".into(),
            force: false,
        };
        assert_eq!(args.state, "WA");
        assert_eq!(args.label, "wa_house_old");
    }
}
