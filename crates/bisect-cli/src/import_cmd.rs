use crate::args::ImportArgs;
use bisect_report::{
    validate_geoid_format_batch,
};
/// import_cmd.rs — `BISECT import` command dispatch.
///
/// Imports a GeoJSON plan (or GerryChain JSON) into the RPLAN format.
/// Dispatches to bisect_report::import_plan_to_rplan or import_gerrychain_to_assignments.
use std::path::PathBuf;

/// Return true if the path has a shapefile-family extension (.shp, .shx, .dbf, .prj, .zip).
pub fn is_shapefile_extension(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("shp") | Some("shx") | Some("dbf") | Some("prj") | Some("zip")
    )
}

/// Build the shapefile guidance error message for a given extension.
pub fn shapefile_error_message(path: &std::path::Path) -> String {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("shp");
    let (title, extra) = if ext == "zip" {
        (
            "ZIP archives (common for Census TIGER downloads) are not directly supported by bisect import.",
            "Extract the archive first, then convert the contained .shp to GeoJSON.",
        )
    } else {
        (
            &*format!("Shapefile format (.{ext}) is not directly supported by bisect import."),
            "Convert to GeoJSON first using one of:",
        )
    };
    format!(
        "ERROR: {title}\n\
         {extra}\n  \
         ogr2ogr -f GeoJSON output.geojson input.shp\n  \
         qgis: Layer > Save As > GeoJSON\n  \
         python: geopandas.read_file('input.shp').to_file('output.geojson', driver='GeoJSON')\n\
         Then: bisect import --format geojson --file output.geojson --label <label>"
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

    let assignments=bisect_report::import_source::parse(&args.format,&file_path.to_string_lossy(),&content)?;
    validate_geoid_format_batch(&assignments).map_err(|e|anyhow::anyhow!("GEOID validation failed: {e}"))?;
    let settings=bisect_report::import_source::Settings{label:args.label.clone(),state:args.state.clone(),year:args.year.clone(),format:args.format.clone(),source_name:file_path.display().to_string(),created_at:bisect_report::now_iso8601(),as_civic_counter_proposal:args.as_civic_counter_proposal,submitted_by:args.submitted_by.clone(),submitted_at:args.submitted_at.clone()};
    let plan=bisect_report::import_source::legacy(&settings,assignments);
    let legacy_files=bisect_report::import_source::legacy_files(&args.label,&plan)?;
    for name in [format!("{}.rplan",args.label),"final_assignments.json".into()]{std::fs::write(plan_dir.join(&name),&legacy_files[&name])?;}
    // Preserve the native failure boundary: a rejected civic manifest leaves
    // the earlier legacy RPLAN/assignments files, without promoting a manifest.
    let manifest=bisect_report::import_source::manifest(&settings,plan.metadata.num_districts,&bisect_report::now_iso8601(),&bisect_report::now_iso8601())?;
    bisect_report::manifest::callais_preflight(&manifest)?;
    bisect_report::write_manifest_atomic(&plan_dir,&manifest)?;
    let rplan_path=plan_dir.join(format!("{}.rplan",args.label));
    let assignments=&plan.assignments;let num_districts=plan.metadata.num_districts;

    eprintln!(
        "[OK] Imported {} tracts → {} districts. RPLAN: {}",
        assignments.len(),
        num_districts,
        rplan_path.display()
    );

    Ok(())
}

pub use bisect_report::assignment_import::{import_assignments_from_geojson_properties, import_assignments_from_dra_csv};

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
        assert!(
            msg.contains("ogr2ogr"),
            "error should mention ogr2ogr, got: {msg}"
        );
        assert!(
            msg.contains("GeoJSON"),
            "error should mention GeoJSON conversion, got: {msg}"
        );
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
        assert!(
            msg.contains("ogr2ogr"),
            "error should mention ogr2ogr, got: {msg}"
        );
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
        assert!(
            result.is_ok(),
            "should parse DRA CSV with header: {:?}",
            result.err()
        );
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
        assert!(
            result.is_ok(),
            "should parse headerless DRA CSV: {:?}",
            result.err()
        );
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 3);
        assert_eq!(assignments["53001000100"], 1);
    }

    #[test]
    fn test_import_dra_csv_reversed_columns() {
        let csv = "DISTRICT,GEOID\n1,53001000100\n2,53001000200\n";
        let result = import_assignments_from_dra_csv(csv);
        assert!(
            result.is_ok(),
            "should parse reversed-column DRA CSV: {:?}",
            result.err()
        );
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 2);
        assert_eq!(assignments["53001000100"], 1);
        assert_eq!(assignments["53001000200"], 2);
    }

    #[test]
    fn test_import_dra_csv_zero_district_fails() {
        let csv = "GEOID,DISTRICT\n53001000100,0\n";
        let result = import_assignments_from_dra_csv(csv);
        assert!(result.is_err(), "district 0 must fail");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("[INPUT]") && msg.contains("invalid district 0"),
            "error must classify and explain invalid district 0: {msg}"
        );
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

    // ── 15 additional tests for import_cmd ────────────────────────────────────

    #[test]
    fn test_is_shapefile_extension_shp() {
        assert!(is_shapefile_extension(std::path::Path::new("input.shp")));
    }

    #[test]
    fn test_is_shapefile_extension_shx() {
        assert!(is_shapefile_extension(std::path::Path::new("input.shx")));
    }

    #[test]
    fn test_is_shapefile_extension_dbf() {
        assert!(is_shapefile_extension(std::path::Path::new("input.dbf")));
    }

    #[test]
    fn test_is_shapefile_extension_prj() {
        assert!(is_shapefile_extension(std::path::Path::new("input.prj")));
    }

    #[test]
    fn test_is_shapefile_extension_zip() {
        assert!(is_shapefile_extension(std::path::Path::new(
            "tiger_2020.zip"
        )));
    }

    #[test]
    fn test_is_shapefile_extension_geojson_not_shapefile() {
        assert!(!is_shapefile_extension(std::path::Path::new(
            "plan.geojson"
        )));
    }

    #[test]
    fn test_is_shapefile_extension_json_not_shapefile() {
        assert!(!is_shapefile_extension(std::path::Path::new("plan.json")));
    }

    #[test]
    fn test_is_shapefile_extension_csv_not_shapefile() {
        assert!(!is_shapefile_extension(std::path::Path::new(
            "dra_export.csv"
        )));
    }

    #[test]
    fn test_shapefile_error_message_shp_mentions_qgis() {
        let msg = shapefile_error_message(std::path::Path::new("plan.shp"));
        assert!(
            msg.contains("qgis") || msg.contains("QGIS") || msg.contains("qgis"),
            "error must mention QGIS alternative: {msg}"
        );
    }

    #[test]
    fn test_shapefile_error_message_contains_python_geopandas() {
        let msg = shapefile_error_message(std::path::Path::new("plan.shp"));
        assert!(
            msg.contains("geopandas"),
            "error must mention geopandas Python option: {msg}"
        );
    }

    #[test]
    fn test_import_dbf_extension_gives_error() {
        let args = ImportArgs {
            file: std::path::PathBuf::from("plan.dbf"),
            state: "CA".into(),
            year: "2020".into(),
            label: "ca_test".into(),
            version: "test".into(),
            format: "geojson".into(),
            output_base: "outputs".into(),
            as_civic_counter_proposal: false,
            submitted_by: None,
            submitted_at: None,
        };
        let result = run_import(&args);
        assert!(result.is_err(), "importing .dbf should fail");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Shapefile") || msg.contains("shapefile"),
            "error must mention shapefile format: {msg}"
        );
    }

    #[test]
    fn test_import_dra_csv_empty_fails() {
        let result = import_assignments_from_dra_csv("");
        assert!(result.is_err(), "empty CSV must fail");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("empty"), "error must mention empty CSV: {msg}");
    }

    #[test]
    fn test_import_dra_csv_no_data_rows_fails() {
        // Header line only, no data
        let csv = "GEOID,DISTRICT\n";
        let result = import_assignments_from_dra_csv(csv);
        assert!(
            result.is_err(),
            "CSV with header only must fail with no assignments"
        );
    }

    #[test]
    fn test_import_dra_csv_large_dataset() {
        // 100 rows
        let mut csv = String::from("GEOID,DISTRICT\n");
        for i in 0..100usize {
            csv.push_str(&format!("{:011},1\n", 53_001_000_100u64 + i as u64));
        }
        let result = import_assignments_from_dra_csv(&csv);
        assert!(result.is_ok(), "100-row CSV must parse successfully");
        assert_eq!(result.unwrap().len(), 100);
    }

    #[test]
    fn test_import_dra_csv_auto_detect_reversed_columns_no_header() {
        // Reversed with no header: district (small int) comes first,
        // but auto-detect should handle 5-char FIPS + 11-char GEOID swap.
        // Let's test the actual auto-swap: small district val on left, 11-char geoid on right.
        let csv = "1,53001000100\n2,53001000200\n";
        let result = import_assignments_from_dra_csv(csv);
        assert!(
            result.is_ok(),
            "auto-swap must handle reversed no-header CSV: {:?}",
            result.err()
        );
        let assignments = result.unwrap();
        assert_eq!(assignments["53001000100"], 1);
        assert_eq!(assignments["53001000200"], 2);
    }

    #[test]
    fn test_import_geojson_properties_geoid_lowercase_and_uppercase() {
        // Both "geoid" and "GEOID" keys must be detected
        let geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"geoid": "53001000100", "district": 1}
                },
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"GEOID": "53001000200", "district": 2}
                }
            ]
        });
        let result = import_assignments_from_geojson_properties(&geojson.to_string());
        assert!(
            result.is_ok(),
            "must handle both geoid and GEOID: {:?}",
            result.err()
        );
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 2);
        assert_eq!(assignments["53001000100"], 1);
        assert_eq!(assignments["53001000200"], 2);
    }

    #[test]
    fn test_import_geojson_properties_zero_district_fails() {
        let geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"geoid": "53001000100", "district": 0}
                }
            ]
        });
        let result = import_assignments_from_geojson_properties(&geojson.to_string());
        assert!(result.is_err(), "district 0 must fail");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("[INPUT]") && msg.contains("invalid district 0"),
            "error must classify and explain invalid district 0: {msg}"
        );
    }

    #[test]
    fn test_import_geojson_properties_non_integer_district_fails() {
        let geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"geoid": "53001000100", "district": "one"}
                }
            ]
        });
        let result = import_assignments_from_geojson_properties(&geojson.to_string());
        assert!(result.is_err(), "non-integer district must fail");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("[INPUT]") && msg.contains("integer district"),
            "error must classify and explain malformed district: {msg}"
        );
    }
}
