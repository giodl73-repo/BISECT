use bisect_core::state_code_to_fips as core_fips;
use serde_json::Value;
/// import.rs — GeoJSON district polygon → PIP tract assignment + nearest fallback.
///
/// Spec 6 / Scenario 3:
/// - PIP (point-in-polygon) assigns each tract centroid to the district that contains it
/// - If no polygon contains the centroid, fall back to nearest polygon centroid
/// - source="imported" is recorded in RPLAN metadata
use std::collections::HashMap;

/// Assign each census tract centroid to the district polygon containing it.
/// Falls back to the nearest polygon centroid for tracts outside all polygons.
///
/// `geojson_str` — RFC 7946 GeoJSON FeatureCollection with district polygons.
///   Each feature must have a "district_id" property (integer).
/// `tract_centroids` — map from GEOID (11-char) to (lon, lat) coordinates.
///
/// Returns: HashMap<GEOID, district_id>
pub fn import_geojson_plan(
    geojson_str: &str,
    tract_centroids: &HashMap<String, (f64, f64)>,
) -> anyhow::Result<HashMap<String, usize>> {
    use geo::algorithm::centroid::Centroid;
    use geo::{Contains, Point};

    let fc: Value = serde_json::from_str(geojson_str)?;
    if fc["type"].as_str() != Some("FeatureCollection") {
        anyhow::bail!("Expected GeoJSON FeatureCollection, got: {}", fc["type"]);
    }

    let features = fc["features"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("GeoJSON 'features' must be an array"))?;

    // Parse each feature into (district_id, geo::Polygon)
    let mut district_polygons: Vec<(usize, geo::Polygon)> = Vec::new();
    let mut district_centroids: Vec<(usize, (f64, f64))> = Vec::new();

    for feature in features {
        let district_id = parse_district_id(feature)?;

        let geom = &feature["geometry"];
        if geom.is_null() {
            continue;
        }

        let geo_type = geom["type"].as_str().unwrap_or("");
        match geo_type {
            "Polygon" => {
                let poly = parse_polygon(geom)?;
                let centroid = poly
                    .centroid()
                    .ok_or_else(|| anyhow::anyhow!("[INPUT] Polygon centroid is undefined"))?;
                district_centroids.push((district_id, (centroid.x(), centroid.y())));
                district_polygons.push((district_id, poly));
            }
            "MultiPolygon" => {
                // Use first sub-polygon for PIP; all sub-polygons for centroid
                let arr = geom["coordinates"].as_array().ok_or_else(|| {
                    anyhow::anyhow!("[INPUT] MultiPolygon coordinates must be an array")
                })?;
                if arr.is_empty() {
                    anyhow::bail!("[INPUT] MultiPolygon coordinates must not be empty");
                }
                for sub_poly_coords in arr {
                    let sub_geom = serde_json::json!({
                        "type": "Polygon",
                        "coordinates": sub_poly_coords,
                    });
                    let poly = parse_polygon(&sub_geom)?;
                    let centroid = poly
                        .centroid()
                        .ok_or_else(|| anyhow::anyhow!("[INPUT] Polygon centroid is undefined"))?;
                    district_centroids.push((district_id, (centroid.x(), centroid.y())));
                    district_polygons.push((district_id, poly));
                }
            }
            other => {
                anyhow::bail!(
                    "[INPUT] unsupported GeoJSON geometry type '{}' for district_id {}",
                    if other.is_empty() { "<missing>" } else { other },
                    district_id
                );
            }
        }
    }

    if district_polygons.is_empty() {
        anyhow::bail!("[INPUT] GeoJSON contains no usable Polygon or MultiPolygon geometry");
    }

    // PIP assignment: for each tract centroid, find the containing polygon
    let mut assignments = HashMap::with_capacity(tract_centroids.len());

    for (geoid, &(lon, lat)) in tract_centroids {
        let pt = Point::new(lon, lat);
        let mut assigned = None;

        for (dist_id, poly) in &district_polygons {
            if poly.contains(&pt) {
                assigned = Some(*dist_id);
                break;
            }
        }

        if assigned.is_none() {
            // Fallback: nearest polygon centroid
            assigned = nearest_centroid(lon, lat, &district_centroids);
        }

        let dist = assigned.ok_or_else(|| {
            anyhow::anyhow!("[INPUT] no polygon or fallback centroid could assign GEOID {geoid}")
        })?;
        assignments.insert(geoid.clone(), dist);
    }

    Ok(assignments)
}

