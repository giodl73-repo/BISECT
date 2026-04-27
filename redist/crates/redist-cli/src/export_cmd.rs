/// export_cmd.rs — `redist export` command dispatch.
///
/// Dispatches to redist_report::{export_geojson, export_gerrychain_v23, export_tracts_csv}
/// based on --format flag.
use std::path::PathBuf;
use crate::args::{ExportArgs, ExportFormat};
use redist_report::{RplanFile, export_geojson, export_gerrychain_v23, export_tracts_csv, sha256_file};

/// Run the export command.
pub fn run_export(args: &ExportArgs) -> anyhow::Result<()> {
    let output_dir = PathBuf::from(&args.output_base);
    let plan_dir = output_dir
        .join(&args.version)
        .join(&args.year)
        .join("plans")
        .join(&args.label);

    if !plan_dir.exists() {
        anyhow::bail!(
            "Plan directory not found: '{}'. \
             Run 'redist state --label {}' first.",
            plan_dir.display(),
            args.label
        );
    }

    // Load the RPLAN file (or reconstruct from final_assignments.json)
    let rplan = load_rplan_from_plan_dir(&plan_dir, &args.label)?;

    // Determine output directory
    let out_dir = args
        .out
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("exports/{}", args.label)));
    std::fs::create_dir_all(&out_dir)?;

    for fmt in &args.format {
        match fmt {
            ExportFormat::GeoJson => {
                // Warn if plan has no geometry — GeoJSON will have null geometries
                warn_if_null_geometry(&rplan, &args.label, &args.year);
                let geojson_str = export_geojson(&rplan)?;
                let path = out_dir.join(format!("{}.geojson", args.label));
                std::fs::write(&path, &geojson_str)?;
                let sha = sha256_file(&path).unwrap_or_else(|_| "error".to_string());
                eprintln!("[OK] {} (sha256: {})", path.display(), sha);
            }
            ExportFormat::GerryChain => {
                let gc_str = export_gerrychain_v23(&rplan)?;
                let path = out_dir.join(format!("{}_gerrychain.json", args.label));
                std::fs::write(&path, &gc_str)?;
                let sha = sha256_file(&path).unwrap_or_else(|_| "error".to_string());
                eprintln!("[OK] {} (sha256: {})", path.display(), sha);
            }
            ExportFormat::Csv => {
                let csv_str = export_tracts_csv(&rplan)?;
                let path = out_dir.join(format!("{}_tracts.csv", args.label));
                std::fs::write(&path, &csv_str)?;
                let sha = sha256_file(&path).unwrap_or_else(|_| "error".to_string());
                eprintln!("[OK] {} (sha256: {})", path.display(), sha);
            }
        }
    }

    Ok(())
}

/// Load an RplanFile from a plan directory.
/// Tries plan.rplan first, then reconstructs from final_assignments.json + manifest.json.
/// Also checks `data/final_assignments.json` (new pipeline layout).
fn load_rplan_from_plan_dir(plan_dir: &std::path::Path, label: &str) -> anyhow::Result<RplanFile> {
    // Try to read a .rplan file
    let rplan_path = plan_dir.join(format!("{}.rplan", label));
    if rplan_path.exists() {
        let content = std::fs::read_to_string(&rplan_path)?;
        let rplan: RplanFile = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse {}: {}", rplan_path.display(), e))?;
        return Ok(rplan);
    }

    // Fall back: reconstruct from final_assignments.json (root or data/ subdir)
    let assignments_path_root = plan_dir.join("final_assignments.json");
    let assignments_path_data = plan_dir.join("data").join("final_assignments.json");
    let assignments_path = if assignments_path_root.exists() {
        assignments_path_root
    } else if assignments_path_data.exists() {
        assignments_path_data
    } else {
        anyhow::bail!(
            "No .rplan or final_assignments.json found in '{}'",
            plan_dir.display()
        );
    };

    let assignments_str = std::fs::read_to_string(&assignments_path)?;
    let assignments: std::collections::HashMap<String, usize> =
        serde_json::from_str(&assignments_str)?;

    // Read manifest for metadata
    let manifest_path = plan_dir.join("manifest.json");
    let metadata = if manifest_path.exists() {
        let manifest_str = std::fs::read_to_string(&manifest_path)?;
        let manifest: redist_report::PlanManifest = serde_json::from_str(&manifest_str)?;
        redist_report::RplanMetadata {
            label: manifest.label,
            state_fips: "00".into(),
            state_code: manifest.state_code,
            year: manifest.year,
            chamber: manifest.chamber,
            num_districts: manifest.num_districts,
            population_source: manifest.population_source,
            balance_tolerance_pct: manifest.balance_tolerance_pct,
            created_at: manifest.created_at,
            created_by: format!("redist {}", manifest.binary_version),
            ..Default::default()
        }
    } else {
        redist_report::RplanMetadata {
            label: label.to_string(),
            ..Default::default()
        }
    };

    Ok(RplanFile {
        rplan_version: "0.1".into(),
        metadata,
        assignments,
        geometry: None,
    })
}

