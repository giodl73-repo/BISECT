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
/// Steps:
///   1. Find TIGER shapefile at data/{year}/tiger/tracts/{fips}/tl_{year}_{fips}_tract.shp
///   2. Decode each tract's WKB geometry
///   3. Join to district assignments via GEOID
///   4. Dissolve (union) per district
///
/// Returns HashMap<district_id, MultiPolygon> for all districts present in assignments.
pub fn load_district_geometries(
    state_name: &str,
    state_code: &str,
    year: &str,
    assignments: &HashMap<String, usize>,
    data_root: &Path,
) -> anyhow::Result<HashMap<usize, MultiPolygon<f64>>> {
    let fips = state_code_to_fips(state_code)
        .ok_or_else(|| anyhow::anyhow!("Unknown state code: {state_code}"))?;

    let tiger_path = data_root
        .join(year)
        .join("tiger")
        .join("tracts")
        .join(fips)
        .join(format!("tl_{year}_{fips}_tract.shp"));

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

    // Map tract index → district via GEOID
    let tract_assignments: Vec<usize> = tract_records.iter()
        .map(|tr| assignments.get(&tr.geoid).copied().unwrap_or(0))
        .collect();

    let num_districts = assignments.values().copied().max().unwrap_or(1);

    // Log unmatched tracts at debug level
    let unmatched = tract_records.iter()
        .filter(|tr| !assignments.contains_key(&tr.geoid))
        .count();
    if unmatched > 0 {
        eprintln!("WARNING: {unmatched}/{} tracts had no assignment match for {state_name}",
            tract_records.len());
    }

    Ok(group_dissolve(&geoms, &tract_assignments, num_districts))
}

/// Map state FIPS code from 2-letter postal code.
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
        // 4 polygons (as WKB-encoded geometries) → 2 districts
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
