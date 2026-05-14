use geo::{Area, BoundingRect, Centroid, ConvexHull, EuclideanLength};
use geo_types::{Coord, Point, Polygon};
/// Compactness metrics: Polsby-Popper, Reock, Convex Hull Ratio,
/// Schwartzberg, Length-Width Ratio, and Population-Weighted Compactness.
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

/// All compactness metrics for a single district.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompactnessMetrics {
    pub district: usize,
    /// Polsby-Popper = 4π × area / perimeter². Range [0,1]; 1 = perfect circle.
    pub polsby_popper: f64,
    /// Reock = area / minimum_bounding_circle_area. Range [0,1].
    pub reock: f64,
    /// Convex Hull Ratio = area / convex_hull_area. Range [0,1].
    pub convex_hull_ratio: f64,
    /// Schwartzberg = P / (2√(πA)) = 1/√PP. Range [1, ∞); 1 = perfect circle.
    pub schwartzberg: f64,
    /// Length-Width Ratio = long_side / short_side of minimum bounding rectangle.
    /// Range [1, ∞); 1 = square or circle.
    pub length_width_ratio: f64,
    /// Perimeter in metres (projected CRS).
    pub perimeter_m: f64,
    /// Area in square metres (projected CRS).
    pub area_m2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoundingCircle {
    pub center_x: f64,
    pub center_y: f64,
    pub radius: f64,
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
    Ok((score.min(1.0), perimeter)) // cap at 1.0 like Python
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
    let centroid = polygon.centroid().ok_or(CompactnessError::EmptyGeometry)?;
    let radius = max_distance_to_boundary(polygon, centroid);
    if radius == 0.0 {
        return Ok(0.0);
    }
    let circle_area = PI * radius * radius;
    Ok((area / circle_area).min(1.0))
}

/// Exact polygon-vertex minimum-bounding-circle Reock.
///
/// This is intentionally separate from `reock()`, which remains the production
/// Python-parity centroid-radius proxy. For a polygon with straight edges, the
/// minimum circle containing all exterior vertices also contains every exterior
/// edge segment because circles are convex.
pub fn exact_reock(polygon: &Polygon<f64>) -> Result<f64, CompactnessError> {
    let area = polygon.unsigned_area();
    if area == 0.0 {
        return Err(CompactnessError::EmptyGeometry);
    }
    let circle = minimum_bounding_circle(polygon)?;
    if circle.radius == 0.0 {
        return Ok(0.0);
    }
    Ok((area / (PI * circle.radius * circle.radius)).min(1.0))
}

