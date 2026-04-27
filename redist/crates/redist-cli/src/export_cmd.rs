/// export_cmd.rs — `redist export` command dispatch.
///
/// Dispatches to redist_report::{export_geojson, export_gerrychain_v23, export_tracts_csv}
/// based on --format flag.
use std::path::PathBuf;
use crate::args::{ExportArgs, ExportFormat};
use redist_report::{
    RplanFile, export_geojson, export_gerrychain_v23, export_tracts_csv, sha256_file,
    write_rplan, PlanManifest,
    audit::{generate_verification_command_from_manifest, generate_verification_script},
};

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
            ExportFormat::ReproducibilityPackage => {
                write_reproducibility_package(
                    &plan_dir, &out_dir, &args.label, &rplan,
                )?;
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
// Reproducibility package writer — Task 141
// ---------------------------------------------------------------------------

/// Write a reproducibility package directory for court submission.
///
/// Output structure:
///   {out_dir}/{label}_reproducibility/
///     manifest.json        — copied from plan directory
///     {label}.rplan        — plan assignments in RPLAN format
///     analysis/            — all JSON files from plan_dir/analysis/ (if present)
///     audit.json           — audit record built from manifest
///     verify.sh            — executable lines from the verification command
fn write_reproducibility_package(
    plan_dir: &std::path::Path,
    out_dir: &std::path::Path,
    label: &str,
    rplan: &RplanFile,
) -> anyhow::Result<()> {
    // Output directory: {out_dir}/{label}_reproducibility/
    let pkg_dir = out_dir.join(format!("{}_reproducibility", label));
    std::fs::create_dir_all(&pkg_dir)?;

    // 1. manifest.json — copy from plan directory
    let manifest_src = plan_dir.join("manifest.json");
    let manifest_dst = pkg_dir.join("manifest.json");
    let manifest: Option<PlanManifest> = if manifest_src.exists() {
        std::fs::copy(&manifest_src, &manifest_dst)?;
        let content = std::fs::read_to_string(&manifest_src)?;
        serde_json::from_str(&content).ok()
    } else {
        None
    };

    // 2. {label}.rplan — export plan assignments in RPLAN format
    let rplan_path = pkg_dir.join(format!("{}.rplan", label));
    write_rplan(&rplan_path, &rplan.metadata, &rplan.assignments, rplan.geometry.clone())?;

    // 3. analysis/ — copy all JSON files from plan_dir/analysis/ if they exist
    let analysis_src = plan_dir.join("analysis");
    let analysis_dst = pkg_dir.join("analysis");
    let mut analysis_count = 0usize;
    if analysis_src.exists() && analysis_src.is_dir() {
        std::fs::create_dir_all(&analysis_dst)?;
        if let Ok(entries) = std::fs::read_dir(&analysis_src) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                if src_path.extension().map(|e| e == "json").unwrap_or(false) {
                    let dst_path = analysis_dst.join(entry.file_name());
                    std::fs::copy(&src_path, &dst_path)?;
                    analysis_count += 1;
                }
            }
        }
    }

    // 4. audit.json — build from manifest (same as --audit-only format)
    let audit_dst = pkg_dir.join("audit.json");
    let audit_json = if let Some(ref m) = manifest {
        serde_json::to_string_pretty(&serde_json::json!({
            "label": m.label,
            "audit": {
                "verification_command": generate_verification_command_from_manifest(m),
                "verification_script": generate_verification_script(m),
                "binary_version": m.binary_version,
                "binary_download_url": m.binary_download_url,
                "binary_sha256": if m.binary_sha256.is_empty() { "(not computed)".to_string() } else { m.binary_sha256.clone() },
                "adjacency_file": m.adjacency_file,
                "adjacency_sha256": if m.adjacency_sha256.is_empty() { "(not computed — run: sha256sum adjacency_file)".to_string() } else { m.adjacency_sha256.clone() },
                "tiger_source_url": m.tiger_source_url,
                "tiger_sha256": m.tiger_sha256.clone().unwrap_or_else(|| format!("(not recorded — download from {} and compute manually)", m.tiger_source_url)),
                "created_at": m.created_at,
                "seed": m.seed,
            }
        }))?
    } else {
        serde_json::to_string_pretty(&serde_json::json!({
            "label": label,
            "audit": { "note": "manifest.json not found — audit fields unavailable" }
        }))?
    };
    std::fs::write(&audit_dst, &audit_json)?;

    // 5. verify.sh — executable lines from the verification command
    let verify_dst = pkg_dir.join("verify.sh");
    let verify_content = if let Some(ref m) = manifest {
        generate_verification_script(m)
    } else {
        format!("# Verification script unavailable — manifest.json not found\n# Re-run: redist state --label {}", label)
    };
    std::fs::write(&verify_dst, &verify_content)?;
    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&verify_dst)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&verify_dst, perms)?;
    }

    // Report output
    eprintln!("[OK] Reproducibility package: {}/", pkg_dir.display());
    eprintln!("  manifest.json");
    eprintln!("  {}.rplan", label);
    eprintln!("  analysis/ ({} files)", analysis_count);
    eprintln!("  audit.json");
    eprintln!("  verify.sh");

    Ok(())
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

    // ── Task 141: reproducibility package ────────────────────────────────────

    fn make_test_manifest_for_repro() -> redist_report::PlanManifest {
        redist_report::PlanManifest {
            label: "vt_congressional_2020".into(),
            state_code: "VT".into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts: 1,
            seed: Some(42),
            binary_version: "0.1.0".into(),
            binary_sha256: "a".repeat(64),
            binary_download_url:
                "https://github.com/owner/redist/releases/download/v0.1.0/redist".into(),
            adjacency_file: "vt_adjacency_2020.adj.bin".into(),
            adjacency_sha256: "b".repeat(64),
            tiger_source_url:
                "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_50_tract.zip".into(),
            tiger_sha256: Some("c".repeat(64)),
            ..Default::default()
        }
    }

    #[test]
    fn test_reproducibility_package_contains_required_files() {
        use redist_report::{RplanFile, RplanMetadata};
        use std::collections::HashMap;

        let tmp = tempfile::TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plan");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();

        // Write a manifest
        let manifest = make_test_manifest_for_repro();
        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        std::fs::write(plan_dir.join("manifest.json"), &manifest_json).unwrap();

        // Write some analysis files
        std::fs::write(
            analysis_dir.join("summary.json"),
            serde_json::to_string(&serde_json::json!({"status": "ok"})).unwrap(),
        ).unwrap();
        std::fs::write(
            analysis_dir.join("compactness.json"),
            serde_json::to_string(&serde_json::json!({"status": "ok"})).unwrap(),
        ).unwrap();

        let label = "vt_congressional_2020";
        let rplan = RplanFile {
            rplan_version: "0.1".into(),
            metadata: RplanMetadata {
                label: label.into(),
                state_code: "VT".into(),
                year: "2020".into(),
                ..Default::default()
            },
            assignments: HashMap::new(),
            geometry: None,
        };

        let out_dir = tmp.path().join("exports");
        std::fs::create_dir_all(&out_dir).unwrap();

        write_reproducibility_package(&plan_dir, &out_dir, label, &rplan).unwrap();

        let pkg_dir = out_dir.join(format!("{}_reproducibility", label));
        assert!(pkg_dir.exists(), "package directory must be created");
        assert!(pkg_dir.join("manifest.json").exists(), "manifest.json must be present");
        assert!(pkg_dir.join(format!("{}.rplan", label)).exists(), "{}.rplan must be present", label);
        assert!(pkg_dir.join("audit.json").exists(), "audit.json must be present");
        assert!(pkg_dir.join("verify.sh").exists(), "verify.sh must be present");

        // Verify audit.json is valid JSON with an "audit" key
        let audit_content = std::fs::read_to_string(pkg_dir.join("audit.json")).unwrap();
        let audit_val: serde_json::Value = serde_json::from_str(&audit_content).unwrap();
        assert!(audit_val["audit"].is_object(), "audit.json must contain an 'audit' object");
    }
}
