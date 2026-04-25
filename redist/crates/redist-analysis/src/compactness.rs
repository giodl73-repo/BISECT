/// Compactness metrics: Polsby-Popper, Reock, Convex Hull Ratio.
///
/// **CRITICAL PRECONDITION**: all input WKB geometries MUST be in a projected
/// equal-area CRS (metres), not WGS84/NAD83 degrees. Use:
///   EPSG:5070 (Albers Equal Area) for CONUS
///   EPSG:3338 for Alaska
///   EPSG:6364 for Hawaii
/// The formulas use `.area()` and `.length()` from the `geo` crate which
/// compute planar (Cartesian) values. WGS84 degree inputs produce nonsense.
///
/// Python equivalents (analyze_district_compactness.py):
///   polsby_popper → geometry.area, geometry.length → 4π*A/P²
///   reock → centroid + max boundary distance → A / πr²
///   convex_hull_ratio → A / convex_hull.area
use std::f64::consts::PI;
use geo::{Area, EuclideanLength, Centroid, ConvexHull};
use geo_types::{Polygon, MultiPolygon, Coord, Point};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompactnessError {
    #[error("empty geometry: cannot compute compactness for a polygon with zero area")]
    EmptyGeometry,
    #[error("zero perimeter: polygon has non-zero area but zero perimeter (degenerate geometry)")]
    ZeroPerimeter,
    #[error("WKB parse error: {0}")]
    WkbError(String),
}

/// All three compactness metrics for a single district.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompactnessMetrics {
    pub district: usize,
    /// Polsby-Popper = 4π × area / perimeter². Range [0,1]; 1 = perfect circle.
    pub polsby_popper: f64,
    /// Reock = area / minimum_bounding_circle_area. Range [0,1].
    pub reock: f64,
    /// Convex Hull Ratio = area / convex_hull_area. Range [0,1].
    pub convex_hull_ratio: f64,
    /// Perimeter in metres (projected CRS).
    pub perimeter_m: f64,
    /// Area in square metres (projected CRS).
    pub area_m2: f64,
}

// ---------------------------------------------------------------------------
// Core formulas — each matches the Python implementation exactly
// ---------------------------------------------------------------------------

/// Polsby-Popper score: 4π × A / P²
///
/// Python: `(4 * np.pi * area) / (perimeter ** 2)`, capped at 1.0.
/// Requires projected coordinates (metres). Returns (score, perimeter_m).
pub fn polsby_popper(polygon: &Polygon<f64>) -> Result<(f64, f64), CompactnessError> {
    let area = polygon.unsigned_area();
    if area == 0.0 {
        return Err(CompactnessError::EmptyGeometry);
    }
    let perimeter = polygon_perimeter(polygon);
    if perimeter == 0.0 {
        return Err(CompactnessError::ZeroPerimeter);
    }
    let score = (4.0 * PI * area) / (perimeter * perimeter);
    Ok((score.min(1.0), perimeter))  // cap at 1.0 like Python
}

/// Reock score: A / (π × r²) where r = max distance from centroid to boundary.
///
/// Python: centroid + max_dist to any boundary point → circle_area = π*r²
/// This is an approximation of the minimum bounding circle (not the true MBC
/// via Welzl's algorithm, but matching Python's approximation exactly).
pub fn reock(polygon: &Polygon<f64>) -> Result<f64, CompactnessError> {
    let area = polygon.unsigned_area();
    if area == 0.0 {
        return Err(CompactnessError::EmptyGeometry);
    }
    let centroid = polygon.centroid()
        .ok_or(CompactnessError::EmptyGeometry)?;
    let radius = max_distance_to_boundary(polygon, centroid);
    if radius == 0.0 {
        return Ok(0.0);
    }
    let circle_area = PI * radius * radius;
    Ok((area / circle_area).min(1.0))
}

/// Convex Hull Ratio: A / convex_hull_area.
///
/// Python: `area / geometry.convex_hull.area`.
pub fn convex_hull_ratio(polygon: &Polygon<f64>) -> Result<f64, CompactnessError> {
    let area = polygon.unsigned_area();
    if area == 0.0 {
        return Err(CompactnessError::EmptyGeometry);
    }
    let hull = polygon.convex_hull();
    let hull_area = hull.unsigned_area();
    if hull_area == 0.0 {
        return Ok(0.0);
    }
    Ok((area / hull_area).min(1.0))
}