pub fn minimum_bounding_circle(polygon: &Polygon<f64>) -> Result<BoundingCircle, CompactnessError> {
    let points = exterior_points(polygon);
    if points.is_empty() || polygon.unsigned_area() == 0.0 {
        return Err(CompactnessError::EmptyGeometry);
    }
    Ok(smallest_enclosing_circle(&points))
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

/// Schwartzberg score: P / (2√(πA)).
///
/// Mathematical identity: S = 1 / √PP.
///
/// Proof: PP = 4πA/P², so √PP = 2√(πA)/P, thus 1/√PP = P/(2√(πA)) = S. ∎
///
/// Range [1, ∞); S = 1 for a perfect circle (PP = 1 → S = 1/√1 = 1).
/// A square has PP = π/4, so S = 1/√(π/4) = 2/√π ≈ 1.128.
///
/// This reuses `polsby_popper()` to avoid recomputing area and perimeter.
pub fn schwartzberg(polygon: &Polygon<f64>) -> Result<f64, CompactnessError> {
    let (pp, _) = polsby_popper(polygon)?;
    // pp is capped at 1.0 already; sqrt is safe because pp > 0 (EmptyGeometry
    // and ZeroPerimeter errors were already handled inside polsby_popper).
    Ok(1.0 / pp.sqrt())
}

/// Length-Width Ratio: long_side / short_side of the minimum bounding rectangle.
///
/// The minimum bounding rectangle (MBR) is found via rotating calipers on the
/// convex hull: for each edge of the hull we rotate so that edge is horizontal,
/// compute the axis-aligned bounding box in that frame, and keep the box with
/// the smallest area. This is the standard O(n) rotating-calipers result after
/// computing the O(n log n) convex hull.
///
/// Range [1, ∞); 1 = square or circle; 2 = 2:1 rectangle.
pub fn length_width_ratio(polygon: &Polygon<f64>) -> Result<f64, CompactnessError> {
    let area = polygon.unsigned_area();
    if area == 0.0 {
        return Err(CompactnessError::EmptyGeometry);
    }

    let hull = polygon.convex_hull();
    let coords: Vec<Coord<f64>> = hull.exterior().coords().cloned().collect();

    // Need at least 3 distinct vertices (plus the closing repeat = 4 points).
    if coords.len() < 3 {
        // Degenerate hull — fall back to 1.0 (no elongation detectable)
        return Ok(1.0);
    }

    let n = coords.len() - 1; // exclude closing duplicate

    // For each hull edge, rotate coordinates so that edge is horizontal,
    // compute axis-aligned bounding box, track minimum-area box and its ratio.
    let mut min_box_area = f64::MAX;
    let mut best_ratio: f64 = 1.0;

    for i in 0..n {
        let c0 = coords[i];
        let c1 = coords[(i + 1) % n];

        let dx = c1.x - c0.x;
        let dy = c1.y - c0.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 {
            continue;
        }

        // Rotation that maps this edge to the x-axis:
        //   x' =  x·cos_a + y·sin_a
        //   y' = -x·sin_a + y·cos_a
        let cos_a = dx / len;
        let sin_a = dy / len;

        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;

        for c in &coords[..n] {
            let xr = c.x * cos_a + c.y * sin_a;
            let yr = -c.x * sin_a + c.y * cos_a;
            if xr < x_min {
                x_min = xr;
            }
            if xr > x_max {
                x_max = xr;
            }
            if yr < y_min {
                y_min = yr;
            }
            if yr > y_max {
                y_max = yr;
            }
        }

        let w = x_max - x_min; // dimension along edge
        let h = y_max - y_min; // dimension perpendicular to edge
        let box_area = w * h;

        if box_area > 0.0 && box_area < min_box_area {
            min_box_area = box_area;
            best_ratio = if w >= h { w / h } else { h / w };
        }
    }

    if min_box_area == f64::MAX {
        // All edges had zero length — degenerate polygon
        return Ok(1.0);
    }

    Ok(best_ratio.max(1.0)) // guarantee ≥ 1.0 despite floating-point noise
}

/// Axis-aligned length-width diagnostic: long_side / short_side of the AABB.
///
/// This helper is intentionally separate from [`length_width_ratio`]. The
/// production compactness metric uses the rotation-invariant minimum bounding
/// rectangle; AABB is exposed only for diagnostics and paper evidence about
/// orientation dependence.
pub fn axis_aligned_length_width_ratio(polygon: &Polygon<f64>) -> Result<f64, CompactnessError> {
    let area = polygon.unsigned_area();
    if area == 0.0 {
        return Err(CompactnessError::EmptyGeometry);
    }

    let rect = polygon
        .bounding_rect()
        .ok_or(CompactnessError::EmptyGeometry)?;
    let width = rect.max().x - rect.min().x;
    let height = rect.max().y - rect.min().y;
    if width <= 0.0 || height <= 0.0 {
        return Err(CompactnessError::EmptyGeometry);
    }

    Ok((width.max(height) / width.min(height)).max(1.0))
}

/// Population-Weighted Compactness (Moment of Inertia).
///
/// PWC = Σᵢ popᵢ · d(district_centroid, tractᵢ)² / total_pop
///
/// This measures how spread out the population is relative to the district
/// centroid. Lower values indicate more compact population distribution.
///
/// Units: square metres (when centroids are in projected CRS metres).
///
/// This function does NOT appear in `all_metrics()` because it requires
/// tract-level population data that is not available from polygon geometry alone.
///
/// # Arguments
/// * `centroids` — (lon, lat) or (x, y) coordinate of each tract centroid
/// * `populations` — population count for each tract
/// * `district_centroid` — (x, y) of the district centroid
///
/// Returns 0.0 if there are no tracts or total population is zero.
pub fn population_weighted_compactness(
    centroids: &[(f64, f64)],
    populations: &[u64],
    district_centroid: (f64, f64),
) -> f64 {
    assert_eq!(
        centroids.len(),
        populations.len(),
        "centroids and populations must have the same length"
    );

    let total_pop: u64 = populations.iter().sum();
    if total_pop == 0 {
        return 0.0;
    }

    let (cx, cy) = district_centroid;
    let weighted_sum: f64 = centroids
        .iter()
        .zip(populations.iter())
        .map(|(&(x, y), &pop)| {
            let dx = x - cx;
            let dy = y - cy;
            let d2 = dx * dx + dy * dy;
            d2 * pop as f64
        })
        .sum();

    weighted_sum / total_pop as f64
}

/// Compute all five geometry-based metrics for a district polygon.
pub fn all_metrics(
    district: usize,
    polygon: &Polygon<f64>,
) -> Result<CompactnessMetrics, CompactnessError> {
    let (pp, perimeter) = polsby_popper(polygon)?;
    let reock_score = reock(polygon)?;
    let chr = convex_hull_ratio(polygon)?;
    let schwartz = schwartzberg(polygon)?;
    let lw = length_width_ratio(polygon)?;
    let area = polygon.unsigned_area();
    Ok(CompactnessMetrics {
        district,
        polsby_popper: pp,
        reock: reock_score,
        convex_hull_ratio: chr,
        schwartzberg: schwartz,
        length_width_ratio: lw,
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
    polygon
        .exterior()
        .coords()
        .map(|coord| {
            let dx = coord.x - centroid.x();
            let dy = coord.y - centroid.y();
            (dx * dx + dy * dy).sqrt()
        })
        .fold(0.0_f64, f64::max)
}

fn exterior_points(polygon: &Polygon<f64>) -> Vec<Coord<f64>> {
    let mut points: Vec<Coord<f64>> = polygon.exterior().coords().cloned().collect();
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    points
}

fn smallest_enclosing_circle(points: &[Coord<f64>]) -> BoundingCircle {
    if points.len() == 1 {
        return BoundingCircle {
            center_x: points[0].x,
            center_y: points[0].y,
            radius: 0.0,
        };
    }

    let mut best = BoundingCircle {
        center_x: 0.0,
        center_y: 0.0,
        radius: f64::INFINITY,
    };
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let circle = circle_from_diameter(points[i], points[j]);
            if contains_all(&circle, points) && circle.radius < best.radius {
                best = circle;
            }
        }
    }
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            for k in (j + 1)..points.len() {
                if let Some(circle) = circle_from_three_points(points[i], points[j], points[k]) {
                    if contains_all(&circle, points) && circle.radius < best.radius {
                        best = circle;
                    }
                }
            }
        }
    }
    best
}

