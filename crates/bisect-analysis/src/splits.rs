/// County and municipal split analysis.
///
/// County splits are detected by parsing the 5-char county FIPS prefix from each
/// 11-char Census tract GEOID. Municipal splits require a place-to-tract lookup
/// table (from `bisect fetch --type geography`).
use std::collections::{HashMap, HashSet};

use crate::county_names::county_name as lookup_county_name;
use crate::split_standards::{get_split_standard, SplitStandard};
use thiserror::Error;

// ---------------------------------------------------------------------------
// County split types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct CountySplitResult {
    pub total: usize,
    pub split: usize,
    /// (total_counties - split_counties) / total_counties
    pub preservation_score: f64,
    pub split_list: Vec<CountySplit>,
    pub legal_standard: Option<String>,
    pub compliance_assessment: Option<String>,
    pub disclaimer: Option<String>,
    /// State-specific note about entity count (e.g., VA's 133 = 95 counties + 38 cities)
    pub entity_note: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CountySplit {
    pub county_fips: String,
    pub county_name: Option<String>,
    pub districts_containing: Vec<usize>,
    pub tract_count: usize,
    /// Number of distinct districts this county is split across.
    pub split_severity: usize,
}

// ---------------------------------------------------------------------------
// Municipal split types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct MunicipalSplitResult {
    /// False when place_to_tracts is empty (data not available).
    pub available: bool,
    pub total: usize,
    pub split: usize,
    pub preservation_score: f64,
    pub split_list: Vec<MunicipalSplit>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MunicipalSplit {
    pub place_fips: String,
    pub place_name: String,
    pub districts_containing: Vec<usize>,
    pub split_severity: usize,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SplitError {
    #[error("[INPUT] explicit county identities require at least one unit")]
    NoExplicitUnits,
    #[error("[INPUT] county identity length {counties} does not match ordered unit length {units}")]
    CountyIdentityLength { units: usize, counties: usize },
    #[error("[INPUT] municipal identity length {municipalities} does not match ordered unit length {units}")]
    MunicipalIdentityLength { units: usize, municipalities: usize },
    #[error("[INPUT] explicit municipal identities do not cover the assignment universe")]
    MunicipalAssignmentMismatch,
    #[error("[INPUT] duplicate explicit unit identity {unit}")]
    DuplicateUnitIdentity { unit: String },
    #[error("[INPUT] missing county identity for unit {unit}; missing is not zero splits")]
    MissingCountyIdentity { unit: String },
    #[error("[INPUT] county identity supplied for an unassigned unit {unit}")]
    UnexpectedCountyIdentity { unit: String },
    #[error("[INPUT] invalid explicit {kind} identity {id:?}; use 1 through 200 bytes without control characters or blank-only IDs")]
    InvalidExplicitIdentity { kind: &'static str, id: String },
    #[error("[INPUT] GEOID {geoid} cannot be sliced at byte 5 to extract county FIPS")]
    InvalidCountyFipsBoundary { geoid: String },
    #[error("[INPUT] assignment for tract {geoid} uses invalid district 0")]
    InvalidDistrictLabel { geoid: String },
}

/// Build an exact opaque unit-to-county map from a portable context's order.
/// No Census prefix, FIPS name or jurisdiction policy is inferred.
pub fn explicit_counties_from_ordered_units(
    units: &[String], counties: &[Option<String>],
) -> Result<HashMap<String, String>, SplitError> {
    if units.is_empty() { return Err(SplitError::NoExplicitUnits); }
    if units.len() != counties.len() { return Err(SplitError::CountyIdentityLength { units: units.len(), counties: counties.len() }); }
    let mut result = HashMap::new();
    for (unit, county) in units.iter().zip(counties) {
        validate_explicit_identity(unit, "unit")?;
        let county = county.as_ref().ok_or_else(|| SplitError::MissingCountyIdentity { unit: unit.clone() })?;
        validate_explicit_identity(county, "county")?;
        if result.insert(unit.clone(), county.clone()).is_some() { return Err(SplitError::DuplicateUnitIdentity { unit: unit.clone() }); }
    }
    Ok(result)
}

fn validate_explicit_identity(id: &str, kind: &'static str) -> Result<(), SplitError> {
    if id.trim().is_empty() || id.len() > 200 || id.chars().any(char::is_control) {
        return Err(SplitError::InvalidExplicitIdentity { kind, id: id.into() });
    }
    Ok(())
}


/// Optional RCTX membership for opaque custom units. Null means unknown.
pub struct ExplicitMunicipalIdentities {
    by_unit: std::collections::BTreeMap<String, Option<String>>,
}

impl ExplicitMunicipalIdentities {
    pub fn from_ordered_units(units: &[String], ids: &[Option<String>]) -> Result<Self, SplitError> {
        if units.is_empty() { return Err(SplitError::NoExplicitUnits); }
        if units.len() != ids.len() {
            return Err(SplitError::MunicipalIdentityLength { units: units.len(), municipalities: ids.len() });
        }
        let mut by_unit = std::collections::BTreeMap::new();
        for (unit, id) in units.iter().zip(ids) {
            validate_explicit_identity(unit, "unit")?;
            if let Some(id) = id { validate_explicit_identity(id, "municipal")?; }
            if by_unit.insert(unit.clone(), id.clone()).is_some() {
                return Err(SplitError::DuplicateUnitIdentity { unit: unit.clone() });
            }
        }
        Ok(Self { by_unit })
    }

    pub fn known_units(&self) -> usize {
        self.by_unit.values().filter(|id| id.is_some()).count()
    }

    /// Attach evidence only for an explicit municipal column. An incomplete
    /// column preserves the county report without certifying zero splits.
    pub fn attach(
        &self, report: &mut serde_json::Value, assignments: &HashMap<String, usize>,
        place_names: &HashMap<String, String>,
    ) -> Result<(), SplitError> {
        use sha2::{Digest, Sha256};
        if assignments.len() != self.by_unit.len() ||
            self.by_unit.keys().any(|unit| !assignments.contains_key(unit)) {
            return Err(SplitError::MunicipalAssignmentMismatch);
        }
        let canonical = serde_json::to_vec(&self.by_unit).expect("string-only membership serializes");
        let mut hash = Sha256::new();
        hash.update(b"bisect-explicit-municipal-identity-v1\0");
        hash.update(canonical);
        report["municipal_identity_schema"] = serde_json::json!("bisect-explicit-municipal-membership-v1");
        report["municipal_identity_source"] = serde_json::json!("matching-rctx-explicit-municipalities");
        report["municipal_identity_sha256"] = serde_json::json!(format!("{:x}", hash.finalize()));
        let known = self.known_units();
        let missing = self.by_unit.len() - known;
        if missing != 0 {
            report["municipalities"] = serde_json::json!({
                "available":false, "status":"incomplete-explicit-membership",
                "known_units":known, "missing_units":missing,
                "total":null, "split":null, "preservation_score":null, "split_list":[]
            });
            return Ok(());
        }
        let mut groups: std::collections::BTreeMap<&str, (std::collections::BTreeSet<usize>, usize)> =
            std::collections::BTreeMap::new();
        for (unit, id) in &self.by_unit {
            let district = assignments[unit];
            if district == 0 { return Err(SplitError::InvalidDistrictLabel { geoid: unit.clone() }); }
            let group = groups.entry(id.as_deref().expect("complete membership")).or_default();
            group.0.insert(district);
            group.1 += 1;
        }
        let splits: Vec<_> = groups.iter().filter(|(_, (districts, _))| districts.len() > 1)
            .map(|(&id, (districts, units))| serde_json::json!({
                "municipality_id":id, "municipality_name":place_names.get(id),
                "districts_containing":districts, "unit_count":units, "split_severity":districts.len()
            })).collect();
        report["municipalities"] = serde_json::json!({
            "available":true, "status":"complete-explicit-membership",
            "known_units":known, "missing_units":0, "total":groups.len(), "split":splits.len(),
            "preservation_score":(groups.len() - splits.len()) as f64 / groups.len() as f64,
            "split_list":splits
        });
        Ok(())
    }
}

/// Operational splits over supplied county identities, shared by desktop/WASM.
/// This deliberately has a separate report schema from Census FIPS inference.
pub fn assemble_explicit_splits_report(
    assignments: &HashMap<String, usize>, county_ids: &HashMap<String, String>,
    state: &str, year: &str, county_names: Option<&HashMap<String, String>>,
    place_to_units: &HashMap<String, Vec<String>>, place_names: &HashMap<String, String>,
) -> Result<serde_json::Value, SplitError> {
    if assignments.is_empty() { return Err(SplitError::NoExplicitUnits); }
    for unit in county_ids.keys() { if !assignments.contains_key(unit) { return Err(SplitError::UnexpectedCountyIdentity { unit: unit.clone() }); } }
    let mut groups: std::collections::BTreeMap<String, (std::collections::BTreeSet<usize>, usize)> = std::collections::BTreeMap::new();
    for (unit, &district) in assignments {
        validate_explicit_identity(unit, "unit")?;
        if district == 0 { return Err(SplitError::InvalidDistrictLabel { geoid: unit.clone() }); }
        let county = county_ids.get(unit).ok_or_else(|| SplitError::MissingCountyIdentity { unit: unit.clone() })?;
        validate_explicit_identity(county, "county")?;
        let group = groups.entry(county.clone()).or_default(); group.0.insert(district); group.1 += 1;
    }
    let split_list: Vec<_> = groups.iter().filter(|(_, (districts, _))| districts.len() > 1).map(|(county, (districts, units))| serde_json::json!({
        "county_id": county, "county_name": county_names.and_then(|names| names.get(county)),
        "districts_containing": districts, "unit_count": units, "split_severity": districts.len(),
    })).collect();
    let municipal = try_analyze_municipal_splits(assignments, place_to_units, place_names)?;
    let municipalities: Vec<_> = municipal.split_list.iter().map(|place| serde_json::json!({
        "municipality_id": place.place_fips, "municipality_name": place.place_name,
        "districts_containing": place.districts_containing, "split_severity": place.split_severity,
    })).collect();
    // Keep the effective identities bound even when only splits.json is retained.
    // This is a new domain-separated identity, not a replacement RPLAN/RCTX hash.
    use sha2::{Digest, Sha256};
    let ordered: std::collections::BTreeMap<_,_> = county_ids.iter().collect();
    let canonical = serde_json::to_vec(&ordered).expect("string-only county map serializes");
    let mut identity = Sha256::new(); identity.update(b"bisect-explicit-county-identity-v1\0"); identity.update(canonical);
    let identity = format!("{:x}", identity.finalize());
    Ok(serde_json::json!({"analyzer":"splits", "state":state, "year":year,
        "subdivision_identity":"explicit-unit-county-map-v1",
        "county_identity_sha256":identity,
        "counties":{"available":true,"subdivision_term":"county","subdivision_term_plural":"counties",
            "total":groups.len(),"split":split_list.len(),"preservation_score":(groups.len()-split_list.len()) as f64/groups.len() as f64,
            "split_list":split_list,"legal_standard":serde_json::Value::Null,"compliance_assessment":serde_json::Value::Null,
            "disclaimer":"Operational counts over supplied county identities. IDs are opaque claims, not inferred Census FIPS or a legal preservation finding."},
        "municipalities":{"available":municipal.available,"total":municipal.total,"split":municipal.split,
            "preservation_score":municipal.preservation_score,"split_list":municipalities}
    }))
}

// ---------------------------------------------------------------------------
// GEOID helpers
// ---------------------------------------------------------------------------

/// Parse county FIPS from an 11-char (or longer) Census tract GEOID.
/// "530330001001" -> "53033" (state 53, county 033)
pub fn county_fips_from_geoid(geoid: &str) -> &str {
    try_county_fips_from_geoid(geoid).expect("GEOID has a valid county FIPS byte boundary")
}

pub fn try_county_fips_from_geoid(geoid: &str) -> Result<&str, SplitError> {
    if geoid.len() >= 5 {
        if geoid.is_char_boundary(5) {
            Ok(&geoid[..5])
        } else {
            Err(SplitError::InvalidCountyFipsBoundary {
                geoid: geoid.to_string(),
            })
        }
    } else {
        Ok(geoid)
    }
}

// ---------------------------------------------------------------------------
// County split analysis
// ---------------------------------------------------------------------------

/// Analyze county splits in `assignments`.
///
/// `county_names` — optional map from county FIPS to name (for display).
/// `state_code`   — optional two-letter state code for legal standard lookup.
pub fn analyze_county_splits(
    assignments: &HashMap<String, usize>,
    county_names: Option<&HashMap<String, String>>,
) -> CountySplitResult {
    try_analyze_county_splits(assignments, county_names)
        .expect("county split assignment GEOIDs are valid")
}

pub fn try_analyze_county_splits(
    assignments: &HashMap<String, usize>,
    county_names: Option<&HashMap<String, String>>,
) -> Result<CountySplitResult, SplitError> {
    try_analyze_county_splits_with_state(assignments, county_names, None)
}

/// Same as `analyze_county_splits` but with state code for legal standard lookup.
pub fn analyze_county_splits_with_state(
    assignments: &HashMap<String, usize>,
    county_names: Option<&HashMap<String, String>>,
    state_code: Option<&str>,
) -> CountySplitResult {
    try_analyze_county_splits_with_state(assignments, county_names, state_code)
        .expect("county split assignment GEOIDs are valid")
}

pub fn try_analyze_county_splits_with_state(
    assignments: &HashMap<String, usize>,
    county_names: Option<&HashMap<String, String>>,
    state_code: Option<&str>,
) -> Result<CountySplitResult, SplitError> {
    // Build map: county_fips -> set of districts
    let mut county_to_districts: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut county_to_tracts: HashMap<String, usize> = HashMap::new();

    for (geoid, &district) in assignments {
        if district == 0 {
            return Err(SplitError::InvalidDistrictLabel {
                geoid: geoid.clone(),
            });
        }
        let fips = try_county_fips_from_geoid(geoid)?.to_string();
        county_to_districts
            .entry(fips.clone())
            .or_default()
            .insert(district);
        *county_to_tracts.entry(fips).or_default() += 1;
    }

    let total = county_to_districts.len();

    let mut split_list: Vec<CountySplit> = county_to_districts
        .iter()
        .filter(|(_, dists)| dists.len() > 1)
        .map(|(fips, dists)| {
            let mut d_vec: Vec<usize> = dists.iter().cloned().collect();
            d_vec.sort();
            CountySplit {
                county_name: county_names
                    .and_then(|m| m.get(fips))
                    .map(|s| s.as_str())
                    .or_else(|| lookup_county_name(fips))
                    .map(|s| s.to_string()),
                tract_count: *county_to_tracts.get(fips).unwrap_or(&0),
                split_severity: dists.len(),
                districts_containing: d_vec,
                county_fips: fips.clone(),
            }
        })
        .collect();
    split_list.sort_by(|a, b| a.county_fips.cmp(&b.county_fips));

    let split = split_list.len();
    let preservation_score = if total == 0 {
        1.0
    } else {
        (total - split) as f64 / total as f64
    };

    // Look up legal standard.
    let standard: Option<SplitStandard> = state_code.and_then(|sc| get_split_standard(sc));
    let legal_standard = standard.as_ref().map(|s| s.legal_standard.clone());
    let disclaimer = standard.as_ref().map(|s| s.disclaimer.clone());
    let compliance_assessment = standard.as_ref().map(|s| {
        s.compliance_assessment_template
            .replace("{n}", &split.to_string())
            .replace("{m}", &split.to_string())
    });

    // State-specific entity note (prominently surface VA's 133-entity count so
    // practitioners don't mistakenly expect only 95 counties)
    let entity_note = state_code.and_then(|sc| match sc {
        "VA" => Some(format!(
            "Virginia has 133 total preservation entities: 95 counties + 38 independent cities. \
             Both appear in this split analysis. Total found in this plan: {}.",
            total
        )),
        _ => None,
    });

    Ok(CountySplitResult {
        total,
        split,
        preservation_score,
        split_list,
        legal_standard,
        compliance_assessment,
        disclaimer,
        entity_note,
    })
}

// ---------------------------------------------------------------------------
// Municipal split analysis
// ---------------------------------------------------------------------------

/// Analyze municipal splits.
///
/// `place_to_tracts` — map from place FIPS -> list of tract GEOIDs in that place.
/// `place_names`     — map from place FIPS -> human-readable name.
pub fn analyze_municipal_splits(
    assignments: &HashMap<String, usize>,
    place_to_tracts: &HashMap<String, Vec<String>>,
    place_names: &HashMap<String, String>,
) -> MunicipalSplitResult {
    try_analyze_municipal_splits(assignments, place_to_tracts, place_names)
        .expect("municipal split assignment labels are valid")
}

pub fn try_analyze_municipal_splits(
    assignments: &HashMap<String, usize>,
    place_to_tracts: &HashMap<String, Vec<String>>,
    place_names: &HashMap<String, String>,
) -> Result<MunicipalSplitResult, SplitError> {
    if place_to_tracts.is_empty() {
        return Ok(MunicipalSplitResult {
            available: false,
            total: 0,
            split: 0,
            preservation_score: 1.0,
            split_list: vec![],
        });
    }

    let mut split_list: Vec<MunicipalSplit> = Vec::new();
    let total = place_to_tracts.len();

    for (place_fips, tracts) in place_to_tracts {
        let mut dists_in_place: HashSet<usize> = HashSet::new();
        for t in tracts {
            if let Some(&d) = assignments.get(t) {
                if d == 0 {
                    return Err(SplitError::InvalidDistrictLabel { geoid: t.clone() });
                }
                dists_in_place.insert(d);
            }
        }
        if dists_in_place.len() > 1 {
            let mut d_vec: Vec<usize> = dists_in_place.iter().cloned().collect();
            d_vec.sort();
            split_list.push(MunicipalSplit {
                place_fips: place_fips.clone(),
                place_name: place_names
                    .get(place_fips)
                    .cloned()
                    .unwrap_or_else(|| place_fips.clone()),
                split_severity: dists_in_place.len(),
                districts_containing: d_vec,
            });
        }
    }
    split_list.sort_by(|a, b| a.place_fips.cmp(&b.place_fips));

    let split = split_list.len();
    let preservation_score = if total == 0 {
        1.0
    } else {
        (total - split) as f64 / total as f64
    };

    Ok(MunicipalSplitResult {
        available: true,
        total,
        split,
        preservation_score,
        split_list,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_assignments(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|&(k, v)| (k.to_string(), v)).collect()
    }

    #[test]
    fn test_county_fips_from_geoid_king_county() {
        assert_eq!(county_fips_from_geoid("530330001001"), "53033");
    }

    #[test]
    fn test_county_fips_from_geoid_alabama() {
        assert_eq!(county_fips_from_geoid("010010020100"), "01001");
    }

    #[test]
    fn test_county_split_single_district_no_split() {
        // All King County tracts in district 1 -> no split
        let assignments = make_assignments(&[("53033001", 1), ("53033002", 1)]);
        let result = analyze_county_splits(&assignments, None);
        assert_eq!(result.split, 0);
        assert!((result.preservation_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_county_split_across_two_districts() {
        // King County split across districts 1 and 2
        let assignments = make_assignments(&[("53033000001", 1), ("53033000002", 2)]);
        let result = analyze_county_splits(&assignments, None);
        assert_eq!(result.split, 1);
        assert_eq!(result.split_list[0].county_fips, "53033");
        assert_eq!(result.split_list[0].districts_containing.len(), 2);
        assert_eq!(result.split_list[0].split_severity, 2);
    }

    #[test]
    fn test_county_split_severity_three_districts() {
        // One county split across three districts
        let assignments =
            make_assignments(&[("53033000001", 1), ("53033000002", 2), ("53033000003", 3)]);
        let result = analyze_county_splits(&assignments, None);
        assert_eq!(result.split, 1);
        assert_eq!(result.split_list[0].split_severity, 3);
    }

    #[test]
    fn test_county_preservation_score_formula() {
        // 2 counties total, 1 split -> preservation_score = (2-1)/2 = 0.5
        let assignments = make_assignments(&[
            ("53033000001", 1), // King County
            ("53033000002", 2), // King County split
            ("53035000001", 3), // Kitsap County (whole)
        ]);
        let result = analyze_county_splits(&assignments, None);
        assert_eq!(result.split, 1);
        assert!((result.preservation_score - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_municipal_split_detected() {
        let assignments = make_assignments(&[("53033005000", 7), ("53033005100", 9)]);
        let mut place_to_tracts = HashMap::new();
        place_to_tracts.insert(
            "5363000".to_string(),
            vec!["53033005000".to_string(), "53033005100".to_string()],
        );
        let mut place_names = HashMap::new();
        place_names.insert("5363000".to_string(), "Seattle".to_string());

        let result = analyze_municipal_splits(&assignments, &place_to_tracts, &place_names);
        assert_eq!(result.split, 1);
        assert_eq!(result.split_list[0].place_name, "Seattle");
        assert_eq!(result.split_list[0].split_severity, 2);
    }

    #[test]
    fn test_municipal_data_absent_returns_unavailable() {
        let result = analyze_municipal_splits(&HashMap::new(), &HashMap::new(), &HashMap::new());
        assert!(!result.available);
    }

    // ── Virginia independent cities ───────────────────────────────────────────

    #[test]
    fn test_va_independent_city_tracked_separately_from_county() {
        // Richmond city (51760) and Henrico County (51087) are legally separate in VA.
        // Census GEOIDs encode them with different 5-char FIPS prefixes, so our
        // county_fips_from_geoid() correctly treats them as separate entities.
        let assignments = make_assignments(&[
            ("51760000001", 1), // Richmond city, tract 1, district 1
            ("51760000002", 2), // Richmond city, tract 2, district 2 (city is split)
            ("51087000001", 1), // Henrico County (adjacent), district 1 (not split)
        ]);
        let result = analyze_county_splits(&assignments, None);
        assert_eq!(
            result.total, 2,
            "should see 2 entities: Richmond city + Henrico County"
        );
        assert_eq!(
            result.split, 1,
            "only Richmond city is split — Henrico is intact"
        );
        assert_eq!(
            result.split_list[0].county_fips, "51760",
            "split entity must be Richmond city"
        );
        assert_eq!(result.split_list[0].split_severity, 2);
    }

    #[test]
    fn test_va_state_code_legal_standard_mentions_independent_cities() {
        let assignments = make_assignments(&[("51760000001", 1), ("51760000002", 2)]);
        let result = analyze_county_splits_with_state(&assignments, None, Some("VA"));
        let standard = result
            .legal_standard
            .expect("VA must have a legal standard");
        assert!(
            standard.contains("independent") || standard.contains("city"),
            "VA legal standard must mention independent cities, got: {standard}"
        );
    }

    // ── Nevada urban-rural skew ───────────────────────────────────────────────

    #[test]
    fn test_nv_clark_county_split_detected() {
        // Clark County (32003) contains ~73% of Nevada's population.
        // It necessarily spans multiple legislative districts in any valid plan.
        let assignments = make_assignments(&[
            ("32003000001", 1),
            ("32003000002", 1),
            ("32003000003", 2),
            ("32003000004", 3),
            ("32031000001", 4), // Washoe County (Reno) — whole in district 4
        ]);
        let result = analyze_county_splits_with_state(&assignments, None, Some("NV"));
        assert_eq!(result.total, 2, "Clark + Washoe = 2 county entities");
        assert_eq!(result.split, 1, "only Clark County is split");
        assert_eq!(result.split_list[0].county_fips, "32003");
        assert_eq!(
            result.split_list[0].split_severity, 3,
            "Clark spans districts 1, 2, 3"
        );
        // NV standard should acknowledge Clark County as expected split
        let disclaimer = result.disclaimer.expect("NV must have disclaimer");
        assert!(
            disclaimer.contains("Clark") || disclaimer.contains("expected"),
            "NV disclaimer must note Clark County splits are expected"
        );
    }

    #[test]
    fn test_nv_clark_county_name_resolved_from_lookup() {
        let assignments = make_assignments(&[("32003000001", 1), ("32003000002", 2)]);
        let result = analyze_county_splits(&assignments, None);
        let split = &result.split_list[0];
        // Clark County should resolve its name from the static lookup
        assert_eq!(split.county_name.as_deref(), Some("Clark County"));
    }

    // ── Louisiana parishes ────────────────────────────────────────────────────

    #[test]
    fn test_la_parish_split_detected() {
        // Orleans Parish (22071) should be split across two districts
        let assignments = make_assignments(&[
            ("22071000001", 1),
            ("22071000002", 2),
            ("22087000001", 3), // Saint Bernard Parish — whole
        ]);
        let result = analyze_county_splits_with_state(&assignments, None, Some("LA"));
        assert_eq!(result.split, 1);
        assert_eq!(result.split_list[0].county_fips, "22071");
        // LA standard should use "parish" terminology
        let standard = result.legal_standard.expect("LA must have legal standard");
        assert!(
            standard.contains("parish"),
            "LA standard must use 'parish' term"
        );
    }

    #[test]
    fn test_la_parish_name_from_lookup() {
        let assignments = make_assignments(&[("22071000001", 1), ("22071000002", 2)]);
        let result = analyze_county_splits(&assignments, None);
        let split = &result.split_list[0];
        assert_eq!(split.county_name.as_deref(), Some("Orleans Parish"));
    }

    // ── analyze_county_splits edge cases ────────────────────────────────────

    #[test]
    fn test_county_split_empty_assignments_returns_zero() {
        let result = analyze_county_splits(&HashMap::new(), None);
        assert_eq!(result.total, 0);
        assert_eq!(result.split, 0);
        assert!(
            (result.preservation_score - 1.0).abs() < 1e-9,
            "empty plan must have preservation_score=1.0"
        );
    }

    #[test]
    fn test_county_split_multiple_counties_all_preserved() {
        // Three different counties, each in their own single district → no splits
        let assignments = make_assignments(&[
            ("53033000001", 1), // King County
            ("53035000001", 2), // Kitsap County
            ("53053000001", 3), // Pierce County
        ]);
        let result = analyze_county_splits(&assignments, None);
        assert_eq!(result.split, 0);
        assert_eq!(result.total, 3);
        assert!((result.preservation_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_county_split_tract_count_reported() {
        // King County has 3 tracts but is split
        let assignments =
            make_assignments(&[("53033000001", 1), ("53033000002", 1), ("53033000003", 2)]);
        let result = analyze_county_splits(&assignments, None);
        assert_eq!(result.split, 1);
        assert_eq!(result.split_list[0].tract_count, 3);
    }

    #[test]
    fn test_county_fips_from_geoid_short_geoid_returns_as_is() {
        // GEOID shorter than 5 chars returns itself unchanged
        assert_eq!(county_fips_from_geoid("530"), "530");
    }

    #[test]
    fn test_try_county_fips_from_geoid_rejects_invalid_utf8_boundary() {
        let err = try_county_fips_from_geoid("ééé")
            .expect_err("non-boundary byte 5 must be rejected instead of panicking");
        assert_eq!(
            err,
            SplitError::InvalidCountyFipsBoundary {
                geoid: "ééé".to_string()
            }
        );
    }

    #[test]
    fn test_try_analyze_county_splits_rejects_invalid_geoid_boundary() {
        let assignments = make_assignments(&[("ééé", 1)]);
        let err = try_analyze_county_splits(&assignments, None)
            .expect_err("invalid GEOID boundary must be rejected");
        assert_eq!(
            err,
            SplitError::InvalidCountyFipsBoundary {
                geoid: "ééé".to_string()
            }
        );
    }

    #[test]
    fn test_try_analyze_county_splits_rejects_zero_district_label() {
        let assignments = make_assignments(&[("53033000001", 0)]);
        let err = try_analyze_county_splits(&assignments, None)
            .expect_err("zero district labels must be rejected");
        assert_eq!(
            err,
            SplitError::InvalidDistrictLabel {
                geoid: "53033000001".to_string()
            }
        );
    }

    #[test]
    fn test_municipal_split_no_split_when_place_in_single_district() {
        let assignments = make_assignments(&[("53033005000", 3), ("53033005100", 3)]);
        let mut place_to_tracts = HashMap::new();
        place_to_tracts.insert(
            "5363000".to_string(),
            vec!["53033005000".to_string(), "53033005100".to_string()],
        );
        let place_names = HashMap::new();
        let result = analyze_municipal_splits(&assignments, &place_to_tracts, &place_names);
        assert!(result.available);
        assert_eq!(result.split, 0);
        assert!((result.preservation_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_municipal_split_unknown_place_uses_fips_as_name() {
        // Place FIPS not in place_names → name falls back to FIPS
        let assignments = make_assignments(&[("53033005000", 1), ("53033005100", 2)]);
        let mut place_to_tracts = HashMap::new();
        place_to_tracts.insert(
            "9999999".to_string(),
            vec!["53033005000".to_string(), "53033005100".to_string()],
        );
        let result = analyze_municipal_splits(&assignments, &place_to_tracts, &HashMap::new());
        assert_eq!(result.split, 1);
        assert_eq!(result.split_list[0].place_name, "9999999");
    }

    #[test]
    fn test_try_analyze_municipal_splits_rejects_zero_district_label() {
        let assignments = make_assignments(&[("53033005000", 0)]);
        let mut place_to_tracts = HashMap::new();
        place_to_tracts.insert("5363000".to_string(), vec!["53033005000".to_string()]);
        let err = try_analyze_municipal_splits(&assignments, &place_to_tracts, &HashMap::new())
            .expect_err("zero district labels must be rejected");
        assert_eq!(
            err,
            SplitError::InvalidDistrictLabel {
                geoid: "53033005000".to_string()
            }
        );
    }
}

/// Assemble the native CLI splits report from explicit lookups.
pub fn assemble_splits_report(assignments:&HashMap<String,usize>,state:&str,year:&str,county_names:Option<&HashMap<String,String>>,place_to_tracts:&HashMap<String,Vec<String>>,place_names:&HashMap<String,String>)->Result<serde_json::Value,SplitError>{
 let standard=get_split_standard(state);
 let county_result=try_analyze_county_splits_with_state(assignments,county_names,Some(state))?;
 let municipal_result=try_analyze_municipal_splits(assignments,place_to_tracts,place_names)?;
 let (sub_term,sub_term_plural)=standard.as_ref().map(|s|(s.subdivision_term.clone(),s.subdivision_term_plural.clone())).unwrap_or_else(||("county".into(),"counties".into()));
 Ok(serde_json::json!({"analyzer":"splits","state":state,"year":year,
 "counties":{"subdivision_term":sub_term,"subdivision_term_plural":sub_term_plural,"total":county_result.total,"split":county_result.split,"preservation_score":county_result.preservation_score,"split_list":county_result.split_list,"legal_standard":county_result.legal_standard,"compliance_assessment":county_result.compliance_assessment,"disclaimer":county_result.disclaimer},
 "municipalities":{"available":municipal_result.available,"total":municipal_result.total,"split":municipal_result.split,"preservation_score":municipal_result.preservation_score,"split_list":municipal_result.split_list}}))
}

#[cfg(test)]
mod explicit_county_tests {
    use super::*;
    fn assignments() -> HashMap<String,usize> { [("ward-A",1),("ward-B",2),("numeric-looking-A",1),("numeric-looking-B",2)].into_iter().map(|(u,d)|(u.into(),d)).collect() }
    fn identities() -> HashMap<String,String> { [("ward-A","county-α"),("ward-B","county-α"),("numeric-looking-A","44001"),("numeric-looking-B","44001")].into_iter().map(|(u,c)|(u.into(),c.into())).collect() }
    #[test]
    fn opaque_ids_are_not_fips_or_policy_findings() {
        let report=assemble_explicit_splits_report(&assignments(),&identities(),"RI","2024",None,&HashMap::new(),&HashMap::new()).unwrap();
        assert_eq!(report["counties"]["total"],2);assert_eq!(report["counties"]["split"],2);
        assert_eq!(report["counties"]["split_list"][0]["county_id"],"44001");
        assert!(report["counties"]["split_list"][0]["county_name"].is_null());
        assert_eq!(report["counties"]["split_list"][1]["unit_count"],2);
        assert!(report["counties"]["legal_standard"].is_null());assert!(report["counties"]["compliance_assessment"].is_null());
        assert_eq!(report["municipalities"]["available"],false);
    }
    #[test]
    fn optional_names_and_municipal_membership_are_explicit() {
        let names=HashMap::from([("county-α".into(),"Supplied county name".into())]);
        let places=HashMap::from([("place-1".into(),vec!["ward-A".into(),"ward-B".into()])]);
        let report=assemble_explicit_splits_report(&assignments(),&identities(),"RI","2024",Some(&names),&places,&HashMap::new()).unwrap();
        assert_eq!(report["counties"]["split_list"][1]["county_name"],"Supplied county name");
        assert_eq!(report["municipalities"]["split_list"][0]["municipality_id"],"place-1");
        assert!(report["municipalities"]["split_list"][0].get("place_fips").is_none());
    }
    #[test]
    fn incomplete_extra_and_invalid_identities_refuse_counts() {
        let mut ids=identities();ids.remove("ward-A");
        assert!(matches!(assemble_explicit_splits_report(&assignments(),&ids,"ZZ","2024",None,&HashMap::new(),&HashMap::new()),Err(SplitError::MissingCountyIdentity{..})));
        let mut ids=identities();ids.insert("unassigned".into(),"extra".into());
        assert!(matches!(assemble_explicit_splits_report(&assignments(),&ids,"ZZ","2024",None,&HashMap::new(),&HashMap::new()),Err(SplitError::UnexpectedCountyIdentity{..})));
        for invalid in [" ".to_string(),"bad\ncounty".into(),"α".repeat(101)] {
            let mut ids=identities();ids.insert("ward-A".into(),invalid);
            assert!(matches!(assemble_explicit_splits_report(&assignments(),&ids,"ZZ","2024",None,&HashMap::new(),&HashMap::new()),Err(SplitError::InvalidExplicitIdentity{..})));
        }
        let mut zero=assignments();zero.insert("ward-A".into(),0);assert!(matches!(assemble_explicit_splits_report(&zero,&identities(),"ZZ","2024",None,&HashMap::new(),&HashMap::new()),Err(SplitError::InvalidDistrictLabel{..})));
    }
    #[test]
    fn ordered_context_requires_complete_unique_alignment() {
        let units=vec!["A".into(),"B".into()];
        assert!(matches!(explicit_counties_from_ordered_units(&units,&[Some("county".into())]),Err(SplitError::CountyIdentityLength{..})));
        assert!(matches!(explicit_counties_from_ordered_units(&units,&[Some("county".into()),None]),Err(SplitError::MissingCountyIdentity{..})));
        assert!(matches!(explicit_counties_from_ordered_units(&["A".into(),"A".into()],&[Some("c".into()),Some("c".into())]),Err(SplitError::DuplicateUnitIdentity{..})));
        assert!(matches!(explicit_counties_from_ordered_units(&[],&[]),Err(SplitError::NoExplicitUnits)));
        assert_eq!(explicit_counties_from_ordered_units(&units,&[Some("county".into()),Some("other".into())]).unwrap()["B"],"other");
    }
}