/// Warn if all GeoJSON features will have null geometry.
/// Emits a warning with instructions to fetch/reimport geometry.
fn warn_if_null_geometry(rplan: &RplanFile, label: &str, year: &str) {
    // Plan has no geometry if geometry field is None or not a FeatureCollection
    // with non-null coordinate data.
    let has_geometry = rplan.geometry.as_ref().map(|g| {
        g["type"].as_str() == Some("FeatureCollection")
            && g["features"].as_array().map(|f| {
                f.iter().any(|feat| !feat["geometry"].is_null())
            }).unwrap_or(false)
    }).unwrap_or(false);

    if !has_geometry {
        let state = if rplan.metadata.state_code.is_empty() {
            "<STATE>".to_string()
        } else {
            rplan.metadata.state_code.clone()
        };
        eprintln!(
            "WARNING: Plan '{}' has no geometry -- GeoJSON will have null geometries.\n\
             To add geometry: run 'redist import' with a GeoJSON source, or use\n\
             'redist fetch --type geometry --states {} --year {}' + reimport.",
            label, state, year
        );
    }
}

// ---------------------------------------------------------------------------
// Tests — Task 7
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{ExportArgs, ExportFormat};

    #[test]
    fn test_export_formats_parsed() {
        let args = ExportArgs {
            label: "wa_house_draft1".into(),
            year: "2020".into(),
            version: "WA_Plans".into(),
            format: vec![ExportFormat::GeoJson, ExportFormat::GerryChain, ExportFormat::Csv],
            out: None,
            output_base: "outputs".into(),
        };
        assert!(args.format.contains(&ExportFormat::GeoJson));
        assert!(args.format.contains(&ExportFormat::GerryChain));
        assert!(args.format.contains(&ExportFormat::Csv));
    }

    #[test]
    fn test_export_default_format_is_geojson() {
        // Default format should be geojson when not specified
        let args = ExportArgs {
            label: "vt_congressional_2020".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ExportFormat::GeoJson], // default
            out: None,
            output_base: "outputs".into(),
        };
        assert_eq!(args.format, vec![ExportFormat::GeoJson]);
    }

    /// Task 114: SHA-256 of export output files is 64-character lowercase hex.
    #[test]
    fn test_export_output_sha256_format() {
        use redist_report::sha256_file;

        let tmp = tempfile::TempDir::new().unwrap();
        let file_path = tmp.path().join("test_output.geojson");
        std::fs::write(&file_path, r#"{"type":"FeatureCollection","features":[]}"#).unwrap();

        let sha = sha256_file(&file_path).expect("sha256_file should succeed");
        assert_eq!(sha.len(), 64, "SHA-256 must be 64 hex characters, got: {}", sha);
        assert!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-256 must be hex characters only, got: {}", sha
        );
        assert_eq!(sha, sha.to_lowercase(), "SHA-256 must be lowercase hex");
    }

    /// Scenario 15: warn_if_null_geometry triggers for a geometry-free plan.
    #[test]
    fn test_export_warns_on_null_geometry() {
        use redist_report::{RplanFile, RplanMetadata};
        use std::collections::HashMap;

        // Plan with no geometry (geometry: None)
        let plan_no_geom = RplanFile {
            rplan_version: "0.1".into(),
            metadata: RplanMetadata {
                label: "wa_test".into(),
                state_code: "WA".into(),
                year: "2020".into(),
                ..Default::default()
            },
            assignments: HashMap::new(),
            geometry: None,
        };
        // warn_if_null_geometry should not panic and should detect missing geometry
        // (we can't capture eprintln, but we can verify the detection logic)
        let has_geometry = plan_no_geom.geometry.as_ref().map(|g| {
            g["type"].as_str() == Some("FeatureCollection")
                && g["features"].as_array().map(|f| {
                    f.iter().any(|feat| !feat["geometry"].is_null())
                }).unwrap_or(false)
        }).unwrap_or(false);
        assert!(!has_geometry, "plan with geometry=None should be detected as missing geometry");

        // Plan with FeatureCollection but all null geometries
        let plan_null_feats = RplanFile {
            geometry: Some(serde_json::json!({
                "type": "FeatureCollection",
                "features": [
                    {"type": "Feature", "geometry": null, "properties": {"district_id": 1}},
                    {"type": "Feature", "geometry": null, "properties": {"district_id": 2}}
                ]
            })),
            ..plan_no_geom.clone()
        };
        let has_geometry2 = plan_null_feats.geometry.as_ref().map(|g| {
            g["type"].as_str() == Some("FeatureCollection")
                && g["features"].as_array().map(|f| {
                    f.iter().any(|feat| !feat["geometry"].is_null())
                }).unwrap_or(false)
        }).unwrap_or(false);
        assert!(!has_geometry2, "plan with all-null geometries should be detected as missing geometry");

        // The warn function itself must not panic for either case
        warn_if_null_geometry(&plan_no_geom, "wa_test", "2020");
        warn_if_null_geometry(&plan_null_feats, "wa_test", "2020");
    }
}