fn parse_district_id(feature: &Value) -> anyhow::Result<usize> {
    let district_id = feature["properties"]["district_id"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("[INPUT] Feature missing integer 'district_id' property"))?;
    if district_id == 0 {
        anyhow::bail!("[INPUT] GeoJSON feature uses invalid district_id 0");
    }
    usize::try_from(district_id)
        .map_err(|_| anyhow::anyhow!("[INPUT] district_id value {district_id} is too large"))
}

/// Find the nearest district centroid to (lon, lat).
fn nearest_centroid(lon: f64, lat: f64, centroids: &[(usize, (f64, f64))]) -> Option<usize> {
    centroids
        .iter()
        .min_by(|a, b| {
            let da = (a.1 .0 - lon).powi(2) + (a.1 .1 - lat).powi(2);
            let db = (b.1 .0 - lon).powi(2) + (b.1 .1 - lat).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(dist_id, _)| *dist_id)
}

/// Parse a GeoJSON Polygon geometry into a geo::Polygon.
fn parse_polygon(geom: &Value) -> anyhow::Result<geo::Polygon> {
    let rings = geom["coordinates"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("[INPUT] Polygon coordinates must be an array"))?;
    if rings.is_empty() {
        anyhow::bail!("[INPUT] Polygon coordinates must include an exterior ring");
    }

    let exterior = parse_coordinate_ring(&rings[0], "exterior")?;

    if exterior.len() < 4 {
        anyhow::bail!("[INPUT] Polygon exterior ring must contain at least 4 coordinates");
    }

    let interiors: Vec<geo::LineString> = rings[1..]
        .iter()
        .enumerate()
        .map(|(idx, ring)| {
            let coords = parse_coordinate_ring(ring, &format!("interior ring {idx}"))?;
            if coords.len() >= 4 {
                Ok(geo::LineString::new(coords))
            } else {
                anyhow::bail!(
                    "[INPUT] Polygon interior ring {idx} must contain at least 4 coordinates"
                )
            }
        })
        .collect::<anyhow::Result<_>>()?;

    Ok(geo::Polygon::new(geo::LineString::new(exterior), interiors))
}

fn parse_coordinate_ring(ring: &Value, label: &str) -> anyhow::Result<Vec<geo::Coord>> {
    let coords = ring
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("[INPUT] Polygon {label} must be an array"))?;
    coords
        .iter()
        .enumerate()
        .map(|(idx, coord)| {
            let arr = coord.as_array().ok_or_else(|| {
                anyhow::anyhow!("[INPUT] Polygon {label} coordinate {idx} must be an array")
            })?;
            let lon = arr.first().and_then(|v| v.as_f64()).ok_or_else(|| {
                anyhow::anyhow!(
                    "[INPUT] Polygon {label} coordinate {idx} longitude must be a number"
                )
            })?;
            let lat = arr.get(1).and_then(|v| v.as_f64()).ok_or_else(|| {
                anyhow::anyhow!(
                    "[INPUT] Polygon {label} coordinate {idx} latitude must be a number"
                )
            })?;
            Ok(geo::Coord { x: lon, y: lat })
        })
        .collect()
}

/// Full import pipeline: GeoJSON file path → RplanFile written in memory.
/// Sets `source = "imported"` and `source_file` in RPLAN metadata.
pub fn import_plan_to_rplan(
    geojson_path: &str,
    state_code: &str,
    year: &str,
    label: &str,
    _version: &str,
    tract_centroids: &HashMap<String, (f64, f64)>,
) -> anyhow::Result<crate::rplan::RplanFile> {
    let content = std::fs::read_to_string(geojson_path)
        .map_err(|e| anyhow::anyhow!("cannot read GeoJSON file '{}': {}", geojson_path, e))?;

    let assignments = import_geojson_plan(&content, tract_centroids)?;

    // Validate GEOID format
    crate::rplan::validate_geoid_format_batch(&assignments)?;

    // Count distinct districts
    let num_districts = assignments.values().copied().max().unwrap_or(0);

    let mut metadata = crate::rplan::RplanMetadata {
        label: label.to_string(),
        state_fips: core_fips(&state_code.to_uppercase())
            .unwrap_or("00")
            .to_string(),
        state_code: state_code.to_uppercase(),
        year: year.to_string(),
        chamber: "imported".to_string(),
        num_districts,
        population_source: "unknown".to_string(),
        balance_tolerance_pct: 0.0,
        created_at: crate::paths::now_iso8601(),
        created_by: "bisect import".to_string(),
        ..Default::default()
    };
    // Mark as imported
    metadata.notes = Some(format!("imported from {}", geojson_path));

    Ok(crate::rplan::RplanFile {
        rplan_version: "0.1".into(),
        metadata,
        assignments,
        geometry: None,
    })
}