fn circle_from_diameter(a: Coord<f64>, b: Coord<f64>) -> BoundingCircle {
    let center_x = (a.x + b.x) / 2.0;
    let center_y = (a.y + b.y) / 2.0;
    BoundingCircle {
        center_x,
        center_y,
        radius: distance(
            Coord {
                x: center_x,
                y: center_y,
            },
            a,
        ),
    }
}

fn circle_from_three_points(a: Coord<f64>, b: Coord<f64>, c: Coord<f64>) -> Option<BoundingCircle> {
    let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
    if d.abs() < 1e-12 {
        return None;
    }
    let a2 = a.x * a.x + a.y * a.y;
    let b2 = b.x * b.x + b.y * b.y;
    let c2 = c.x * c.x + c.y * c.y;
    let center_x = (a2 * (b.y - c.y) + b2 * (c.y - a.y) + c2 * (a.y - b.y)) / d;
    let center_y = (a2 * (c.x - b.x) + b2 * (a.x - c.x) + c2 * (b.x - a.x)) / d;
    let center = Coord {
        x: center_x,
        y: center_y,
    };
    Some(BoundingCircle {
        center_x,
        center_y,
        radius: distance(center, a),
    })
}

fn contains_all(circle: &BoundingCircle, points: &[Coord<f64>]) -> bool {
    let center = Coord {
        x: circle.center_x,
        y: circle.center_y,
    };
    points
        .iter()
        .all(|&point| distance(center, point) <= circle.radius + 1e-9)
}

