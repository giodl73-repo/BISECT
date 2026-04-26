/// RPLAN v0.1 — redistricting plan interchange format.
///
/// RPLAN is a JSON file with:
///   - `rplan_version`: version string at root level ONLY (never inside metadata)
///   - `metadata`: RplanMetadata (all human-readable plan metadata)
///   - `assignments`: HashMap<geoid, district> (11-char numeric GEOIDs → 1-based district)
///   - `geometry`: optional GeoJSON FeatureCollection
///
/// Supported GEOID format: exactly 11 numeric characters (Census tract GEOIDs).
use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RplanError {
    #[error("GEOID format error: {0}")]
    InvalidGeoid(String),
    #[error("District range error: district {district} out of range [1, {num_districts}]")]
    InvalidDistrictRange { district: usize, num_districts: usize },
    #[error("Schema validation error: {0}")]
    SchemaError(String),
    #[error("Version error: {0}")]
    VersionError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Top-level RPLAN file structure.
/// `rplan_version` MUST be at root level — never duplicated inside `metadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RplanFile {
    pub rplan_version: String,
    pub metadata: RplanMetadata,
    pub assignments: HashMap<String, usize>,
    pub geometry: Option<serde_json::Value>,
}

/// Plan metadata. Does NOT include `rplan_version` — that lives at root.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RplanMetadata {
    pub label: String,
    pub state_fips: String,
    pub state_code: String,
    pub year: String,
    pub chamber: String,
    pub num_districts: usize,
    pub population_source: String,
    pub balance_tolerance_pct: f64,
    pub created_at: String,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_manifest: Option<serde_json::Value>,
    // NOTE: rplan_version is intentionally absent from metadata — it lives at root only
}

/// Validation result containing warnings (non-fatal) and errors (fatal).
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Validate a 11-character numeric GEOID (Census tract format).
pub fn validate_geoid_format(geoid: &str) -> Result<(), RplanError> {
    if geoid.len() != 11 {
        return Err(RplanError::InvalidGeoid(format!(
            "GEOID '{}' must be exactly 11 characters, got {}",
            geoid,
            geoid.len()
        )));
    }
    if !geoid.chars().all(|c| c.is_ascii_digit()) {
        return Err(RplanError::InvalidGeoid(format!(
            "GEOID '{}' must be all numeric digits",
            geoid
        )));
    }
    Ok(())
}

/// Validate that a district assignment value is in [1, num_districts].
/// `_value` is reserved for future use (e.g. district metadata).
pub fn validate_district_range(
    district: usize,
    _value: usize,
    num_districts: usize,
) -> Result<(), RplanError> {
    if district == 0 || district > num_districts {
        return Err(RplanError::InvalidDistrictRange { district, num_districts });
    }
    Ok(())
}

/// Validate a parsed RplanFile for schema correctness, version compatibility,
/// GEOID format, and district range.
pub fn validate_rplan(rplan: &RplanFile) -> Result<ValidationResult, RplanError> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // 1. Version check: parse "major.minor"
    let version_str = &rplan.rplan_version;
    let parts: Vec<&str> = version_str.splitn(2, '.').collect();
    if parts.len() != 2 {
        errors.push(format!(
            "rplan_version '{}' must be in 'major.minor' format",
            version_str
        ));
    } else {
        let major: u32 = parts[0].parse().unwrap_or(u32::MAX);
        let minor: u32 = parts[1].parse().unwrap_or(u32::MAX);
        if major != 0 {
            return Err(RplanError::VersionError(format!(
                "Unsupported major version {} in '{}'. This reader supports major version 0.",
                major, version_str
            )));
        }
        if minor > 1 {
            warnings.push(format!(
                "RPLAN minor version {} is newer than this reader (0.1). \
                 Some fields may be ignored.",
                minor
            ));
        }
    }

    // 2. Required metadata fields
    let meta = &rplan.metadata;
    if meta.label.is_empty() {
        errors.push("metadata.label is required and must not be empty".into());
    }
    if meta.state_fips.is_empty() {
        errors.push("metadata.state_fips is required".into());
    }
    if meta.state_code.is_empty() {
        errors.push("metadata.state_code is required".into());
    }
    if meta.year.is_empty() {
        errors.push("metadata.year is required".into());
    }
    if meta.chamber.is_empty() {
        errors.push("metadata.chamber is required".into());
    }
    if meta.num_districts == 0 {
        errors.push("metadata.num_districts must be > 0".into());
    }
    if meta.population_source.is_empty() {
        errors.push("metadata.population_source is required".into());
    }
    if meta.created_at.is_empty() {
        errors.push("metadata.created_at is required".into());
    }
    if meta.created_by.is_empty() {
        errors.push("metadata.created_by is required".into());
    }

    // 3. GEOID format + district range validation
    for (geoid, &district) in &rplan.assignments {
        if let Err(e) = validate_geoid_format(geoid) {
            errors.push(format!("GEOID validation failed: {}", e));
        }
        if meta.num_districts > 0 {
            if let Err(e) = validate_district_range(district, district, meta.num_districts) {
                errors.push(format!("District range error for GEOID {}: {}", geoid, e));
            }
        }
    }

    // 4. GeoJSON geometry check (if present)
    if let Some(geom) = &rplan.geometry {
        if geom["type"].as_str() != Some("FeatureCollection") {
            errors.push(format!(
                "geometry must be a GeoJSON FeatureCollection, got type='{}'",
                geom["type"].as_str().unwrap_or("unknown")
            ));
        }
    }

    let valid = errors.is_empty();
    if valid {
        Ok(ValidationResult { valid, warnings, errors })
    } else {
        Err(RplanError::SchemaError(errors.join("; ")))
    }
}

