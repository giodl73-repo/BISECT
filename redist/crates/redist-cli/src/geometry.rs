/// Shared district geometry helper.
///
/// Both map_cmd.rs (rendering) and the compactness analyzer need to:
///   1. Load TIGER tract shapefiles for a state
///   2. Decode WKB bytes to geo_types::Geometry
///   3. Group by district assignment via group_dissolve
///
/// This module is the single place that does all three steps.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use geo_types::{Geometry, MultiPolygon};
use redist_map::{wkb_to_geometry, group_dissolve};
use redist_data::tiger::read_tiger_tracts;
use redist_core::state_code_to_fips;

/// Resolve raw assignments (may be index-keyed or GEOID-keyed) to GEOID-keyed form.
/// If keys are shorter than 11 chars, treats them as adjacency indices and resolves
/// via the _geoids.json file in the adjacency store.
pub fn resolve_to_geoid_assignments(
    raw: HashMap<String, usize>,
    output_root: &Path,
    state_code: &str,
    year: &str,
) -> HashMap<String, usize> {
    let uses_index = raw.keys().next().map(|k| k.len() < 11).unwrap_or(false);
    if !uses_index {
        return raw;
    }
    let state_code_lower = state_code.to_lowercase();
    let geoid_file = format!("{state_code_lower}_adjacency_{year}_geoids.json");
    let geoid_candidates = [
        output_root.join("data").join(year).join("adjacency").join(&geoid_file),
        PathBuf::from("outputs/V3/data").join(year).join("adjacency").join(&geoid_file),
        PathBuf::from("outputs/V4/data").join(year).join("adjacency").join(&geoid_file),
    ];
    let geoid_path = geoid_candidates.iter().find(|p| p.exists());
    if let Some(geoid_path) = geoid_path {
        let raw_geoids: HashMap<String, String> =
            match std::fs::read_to_string(geoid_path)
                .and_then(|s| serde_json::from_str(&s)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
            {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("WARNING: could not read geoid mapping {}: {e}", geoid_path.display());
                    return raw;
                }
            };
        raw_geoids.into_iter()
            .filter_map(|(idx, geoid)| raw.get(&idx).map(|&d| (geoid, d)))
            .collect()
    } else {
        eprintln!("WARNING: geoid mapping not found for {state_code_lower} {year}; \
            run: python scripts/data/generate_adj_bin.py --year {year} --states {state_code}");
        raw
    }
}