/// Compute all three metrics for a district polygon.
pub fn all_metrics(
    district: usize,
    polygon: &Polygon<f64>,
) -> Result<CompactnessMetrics, CompactnessError> {
    let (pp, perimeter) = polsby_popper(polygon)?;
    let reock_score = reock(polygon)?;
    let chr = convex_hull_ratio(polygon)?;
    let area = polygon.unsigned_area();
    Ok(CompactnessMetrics {
        district,
        polsby_popper: pp,
        reock: reock_score,
        convex_hull_ratio: chr,
        perimeter_m: perimeter,
        area_m2: area,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Polygon perimeter = exterior ring length only.
///
/// Matches Python Shapely: `geometry.length` on a Polygon returns only the
/// exterior ring perimeter, NOT including interior holes. Holes are excluded
/// to maintain parity with `analyze_district_compactness.py:polsby_popper_score`.
fn polygon_perimeter(polygon: &Polygon<f64>) -> f64 {
    polygon.exterior().euclidean_length()
}

/// Max Euclidean distance from centroid to any exterior boundary coordinate.
/// Matches Python's `minimum_bounding_circle` approximation.
fn max_distance_to_boundary(polygon: &Polygon<f64>, centroid: Point<f64>) -> f64 {
    polygon.exterior().coords()
        .map(|coord| {
            let dx = coord.x - centroid.x();
            let dy = coord.y - centroid.y();
            (dx * dx + dy * dy).sqrt()
        })
        .fold(0.0_f64, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::LineString;

    /// A perfect circle approximated as a 360-gon.
    fn circle_polygon(cx: f64, cy: f64, r: f64) -> Polygon<f64> {
        let n = 360usize;
        let coords: Vec<Coord<f64>> = (0..=n).map(|i| {
            let theta = 2.0 * PI * i as f64 / n as f64;
            Coord { x: cx + r * theta.cos(), y: cy + r * theta.sin() }
        }).collect();
        Polygon::new(LineString::new(coords), vec![])
    }

    fn unit_square(origin_x: f64, origin_y: f64) -> Polygon<f64> {
        // 1000m × 1000m square in projected coordinates
        let x0 = origin_x;
        let y0 = origin_y;
        let s = 1000.0_f64;
        Polygon::new(
            LineString::new(vec![
                Coord { x: x0,     y: y0 },
                Coord { x: x0 + s, y: y0 },
                Coord { x: x0 + s, y: y0 + s },
                Coord { x: x0,     y: y0 + s },
                Coord { x: x0,     y: y0 },
            ]),
            vec![],
        )
    }

    // ── Polsby-Popper ────────────────────────────────────────────────────────

    #[test]
    fn test_pp_circle_approaches_1() {
        let poly = circle_polygon(1_000_000.0, 1_000_000.0, 5000.0);
        let (pp, _) = polsby_popper(&poly).unwrap();
        // 360-gon ≈ circle — PP should be very close to 1.0
        assert!(pp > 0.999, "circle PP should be ~1.0, got {pp:.6}");
        assert!(pp <= 1.0, "PP must not exceed 1.0, got {pp:.6}");
    }

    #[test]
    fn test_pp_square_is_pi_over_4() {
        // Square: A=s², P=4s → PP = 4π·s²/(4s)² = π/4 ≈ 0.7854
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let (pp, _) = polsby_popper(&poly).unwrap();
        let expected = PI / 4.0;
        assert!((pp - expected).abs() < 1e-6,
            "square PP should be π/4={expected:.6}, got {pp:.6}");
    }

    #[test]
    fn test_pp_perimeter_returned_correctly() {
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let (_, perimeter) = polsby_popper(&poly).unwrap();
        // 4 × 1000m = 4000m
        assert!((perimeter - 4000.0).abs() < 0.001, "perimeter should be 4000m, got {perimeter}");
    }

    #[test]
    fn test_pp_empty_geometry_error() {
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        assert!(matches!(polsby_popper(&empty), Err(CompactnessError::EmptyGeometry)));
    }

    #[test]
    fn test_pp_capped_at_1() {
        // Numerically degenerate polygon could exceed 1 without capping
        let poly = circle_polygon(0.0, 0.0, 1.0);
        let (pp, _) = polsby_popper(&poly).unwrap();
        assert!(pp <= 1.0);
    }

    // ── Reock ────────────────────────────────────────────────────────────────

    #[test]
    fn test_reock_circle_approaches_1() {
        let poly = circle_polygon(1_000_000.0, 1_000_000.0, 5000.0);
        let r = reock(&poly).unwrap();
        // True circle: area = πr², bounding circle area = πr² → Reock = 1
        assert!(r > 0.999, "circle Reock should be ~1.0, got {r:.6}");
    }

    #[test]
    fn test_reock_square_is_pi_over_4() {
        // Square (s×s): area = s², bounding circle radius = s√2/2
        // circle_area = π(s/√2)² = πs²/2
        // reock = s² / (πs²/2) = 2/π ≈ 0.6366
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let r = reock(&poly).unwrap();
        let expected = 2.0 / PI;
        assert!((r - expected).abs() < 0.001,
            "square Reock should be 2/π={expected:.6}, got {r:.6}");
    }

    #[test]
    fn test_reock_empty_error() {
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        assert!(matches!(reock(&empty), Err(CompactnessError::EmptyGeometry)));
    }

    // ── Convex Hull Ratio ────────────────────────────────────────────────────

    #[test]
    fn test_convex_hull_ratio_convex_polygon_is_1() {
        // A square is already convex — hull ratio = 1
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let chr = convex_hull_ratio(&poly).unwrap();
        assert!((chr - 1.0).abs() < 1e-6, "square CHR should be 1.0, got {chr:.6}");
    }

    #[test]
    fn test_convex_hull_ratio_l_shape() {
        // L-shaped polygon: area < convex hull area → CHR < 1
        let l_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 1_000_000.0, y: 1_000_000.0 },
                Coord { x: 1_002_000.0, y: 1_000_000.0 },
                Coord { x: 1_002_000.0, y: 1_001_000.0 },
                Coord { x: 1_001_000.0, y: 1_001_000.0 },
                Coord { x: 1_001_000.0, y: 1_002_000.0 },
                Coord { x: 1_000_000.0, y: 1_002_000.0 },
                Coord { x: 1_000_000.0, y: 1_000_000.0 },
            ]),
            vec![],
        );
        let chr = convex_hull_ratio(&l_shape).unwrap();
        assert!(chr < 1.0, "L-shape CHR should be < 1.0, got {chr:.6}");
        assert!(chr > 0.5, "L-shape CHR should be reasonable, got {chr:.6}");
    }

    // ── all_metrics ──────────────────────────────────────────────────────────

    #[test]
    fn test_all_metrics_square() {
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let m = all_metrics(1, &poly).unwrap();
        assert_eq!(m.district, 1);
        assert!((m.polsby_popper - PI / 4.0).abs() < 1e-6);
        assert!((m.area_m2 - 1_000_000.0).abs() < 0.001); // 1km² in m²
        assert!((m.perimeter_m - 4000.0).abs() < 0.001);
        assert!(m.convex_hull_ratio > 0.99); // square is convex
    }

    // ── Bounds and Python parity ─────────────────────────────────────────────

    #[test]
    fn test_all_scores_nonnegative() {
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let (pp, _) = polsby_popper(&poly).unwrap();
        assert!(pp >= 0.0, "PP must be >= 0, got {pp}");
        let r = reock(&poly).unwrap();
        assert!(r >= 0.0, "Reock must be >= 0, got {r}");
        let chr = convex_hull_ratio(&poly).unwrap();
        assert!(chr >= 0.0, "CHR must be >= 0, got {chr}");
    }

    #[test]
    fn test_perimeter_excludes_holes() {
        // Polygon with a hole: Python .length returns only exterior (4000m)
        // Rust must match — do NOT add hole perimeters
        let exterior = LineString::new(vec![
            Coord { x: 1_000_000.0, y: 1_000_000.0 },
            Coord { x: 1_001_000.0, y: 1_000_000.0 },
            Coord { x: 1_001_000.0, y: 1_001_000.0 },
            Coord { x: 1_000_000.0, y: 1_001_000.0 },
            Coord { x: 1_000_000.0, y: 1_000_000.0 },
        ]);
        let hole = LineString::new(vec![
            Coord { x: 1_000_250.0, y: 1_000_250.0 },
            Coord { x: 1_000_750.0, y: 1_000_250.0 },
            Coord { x: 1_000_750.0, y: 1_000_750.0 },
            Coord { x: 1_000_250.0, y: 1_000_750.0 },
            Coord { x: 1_000_250.0, y: 1_000_250.0 },
        ]);
        let poly = Polygon::new(exterior, vec![hole]);
        let (_, perimeter) = polsby_popper(&poly).unwrap();
        // Exterior only = 4000m; hole = 2000m
        // Must be 4000m, NOT 6000m
        assert!((perimeter - 4000.0).abs() < 0.001,
            "perimeter should be 4000m (exterior only), got {perimeter}");
    }

    // ── Python parity ────────────────────────────────────────────────────────

    #[test]
    fn test_pp_formula_matches_python_exactly() {
        // Python: score = (4 * np.pi * area) / (perimeter ** 2)
        // For a 2000×500m rectangle: A=1,000,000, P=5000
        let rect = Polygon::new(
            LineString::new(vec![
                Coord { x: 1_000_000.0, y: 1_000_000.0 },
                Coord { x: 1_002_000.0, y: 1_000_000.0 },
                Coord { x: 1_002_000.0, y: 1_000_500.0 },
                Coord { x: 1_000_000.0, y: 1_000_500.0 },
                Coord { x: 1_000_000.0, y: 1_000_000.0 },
            ]),
            vec![],
        );
        let area = 2000.0 * 500.0;
        let perimeter = 2.0 * (2000.0 + 500.0);
        let expected_pp = (4.0 * PI * area) / (perimeter * perimeter);
        let (pp, p) = polsby_popper(&rect).unwrap();
        assert!((pp - expected_pp).abs() < 1e-9, "PP {pp} != expected {expected_pp}");
        assert!((p - perimeter).abs() < 0.001, "perimeter {p} != {perimeter}");
    }
}