/// Parse a JSON string as an RplanFile and validate it.
pub fn validate_rplan_str(json: &str) -> Result<ValidationResult, RplanError> {
    let rplan: RplanFile = serde_json::from_str(json)?;
    validate_rplan(&rplan)
}

/// Write an RplanFile to disk at `path` (serialized as JSON).
pub fn write_rplan(
    path: &Path,
    metadata: &RplanMetadata,
    assignments: &HashMap<String, usize>,
    geometry: Option<serde_json::Value>,
) -> anyhow::Result<()> {
    let rplan = RplanFile {
        rplan_version: "0.1".into(),
        metadata: metadata.clone(),
        assignments: assignments.clone(),
        geometry,
    };
    let json = serde_json::to_string_pretty(&rplan)?;
    std::fs::write(path, json)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Test helpers (pub(crate) so tests can use them)
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) fn make_test_rplan(assignments: HashMap<String, usize>) -> RplanFile {
    RplanFile {
        rplan_version: "0.1".into(),
        metadata: RplanMetadata {
            label: "test_plan".into(),
            state_fips: "53".into(),
            state_code: "WA".into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts: 12,
            population_source: "total".into(),
            balance_tolerance_pct: 0.5,
            created_at: "2026-04-26T00:00:00Z".into(),
            created_by: "redist test".into(),
            ..Default::default()
        },
        assignments,
        geometry: None,
    }
}

#[cfg(test)]
pub(crate) fn make_minimal_valid_rplan() -> RplanFile {
    let mut assignments = HashMap::new();
    assignments.insert("53033000100".to_string(), 1usize);
    RplanFile {
        rplan_version: "0.1".into(),
        metadata: RplanMetadata {
            label: "wa_test".into(),
            state_fips: "53".into(),
            state_code: "WA".into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts: 10,
            population_source: "total".into(),
            balance_tolerance_pct: 0.5,
            created_at: "2026-04-26T00:00:00Z".into(),
            created_by: "test".into(),
            ..Default::default()
        },
        assignments,
        geometry: None,
    }
}

#[cfg(test)]
pub(crate) fn make_rplan_with_bad_district(num_districts: usize, bad_district: usize) -> RplanFile {
    let mut assignments = HashMap::new();
    assignments.insert("53033000100".to_string(), bad_district);
    RplanFile {
        rplan_version: "0.1".into(),
        metadata: RplanMetadata {
            label: "bad_plan".into(),
            state_fips: "53".into(),
            state_code: "WA".into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts,
            population_source: "total".into(),
            balance_tolerance_pct: 0.5,
            created_at: "2026-04-26T00:00:00Z".into(),
            created_by: "test".into(),
            ..Default::default()
        },
        assignments,
        geometry: None,
    }
}