/// Load dissolved district geometries for a state.
///
/// The Rust CLI writes `final_assignments.json` with **tract-index** keys (not GEOIDs):
///   `{"174": 1, "22": 1, ...}` (index → district_id, 1-based)
///
/// The adjacency `_geoids.json` maps index → GEOID:
///   `{"0": "50005957100", "1": "50005957400", ...}`
///
/// Steps:
///   1. Detect key format (index if len < 11, GEOID otherwise)
///   2. If index-keyed: load `_geoids.json` to build GEOID → district map
///   3. Load TIGER shapefile for the correct geographic resolution
///   4. Join by GEOID, group_dissolve per district
///
/// `resolution` controls which TIGER shapefile is loaded:
///   - "tract" (default): `tl_{year}_{fips}_tract.shp`
///   - "block_group" / "block-group": `tl_{year}_{fips}_bg.shp`
///   - "block": `tl_{year}_{fips}_tabblock20.shp` (2020) — not yet fully supported
///
/// Returns HashMap<district_id, MultiPolygon> for all districts.
pub fn load_district_geometries(
    state_name: &str,
    state_code: &str,
    year: &str,
    version: &str,
    assignments: &HashMap<String, usize>,
    data_root: &Path,
    resolution: &str,
) -> anyhow::Result<HashMap<usize, MultiPolygon<f64>>> {
    let fips = state_code_to_fips(state_code)
        .ok_or_else(|| anyhow::anyhow!("Unknown state code: {state_code}"))?;

    // Determine if assignments use tract indices or GEOIDs
    let uses_index_keys = assignments.keys()
        .next()
        .map(|k| k.len() < 11)
        .unwrap_or(false);

    // Build GEOID → district_id map
    // For block groups the geoid file uses the bg_ prefix convention
    let geoid_to_district: HashMap<String, usize> = if uses_index_keys {
        let state_code_lower = state_code.to_lowercase();
        // Geoid filename depends on resolution
        let geoid_file = match resolution {
            "block_group" | "block-group" => {
                format!("{state_code_lower}_bg_adjacency_{year}_geoids.json")
            }
            _ => format!("{state_code_lower}_adjacency_{year}_geoids.json"),
        };
        // Geoid files live in the shared data directory, not the year-specific states dir
        let geoid_candidates = [
            PathBuf::from("outputs").join(version).join("data").join(year).join("adjacency").join(&geoid_file),
            PathBuf::from("outputs/V3/data").join(year).join("adjacency").join(&geoid_file),
            PathBuf::from("outputs/V4/data").join(year).join("adjacency").join(&geoid_file),
        ];
        let geoid_path = geoid_candidates.iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow::anyhow!(
                "Geoid mapping not found for {state_name} ({resolution} resolution). \
                 Run: python scripts/data/generate_adj_bin.py --year {year} --states {state_code}"
            ))?;

        let raw: HashMap<String, String> = serde_json::from_str(
            &std::fs::read_to_string(geoid_path)?
        )?;
        // raw: {"0": "50005957100", ...} — join with assignments on index
        raw.into_iter()
            .filter_map(|(idx, geoid)| {
                assignments.get(&idx).map(|&dist| (geoid, dist))
            })
            .collect()
    } else {
        // Assignments already keyed by GEOID
        assignments.iter().map(|(k, &v)| (k.clone(), v)).collect()
    };

    // TIGER files: choose stem based on resolution
    //   tract:       data/{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.shp
    //   block_group: data/{year}/tiger/tracts/tl_{year}_{fips}_bg/tl_{year}_{fips}_bg.shp
    let (stem, subdir) = match resolution {
        "block_group" | "block-group" => (
            format!("tl_{year}_{fips}_bg"),
            "tracts",  // block group shapefiles live alongside tracts
        ),
        _ => (
            format!("tl_{year}_{fips}_tract"),
            "tracts",
        ),
    };
    let tiger_path = data_root
        .join(year)
        .join("tiger")
        .join(subdir)
        .join(&stem)
        .join(format!("{stem}.shp"));

    if !tiger_path.exists() {
        anyhow::bail!(
            "{resolution} TIGER shapefile not found: {}. \
             Run: redist fetch --year {year} --states {state_code} (for tracts), \
             or download block group TIGER files manually.",
            tiger_path.display()
        );
    }

    let tract_records = read_tiger_tracts(&tiger_path)
        .map_err(|e| anyhow::anyhow!("Failed to read TIGER shapefile: {e}"))?;

    let geoms: Vec<Geometry<f64>> = tract_records.iter()
        .map(|tr| wkb_to_geometry(&tr.geometry_wkb))
        .collect::<anyhow::Result<Vec<_>>>()?;

    let tract_assignments: Vec<usize> = tract_records.iter()
        .map(|tr| geoid_to_district.get(&tr.geoid).copied().unwrap_or(0))
        .collect();

    let num_districts = assignments.values().copied().max().unwrap_or(1);

    let unmatched = tract_records.iter()
        .filter(|tr| !geoid_to_district.contains_key(&tr.geoid))
        .count();
    if unmatched > 0 {
        eprintln!("WARNING: {unmatched}/{} {resolution} units had no assignment match for {state_name}",
            tract_records.len());
    }

    Ok(group_dissolve(&geoms, &tract_assignments, num_districts))
}

