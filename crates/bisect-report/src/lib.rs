/// BISECT-report: RPLAN interchange format, PlanManifest, plan path management,
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
#[cfg(feature = "filesystem")]
pub mod audit;
#[cfg(feature = "filesystem")]
pub mod canonical;
#[cfg(feature = "filesystem")]
pub mod civic_gate;
pub mod comparison;
#[cfg(any(feature = "filesystem", feature = "report-render"))]
pub mod comparison_delivery;
pub mod tui_library;
pub mod tui_session;
pub mod assignment_discovery;
pub mod binary_provenance;
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod commission_audit;
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod commission_manifest;
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod report_delivery;
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod export;
#[cfg(any(feature = "filesystem", feature = "report-render"))]
pub mod html;
pub mod html_comparison;
#[cfg(any(feature = "filesystem", feature = "polygon-import"))]
pub mod import;
#[cfg(feature = "filesystem")]
pub mod manifest;
pub mod manifest_schema;
pub mod legacy_migration;
pub mod label_chain;
pub mod label_registry;
pub use manifest_schema::PlanManifest;
pub mod moe;
pub mod narrative;
#[cfg(any(feature = "filesystem", feature = "report-render"))]
pub mod narrative_manifest;
#[cfg(feature = "filesystem")]
pub mod paths;
#[cfg(feature = "filesystem")]
pub mod report;
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod report_assembly;
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod report_package;
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod rplan;

#[cfg(feature = "filesystem")]
pub use audit::{
    generate_verification_command, generate_verification_script, sha256_file as audit_sha256_file,
    ExternalAnalyzerRecord,
};
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub use export::{
    export_geojson, export_gerrychain_v23, export_tracts_csv, import_gerrychain_to_assignments,
};
#[cfg(any(feature = "filesystem", feature = "report-render"))]
pub use html::render_html_report;
#[cfg(all(feature = "filesystem", not(feature = "polygon-import")))]
pub use import::import_geojson_plan;
#[cfg(feature = "filesystem")]
pub use import::import_plan_to_rplan;
#[cfg(feature = "filesystem")]
pub use manifest::{
    check_incomplete_plan, sha256_bytes, sha256_file, tiger_source_url,
    write_manifest_atomic,
};
pub use manifest_schema::default_label;
#[cfg(feature = "filesystem")]
pub use paths::{
    check_plan_collision, create_plan_dir, now_iso8601, plan_output_dir, state_output_dir,
};
#[cfg(feature = "filesystem")]
pub use report::{
    assemble_report, check_required_analysis_files, Report, ReportContext, ReportSections,
    REQUIRED_ANALYSIS_FILES,
};
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub use rplan::{
    to_rplan_v02_document, validate_district_range, validate_geoid_format,
    validate_geoid_format_batch, validate_rplan, validate_rplan_str, RplanError,
    RplanFile, RplanMetadata, ValidationResult,
};

#[cfg(feature = "filesystem")]
pub use rplan::write_rplan;

#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod assignment_import;

#[cfg(feature = "polygon-import")]
pub use import::{import_geojson_plan, import_geojson_plan_detailed, GeojsonAssignmentResult};

#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod reproducibility;
pub mod verification;
pub mod assignment_verification;

pub mod plan_doctor;
pub mod label_rename;
pub mod assignment_output;
#[cfg(any(feature = "filesystem", feature = "interchange"))]
pub mod format_validation;

pub mod label_indexes;

pub mod tui_compare;

pub mod tui_doctor;

pub mod tui_verify;

pub mod tui_run;

#[cfg(any(feature="filesystem",feature="interchange"))]
pub mod run_target;

pub mod label_compare;
pub mod plan_hub;
#[cfg(any(feature="filesystem",feature="interchange"))]
pub mod export_source;

#[cfg(feature="label-import")]pub mod label_import;

#[cfg(any(feature="filesystem",feature="interchange"))]
pub mod import_source;

pub mod label_summary;
