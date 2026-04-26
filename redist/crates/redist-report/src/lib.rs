/// redist-report: RPLAN interchange format, PlanManifest, plan path management.
///
/// Public API:
///   - rplan: RplanFile, RplanMetadata, validate_rplan, validate_rplan_str, write_rplan
///   - manifest: PlanManifest, sha256_bytes, sha256_file, default_label,
///               write_manifest_atomic, check_incomplete_plan, tiger_source_url
///   - paths: plan_output_dir, state_output_dir, create_plan_dir, check_plan_collision
pub mod manifest;
pub mod paths;
pub mod rplan;

pub use manifest::{
    PlanManifest, check_incomplete_plan, default_label, sha256_bytes, sha256_file,
    tiger_source_url, write_manifest_atomic,
};
pub use paths::{check_plan_collision, create_plan_dir, plan_output_dir, state_output_dir};
pub use rplan::{
    RplanError, RplanFile, RplanMetadata, ValidationResult, validate_geoid_format,
    validate_district_range, validate_rplan, validate_rplan_str, write_rplan,
};