/// Reverse-lookup: state directory name (e.g. "rhode_island") → 2-letter code (e.g. "RI").
pub fn state_name_to_code(name: &str) -> Option<&'static str> {
    match name {
        "alabama" => Some("AL"), "alaska" => Some("AK"), "arizona" => Some("AZ"),
        "arkansas" => Some("AR"), "california" => Some("CA"), "colorado" => Some("CO"),
        "connecticut" => Some("CT"), "delaware" => Some("DE"), "florida" => Some("FL"),
        "georgia" => Some("GA"), "hawaii" => Some("HI"), "idaho" => Some("ID"),
        "illinois" => Some("IL"), "indiana" => Some("IN"), "iowa" => Some("IA"),
        "kansas" => Some("KS"), "kentucky" => Some("KY"), "louisiana" => Some("LA"),
        "maine" => Some("ME"), "maryland" => Some("MD"), "massachusetts" => Some("MA"),
        "michigan" => Some("MI"), "minnesota" => Some("MN"), "mississippi" => Some("MS"),
        "missouri" => Some("MO"), "montana" => Some("MT"), "nebraska" => Some("NE"),
        "nevada" => Some("NV"), "new_hampshire" => Some("NH"), "new_jersey" => Some("NJ"),
        "new_mexico" => Some("NM"), "new_york" => Some("NY"), "north_carolina" => Some("NC"),
        "north_dakota" => Some("ND"), "ohio" => Some("OH"), "oklahoma" => Some("OK"),
        "oregon" => Some("OR"), "pennsylvania" => Some("PA"), "rhode_island" => Some("RI"),
        "south_carolina" => Some("SC"), "south_dakota" => Some("SD"), "tennessee" => Some("TN"),
        "texas" => Some("TX"), "utah" => Some("UT"), "vermont" => Some("VT"),
        "virginia" => Some("VA"), "washington" => Some("WA"), "west_virginia" => Some("WV"),
        "wisconsin" => Some("WI"), "wyoming" => Some("WY"), "district_of_columbia" => Some("DC"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::polygon;
    use geo::Area;

    #[test]
    fn test_fips_lookup_vt() {
        assert_eq!(state_code_to_fips("VT"), Some("50"));
    }

    #[test]
    fn test_fips_lookup_ca() {
        assert_eq!(state_code_to_fips("CA"), Some("06"));
    }

    #[test]
    fn test_fips_lookup_unknown() {
        assert_eq!(state_code_to_fips("ZZ"), None);
    }

    #[test]
    fn test_group_dissolve_two_districts() {
        use geozero::{CoordDimensions, ToWkb};

        let p1: Geometry<f64> = Geometry::Polygon(
            polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.),(x:0.,y:0.)]
        );
        let p2: Geometry<f64> = Geometry::Polygon(
            polygon![(x:1.,y:0.),(x:2.,y:0.),(x:2.,y:1.),(x:1.,y:1.),(x:1.,y:0.)]
        );
        let p3: Geometry<f64> = Geometry::Polygon(
            polygon![(x:0.,y:1.),(x:1.,y:1.),(x:1.,y:2.),(x:0.,y:2.),(x:0.,y:1.)]
        );
        let p4: Geometry<f64> = Geometry::Polygon(
            polygon![(x:1.,y:1.),(x:2.,y:1.),(x:2.,y:2.),(x:1.,y:2.),(x:1.,y:1.)]
        );

        let geoms = vec![p1, p2, p3, p4];
        let assignments = vec![1usize, 1, 2, 2];

        let districts = group_dissolve(&geoms, &assignments, 2);
        assert_eq!(districts.len(), 2);
        assert!((districts[&1].unsigned_area() - 2.0).abs() < 1e-9);
        assert!((districts[&2].unsigned_area() - 2.0).abs() < 1e-9);
    }

    // ── state_name_to_code ────────────────────────────────────────────────────

    #[test]
    fn state_name_to_code_vermont() {
        assert_eq!(state_name_to_code("vermont"), Some("VT"));
    }

    #[test]
    fn state_name_to_code_california() {
        assert_eq!(state_name_to_code("california"), Some("CA"));
    }

    #[test]
    fn state_name_to_code_new_york() {
        assert_eq!(state_name_to_code("new_york"), Some("NY"));
    }

    #[test]
    fn state_name_to_code_new_hampshire() {
        assert_eq!(state_name_to_code("new_hampshire"), Some("NH"));
    }

    #[test]
    fn state_name_to_code_north_carolina() {
        assert_eq!(state_name_to_code("north_carolina"), Some("NC"));
    }

    #[test]
    fn state_name_to_code_rhode_island() {
        assert_eq!(state_name_to_code("rhode_island"), Some("RI"));
    }

    #[test]
    fn state_name_to_code_district_of_columbia() {
        assert_eq!(state_name_to_code("district_of_columbia"), Some("DC"));
    }

    #[test]
    fn state_name_to_code_unknown_returns_none() {
        assert_eq!(state_name_to_code("unknown_state"), None);
    }

    #[test]
    fn state_name_to_code_empty_returns_none() {
        assert_eq!(state_name_to_code(""), None);
    }

    #[test]
    fn state_name_to_code_case_sensitive_uppercase_returns_none() {
        // The function only matches lowercase_underscore form.
        assert_eq!(state_name_to_code("California"), None);
    }

    #[test]
    fn state_name_to_code_texas() {
        assert_eq!(state_name_to_code("texas"), Some("TX"));
    }

    #[test]
    fn state_name_to_code_alaska() {
        assert_eq!(state_name_to_code("alaska"), Some("AK"));
    }

    #[test]
    fn state_name_to_code_hawaii() {
        assert_eq!(state_name_to_code("hawaii"), Some("HI"));
    }

    #[test]
    fn state_name_to_code_all_50_have_some() {
        // Quick smoke test: none of the 50 canonical lowercase names returns None.
        let names = [
            "alabama", "alaska", "arizona", "arkansas", "california", "colorado",
            "connecticut", "delaware", "florida", "georgia", "hawaii", "idaho",
            "illinois", "indiana", "iowa", "kansas", "kentucky", "louisiana",
            "maine", "maryland", "massachusetts", "michigan", "minnesota",
            "mississippi", "missouri", "montana", "nebraska", "nevada",
            "new_hampshire", "new_jersey", "new_mexico", "new_york",
            "north_carolina", "north_dakota", "ohio", "oklahoma", "oregon",
            "pennsylvania", "rhode_island", "south_carolina", "south_dakota",
            "tennessee", "texas", "utah", "vermont", "virginia", "washington",
            "west_virginia", "wisconsin", "wyoming",
        ];
        for name in &names {
            assert!(state_name_to_code(name).is_some(), "expected Some for: {name}");
        }
    }

    // ── resolve_to_geoid_assignments ─────────────────────────────────────────

    #[test]
    fn resolve_to_geoid_assignments_long_keys_passthrough() {
        // Keys already >= 11 chars → treated as GEOIDs, returned unchanged.
        let mut raw = HashMap::new();
        raw.insert("50005957100".to_string(), 1usize);
        raw.insert("50005957200".to_string(), 2usize);
        let result = resolve_to_geoid_assignments(
            raw.clone(),
            std::path::Path::new("/nonexistent"),
            "VT",
            "2020",
        );
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("50005957100"), Some(&1));
        assert_eq!(result.get("50005957200"), Some(&2));
    }

    #[test]
    fn resolve_to_geoid_assignments_empty_map_passthrough() {
        let raw: HashMap<String, usize> = HashMap::new();
        let result = resolve_to_geoid_assignments(
            raw,
            std::path::Path::new("/nonexistent"),
            "VT",
            "2020",
        );
        assert!(result.is_empty(), "empty input must produce empty output");
    }

    #[test]
    fn resolve_to_geoid_assignments_short_keys_without_geoid_file_returns_original() {
        // Keys < 11 chars look like indices, but geoid file doesn't exist →
        // falls back to returning the raw map unchanged (with a warning).
        let mut raw = HashMap::new();
        raw.insert("0".to_string(), 1usize);
        raw.insert("1".to_string(), 2usize);
        let result = resolve_to_geoid_assignments(
            raw.clone(),
            std::path::Path::new("/nonexistent"),
            "VT",
            "2020",
        );
        // Returns the original raw map since geoid file doesn't exist.
        assert_eq!(result.len(), 2);
    }

    // ── group_dissolve single district ───────────────────────────────────────

    #[test]
    fn group_dissolve_single_district_four_unit_squares() {
        // Four unit squares all in district 1 → combined area = 4.0
        let make_sq = |x0: f64, y0: f64| -> Geometry<f64> {
            Geometry::Polygon(polygon![
                (x: x0,     y: y0),
                (x: x0+1.0, y: y0),
                (x: x0+1.0, y: y0+1.0),
                (x: x0,     y: y0+1.0),
                (x: x0,     y: y0)
            ])
        };
        let geoms = vec![make_sq(0.0,0.0), make_sq(1.0,0.0), make_sq(2.0,0.0), make_sq(3.0,0.0)];
        let assignments = vec![1usize, 1, 1, 1];
        let districts = group_dissolve(&geoms, &assignments, 1);
        assert_eq!(districts.len(), 1);
        assert!((districts[&1].unsigned_area() - 4.0).abs() < 1e-9,
            "four unit squares in one district should have area 4.0");
    }

    #[test]
    fn group_dissolve_three_districts_equal_area() {
        // Three unit squares in three separate districts → each area = 1.0
        let make_sq = |x0: f64| -> Geometry<f64> {
            Geometry::Polygon(polygon![
                (x: x0,     y: 0.0),
                (x: x0+1.0, y: 0.0),
                (x: x0+1.0, y: 1.0),
                (x: x0,     y: 1.0),
                (x: x0,     y: 0.0)
            ])
        };
        let geoms = vec![make_sq(0.0), make_sq(1.0), make_sq(2.0)];
        let assignments = vec![1usize, 2, 3];
        let districts = group_dissolve(&geoms, &assignments, 3);
        assert_eq!(districts.len(), 3);
        for d in 1..=3 {
            assert!((districts[&d].unsigned_area() - 1.0).abs() < 1e-9,
                "district {d} should have area 1.0");
        }
    }
}