// ---------------------------------------------------------------------------
// Tests — Tasks 1 + 2
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Task 1: RPLAN structure tests ---

    #[test]
    fn test_rplan_version_top_level_only() {
        let plan = RplanFile {
            rplan_version: "0.1".into(),
            metadata: RplanMetadata {
                label: "wa_house_draft1".into(),
                state_fips: "53".into(),
                ..Default::default()
            },
            assignments: HashMap::new(),
            geometry: None,
        };
        let json = serde_json::to_value(&plan).unwrap();
        assert!(json["rplan_version"].is_string());
        assert!(
            json["metadata"].get("rplan_version").is_none(),
            "rplan_version must NOT appear inside metadata"
        );
    }

    #[test]
    fn test_rplan_geoid_format_valid() {
        let result = validate_geoid_format("53033000100");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rplan_geoid_format_too_short() {
        let result = validate_geoid_format("5303300010"); // 10 chars
        assert!(result.is_err());
    }

    #[test]
    fn test_rplan_geoid_format_non_numeric() {
        let result = validate_geoid_format("5303300010A");
        assert!(result.is_err());
    }

    #[test]
    fn test_rplan_district_range_valid() {
        let result = validate_district_range(1, 5, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rplan_district_range_zero_invalid() {
        let result = validate_district_range(0, 5, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_rplan_district_range_exceeds_num_districts() {
        let result = validate_district_range(11, 5, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_rplan_roundtrip_preserves_assignments() {
        let mut assignments = HashMap::new();
        assignments.insert("53033000100".to_string(), 7usize);
        assignments.insert("53033000101".to_string(), 7usize);
        assignments.insert("53033000200".to_string(), 12usize);
        let plan = make_test_rplan(assignments.clone());
        let json = serde_json::to_string(&plan).unwrap();
        let decoded: RplanFile = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.assignments, assignments);
    }

    #[test]
    fn test_rplan_geometry_null_allowed() {
        let plan = make_test_rplan(HashMap::new());
        let json = serde_json::to_value(&plan).unwrap();
        assert!(json["geometry"].is_null());
    }

    // --- Task 2: Validator tests ---

    #[test]
    fn test_validate_missing_required_field_fails() {
        // metadata.label missing → schema error
        let json = r#"{"rplan_version":"0.1","metadata":{"state_fips":"53","state_code":"","year":"","chamber":"","num_districts":0,"population_source":"","balance_tolerance_pct":0.0,"created_at":"","created_by":""},"assignments":{},"geometry":null}"#;
        let result = validate_rplan_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_geoid_key_fails() {
        let json = r#"{
            "rplan_version": "0.1",
            "metadata": {"label":"x","state_fips":"53","state_code":"WA","year":"2020","chamber":"congressional","num_districts":1,"population_source":"total","balance_tolerance_pct":0.5,"created_at":"2026-04-26T00:00:00Z","created_by":"test"},
            "assignments": {"530330": 1},
            "geometry": null
        }"#;
        let result = validate_rplan_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("GEOID"));
    }

    #[test]
    fn test_validate_district_out_of_range_fails() {
        // num_districts=3 but assignment value=5
        let result = validate_rplan(&make_rplan_with_bad_district(3, 5));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_rplan_passes() {
        let rplan = make_minimal_valid_rplan();
        assert!(validate_rplan(&rplan).is_ok());
    }

    #[test]
    fn test_validate_unknown_major_version_fails() {
        let mut rplan = make_minimal_valid_rplan();
        rplan.rplan_version = "2.0".into();
        let result = validate_rplan(&rplan);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("major version"));
    }

    #[test]
    fn test_validate_unknown_minor_version_warns_not_fails() {
        // Minor version bump: 0.2 with 0.1 reader → warn only
        let mut rplan = make_minimal_valid_rplan();
        rplan.rplan_version = "0.2".into();
        let result = validate_rplan(&rplan);
        // Should succeed with a warning, not fail
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert!(!vr.warnings.is_empty(), "Should have a warning about newer minor version");
    }

    // --- write_rplan integration ---

    #[test]
    fn test_write_rplan_produces_valid_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut assignments = HashMap::new();
        assignments.insert("53033000100".to_string(), 1usize);
        assignments.insert("53033000101".to_string(), 2usize);
        let metadata = RplanMetadata {
            label: "wa_test".into(),
            state_fips: "53".into(),
            state_code: "WA".into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts: 10,
            population_source: "total".into(),
            balance_tolerance_pct: 0.5,
            created_at: "2026-04-26T00:00:00Z".into(),
            created_by: "redist test".into(),
            ..Default::default()
        };
        let path = tmp.path().join("plan.rplan");
        write_rplan(&path, &metadata, &assignments, None).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(validate_rplan_str(&content).is_ok());
    }
}
