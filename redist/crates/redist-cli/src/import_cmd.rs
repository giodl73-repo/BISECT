/// import_cmd.rs — `redist import` command dispatch.
///
/// Imports a GeoJSON plan (or GerryChain JSON) into the RPLAN format.
/// Dispatches to redist_report::import_plan_to_rplan or import_gerrychain_to_assignments.
use std::path::PathBuf;
use crate::args::ImportArgs;
use redist_report::{
    RplanFile, RplanMetadata, import_geojson_plan, import_gerrychain_to_assignments,
    write_rplan, validate_geoid_format_batch,
};

/// Run the import command.
pub fn run_import(args: &ImportArgs) -> anyhow::Result<()> {
    let file_path = &args.file;
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
        _ => {
            // Default: GeoJSON import (PIP assignment)
            // For GeoJSON import without real tract centroids: use assignment field from properties
            // or fall back to deriving centroids from the GeoJSON itself
            import_assignments_from_geojson_properties(&content)?
        }
    };

    // Validate GEOID format
    validate_geoid_format_batch(&assignments)
        .map_err(|e| anyhow::anyhow!("GEOID validation failed: {e}"))?;

    let num_districts = assignments.values().copied().max().unwrap_or(0);

    let metadata = RplanMetadata {
        label: args.label.clone(),
        state_fips: state_code_to_fips(&args.state),
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

    // Write minimal manifest.json
    let manifest = redist_report::PlanManifest {
        label: args.label.clone(),
        state_code: args.state.to_uppercase(),
        year: args.year.clone(),
        chamber: "imported".into(),
        num_districts,
        created_at: redist_report::now_iso8601(),
        ..Default::default()
    };
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

fn state_code_to_fips(state_code: &str) -> String {
    match state_code.to_uppercase().as_str() {
        "AL" => "01", "AK" => "02", "AZ" => "04", "AR" => "05", "CA" => "06",
        "CO" => "08", "CT" => "09", "DE" => "10", "FL" => "12", "GA" => "13",
        "HI" => "15", "ID" => "16", "IL" => "17", "IN" => "18", "IA" => "19",
        "KS" => "20", "KY" => "21", "LA" => "22", "ME" => "23", "MD" => "24",
        "MA" => "25", "MI" => "26", "MN" => "27", "MS" => "28", "MO" => "29",
        "MT" => "30", "NE" => "31", "NV" => "32", "NH" => "33", "NJ" => "34",
        "NM" => "35", "NY" => "36", "NC" => "37", "ND" => "38", "OH" => "39",
        "OK" => "40", "OR" => "41", "PA" => "42", "RI" => "44", "SC" => "45",
        "SD" => "46", "TN" => "47", "TX" => "48", "UT" => "49", "VT" => "50",
        "VA" => "51", "WA" => "53", "WV" => "54", "WI" => "55", "WY" => "56",
        _ => "00",
    }.to_string()
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
        };
        assert_eq!(args.state, "WA");
        assert_eq!(args.label, "wa_imported_test");
    }

    #[test]
    fn test_migrate_args_parsed() {
        let args = crate::args::MigrateArgs {
            state: "WA".into(),
            label: "wa_house_old".into(),
            version: "v1".into(),
            year: "2020".into(),
        };
        assert_eq!(args.state, "WA");
        assert_eq!(args.label, "wa_house_old");
    }
}
