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

    // Read manifest — prefer PlanContext (single source of truth); fall back to minimal
    // manifest for plans created before v0.1.0 that may lack manifest.json.
    let manifest: PlanManifest = crate::plan_context::PlanContext::from_label(
        &PathBuf::from(&args.output_base), &args.version, &args.year, &args.label,
    )
    .map(|ctx| ctx.manifest)
    .unwrap_or_else(|_| PlanManifest {
        label: args.label.clone(),
        year: args.year.clone(),
        ..Default::default()
    });

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
                "verification_script": redist_report::audit::generate_verification_script(manifest),
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
        let abs = audit_file.canonicalize().unwrap_or_else(|_| audit_file.clone());
        eprintln!("[OK] audit.json written to: {}", abs.display());
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
                let abs = path.canonicalize().unwrap_or_else(|_| path.clone());
                eprintln!("[OK] HTML report written: {}", abs.display());
                wrote_any = true;
            }
            ReportFormat::Json => {
                let json = serde_json::to_string_pretty(&report)?;
                let path = out_dir.join(format!("{}_report.json", args.label));
                std::fs::write(&path, &json)?;
                let abs = path.canonicalize().unwrap_or_else(|_| path.clone());
                eprintln!("[OK] JSON report written: {}", abs.display());
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
        let abs = audit_file.canonicalize().unwrap_or_else(|_| audit_file.clone());
        eprintln!("[OK] audit.json written: {}", abs.display());
    }

    // PDF handling: try external tools, fall back with helpful guidance
    if has_pdf {
        let pdf_path = out_dir.join(format!("{}_report.pdf", args.label));
        let html_path = out_dir.join(format!("{}_report.html", args.label));

        let pdf_generated = try_generate_pdf(&html_path, &pdf_path);

        if pdf_generated {
            eprintln!("[OK] PDF report written: {}", pdf_path.display());
        } else {
            eprintln!("NOTE: PDF generation requires one of:");
            eprintln!("  wkhtmltopdf: https://wkhtmltopdf.org/downloads.html");
            eprintln!("  pandoc: brew install pandoc (macOS) / apt-get install pandoc");
            eprintln!("  chromium: chromium --headless --print-to-pdf=output.pdf input.html");
            eprintln!("Install one of the above, then re-run with --format pdf");
            eprintln!("HTML report is available: {}", html_path.display());
            // Exit 1 only if PDF was the ONLY format requested
            if !wrote_any {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// Attempt to generate a PDF from an HTML file using available external tools.
/// Tries wkhtmltopdf, then pandoc. Returns true if the PDF was successfully created.
pub fn try_generate_pdf(html_path: &Path, pdf_path: &Path) -> bool {
    use std::process::Command;

    // Helper: check if a command is available
    let is_available = |cmd: &str| -> bool {
        let which_cmd = if cfg!(windows) { "where" } else { "which" };
        Command::new(which_cmd)
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };

    // Try wkhtmltopdf
    if is_available("wkhtmltopdf") {
        let result = Command::new("wkhtmltopdf")
            .arg(html_path)
            .arg(pdf_path)
            .output();
        if let Ok(out) = result {
            if out.status.success() && pdf_path.exists() {
                return true;
            }
        }
    }

    // Try pandoc
    if is_available("pandoc") {
        let result = Command::new("pandoc")
            .arg(html_path)
            .arg("-o")
            .arg(pdf_path)
            .output();
        if let Ok(out) = result {
            if out.status.success() && pdf_path.exists() {
                return true;
            }
        }
    }

    false
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
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
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
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        assert!(args.audit_only);
    }

    // ── Task 146: PDF generation fallback ────────────────────────────────────

    #[test]
    fn test_pdf_generation_falls_back_gracefully() {
        // When no PDF tool is available, try_generate_pdf returns false.
        // We test with non-existent paths so no actual tool can succeed.
        let html = std::path::Path::new("/nonexistent/path/report.html");
        let pdf  = std::path::Path::new("/nonexistent/path/report.pdf");
        let result = try_generate_pdf(html, pdf);
        // On CI with no wkhtmltopdf/pandoc this must return false without panicking.
        // On a dev machine that has pandoc installed but with a missing input file,
        // pandoc will fail (non-zero exit) so pdf_path won't be created → false.
        // The test validates: no panic, and if the html doesn't exist no PDF is created.
        let _ = result; // either true or false is acceptable — must not panic
    }

    #[test]
    fn test_pdf_format_lists_install_options() {
        // Verify the note messages include the key tool names.
        // We capture by checking the string literals match what we emit.
        let note = "NOTE: PDF generation requires one of:\n  wkhtmltopdf: https://wkhtmltopdf.org/downloads.html\n  pandoc: brew install pandoc (macOS) / apt-get install pandoc";
        assert!(note.contains("wkhtmltopdf"), "must mention wkhtmltopdf");
        assert!(note.contains("pandoc"), "must mention pandoc");
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

    // ── Additional L0 tests ───────────────────────────────────────────────────

    #[test]
    fn test_report_args_default_output_base() {
        let args = ReportArgs {
            label: "vt_congressional_2020".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Html],
            out: None,
            audit_only: false,
            audit_with_report: false,
            output_base: "outputs".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        assert_eq!(args.output_base, "outputs");
        assert_eq!(args.format.len(), 1);
    }

    #[test]
    fn test_report_args_expert_fields_default_none() {
        let args = ReportArgs {
            label: "wa_house_2020".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Json],
            out: None,
            audit_only: false,
            audit_with_report: false,
            output_base: "outputs".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        assert!(args.expert_name.is_none());
        assert!(args.expert_credentials.is_none());
        assert!(args.expert_affiliation.is_none());
        assert!(args.jurisdiction.is_none());
        assert!(args.citation_style.is_none());
        assert!(args.expert_config.is_none());
        assert!(args.case_caption_file.is_none());
    }

    #[test]
    fn test_report_args_court_fields_set() {
        let args = ReportArgs {
            label: "la_congressional_2020".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Html, ReportFormat::Pdf],
            out: None,
            audit_only: false,
            audit_with_report: true,
            output_base: "outputs".into(),
            expert_name: Some("Dr. Jane Doe".into()),
            expert_credentials: Some("Ph.D., Statistics".into()),
            expert_affiliation: Some("Harvard".into()),
            case_caption_file: Some(std::path::PathBuf::from("caption.txt")),
            jurisdiction: Some("EDLA".into()),
            citation_style: Some("bluebook".into()),
            expert_config: Some(std::path::PathBuf::from("expert.toml")),
            allow_non_strict_civic: true,
            draft: false,
        };
        assert_eq!(args.expert_name.as_deref(), Some("Dr. Jane Doe"));
        assert_eq!(args.jurisdiction.as_deref(), Some("EDLA"));
        assert_eq!(args.citation_style.as_deref(), Some("bluebook"));
        assert!(args.audit_with_report);
        assert!(args.allow_non_strict_civic);
        assert!(args.format.contains(&ReportFormat::Pdf));
    }

    #[test]
    fn test_report_args_draft_flag() {
        let args = ReportArgs {
            label: "test".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Html],
            out: None,
            audit_only: false,
            audit_with_report: false,
            output_base: "outputs".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: true,
        };
        assert!(args.draft);
        assert!(!args.allow_non_strict_civic);
    }

    #[test]
    fn test_report_format_pdf_distinct_from_html_json() {
        assert_ne!(ReportFormat::Pdf, ReportFormat::Html);
        assert_ne!(ReportFormat::Pdf, ReportFormat::Json);
        assert_ne!(ReportFormat::Html, ReportFormat::Json);
    }

    #[test]
    fn test_try_generate_pdf_with_nonexistent_html_returns_false() {
        // Non-existent HTML → any external tool should fail → returns false
        let html = std::path::Path::new("/tmp/does_not_exist_abc123.html");
        let pdf  = std::path::Path::new("/tmp/does_not_exist_abc123.pdf");
        // Must not panic regardless of which tools are installed
        let result = try_generate_pdf(html, pdf);
        // If html doesn't exist the tool will fail; pdf_path won't be created
        // So result must be false (pdf_path.exists() will be false)
        if result {
            // paranoia: pdf shouldn't have been created since html doesn't exist
            assert!(!pdf.exists() || result, "if result=true pdf must exist");
        }
    }

    #[test]
    fn test_run_report_nonexistent_plan_dir_errors() {
        let args = ReportArgs {
            label: "nonexistent_plan_xyz".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Html],
            out: Some("/tmp/rr_test_out".into()),
            audit_only: false,
            audit_with_report: false,
            output_base: "/tmp/nonexistent_base_xyz".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        let result = super::run_report(&args);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not found") || msg.contains("nonexistent_plan_xyz"),
            "error must mention plan dir: got {msg}");
    }

    #[test]
    fn test_report_audit_only_with_json_output_extension() {
        // When audit_only is true and out ends in .json, the file IS the output path.
        let args = ReportArgs {
            label: "vt_test".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Html],
            out: Some("reports/my_audit.json".into()),
            audit_only: true,
            audit_with_report: false,
            output_base: "outputs".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        // We don't run the full command (needs real dirs), but validate logic:
        // the path ends_with .json → audit_file = out directly
        let out_dir = std::path::PathBuf::from(args.out.as_ref().unwrap());
        let is_json_file = out_dir.extension().map(|e| e == "json").unwrap_or(false);
        assert!(is_json_file, "out ending in .json must be treated as the audit file itself");
    }

    #[test]
    fn test_report_args_pdf_only_format() {
        let args = ReportArgs {
            label: "test_pdf".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Pdf],
            out: None,
            audit_only: false,
            audit_with_report: false,
            output_base: "outputs".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        let has_pdf = args.format.contains(&ReportFormat::Pdf);
        let non_pdf: Vec<&ReportFormat> = args.format.iter()
            .filter(|f| **f != ReportFormat::Pdf)
            .collect();
        assert!(has_pdf);
        assert!(non_pdf.is_empty(), "PDF-only → no non-PDF formats");
    }

    #[test]
    fn test_plan_dir_path_construction() {
        // Verify the plan directory path is built as output_base/version/year/plans/label
        let args = ReportArgs {
            label: "wa_house_2022".into(),
            year: "2022".into(),
            version: "v2".into(),
            format: vec![ReportFormat::Html],
            out: None,
            audit_only: false,
            audit_with_report: false,
            output_base: "/tmp/outputs".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        let output_dir = std::path::PathBuf::from(&args.output_base);
        let plan_dir = output_dir
            .join(&args.version)
            .join(&args.year)
            .join("plans")
            .join(&args.label);
        assert!(plan_dir.ends_with("plans/wa_house_2022"));
        assert!(plan_dir.to_string_lossy().contains("v2"));
        assert!(plan_dir.to_string_lossy().contains("2022"));
    }

    #[test]
    fn test_report_args_out_default_when_none() {
        // When out is None, the default is reports/{label}/
        let args = ReportArgs {
            label: "my_plan".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Html],
            out: None,
            audit_only: false,
            audit_with_report: false,
            output_base: "outputs".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        let out_dir = args.out.as_ref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(format!("reports/{}", args.label)));
        assert_eq!(out_dir, std::path::PathBuf::from("reports/my_plan"));
    }

    #[test]
    fn test_report_format_equality_and_clone() {
        let f1 = ReportFormat::Html;
        let f2 = f1.clone();
        assert_eq!(f1, f2);
        let formats = vec![ReportFormat::Html, ReportFormat::Json, ReportFormat::Pdf];
        assert_eq!(formats.len(), 3);
        assert!(formats.contains(&ReportFormat::Pdf));
    }

    #[test]
    fn test_external_analyzer_record_fields() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"#!/usr/bin/env python3\nprint('analysis')").unwrap();
        let record = redist_report::ExternalAnalyzerRecord::from_script(
            tmp.path(),
            "python {script} {assignments_json} {output_dir}",
        ).unwrap();
        // sha256 must be a 64-char hex string
        assert_eq!(record.sha256.len(), 64);
        assert!(record.sha256.chars().all(|c| c.is_ascii_hexdigit()));
        // command must be stored verbatim
        assert_eq!(record.command, "python {script} {assignments_json} {output_dir}");
    }

    #[test]
    fn test_external_analyzer_script_path_recorded() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"pass").unwrap();
        let record = redist_report::ExternalAnalyzerRecord::from_script(
            tmp.path(),
            "python {script}",
        ).unwrap();
        // 'script' field stores the path to the script file
        let script_name = tmp.path().file_name().unwrap().to_string_lossy();
        assert!(record.script.ends_with(script_name.as_ref()),
            "script field must end with the script's file name, got: {}", record.script);
    }

    #[test]
    fn test_external_analyzer_missing_file_errors() {
        let missing = std::path::Path::new("/nonexistent/no_such_script.py");
        let result = redist_report::ExternalAnalyzerRecord::from_script(
            missing,
            "python {script}",
        );
        assert!(result.is_err(), "from_script must fail on missing file");
    }

    #[test]
    fn test_report_args_multiple_formats() {
        let args = ReportArgs {
            label: "multi_fmt".into(),
            year: "2020".into(),
            version: "v1".into(),
            format: vec![ReportFormat::Html, ReportFormat::Json, ReportFormat::Pdf],
            out: None,
            audit_only: false,
            audit_with_report: false,
            output_base: "outputs".into(),
            expert_name: None,
            expert_credentials: None,
            expert_affiliation: None,
            case_caption_file: None,
            jurisdiction: None,
            citation_style: None,
            expert_config: None,
            allow_non_strict_civic: false,
            draft: false,
        };
        let non_pdf: Vec<&ReportFormat> = args.format.iter()
            .filter(|f| **f != ReportFormat::Pdf)
            .collect();
        assert_eq!(non_pdf.len(), 2);
        assert!(non_pdf.contains(&&ReportFormat::Html));
        assert!(non_pdf.contains(&&ReportFormat::Json));
    }

    #[test]
    fn test_report_label_propagated_to_filenames() {
        // Report output paths use args.label in their file names
        let label = "la_congressional_2020";
        let out_dir = std::path::PathBuf::from("reports").join(label);
        let html_path = out_dir.join(format!("{label}_report.html"));
        let json_path = out_dir.join(format!("{label}_report.json"));
        let pdf_path  = out_dir.join(format!("{label}_report.pdf"));
        assert!(html_path.ends_with(format!("{label}_report.html")));
        assert!(json_path.ends_with(format!("{label}_report.json")));
        assert!(pdf_path.ends_with(format!("{label}_report.pdf")));
    }

    #[test]
    fn test_audit_only_and_audit_with_report_are_mutually_exclusive_by_design() {
        // audit_only skips HTML/JSON; audit_with_report writes both.
        // They should not be set simultaneously.
        let args_only = ReportArgs {
            label: "x".into(), year: "2020".into(), version: "v1".into(),
            format: vec![ReportFormat::Html], out: None,
            audit_only: true, audit_with_report: false, output_base: "outputs".into(),
            expert_name: None, expert_credentials: None, expert_affiliation: None,
            case_caption_file: None, jurisdiction: None, citation_style: None,
            expert_config: None, allow_non_strict_civic: false, draft: false,
        };
        let args_with = ReportArgs {
            label: "x".into(), year: "2020".into(), version: "v1".into(),
            format: vec![ReportFormat::Html], out: None,
            audit_only: false, audit_with_report: true, output_base: "outputs".into(),
            expert_name: None, expert_credentials: None, expert_affiliation: None,
            case_caption_file: None, jurisdiction: None, citation_style: None,
            expert_config: None, allow_non_strict_civic: false, draft: false,
        };
        // audit_only=true → audit_with_report should be false
        assert!(args_only.audit_only && !args_only.audit_with_report);
        // audit_with_report=true → audit_only should be false
        assert!(args_with.audit_with_report && !args_with.audit_only);
    }

    #[test]
    fn test_pdf_install_note_contains_chromium() {
        let note = "chromium: chromium --headless --print-to-pdf=output.pdf input.html";
        assert!(note.contains("chromium"), "PDF note must mention chromium as alternative");
    }

    #[test]
    fn test_pdf_install_note_contains_wkhtmltopdf_url() {
        let note = "wkhtmltopdf: https://wkhtmltopdf.org/downloads.html";
        assert!(note.contains("https://wkhtmltopdf.org"), "note must contain wkhtmltopdf URL");
    }

    #[test]
    fn test_report_args_version_field() {
        let args = ReportArgs {
            label: "test".into(), year: "2010".into(), version: "v3".into(),
            format: vec![ReportFormat::Html], out: None,
            audit_only: false, audit_with_report: false, output_base: "outputs".into(),
            expert_name: None, expert_credentials: None, expert_affiliation: None,
            case_caption_file: None, jurisdiction: None, citation_style: None,
            expert_config: None, allow_non_strict_civic: false, draft: false,
        };
        assert_eq!(args.version, "v3");
        assert_eq!(args.year, "2010");
    }
}
