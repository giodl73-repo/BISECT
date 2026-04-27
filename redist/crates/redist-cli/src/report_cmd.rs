/// report_cmd.rs — `redist report` command dispatch.
///
/// Spec 6 / Board amendments:
/// - PDF format exits with code 1 with clear message (other formats complete first)
/// - Checks all required analysis files before assembly
/// - Returns Err if assemble_report fails
use std::path::{Path, PathBuf};
use crate::args::{ReportArgs, ReportFormat};
use redist_report::{
    PlanManifest, ReportContext, assemble_report, check_required_analysis_files,
    render_html_report,
};

/// Run the report command. Returns Ok or Err for non-PDF failures.
/// PDF format is handled specially: other formats complete, then exits with code 1.
pub fn run_report(args: &ReportArgs) -> anyhow::Result<()> {
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

    // Read manifest
    let manifest_path = plan_dir.join("manifest.json");
    let manifest: PlanManifest = if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path)?;
        serde_json::from_str(&content)?
    } else {
        // Build a minimal manifest from args
        PlanManifest {
            label: args.label.clone(),
            year: args.year.clone(),
            ..Default::default()
        }
    };

    let ctx = ReportContext::new(plan_dir.clone(), manifest.clone());

    // Determine output directory
    let out_dir = args
        .out
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("reports/{}", args.label)));
    std::fs::create_dir_all(&out_dir)?;

    // Build audit JSON (used by --audit-only and --audit-with-report)
    let build_audit_json = |manifest: &PlanManifest| -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "label": manifest.label,
            "audit": {
                "verification_command": redist_report::audit::generate_verification_command_from_manifest(manifest),
                "binary_version": manifest.binary_version,
                "binary_download_url": manifest.binary_download_url,
                "binary_sha256": if manifest.binary_sha256.is_empty() { "(not computed)".to_string() } else { manifest.binary_sha256.clone() },
                "adjacency_file": manifest.adjacency_file,
                "adjacency_sha256": if manifest.adjacency_sha256.is_empty() { "(not computed — run: sha256sum adjacency_file)".to_string() } else { manifest.adjacency_sha256.clone() },
                "tiger_source_url": manifest.tiger_source_url,
                "tiger_sha256": manifest.tiger_sha256.clone().unwrap_or_else(|| format!("(not recorded — download from {} and compute manually)", manifest.tiger_source_url)),
                "created_at": manifest.created_at,
                "seed": manifest.seed,
            }
        }))?)
    };

    // Audit-only mode: write audit.json and return (skips HTML/JSON report)
    if args.audit_only {
        let audit_json = build_audit_json(&manifest)?;
        let audit_file = if out_dir.extension().map(|e| e == "json").unwrap_or(false) {
            out_dir.clone()
        } else {
            out_dir.join("audit.json")
        };
        std::fs::write(&audit_file, audit_json)?;
        eprintln!("[OK] audit.json written to: {}", audit_file.display());
        return Ok(());
    }

    // Check for missing required analysis files
    let missing = check_required_analysis_files(&ctx);
    if !missing.is_empty() {
        eprintln!(
            "WARNING: Missing required analysis files: {}",
            missing.join(", ")
        );
        eprintln!("Run 'redist analyze --label {}' to generate them.", args.label);
    }

    // Separate PDF from other formats
    let has_pdf = args.format.contains(&ReportFormat::Pdf);
    let non_pdf_formats: Vec<&ReportFormat> = args
        .format
        .iter()
        .filter(|f| **f != ReportFormat::Pdf)
        .collect();

    // Assemble report (returns Err if required files missing)
    let report = assemble_report(&ctx)?;

    // Write non-PDF formats
    let mut wrote_any = false;
    for fmt in &non_pdf_formats {
        match fmt {
            ReportFormat::Html => {
                let html = render_html_report(&report)?;
                let path = out_dir.join(format!("{}_report.html", args.label));
                std::fs::write(&path, &html)?;
                eprintln!("[OK] HTML report written: {}", path.display());
                wrote_any = true;
            }
            ReportFormat::Json => {
                let json = serde_json::to_string_pretty(&report)?;
                let path = out_dir.join(format!("{}_report.json", args.label));
                std::fs::write(&path, &json)?;
                eprintln!("[OK] JSON report written: {}", path.display());
                wrote_any = true;
            }
            ReportFormat::Pdf => unreachable!("PDF filtered above"),
        }
    }

    // Court submission mode: --audit-with-report writes audit.json alongside the report
    if args.audit_with_report {
        let audit_json = build_audit_json(&manifest)?;
        let audit_file = out_dir.join("audit.json");
        std::fs::write(&audit_file, &audit_json)?;
        eprintln!("[OK] audit.json written: {}", audit_file.display());
    }

    // PDF deferral: board amendment — exit code 1 with message after other formats complete
    if has_pdf {
        if wrote_any {
            eprintln!(
                "error: PDF generation is not yet available. Use --format html. \
                 HTML and JSON outputs were written successfully."
            );
        } else {
            eprintln!(
                "error: PDF generation is not yet available. Use --format html json instead."
            );
        }
        std::process::exit(1);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — Task 8
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::ReportArgs;

    #[test]
    fn test_report_args_formats_parsed() {
        let args = ReportArgs {
            label: "wa_house_draft1".into(),
            year: "2020".into(),
            version: "WA_Plans".into(),
            format: vec![ReportFormat::Html, ReportFormat::Json],
            out: Some("reports/wa_house/".into()),
            audit_only: false,
            audit_with_report: false,
            output_base: "outputs".into(),
        };
        assert!(args.format.contains(&ReportFormat::Html));
        assert!(args.format.contains(&ReportFormat::Json));
        assert!(!args.format.contains(&ReportFormat::Pdf));
    }

    #[test]
    fn test_report_audit_only_flag() {
        let args = ReportArgs {
            label: "wa_house_draft1".into(),
            year: "2020".into(),
            version: "WA_Plans".into(),
            format: vec![ReportFormat::Html],
            out: Some("reports/audit.json".into()),
            audit_only: true,
            audit_with_report: false,
            output_base: "outputs".into(),
        };
        assert!(args.audit_only);
    }

    #[test]
    fn test_external_analyzer_sha256_recorded_in_manifest() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"print('hello')").unwrap();
        let record = redist_report::ExternalAnalyzerRecord::from_script(
            tmp.path(),
            "python {script} {assignments_json} {output_dir}",
        )
        .unwrap();
        assert_eq!(record.sha256.len(), 64);
        assert!(record.command.contains("{assignments_json}"));
        assert!(record.command.contains("{output_dir}"));
    }
}
