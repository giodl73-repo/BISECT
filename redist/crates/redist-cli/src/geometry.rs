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
///   3. Load TIGER shapefile for tract WKB geometries
///   4. Join by GEOID, group_dissolve per district
///
/// Returns HashMap<district_id, MultiPolygon> for all districts.
pub fn load_district_geometries(
    state_name: &str,
    state_code: &str,
    year: &str,
    version: &str,
    assignments: &HashMap<String, usize>,
    data_root: &Path,
) -> anyhow::Result<HashMap<usize, MultiPolygon<f64>>> {
    let fips = state_code_to_fips(state_code)
        .ok_or_else(|| anyhow::anyhow!("Unknown state code: {state_code}"))?;

    // Determine if assignments use tract indices or GEOIDs
    let uses_index_keys = assignments.keys()
        .next()
        .map(|k| k.len() < 11)
        .unwrap_or(false);

    // Build GEOID → district_id map
    let geoid_to_district: HashMap<String, usize> = if uses_index_keys {
        // Geoid files use state_code_lower prefix: vt_adjacency_2020_geoids.json
        let state_code_lower = state_code.to_lowercase();
        let geoid_file = format!("{state_code_lower}_adjacency_{year}_geoids.json");
        // Geoid files live in the shared data directory, not the year-specific states dir
        let geoid_candidates = [
            PathBuf::from("outputs").join(version).join("data").join(year).join("adjacency").join(&geoid_file),
            PathBuf::from("outputs/V3/data").join(year).join("adjacency").join(&geoid_file),
            PathBuf::from("outputs/V4/data").join(year).join("adjacency").join(&geoid_file),
        ];
        let geoid_path = geoid_candidates.iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow::anyhow!(
                "Geoid mapping not found for {state_name}. \
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

    // TIGER files: data/{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.shp
    let stem = format!("tl_{year}_{fips}_tract");
    let tiger_path = data_root
        .join(year)
        .join("tiger")
        .join("tracts")
        .join(&stem)
        .join(format!("{stem}.shp"));

    if !tiger_path.exists() {
        anyhow::bail!(
            "TIGER shapefile not found: {}. Run: redist fetch --year {year} --states {state_code}",
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
        eprintln!("WARNING: {unmatched}/{} tracts had no assignment match for {state_name}",
            tract_records.len());
    }

    Ok(group_dissolve(&geoms, &tract_assignments, num_districts))
}

/// Map 2-letter postal code → Census FIPS code (zero-padded to 2 digits).
pub fn state_code_to_fips(code: &str) -> Option<&'static str> {
    match code {
        "AL" => Some("01"), "AK" => Some("02"), "AZ" => Some("04"),
        "AR" => Some("05"), "CA" => Some("06"), "CO" => Some("08"),
        "CT" => Some("09"), "DE" => Some("10"), "FL" => Some("12"),
        "GA" => Some("13"), "HI" => Some("15"), "ID" => Some("16"),
        "IL" => Some("17"), "IN" => Some("18"), "IA" => Some("19"),
        "KS" => Some("20"), "KY" => Some("21"), "LA" => Some("22"),
        "ME" => Some("23"), "MD" => Some("24"), "MA" => Some("25"),
        "MI" => Some("26"), "MN" => Some("27"), "MS" => Some("28"),
        "MO" => Some("29"), "MT" => Some("30"), "NE" => Some("31"),
        "NV" => Some("32"), "NH" => Some("33"), "NJ" => Some("34"),
        "NM" => Some("35"), "NY" => Some("36"), "NC" => Some("37"),
        "ND" => Some("38"), "OH" => Some("39"), "OK" => Some("40"),
        "OR" => Some("41"), "PA" => Some("42"), "RI" => Some("44"),
        "SC" => Some("45"), "SD" => Some("46"), "TN" => Some("47"),
        "TX" => Some("48"), "UT" => Some("49"), "VT" => Some("50"),
        "VA" => Some("51"), "WA" => Some("53"), "WV" => Some("54"),
        "WI" => Some("55"), "WY" => Some("56"), "DC" => Some("11"),
        _ => None,
    }
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
}
