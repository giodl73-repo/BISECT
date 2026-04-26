/// Enacted district tract assignment via centroid point-in-polygon (PIP).
///
/// Board amendment (TRENCH): PIP only handles tracts whose centroid falls
/// clearly inside one polygon. For boundary-straddling tracts in dense urban
/// areas the centroid may land in the wrong district — this is a known
/// limitation. The `fallback_count` and `pip_method_note` metadata fields
/// allow practitioners to audit coverage.
///
/// Two-step algorithm:
///   Step 1: geo::Contains — point-in-polygon for each tract centroid.
///   Step 2: nearest-polygon fallback for any unassigned centroid.
///   Step 3: 100% coverage assertion — error if any tract still unassigned.
use geo::algorithm::Contains;
use geo::algorithm::EuclideanDistance;
use geo_types::{MultiPolygon, Point};
use std::collections::HashMap;

/// Metadata produced alongside the assignment map.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EnactedAssignmentMeta {
    pub total_tracts: usize,
    pub pip_assigned: usize,
    pub fallback_count: usize,
    pub pip_method_note: String,
}

/// Assign each tract centroid to an enacted congressional district.
///
/// Returns a map GEOID -> district_id and accompanying metadata.
///
/// Errors if:
///  - `enacted_polygons` is empty (cannot assign anything).
///  - Any tract is still unassigned after the nearest-polygon fallback
///    (which should be logically impossible, but is checked for safety).
pub fn assign_tracts_to_enacted(
    tract_centroids: &[Point<f64>],
    tract_geoids: &[String],
    enacted_polygons: &[MultiPolygon<f64>],
    enacted_ids: &[usize],
) -> anyhow::Result<(HashMap<String, usize>, EnactedAssignmentMeta)> {
    if enacted_polygons.is_empty() {
        anyhow::bail!("coverage: cannot assign tracts — enacted polygon list is empty");
    }
    if enacted_polygons.len() != enacted_ids.len() {
        anyhow::bail!(
            "coverage: enacted_polygons ({}) and enacted_ids ({}) must have the same length",
            enacted_polygons.len(),
            enacted_ids.len()
        );
    }

    let mut assignments: HashMap<String, usize> = HashMap::with_capacity(tract_geoids.len());
    let mut fallback_count = 0usize;

    for (centroid, geoid) in tract_centroids.iter().zip(tract_geoids.iter()) {
        // Step 1: point-in-polygon
        let mut assigned = false;
        for (poly, &id) in enacted_polygons.iter().zip(enacted_ids.iter()) {
            if poly.contains(centroid) {
                assignments.insert(geoid.clone(), id);
                assigned = true;
                break;
            }
        }

        // Step 2: nearest-polygon fallback
        if !assigned {
            let id = assign_nearest(centroid, enacted_polygons, enacted_ids);
            assignments.insert(geoid.clone(), id);
            fallback_count += 1;
        }
    }

    // Step 3: 100% coverage assertion
    let unassigned: Vec<_> = tract_geoids
        .iter()
        .filter(|g| !assignments.contains_key(*g))
        .collect();
    if !unassigned.is_empty() {
        anyhow::bail!(
            "coverage: {} tracts unassigned after enacted assignment: {:?}",
            unassigned.len(),
            &unassigned[..unassigned.len().min(5)]
        );
    }

    let pip_assigned = tract_geoids.len() - fallback_count;
    let meta = EnactedAssignmentMeta {
        total_tracts: tract_geoids.len(),
        pip_assigned,
        fallback_count,
        pip_method_note: "centroid PIP; boundary-straddling tracts may be misassigned".into(),
    };

    Ok((assignments, meta))
}

/// Assign a single centroid to the nearest polygon (minimum Euclidean distance).
///
/// Called when point-in-polygon returns no match (offshore / sliver tracts).
pub fn assign_single_centroid(
    centroid: &Point<f64>,
    polygons: &[MultiPolygon<f64>],
    ids: &[usize],
) -> usize {
    // Step 1: point-in-polygon
    for (poly, &id) in polygons.iter().zip(ids.iter()) {
        if poly.contains(centroid) {
            return id;
        }
    }
    // Step 2: nearest-polygon fallback
    assign_nearest(centroid, polygons, ids)
}