// ---------------------------------------------------------------------------
// Tests — Task 4
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_2_district_geojson() -> String {
        serde_json::to_string(&serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[
                            [-123.0, 47.0], [-122.0, 47.0], [-122.0, 48.0],
                            [-123.0, 48.0], [-123.0, 47.0]
                        ]]
                    },
                    "properties": {"district_id": 1}
                },
                {
                    "type": "Feature",
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[
                            [-122.0, 47.0], [-121.0, 47.0], [-121.0, 48.0],
                            [-122.0, 48.0], [-122.0, 47.0]
                        ]]
                    },
                    "properties": {"district_id": 2}
                }
            ]
        }))
        .unwrap()
    }

    fn fixture_wa_10_district_geojson() -> String {
        let features: Vec<serde_json::Value> = (1..=10usize)
            .map(|d| {
                let lon_base = -124.0 + (d as f64 - 1.0) * 0.5;
                serde_json::json!({
                    "type": "Feature",
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[
                            [lon_base, 46.0],
                            [lon_base + 0.4, 46.0],
                            [lon_base + 0.4, 47.0],
                            [lon_base, 47.0],
                            [lon_base, 46.0],
                        ]]
                    },
                    "properties": {"district_id": d}
                })
            })
            .collect();
        serde_json::to_string(&serde_json::json!({
            "type": "FeatureCollection",
            "features": features,
        }))
        .unwrap()
    }

    fn load_wa_tract_centroids_fixture() -> HashMap<String, (f64, f64)> {
        let mut centroids = HashMap::new();
        // 50 fake WA tract centroids inside the 10-district bands
        for i in 0..10usize {
            for j in 0..5usize {
                let geoid = format!("{:05}{:06}", i + 1, j);
                let lon = -124.0 + i as f64 * 0.5 + 0.1;
                let lat = 46.5;
                centroids.insert(geoid, (lon, lat));
            }
        }
        centroids
    }

    #[test]
    fn test_geojson_import_assigns_all_tracts() {
        let geojson_str = fixture_wa_10_district_geojson();
        let tract_centroids = load_wa_tract_centroids_fixture();
        let assignments = import_geojson_plan(&geojson_str, &tract_centroids).unwrap();
        assert_eq!(
            assignments.len(),
            tract_centroids.len(),
            "all tracts must be assigned; got {} of {}",
            assignments.len(),
            tract_centroids.len()
        );
        for (_, dist) in &assignments {
            assert!(
                *dist >= 1 && *dist <= 10,
                "district must be 1-10, got {}",
                dist
            );
        }
    }

    #[test]
    fn test_import_nearest_polygon_fallback_used_for_border_tracts() {
        let geojson_str = fixture_2_district_geojson();
        let centroids = HashMap::from([
            ("53001001000".to_string(), (-122.5, 47.5)), // inside district 1
            ("53001001001".to_string(), (-999.0, 999.0)), // outside all polygons
        ]);
        let assignments = import_geojson_plan(&geojson_str, &centroids).unwrap();
        assert!(
            assignments.contains_key("53001001001"),
            "border tract must be assigned via nearest-polygon fallback"
        );
    }

    #[test]
    fn test_import_inside_polygon_assigned_correctly() {
        let geojson_str = fixture_2_district_geojson();
        let centroids = HashMap::from([
            ("53001001000".to_string(), (-122.5, 47.5)), // inside district 1
            ("53001001001".to_string(), (-121.5, 47.5)), // inside district 2
        ]);
        let assignments = import_geojson_plan(&geojson_str, &centroids).unwrap();
        assert_eq!(assignments["53001001000"], 1, "should be in district 1");
        assert_eq!(assignments["53001001001"], 2, "should be in district 2");
    }

    #[test]
    fn test_import_rejects_zero_district_id() {
        let geojson_str = serde_json::to_string(&serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[
                        [-123.0, 47.0], [-122.0, 47.0], [-122.0, 48.0],
                        [-123.0, 48.0], [-123.0, 47.0]
                    ]]
                },
                "properties": {"district_id": 0}
            }]
        }))
        .unwrap();
        let centroids = HashMap::from([("53001001000".to_string(), (-122.5, 47.5))]);
        let err = import_geojson_plan(&geojson_str, &centroids).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]") && msg.contains("invalid district_id 0"),
            "error must classify and explain invalid district_id: {msg}"
        );
    }

    #[test]
    fn test_import_rejects_non_integer_district_id() {
        let geojson_str = serde_json::to_string(&serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[
                        [-123.0, 47.0], [-122.0, 47.0], [-122.0, 48.0],
                        [-123.0, 48.0], [-123.0, 47.0]
                    ]]
                },
                "properties": {"district_id": "one"}
            }]
        }))
        .unwrap();
        let centroids = HashMap::from([("53001001000".to_string(), (-122.5, 47.5))]);
        let err = import_geojson_plan(&geojson_str, &centroids).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]") && msg.contains("integer 'district_id'"),
            "error must classify and explain malformed district_id: {msg}"
        );
    }

    #[test]
    fn test_import_rejects_malformed_polygon_coordinate() {
        let geojson_str = serde_json::to_string(&serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[
                        [-123.0, 47.0], ["bad", 47.0], [-122.0, 48.0],
                        [-123.0, 48.0], [-123.0, 47.0]
                    ]]
                },
                "properties": {"district_id": 1}
            }]
        }))
        .unwrap();
        let centroids = HashMap::from([("53001001000".to_string(), (-122.5, 47.5))]);
        let err = import_geojson_plan(&geojson_str, &centroids).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]") && msg.contains("longitude must be a number"),
            "error must classify and explain malformed polygon coordinate: {msg}"
        );
    }

    #[test]
    fn test_import_rejects_no_usable_polygon_geometry() {
        let geojson_str = serde_json::to_string(&serde_json::json!({
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": null,
                "properties": {"district_id": 1}
            }]
        }))
        .unwrap();
        let centroids = HashMap::from([("53001001000".to_string(), (-122.5, 47.5))]);
        let err = import_geojson_plan(&geojson_str, &centroids).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]") && msg.contains("no usable Polygon"),
            "error must classify missing usable geometry: {msg}"
        );
    }

    #[test]
    fn test_import_rejects_unsupported_geometry_type() {
        let geojson_str = serde_json::to_string(&serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": {
                        "type": "Point",
                        "coordinates": [-122.5, 47.5]
                    },
                    "properties": {"district_id": 1}
                },
                {
                    "type": "Feature",
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[
                            [-123.0, 47.0], [-122.0, 47.0], [-122.0, 48.0],
                            [-123.0, 48.0], [-123.0, 47.0]
                        ]]
                    },
                    "properties": {"district_id": 2}
                }
            ]
        }))
        .unwrap();
        let centroids = HashMap::from([("53001001000".to_string(), (-122.5, 47.5))]);
        let err = import_geojson_plan(&geojson_str, &centroids).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("[INPUT]") && msg.contains("unsupported GeoJSON geometry type 'Point'"),
            "error must classify unsupported geometry types: {msg}"
        );
    }

    #[test]
    fn test_import_writes_source_imported_in_notes() {
        let geojson_str = fixture_2_district_geojson();
        let centroids = HashMap::from([("53001001000".to_string(), (-122.5, 47.5))]);
        // Write geojson to temp file
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut tmp.as_file(), geojson_str.as_bytes()).unwrap();
        let rplan = import_plan_to_rplan(
            tmp.path().to_str().unwrap(),
            "WA",
            "2020",
            "wa_imported",
            "test",
            &centroids,
        )
        .unwrap();
        let notes = rplan.metadata.notes.as_deref().unwrap_or("");
        assert!(
            notes.contains("imported"),
            "notes must mention 'imported', got: {notes}"
        );
    }
}
