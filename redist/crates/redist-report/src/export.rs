/// export.rs — GeoJSON RFC 7946, GerryChain v2.3, CSV export.
///
/// Spec 6 / Scenario 3:
/// - GeoJSON coordinates in [lon, lat] order (RFC 7946)
/// - Each feature has "district_id" property
/// - GerryChain v2.3 uses "assignment" (singular) field
/// - CSV has "geoid" and "district" columns
use std::collections::HashMap;
use serde_json::{json, Map, Value};

use crate::rplan::RplanFile;

/// Export district polygons as RFC 7946 GeoJSON FeatureCollection.
/// Each Feature has "district_id" and (if available) "population" in properties.
/// Geometry comes from rplan.geometry (GeoJSON FeatureCollection) if present.
/// If no geometry: produces centroid-only Point features (placeholder).
pub fn export_geojson(plan: &RplanFile) -> anyhow::Result<String> {
    // Build a map of district_id → list of assignments (GEOIDs) for counting
    let mut district_geoids: HashMap<usize, Vec<&str>> = HashMap::new();
    for (geoid, &dist) in &plan.assignments {
        district_geoids.entry(dist).or_default().push(geoid.as_str());
    }

    let features: Vec<Value> = if let Some(geom) = &plan.geometry {
        // Use existing geometry from RPLAN (already dissolved district polygons)
        if geom["type"].as_str() == Some("FeatureCollection") {
            let features_arr = geom["features"].as_array();
            if let Some(arr) = features_arr {
                arr.iter().cloned().collect()
            } else {
                build_empty_district_features(&district_geoids)
            }
        } else {
            build_empty_district_features(&district_geoids)
        }
    } else {
        // No geometry: produce minimal features with district_id and tract count
        build_empty_district_features(&district_geoids)
    };

    let fc = json!({
        "type": "FeatureCollection",
        "features": features,
    });

    Ok(serde_json::to_string_pretty(&fc)?)
}

/// Build placeholder features (no real geometry) with district_id property.
fn build_empty_district_features(
    district_geoids: &HashMap<usize, Vec<&str>>,
) -> Vec<Value> {
    let mut districts: Vec<usize> = district_geoids.keys().copied().collect();
    districts.sort_unstable();

    districts
        .iter()
        .map(|&dist_id| {
            let tract_count = district_geoids.get(&dist_id).map(|v| v.len()).unwrap_or(0);
            json!({
                "type": "Feature",
                "geometry": null,
                "properties": {
                    "district_id": dist_id,
                    "tract_count": tract_count,
                }
            })
        })
        .collect()
}

/// Export in GerryChain v2.3 format.
/// Schema: `{"assignment": {"GEOID": district_int, ...}, "gerrychain_version_target": "2.3"}`
/// Field name is "assignment" (singular) — GerryChain v2.3 convention.
/// RPLAN uses "assignments" (plural) — these must be mapped.
pub fn export_gerrychain_v23(plan: &RplanFile) -> anyhow::Result<String> {
    let assignment: Map<String, Value> = plan
        .assignments
        .iter()
        .map(|(k, v)| (k.clone(), json!(v)))
        .collect();
    let doc = json!({
        "assignment": assignment,
        "gerrychain_version_target": "2.3",
        "label": plan.metadata.label,
        "state": plan.metadata.state_code,
        "year": plan.metadata.year,
        "num_districts": plan.metadata.num_districts,
    });
    Ok(serde_json::to_string_pretty(&doc)?)
}

/// Import GerryChain v2.3 JSON back to an RplanFile assignments map.
/// GerryChain uses "assignment" (singular); we produce "assignments" (plural).
pub fn import_gerrychain_to_assignments(
    gc_json: &str,
) -> anyhow::Result<HashMap<String, usize>> {
    let v: Value = serde_json::from_str(gc_json)?;
    let assignment = v["assignment"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("GerryChain JSON missing 'assignment' field"))?;
    let assignments: HashMap<String, usize> = assignment
        .iter()
        .map(|(k, v)| {
            let dist = v
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("district value must be integer"))
                as Result<u64, _>;
            dist.map(|d| (k.clone(), d as usize))
        })
        .collect::<anyhow::Result<_>>()?;
    Ok(assignments)
}