fn assign_nearest(
    centroid: &Point<f64>,
    polygons: &[MultiPolygon<f64>],
    ids: &[usize],
) -> usize {
    polygons
        .iter()
        .zip(ids.iter())
        .map(|(poly, &id)| (poly.euclidean_distance(centroid), id))
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, id)| id)
        .expect("polygons must not be empty — checked by caller")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::{LineString, Polygon};

    /// Build a simple axis-aligned square polygon.
    fn make_square(x0: f64, y0: f64, x1: f64, y1: f64) -> MultiPolygon<f64> {
        let ring = LineString::from(vec![
            (x0, y0),
            (x1, y0),
            (x1, y1),
            (x0, y1),
            (x0, y0),
        ]);
        MultiPolygon::new(vec![Polygon::new(ring, vec![])])
    }

    #[test]
    fn test_tract_centroid_in_polygon_correct_district() {
        let centroid = Point::new(-120.5, 47.5); // inside the square
        let poly = make_square(-121.0, 47.0, -120.0, 48.0);
        let polygons = vec![poly];
        let ids = vec![3usize];
        let result = assign_single_centroid(&centroid, &polygons, &ids);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_nearest_polygon_fallback_for_coastal_tract() {
        // Centroid just offshore; nearest polygon is the inland one.
        let centroid = Point::new(-124.9, 47.1);
        let poly = make_square(-124.8, 47.0, -124.5, 47.2); // inland
        let polygons = vec![poly];
        let ids = vec![5usize];
        let result = assign_single_centroid(&centroid, &polygons, &ids);
        assert_eq!(result, 5, "Nearest-polygon fallback must assign coastal tract");
    }

    #[test]
    fn test_100_pct_coverage_assertion_all_assigned() {
        let centroids = vec![Point::new(-120.5, 47.5), Point::new(-121.0, 46.0)];
        let geoids = vec!["530330001001".to_string(), "530410002001".to_string()];
        let polygons = vec![
            make_square(-121.5, 47.0, -120.0, 48.0),
            make_square(-122.0, 45.5, -120.5, 46.5),
        ];
        let ids = vec![1usize, 2usize];
        let (result, meta) = assign_tracts_to_enacted(&centroids, &geoids, &polygons, &ids).unwrap();
        assert_eq!(result.len(), 2, "All tracts must be assigned");
        for geoid in &geoids {
            assert!(result.contains_key(geoid), "GEOID {} not in result", geoid);
        }
        assert_eq!(meta.total_tracts, 2);
    }

    #[test]
    fn test_zero_unassigned_tracts_after_fallback() {
        // One centroid outside all polygons -> fallback should still assign it.
        let centroids = vec![
            Point::new(-100.0, 40.0),  // inside polygon 1
            Point::new(0.0, 0.0),      // outside all polygons -> fallback to nearest
        ];
        let geoids = vec!["530330001001".to_string(), "530330002001".to_string()];
        let polygons = vec![
            make_square(-101.0, 39.0, -99.0, 41.0),
            make_square(-120.0, 47.0, -119.0, 48.0),
        ];
        let ids = vec![1usize, 2usize];
        let (result, meta) = assign_tracts_to_enacted(&centroids, &geoids, &polygons, &ids).unwrap();
        assert_eq!(result.len(), geoids.len(), "100% coverage required after fallback");
        assert!(meta.fallback_count >= 1, "At least one fallback expected");
        assert!(!meta.pip_method_note.is_empty());
    }

    #[test]
    fn test_assign_tracts_errors_if_coverage_incomplete_empty_polygons() {
        let centroids = vec![Point::new(-120.5, 47.5)];
        let geoids = vec!["530330001001".to_string()];
        let polygons: Vec<MultiPolygon<f64>> = vec![];
        let ids: Vec<usize> = vec![];
        let result = assign_tracts_to_enacted(&centroids, &geoids, &polygons, &ids);
        assert!(result.is_err(), "Empty polygon list must produce error, not silent gap");
        assert!(result.unwrap_err().to_string().contains("coverage"));
    }

    #[test]
    fn test_pip_method_note_in_metadata() {
        let centroids = vec![Point::new(-120.5, 47.5)];
        let geoids = vec!["530330001001".to_string()];
        let polygons = vec![make_square(-121.0, 47.0, -120.0, 48.0)];
        let ids = vec![1usize];
        let (_, meta) = assign_tracts_to_enacted(&centroids, &geoids, &polygons, &ids).unwrap();
        assert!(
            meta.pip_method_note.contains("centroid PIP"),
            "Metadata must include pip_method_note"
        );
        assert!(
            meta.pip_method_note.contains("boundary-straddling"),
            "Metadata must warn about boundary-straddling tracts"
        );
    }

    #[test]
    fn test_fallback_count_recorded_in_metadata() {
        // Force one fallback by using an offshore centroid.
        let centroids = vec![
            Point::new(-120.5, 47.5),  // inside poly 1
            Point::new(0.0, 0.0),      // outside all -> fallback
        ];
        let geoids = vec!["g1".to_string(), "g2".to_string()];
        let polygons = vec![
            make_square(-121.0, 47.0, -120.0, 48.0),
            make_square(-121.5, 47.5, -120.5, 48.5),
        ];
        let ids = vec![1usize, 2usize];
        let (_, meta) = assign_tracts_to_enacted(&centroids, &geoids, &polygons, &ids).unwrap();
        assert_eq!(
            meta.fallback_count, 1,
            "Exactly one fallback expected for the offshore centroid"
        );
        assert_eq!(meta.pip_assigned, 1);
    }
}