fn distance(a: Coord<f64>, b: Coord<f64>) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::LineString;

    /// A perfect circle approximated as a 360-gon.
    fn circle_polygon(cx: f64, cy: f64, r: f64) -> Polygon<f64> {
        let n = 360usize;
        let coords: Vec<Coord<f64>> = (0..=n)
            .map(|i| {
                let theta = 2.0 * PI * i as f64 / n as f64;
                Coord {
                    x: cx + r * theta.cos(),
                    y: cy + r * theta.sin(),
                }
            })
            .collect();
        Polygon::new(LineString::new(coords), vec![])
    }

    fn unit_square(origin_x: f64, origin_y: f64) -> Polygon<f64> {
        // 1000m × 1000m square in projected coordinates
        let x0 = origin_x;
        let y0 = origin_y;
        let s = 1000.0_f64;
        Polygon::new(
            LineString::new(vec![
                Coord { x: x0, y: y0 },
                Coord { x: x0 + s, y: y0 },
                Coord {
                    x: x0 + s,
                    y: y0 + s,
                },
                Coord { x: x0, y: y0 + s },
                Coord { x: x0, y: y0 },
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
        assert!(
            (pp - expected).abs() < 1e-6,
            "square PP should be π/4={expected:.6}, got {pp:.6}"
        );
    }

    #[test]
    fn test_pp_perimeter_returned_correctly() {
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let (_, perimeter) = polsby_popper(&poly).unwrap();
        // 4 × 1000m = 4000m
        assert!(
            (perimeter - 4000.0).abs() < 0.001,
            "perimeter should be 4000m, got {perimeter}"
        );
    }

    #[test]
    fn test_pp_empty_geometry_error() {
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        assert!(matches!(
            polsby_popper(&empty),
            Err(CompactnessError::EmptyGeometry)
        ));
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
        assert!(
            (r - expected).abs() < 0.001,
            "square Reock should be 2/π={expected:.6}, got {r:.6}"
        );
    }

    #[test]
    fn test_reock_empty_error() {
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        assert!(matches!(
            reock(&empty),
            Err(CompactnessError::EmptyGeometry)
        ));
    }

    // ── Convex Hull Ratio ────────────────────────────────────────────────────

    #[test]
    fn test_convex_hull_ratio_convex_polygon_is_1() {
        // A square is already convex — hull ratio = 1
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let chr = convex_hull_ratio(&poly).unwrap();
        assert!(
            (chr - 1.0).abs() < 1e-6,
            "square CHR should be 1.0, got {chr:.6}"
        );
    }

    #[test]
    fn test_convex_hull_ratio_l_shape() {
        // L-shaped polygon: area < convex hull area → CHR < 1
        let l_shape = Polygon::new(
            LineString::new(vec![
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_000.0,
                },
                Coord {
                    x: 1_002_000.0,
                    y: 1_000_000.0,
                },
                Coord {
                    x: 1_002_000.0,
                    y: 1_001_000.0,
                },
                Coord {
                    x: 1_001_000.0,
                    y: 1_001_000.0,
                },
                Coord {
                    x: 1_001_000.0,
                    y: 1_002_000.0,
                },
                Coord {
                    x: 1_000_000.0,
                    y: 1_002_000.0,
                },
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_000.0,
                },
            ]),
            vec![],
        );
        let chr = convex_hull_ratio(&l_shape).unwrap();
        assert!(chr < 1.0, "L-shape CHR should be < 1.0, got {chr:.6}");
        assert!(chr > 0.5, "L-shape CHR should be reasonable, got {chr:.6}");
    }

    // ── Schwartzberg ─────────────────────────────────────────────────────────

    #[test]
    fn test_schwartzberg_circle_approaches_1() {
        // Circle: PP → 1.0, so S = 1/√1 = 1.0
        let poly = circle_polygon(1_000_000.0, 1_000_000.0, 5000.0);
        let s = schwartzberg(&poly).unwrap();
        assert!(
            s > 0.999 && s <= 1.001,
            "circle Schwartzberg should be ~1.0, got {s:.6}"
        );
    }

    #[test]
    fn test_schwartzberg_square_is_2_over_sqrt_pi() {
        // Square: PP = π/4, so S = 1/√(π/4) = 2/√π ≈ 1.1284
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let s = schwartzberg(&poly).unwrap();
        let expected = 2.0 / PI.sqrt(); // = 2/√π ≈ 1.1284
        assert!(
            (s - expected).abs() < 1e-6,
            "square Schwartzberg should be 2/√π={expected:.6}, got {s:.6}"
        );
    }

    #[test]
    fn test_schwartzberg_identity_with_pp() {
        // Identity: S = 1/√PP — verify for an arbitrary polygon (2:1 rectangle)
        let rect = Polygon::new(
            LineString::new(vec![
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_000.0,
                },
                Coord {
                    x: 1_002_000.0,
                    y: 1_000_000.0,
                },
                Coord {
                    x: 1_002_000.0,
                    y: 1_001_000.0,
                },
                Coord {
                    x: 1_000_000.0,
                    y: 1_001_000.0,
                },
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_000.0,
                },
            ]),
            vec![],
        );
        let (pp, _) = polsby_popper(&rect).unwrap();
        let s = schwartzberg(&rect).unwrap();
        let s_from_identity = 1.0 / pp.sqrt();
        assert!(
            (s - s_from_identity).abs() < 1e-12,
            "S={s:.9} must equal 1/√PP={s_from_identity:.9}"
        );
    }

    #[test]
    fn test_schwartzberg_gte_1() {
        // Schwartzberg is always ≥ 1 (equality only for a perfect circle)
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let s = schwartzberg(&poly).unwrap();
        assert!(s >= 1.0, "Schwartzberg must be >= 1.0, got {s}");
    }

    #[test]
    fn test_schwartzberg_empty_error() {
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        assert!(matches!(
            schwartzberg(&empty),
            Err(CompactnessError::EmptyGeometry)
        ));
    }

    // ── Length-Width Ratio ───────────────────────────────────────────────────

    #[test]
    fn test_lw_ratio_square_is_1() {
        // Square: MBR is the square itself → w = h → ratio = 1.0
        let poly = unit_square(1_000_000.0, 1_000_000.0);
        let lw = length_width_ratio(&poly).unwrap();
        assert!(
            (lw - 1.0).abs() < 1e-6,
            "square LW ratio should be 1.0, got {lw:.6}"
        );
    }

    #[test]
    fn test_lw_ratio_2to1_rectangle_is_2() {
        // 2000m × 1000m rectangle: MBR has sides 2000 and 1000 → ratio = 2.0
        let rect = Polygon::new(
            LineString::new(vec![
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_000.0,
                },
                Coord {
                    x: 1_002_000.0,
                    y: 1_000_000.0,
                },
                Coord {
                    x: 1_002_000.0,
                    y: 1_001_000.0,
                },
                Coord {
                    x: 1_000_000.0,
                    y: 1_001_000.0,
                },
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_000.0,
                },
            ]),
            vec![],
        );
        let lw = length_width_ratio(&rect).unwrap();
        assert!(
            (lw - 2.0).abs() < 1e-6,
            "2:1 rectangle LW ratio should be 2.0, got {lw:.6}"
        );
    }

    #[test]
    fn test_lw_ratio_circle_approx_1() {
        // Circle approximated as 32-gon → MBR is nearly square → ratio ≈ 1.0
        let n = 32usize;
        let r = 5000.0_f64;
        let coords: Vec<Coord<f64>> = (0..=n)
            .map(|i| {
                let theta = 2.0 * PI * i as f64 / n as f64;
                Coord {
                    x: 1_000_000.0 + r * theta.cos(),
                    y: 1_000_000.0 + r * theta.sin(),
                }
            })
            .collect();
        let poly = Polygon::new(LineString::new(coords), vec![]);
        let lw = length_width_ratio(&poly).unwrap();
        // A 32-gon is very close to a circle — LW should be close to 1.0
        assert!(lw < 1.02, "32-gon LW ratio should be ~1.0, got {lw:.6}");
        assert!(lw >= 1.0, "LW ratio must be >= 1.0, got {lw:.6}");
    }

    #[test]
    fn test_lw_ratio_gte_1() {
        // LW ratio is always ≥ 1
        for poly in [
            unit_square(1_000_000.0, 1_000_000.0),
            circle_polygon(1_000_000.0, 1_000_000.0, 5000.0),
        ] {
            let lw = length_width_ratio(&poly).unwrap();
            assert!(lw >= 1.0, "LW ratio must be >= 1.0, got {lw}");
        }
    }

    #[test]
    fn test_lw_ratio_empty_error() {
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        assert!(matches!(
            length_width_ratio(&empty),
            Err(CompactnessError::EmptyGeometry)
        ));
    }

    #[test]
    fn test_lw_mbr_rotation_invariant_aabb_orientation_dependent() {
        let angle = PI / 4.0;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let rotate = |x: f64, y: f64| Coord {
            x: x * cos_a - y * sin_a,
            y: x * sin_a + y * cos_a,
        };
        let rect = Polygon::new(
            LineString::new(vec![
                rotate(0.0, 0.0),
                rotate(3.0, 0.0),
                rotate(3.0, 1.0),
                rotate(0.0, 1.0),
                rotate(0.0, 0.0),
            ]),
            vec![],
        );

        let mbr_lw = length_width_ratio(&rect).unwrap();
        let aabb_lw = axis_aligned_length_width_ratio(&rect).unwrap();

        assert!(
            (mbr_lw - 3.0).abs() < 1e-12,
            "MBR LW should remain 3.0 under rotation, got {mbr_lw:.12}"
        );
        assert!(
            (aabb_lw - 1.0).abs() < 1e-12,
            "AABB LW should collapse to 1.0 for a 45-degree 3:1 rectangle, got {aabb_lw:.12}"
        );
    }

    // ── Population-Weighted Compactness ──────────────────────────────────────

    #[test]
    fn test_pwc_single_point_at_centroid_is_zero() {
        // A single tract whose centroid coincides with the district centroid
        // contributes d² = 0 → PWC = 0
        let centroids = [(1_000_000.0_f64, 1_000_000.0_f64)];
        let populations = [100u64];
        let district_centroid = (1_000_000.0, 1_000_000.0);
        let pwc = population_weighted_compactness(&centroids, &populations, district_centroid);
        assert_eq!(pwc, 0.0, "single point at centroid should give PWC=0");
    }

    #[test]
    fn test_pwc_two_equidistant_equal_pop_points() {
        // Two tracts, equal population, each at distance d from the centroid.
        // PWC = (pop·d² + pop·d²) / (2·pop) = d²
        let d = 1000.0_f64; // 1000 m
        let centroids = [
            (1_000_000.0 - d, 1_000_000.0),
            (1_000_000.0 + d, 1_000_000.0),
        ];
        let populations = [50u64, 50u64];
        let district_centroid = (1_000_000.0, 1_000_000.0);
        let pwc = population_weighted_compactness(&centroids, &populations, district_centroid);
        let expected = d * d; // = 1_000_000.0
        assert!(
            (pwc - expected).abs() < 1e-6,
            "two equal-pop equidistant points: PWC should be d²={expected}, got {pwc}"
        );
    }

    #[test]
    fn test_pwc_zero_population_returns_zero() {
        let centroids = [(1_000_000.0_f64, 1_000_000.0_f64)];
        let populations = [0u64];
        let district_centroid = (0.0, 0.0);
        let pwc = population_weighted_compactness(&centroids, &populations, district_centroid);
        assert_eq!(pwc, 0.0, "zero total population should return 0.0");
    }

    #[test]
    fn test_pwc_population_weighted_correctly() {
        // Tract A: pop=3, d=1 → contributes 3·1=3
        // Tract B: pop=1, d=3 → contributes 1·9=9
        // PWC = (3 + 9) / 4 = 3.0
        let centroids = [
            (1_000_001.0_f64, 1_000_000.0_f64), // d=1
            (1_000_003.0_f64, 1_000_000.0_f64), // d=3
        ];
        let populations = [3u64, 1u64];
        let district_centroid = (1_000_000.0, 1_000_000.0);
        let pwc = population_weighted_compactness(&centroids, &populations, district_centroid);
        let expected = 3.0;
        assert!(
            (pwc - expected).abs() < 1e-9,
            "PWC should be {expected}, got {pwc}"
        );
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
                                             // Schwartzberg for square: 2/√π ≈ 1.1284
        assert!(
            (m.schwartzberg - 2.0 / PI.sqrt()).abs() < 1e-6,
            "all_metrics schwartzberg for square, got {}",
            m.schwartzberg
        );
        // LW ratio for square: 1.0
        assert!(
            (m.length_width_ratio - 1.0).abs() < 1e-6,
            "all_metrics LW ratio for square, got {}",
            m.length_width_ratio
        );
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
        let s = schwartzberg(&poly).unwrap();
        assert!(s >= 1.0, "Schwartzberg must be >= 1.0, got {s}");
        let lw = length_width_ratio(&poly).unwrap();
        assert!(lw >= 1.0, "LW ratio must be >= 1.0, got {lw}");
    }

    #[test]
    fn test_perimeter_excludes_holes() {
        // Polygon with a hole: Python .length returns only exterior (4000m)
        // Rust must match — do NOT add hole perimeters
        let exterior = LineString::new(vec![
            Coord {
                x: 1_000_000.0,
                y: 1_000_000.0,
            },
            Coord {
                x: 1_001_000.0,
                y: 1_000_000.0,
            },
            Coord {
                x: 1_001_000.0,
                y: 1_001_000.0,
            },
            Coord {
                x: 1_000_000.0,
                y: 1_001_000.0,
            },
            Coord {
                x: 1_000_000.0,
                y: 1_000_000.0,
            },
        ]);
        let hole = LineString::new(vec![
            Coord {
                x: 1_000_250.0,
                y: 1_000_250.0,
            },
            Coord {
                x: 1_000_750.0,
                y: 1_000_250.0,
            },
            Coord {
                x: 1_000_750.0,
                y: 1_000_750.0,
            },
            Coord {
                x: 1_000_250.0,
                y: 1_000_750.0,
            },
            Coord {
                x: 1_000_250.0,
                y: 1_000_250.0,
            },
        ]);
        let poly = Polygon::new(exterior, vec![hole]);
        let (_, perimeter) = polsby_popper(&poly).unwrap();
        // Exterior only = 4000m; hole = 2000m
        // Must be 4000m, NOT 6000m
        assert!(
            (perimeter - 4000.0).abs() < 0.001,
            "perimeter should be 4000m (exterior only), got {perimeter}"
        );
    }

    // ── Python parity ────────────────────────────────────────────────────────

    #[test]
    fn test_pp_formula_matches_python_exactly() {
        // Python: score = (4 * np.pi * area) / (perimeter ** 2)
        // For a 2000×500m rectangle: A=1,000,000, P=5000
        let rect = Polygon::new(
            LineString::new(vec![
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_000.0,
                },
                Coord {
                    x: 1_002_000.0,
                    y: 1_000_000.0,
                },
                Coord {
                    x: 1_002_000.0,
                    y: 1_000_500.0,
                },
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_500.0,
                },
                Coord {
                    x: 1_000_000.0,
                    y: 1_000_000.0,
                },
            ]),
            vec![],
        );
        let area = 2000.0 * 500.0;
        let perimeter = 2.0 * (2000.0 + 500.0);
        let expected_pp = (4.0 * PI * area) / (perimeter * perimeter);
        let (pp, p) = polsby_popper(&rect).unwrap();
        assert!(
            (pp - expected_pp).abs() < 1e-9,
            "PP {pp} != expected {expected_pp}"
        );
        assert!(
            (p - perimeter).abs() < 0.001,
            "perimeter {p} != {perimeter}"
        );
    }

    // ── Scenario 27: 1-district (VT congressional) does not panic ────────────

    #[test]
    fn test_compactness_single_district_no_panic() {
        // VT congressional has 1 district = entire state.
        // all_metrics() must return Ok (not panic or divide-by-zero).
        let whole_state = unit_square(1_000_000.0, 1_000_000.0);
        let result = all_metrics(1, &whole_state);
        assert!(
            result.is_ok(),
            "single-district all_metrics must not fail: {:?}",
            result
        );
        let m = result.unwrap();
        assert_eq!(m.district, 1);
        assert!(m.polsby_popper > 0.0 && m.polsby_popper <= 1.0);
        assert!(m.reock > 0.0 && m.reock <= 1.0);
        assert!(m.convex_hull_ratio > 0.0 && m.convex_hull_ratio <= 1.0);
        assert!(m.schwartzberg >= 1.0);
        assert!(m.length_width_ratio >= 1.0);
    }

    #[test]
    fn test_compactness_empty_polygon_returns_error_not_panic() {
        // Empty polygon must return CompactnessError, not panic.
        // The analyze.rs caller skips the district and logs a warning.
        use geo_types::LineString;
        let empty = Polygon::new(LineString::new(vec![]), vec![]);
        let result = all_metrics(1, &empty);
        assert!(result.is_err(), "empty polygon must return error");
    }
}