/// Export tract assignments as GEOID,district CSV (sorted by GEOID).
pub fn export_tracts_csv(plan: &RplanFile) -> anyhow::Result<String> {
    let mut wtr = csv::Writer::from_writer(Vec::new());
    wtr.write_record(["geoid", "district"])?;
    let mut rows: Vec<(&String, &usize)> = plan.assignments.iter().collect();
    rows.sort_by_key(|(k, _)| k.as_str());
    for (geoid, district) in rows {
        wtr.write_record([geoid.as_str(), &district.to_string()])?;
    }
    let bytes = wtr.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

// ---------------------------------------------------------------------------
// Tests — Task 3
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rplan::{RplanFile, RplanMetadata};

    fn fixture_wa_congressional_plan() -> RplanFile {
        let mut assignments = HashMap::new();
        // 10-district WA plan with dummy GEOIDs (11-char, all digits)
        for i in 0..10usize {
            for j in 0..5usize {
                let geoid = format!("{:05}{:06}", i + 1, j);
                assignments.insert(geoid, i + 1);
            }
        }
        RplanFile {
            rplan_version: "0.1".into(),
            metadata: RplanMetadata {
                label: "wa_congressional_2020".into(),
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

    fn fixture_wa_congressional_plan_with_geometry() -> RplanFile {
        let mut plan = fixture_wa_congressional_plan();
        // Add minimal GeoJSON geometry: 10 features with district_id + coordinates
        let features: Vec<Value> = (1..=10usize)
            .map(|d| {
                json!({
                    "type": "Feature",
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[
                            [-120.0 + d as f64 * 0.5, 47.0],
                            [-120.0 + d as f64 * 0.5 + 0.4, 47.0],
                            [-120.0 + d as f64 * 0.5 + 0.4, 47.5],
                            [-120.0 + d as f64 * 0.5, 47.5],
                            [-120.0 + d as f64 * 0.5, 47.0],
                        ]]
                    },
                    "properties": {
                        "district_id": d,
                    }
                })
            })
            .collect();
        plan.geometry = Some(json!({
            "type": "FeatureCollection",
            "features": features,
        }));
        plan
    }

    #[test]
    fn test_geojson_export_has_correct_district_count() {
        let plan = fixture_wa_congressional_plan();
        let geojson_str = export_geojson(&plan).unwrap();
        let v: Value = serde_json::from_str(&geojson_str).unwrap();
        let features = v["features"].as_array().unwrap();
        assert_eq!(
            features.len(),
            10,
            "WA congressional plan must produce 10 GeoJSON features"
        );
    }

    #[test]
    fn test_geojson_district_has_district_id_attribute() {
        let plan = fixture_wa_congressional_plan();
        let geojson_str = export_geojson(&plan).unwrap();
        let v: Value = serde_json::from_str(&geojson_str).unwrap();
        for feature in v["features"].as_array().unwrap() {
            let props = &feature["properties"];
            assert!(
                !props["district_id"].is_null(),
                "each GeoJSON feature must have district_id property"
            );
        }
    }

    #[test]
    fn test_geojson_coordinates_lon_lat_rfc7946() {
        // Scenario 3: WA coordinates must be [lon, lat] — lon < 0, lat 45-49
        let plan = fixture_wa_congressional_plan_with_geometry();
        let geojson_str = export_geojson(&plan).unwrap();
        let v: Value = serde_json::from_str(&geojson_str).unwrap();
        let coords = &v["features"][0]["geometry"]["coordinates"][0][0];
        let lon = coords[0].as_f64().unwrap();
        let lat = coords[1].as_f64().unwrap();
        assert!(lon < 0.0, "longitude must be negative for WA (west)");
        assert!(lat > 40.0 && lat < 50.0, "latitude must be in WA range 40-50");
    }

    #[test]
    fn test_gerrychain_roundtrip_preserves_assignments() {
        let plan = fixture_wa_congressional_plan();
        let gc = export_gerrychain_v23(&plan).unwrap();
        let assignments2 = import_gerrychain_to_assignments(&gc).unwrap();
        assert_eq!(
            plan.assignments, assignments2,
            "GerryChain roundtrip must preserve all tract assignments"
        );
    }

    #[test]
    fn test_gerrychain_export_uses_assignment_singular_field() {
        let plan = fixture_wa_congressional_plan();
        let gc_json = export_gerrychain_v23(&plan).unwrap();
        let v: Value = serde_json::from_str(&gc_json).unwrap();
        assert!(
            v["assignment"].is_object(),
            "GerryChain v2.3 export must use 'assignment' field (singular)"
        );
        assert!(
            v.get("assignments").is_none() || v["assignments"].is_null(),
            "GerryChain export must not include 'assignments' (plural) field"
        );
    }

    #[test]
    fn test_csv_export_has_geoid_and_district_columns() {
        let plan = fixture_wa_congressional_plan();
        let csv_str = export_tracts_csv(&plan).unwrap();
        let mut reader = csv::Reader::from_reader(csv_str.as_bytes());
        let headers = reader.headers().unwrap().clone();
        assert!(
            headers.iter().any(|h| h == "geoid"),
            "CSV must have geoid column"
        );
        assert!(
            headers.iter().any(|h| h == "district"),
            "CSV must have district column"
        );
    }

    #[test]
    fn test_csv_export_row_count_matches_assignments() {
        let plan = fixture_wa_congressional_plan();
        let csv_str = export_tracts_csv(&plan).unwrap();
        let mut reader = csv::Reader::from_reader(csv_str.as_bytes());
        let row_count = reader.records().count();
        assert_eq!(
            row_count,
            plan.assignments.len(),
            "CSV must have one row per assignment"
        );
    }
}
