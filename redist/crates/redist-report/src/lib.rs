/// redist-report: RPLAN interchange format, PlanManifest, plan path management,
/// commission report assembly (HTML/JSON), GeoJSON/GerryChain/CSV export,
/// GeoJSON plan import (PIP), and SHA-256 audit chain.
///
/// Public API:
///   - rplan: RplanFile, RplanMetadata, validate_rplan, validate_rplan_str, write_rplan
///   - manifest: PlanManifest, sha256_bytes, sha256_file, default_label,
///               write_manifest_atomic, check_incomplete_plan, tiger_source_url
///   - paths: plan_output_dir, state_output_dir, create_plan_dir, check_plan_collision
///   - audit: sha256_file, ExternalAnalyzerRecord, generate_verification_command
///   - export: export_geojson, export_gerrychain_v23, export_tracts_csv
///   - import: import_geojson_plan, import_plan_to_rplan
///   - report: Report, assemble_report, check_required_analysis_files, ReportContext
///   - html: render_html_report
pub mod audit;
pub mod canonical;
pub mod civic_gate;
pub mod export;
pub mod html;
pub mod import;
pub mod manifest;
pub mod paths;
pub mod report;
pub mod rplan;

pub use audit::{ExternalAnalyzerRecord, generate_verification_command, generate_verification_script, sha256_file as audit_sha256_file};
pub use export::{export_geojson, export_gerrychain_v23, export_tracts_csv, import_gerrychain_to_assignments};
pub use html::render_html_report;
pub use import::{import_geojson_plan, import_plan_to_rplan};
pub use manifest::{
    PlanManifest, check_incomplete_plan, default_label, sha256_bytes, sha256_file,
    tiger_source_url, write_manifest_atomic,
};
pub use paths::{check_plan_collision, create_plan_dir, now_iso8601, plan_output_dir, state_output_dir};
pub use report::{
    Report, ReportContext, ReportSections, assemble_report,
    check_required_analysis_files, REQUIRED_ANALYSIS_FILES,
};
pub use rplan::{
    RplanError, RplanFile, RplanMetadata, ValidationResult, validate_geoid_format,
    validate_geoid_format_batch, validate_district_range, validate_rplan,
    validate_rplan_str, write_rplan,
};
