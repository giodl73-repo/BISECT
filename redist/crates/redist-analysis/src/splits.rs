/// County and municipal split analysis.
///
/// County splits are detected by parsing the 5-char county FIPS prefix from each
/// 11-char Census tract GEOID. Municipal splits require a place-to-tract lookup
/// table (from `redist fetch --type geography`).
use std::collections::{HashMap, HashSet};

use crate::split_standards::{get_split_standard, SplitStandard};

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

// ---------------------------------------------------------------------------
// GEOID helpers
// ---------------------------------------------------------------------------

/// Parse county FIPS from an 11-char (or longer) Census tract GEOID.
/// "530330001001" -> "53033" (state 53, county 033)
pub fn county_fips_from_geoid(geoid: &str) -> &str {
    if geoid.len() >= 5 {
        &geoid[..5]
    } else {
        geoid
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
    analyze_county_splits_with_state(assignments, county_names, None)
}

/// Same as `analyze_county_splits` but with state code for legal standard lookup.
pub fn analyze_county_splits_with_state(
    assignments: &HashMap<String, usize>,
    county_names: Option<&HashMap<String, String>>,
    state_code: Option<&str>,
) -> CountySplitResult {
    // Build map: county_fips -> set of districts
    let mut county_to_districts: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut county_to_tracts: HashMap<String, usize> = HashMap::new();

    for (geoid, &district) in assignments {
        let fips = county_fips_from_geoid(geoid).to_string();
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
                    .cloned(),
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

    CountySplitResult {
        total,
        split,
        preservation_score,
        split_list,
        legal_standard,
        compliance_assessment,
        disclaimer,
    }
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
    if place_to_tracts.is_empty() {
        return MunicipalSplitResult {
            available: false,
            total: 0,
            split: 0,
            preservation_score: 1.0,
            split_list: vec![],
        };
    }

    let mut split_list: Vec<MunicipalSplit> = Vec::new();
    let total = place_to_tracts.len();

    for (place_fips, tracts) in place_to_tracts {
        let mut dists_in_place: HashSet<usize> = HashSet::new();
        for t in tracts {
            if let Some(&d) = assignments.get(t) {
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

    MunicipalSplitResult {
        available: true,
        total,
        split,
        preservation_score,
        split_list,
    }
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
        let assignments = make_assignments(&[
            ("53033000001", 1),
            ("53033000002", 2),
        ]);
        let result = analyze_county_splits(&assignments, None);
        assert_eq!(result.split, 1);
        assert_eq!(result.split_list[0].county_fips, "53033");
        assert_eq!(result.split_list[0].districts_containing.len(), 2);
        assert_eq!(result.split_list[0].split_severity, 2);
    }

    #[test]
    fn test_county_split_severity_three_districts() {
        // One county split across three districts
        let assignments = make_assignments(&[
            ("53033000001", 1),
            ("53033000002", 2),
            ("53033000003", 3),
        ]);
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
        let assignments = make_assignments(&[
            ("53033005000", 7),
            ("53033005100", 9),
        ]);
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
        let result =
            analyze_municipal_splits(&HashMap::new(), &HashMap::new(), &HashMap::new());
        assert!(!result.available);
    }
}
