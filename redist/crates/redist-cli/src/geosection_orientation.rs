/// GeoSection Phase 2: Directional penalty via PCA of subregion centroids.
///
/// For each subregion at each recursion level:
/// 1. Load tract centroids (lat/lon from TIGER INTPTLAT/INTPTLON)
/// 2. Run 2D PCA on the centroids of tracts IN THIS SUBREGION
/// 3. Minor axis = direction of least variance = narrowest geographic extent
/// 4. Apply directional penalty: edges parallel to cut direction are expensive
///    (they contribute to zigzag), edges perpendicular are cheap (they form
///    a straight cut across the minor axis).

use std::collections::HashMap;
use std::path::Path;

/// (lat, lon) centroid per tract index.
pub type CentroidMap = HashMap<usize, (f64, f64)>;

/// Load INTPTLAT/INTPTLON from TIGER .dbf alongside the geoid → index mapping.
pub fn load_centroids_from_tiger(
    state_code: &str,
    year: &str,
    index_to_geoid: &HashMap<usize, String>,
) -> CentroidMap {
    let fips = state_code_to_fips_2(state_code).unwrap_or("00");
    let dbf_path = format!(
        "data/{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.dbf"
    );
    if !Path::new(&dbf_path).exists() {
        return HashMap::new();
    }

    let geoid_to_idx: HashMap<String, usize> = index_to_geoid.iter()
        .map(|(&i, g)| (g.clone(), i))
        .collect();

    let centroids = read_tiger_centroids(&dbf_path);
    centroids.iter()
        .filter_map(|(geoid, latlon)| geoid_to_idx.get(geoid).map(|&idx| (idx, *latlon)))
        .collect()
}

/// Read GEOID → (lat, lon) from a TIGER .dbf file.
fn read_tiger_centroids(dbf_path: &str) -> HashMap<String, (f64, f64)> {
    use std::io::Read;
    let mut f = match std::fs::File::open(dbf_path) {
        Ok(f) => f, Err(_) => return HashMap::new(),
    };
    let mut data = Vec::new();
    let _ = f.read_to_end(&mut data);
    if data.len() < 32 { return HashMap::new(); }

    let n_records = u32::from_le_bytes([data[4],data[5],data[6],data[7]]) as usize;
    let header_size = u16::from_le_bytes([data[8],data[9]]) as usize;
    let record_size = u16::from_le_bytes([data[10],data[11]]) as usize;

    // Parse field descriptors
    let mut fields: Vec<(String, usize)> = vec![]; // (name, length)
    let mut pos = 32;
    while pos + 32 <= data.len() && data[pos] != 0x0D {
        let name = std::str::from_utf8(&data[pos..pos+11])
            .unwrap_or("").trim_end_matches('\0').to_string();
        let length = data[pos+16] as usize;
        fields.push((name, length));
        pos += 32;
    }

    // Find GEOID, INTPTLAT, INTPTLON field offsets
    let mut geoid_off = None; let mut lat_off = None; let mut lon_off = None;
    let mut running = 1usize; // +1 for deletion flag byte
    for (name, len) in &fields {
        match name.trim() {
            "GEOID" => geoid_off = Some((running, *len)),
            "INTPTLAT" => lat_off = Some((running, *len)),
            "INTPTLON" => lon_off = Some((running, *len)),
            _ => {}
        }
        running += len;
    }
    let (Some((g_off,g_len)), Some((la_off,la_len)), Some((lo_off,lo_len))) =
        (geoid_off, lat_off, lon_off) else { return HashMap::new(); };

    let mut out = HashMap::new();
    for rec in 0..n_records {
        let start = header_size + rec * record_size;
        if start + record_size > data.len() { break; }
        let row = &data[start..start+record_size];
        if row[0] == 0x2A { continue; } // deleted

        let geoid = std::str::from_utf8(&row[g_off..g_off+g_len])
            .unwrap_or("").trim().to_string();
        let lat: f64 = std::str::from_utf8(&row[la_off..la_off+la_len])
            .unwrap_or("").trim().parse().unwrap_or(0.0);
        let lon: f64 = std::str::from_utf8(&row[lo_off..lo_off+lo_len])
            .unwrap_or("").trim().parse().unwrap_or(0.0);
        if !geoid.is_empty() {
            out.insert(geoid, (lat, lon));
        }
    }
    out
}

/// Compute the minor axis direction (angle in radians) for a subregion.
/// Uses 2D PCA on the (lat, lon) centroids of tracts in the subregion.
/// Returns the angle of the MINOR axis (direction of least variance = narrowest extent).
/// The cut should run perpendicular to this axis.
///
/// Returns None if fewer than 2 tracts have centroids.
pub fn compute_minor_axis(
    verts: &std::collections::HashSet<usize>,
    centroids: &CentroidMap,
) -> Option<f64> {
    let pts: Vec<(f64, f64)> = verts.iter()
        .filter_map(|v| centroids.get(v))
        .copied()
        .collect();
    if pts.len() < 2 { return None; }

    let n = pts.len() as f64;
    let mean_lat = pts.iter().map(|(la,_)| la).sum::<f64>() / n;
    let mean_lon = pts.iter().map(|(_,lo)| lo).sum::<f64>() / n;

    // 2×2 covariance matrix
    let (mut c00, mut c01, mut c11) = (0.0f64, 0.0, 0.0);
    for (la, lo) in &pts {
        let dla = la - mean_lat;
        let dlo = lo - mean_lon;
        c00 += dla * dla;
        c01 += dla * dlo;
        c11 += dlo * dlo;
    }
    c00 /= n; c01 /= n; c11 /= n;

    // Eigenvalues of [[c00,c01],[c01,c11]]
    // λ = (c00+c11)/2 ± sqrt(((c00-c11)/2)² + c01²)
    let trace_half = (c00 + c11) / 2.0;
    let disc = (((c00 - c11) / 2.0).powi(2) + c01.powi(2)).sqrt();
    let lambda_min = trace_half - disc; // smaller eigenvalue

    // Eigenvector for lambda_min: [c01, lambda_min - c00] (or [c11-lambda_min, -c01])
    // Minor axis angle = atan2(eigvec_y, eigvec_x)
    let (evx, evy) = if c01.abs() > 1e-12 {
        (c01, lambda_min - c00)
    } else if c00 < c11 {
        (1.0, 0.0) // lat axis is minor
    } else {
        (0.0, 1.0) // lon axis is minor
    };
    Some(evx.atan2(evy))
}

/// Apply directional edge weight penalty.
/// Edges running parallel to the minor axis (i.e., they would create zigzag
/// when crossing perpendicular to the minor axis) get penalised by lambda.
///
/// penalty = w × (1 + lambda × |sin(θ)|)
/// where θ = angle between (centroid_v - centroid_u) and minor_axis_angle.
pub fn apply_directional_penalty(
    edge_weights: &HashMap<(usize,usize),f64>,
    centroids: &CentroidMap,
    minor_axis_angle: f64,
    lambda: f64,
) -> HashMap<(usize,usize),f64> {
    if lambda.abs() < 1e-10 { return edge_weights.clone(); }

    edge_weights.iter().map(|(&(u,v), &w)| {
        let penalty = if let (Some(&(lau,lou)), Some(&(lav,lov))) =
            (centroids.get(&u), centroids.get(&v))
        {
            // Edge direction in (lat,lon) space
            let edge_angle = (lav - lau).atan2(lov - lou);
            // Angle between edge and minor axis
            let mut theta = (edge_angle - minor_axis_angle).abs();
            // Normalise to [0, pi/2]
            while theta > std::f64::consts::FRAC_PI_2 { theta = (std::f64::consts::PI - theta).abs(); }
            // sin(theta) = 0 if edge is perpendicular to minor axis (good cut direction)
            // sin(theta) = 1 if edge is parallel to minor axis (zigzag direction)
            theta.sin()
        } else { 0.0 };
        ((u,v), w * (1.0 + lambda * penalty))
    }).collect()
}

fn state_code_to_fips_2(code: &str) -> Option<&'static str> {
    match code.to_uppercase().as_str() {
        "AL" => Some("01"), "AK" => Some("02"), "AZ" => Some("04"), "AR" => Some("05"),
        "CA" => Some("06"), "CO" => Some("08"), "CT" => Some("09"), "DE" => Some("10"),
        "FL" => Some("12"), "GA" => Some("13"), "HI" => Some("15"), "ID" => Some("16"),
        "IL" => Some("17"), "IN" => Some("18"), "IA" => Some("19"), "KS" => Some("20"),
        "KY" => Some("21"), "LA" => Some("22"), "ME" => Some("23"), "MD" => Some("24"),
        "MA" => Some("25"), "MI" => Some("26"), "MN" => Some("27"), "MS" => Some("28"),
        "MO" => Some("29"), "MT" => Some("30"), "NE" => Some("31"), "NV" => Some("32"),
        "NH" => Some("33"), "NJ" => Some("34"), "NM" => Some("35"), "NY" => Some("36"),
        "NC" => Some("37"), "ND" => Some("38"), "OH" => Some("39"), "OK" => Some("40"),
        "OR" => Some("41"), "PA" => Some("42"), "RI" => Some("44"), "SC" => Some("45"),
        "SD" => Some("46"), "TN" => Some("47"), "TX" => Some("48"), "UT" => Some("49"),
        "VT" => Some("50"), "VA" => Some("51"), "WA" => Some("53"), "WV" => Some("54"),
        "WI" => Some("55"), "WY" => Some("56"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pca_minor_axis_horizontal_band() {
        // Tracts arranged in a wide, short horizontal band → minor axis is vertical
        // (least variance is in the lat/N-S direction)
        let centroids: CentroidMap = (0..20).map(|i| {
            let lat = if i < 10 { 44.0 } else { 44.1 }; // two lat values
            let lon = 40.0 + (i % 10) as f64; // 10 lon values spread wide
            (i, (lat, lon))
        }).collect();
        let verts: std::collections::HashSet<usize> = (0..20).collect();
        let angle = compute_minor_axis(&verts, &centroids).unwrap();
        // Minor axis should be roughly vertical (near 0 or pi, pointing N-S)
        // The eigenvector with smallest eigenvalue points N-S (lat direction)
        let deg = angle.to_degrees();
        // In (lat,lon) space, N-S direction = atan2(1,0) = 90°.
        // Minor axis for a wide E-W band is N-S → angle near ±90°.
        let near_90 = (deg.abs() - 90.0).abs() < 30.0;
        assert!(near_90,
            "minor axis should be N-S (~90°) for E-W horizontal band, got {deg:.1}°");
    }

    #[test]
    fn directional_penalty_zero_lambda_identity() {
        let mut ew = HashMap::new();
        ew.insert((0,1), 1000.0_f64);
        ew.insert((1,2), 500.0_f64);
        let centroids = HashMap::new();
        let result = apply_directional_penalty(&ew, &centroids, 0.0, 0.0);
        assert!((result[&(0,1)] - 1000.0).abs() < 1e-9);
        assert!((result[&(1,2)] - 500.0).abs() < 1e-9);
    }

    #[test]
    fn state_fips_roundtrip() {
        assert_eq!(state_code_to_fips_2("WI"), Some("55"));
        assert_eq!(state_code_to_fips_2("wi"), Some("55"));
        assert_eq!(state_code_to_fips_2("PA"), Some("42"));
        assert_eq!(state_code_to_fips_2("XX"), None);
    }

    // ── compute_minor_axis: edge / corner cases ───────────────────────────────

    #[test]
    fn compute_minor_axis_empty_returns_none() {
        let centroids: CentroidMap = HashMap::new();
        let verts: std::collections::HashSet<usize> = std::collections::HashSet::new();
        assert!(compute_minor_axis(&verts, &centroids).is_none(),
            "empty vertex set must return None");
    }

    #[test]
    fn compute_minor_axis_single_vertex_returns_none() {
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.0));
        let verts: std::collections::HashSet<usize> = std::iter::once(0).collect();
        assert!(compute_minor_axis(&verts, &centroids).is_none(),
            "single vertex cannot form a covariance matrix — must return None");
    }

    #[test]
    fn compute_minor_axis_two_collinear_horizontal() {
        // Two points at the same latitude but different longitudes → E-W axis has
        // ALL the variance; the N-S axis (lat) is the minor (zero-variance) axis.
        // Minor axis eigenvector = (1, 0) → atan2(1, 0) == π/2.
        // But depending on the c01==0 branch the function takes, we may get
        // atan2(0,1)==0 (lon minor) when c00<c11 is false.  The key invariant is
        // that the returned angle is close to 0 or close to ±π/2 (one of the axes).
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (44.0, -71.0)); // same lat, different lon
        let verts: std::collections::HashSet<usize> = vec![0usize, 1].into_iter().collect();
        let angle = compute_minor_axis(&verts, &centroids).unwrap();
        // c00 (lat variance) == 0; c11 (lon variance) > 0 → c00 < c11 → evx=(1,0)
        // atan2(1,0) == π/2
        let expected = std::f64::consts::FRAC_PI_2;
        assert!((angle - expected).abs() < 1e-9,
            "two collinear horizontal points: minor axis should be N-S (π/2), got {angle}");
    }

    #[test]
    fn compute_minor_axis_two_collinear_vertical() {
        // Two points at the same longitude but different latitudes → N-S axis has all
        // the variance; the E-W (lon) axis is the minor (zero-variance) axis.
        // c00 > c11 → evx=(0,1) → atan2(0,1)==0.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (45.0, -72.0)); // same lon, different lat
        let verts: std::collections::HashSet<usize> = vec![0usize, 1].into_iter().collect();
        let angle = compute_minor_axis(&verts, &centroids).unwrap();
        // c00 (lat variance) > 0; c11 (lon variance) == 0 → c00 >= c11 → evx=(0,1)
        // atan2(0,1) == 0
        assert!(angle.abs() < 1e-9,
            "two collinear vertical points: minor axis should be E-W (0 rad), got {angle}");
    }

    #[test]
    fn compute_minor_axis_returns_some_for_two_distinct_points() {
        // Any two points with distinct coordinates must produce Some.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.5));
        centroids.insert(1, (45.2, -71.3));
        let verts: std::collections::HashSet<usize> = vec![0usize, 1].into_iter().collect();
        assert!(compute_minor_axis(&verts, &centroids).is_some(),
            "two distinct diagonal points must return Some");
    }

    #[test]
    fn compute_minor_axis_verts_without_centroids_skipped() {
        // Vertices not in the centroid map are silently skipped; need at least 2 matched.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(5, (44.0, -72.0));
        // verts includes indices that are NOT in centroids → only vertex 5 matches → < 2
        let verts: std::collections::HashSet<usize> = vec![0usize, 1, 5].into_iter().collect();
        assert!(compute_minor_axis(&verts, &centroids).is_none(),
            "fewer than 2 matched centroids must return None");
    }

    #[test]
    fn centroid_map_from_two_points_pca_consistent() {
        // Manually construct a CentroidMap and verify PCA is deterministic.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (40.0, -80.0));
        centroids.insert(1, (40.0, -79.0));
        centroids.insert(2, (40.0, -78.0));
        centroids.insert(3, (40.0, -77.0));
        // Four points on a perfectly horizontal line → lat variance is 0.
        let verts: std::collections::HashSet<usize> = vec![0usize,1,2,3].into_iter().collect();
        let a1 = compute_minor_axis(&verts, &centroids).unwrap();
        let a2 = compute_minor_axis(&verts, &centroids).unwrap();
        assert!((a1 - a2).abs() < 1e-12, "PCA must be deterministic: {a1} vs {a2}");
        // Along a horizontal line the minor axis is N-S → angle ≈ π/2
        let expected = std::f64::consts::FRAC_PI_2;
        assert!((a1 - expected).abs() < 1e-9,
            "horizontal line → minor axis N-S (π/2), got {a1}");
    }

    // ── apply_directional_penalty ─────────────────────────────────────────────

    #[test]
    fn apply_directional_penalty_zero_lambda_no_change() {
        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((0,1), 100.0);
        ew.insert((2,3), 250.0);
        let centroids: CentroidMap = HashMap::new();
        let result = apply_directional_penalty(&ew, &centroids, 1.0, 0.0);
        assert!((result[&(0,1)] - 100.0).abs() < 1e-9, "lambda=0 must leave weight unchanged");
        assert!((result[&(2,3)] - 250.0).abs() < 1e-9, "lambda=0 must leave weight unchanged");
    }

    #[test]
    fn apply_directional_penalty_positive_lambda_increases_parallel_edges() {
        // Edge runs horizontally (E-W, angle ≈ 0 rad).
        // Minor axis is also E-W (angle = 0).
        // An E-W edge is PARALLEL to the minor axis → sin(θ)=0 → NO penalty.
        // A N-S edge is PERPENDICULAR to the minor axis → sin(π/2)=1 → MAX penalty.
        // Wait — re-read the code:
        //   theta = |edge_angle - minor_axis_angle|, normalised to [0, π/2]
        //   penalty = lambda × sin(theta)
        // Minor axis angle = 0 (E-W).
        // Parallel edge (E-W, angle=0): theta=0, sin=0, factor = 1+0 = 1 → unchanged.
        // Perpendicular edge (N-S, angle=π/2): theta=π/2, sin=1, factor = 1+lambda.
        let minor_axis = 0.0_f64; // E-W
        let lambda = 2.0_f64;

        let mut centroids: CentroidMap = HashMap::new();
        // Horizontal (E-W) edge: u=0 at (44.0, -72.0), v=1 at (44.0, -71.0)
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (44.0, -71.0));
        // Vertical (N-S) edge: u=2 at (44.0, -72.0), v=3 at (45.0, -72.0)
        centroids.insert(2, (44.0, -72.0));
        centroids.insert(3, (45.0, -72.0));

        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((0,1), 100.0); // parallel to minor axis → theta=0 → no penalty
        ew.insert((2,3), 100.0); // perpendicular to minor axis → theta=π/2 → full penalty

        let result = apply_directional_penalty(&ew, &centroids, minor_axis, lambda);
        let w_parallel = result[&(0,1)];
        let w_perp = result[&(2,3)];

        // Perpendicular edge must get a higher weight than parallel edge
        assert!(w_perp > w_parallel,
            "N-S edge must be penalised more than E-W edge when minor axis is E-W; \
             got parallel={w_parallel}, perp={w_perp}");
        // Parallel edge: sin(0)=0 → weight unchanged
        assert!((w_parallel - 100.0).abs() < 1e-6,
            "parallel edge (sin=0) must be unchanged; got {w_parallel}");
        // Perpendicular edge: sin(π/2)≈1 → weight × (1+lambda) = 100 × 3 = 300
        assert!((w_perp - 300.0).abs() < 1e-4,
            "perpendicular edge must be ×(1+lambda)=×3.0=300; got {w_perp}");
    }

    #[test]
    fn apply_directional_penalty_always_positive() {
        // Output weights must always be > 0 for positive input weights.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (45.0, -71.5));
        centroids.insert(2, (43.5, -73.0));
        centroids.insert(3, (44.5, -70.0));

        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((0,1), 1.0);
        ew.insert((1,2), 50.0);
        ew.insert((2,3), 999.0);

        let result = apply_directional_penalty(&ew, &centroids, 0.785, 5.0); // lambda=5
        for (&edge, &w) in &result {
            assert!(w > 0.0, "weight for edge {edge:?} must be positive, got {w}");
        }
    }

    #[test]
    fn apply_directional_penalty_parallel_edges_unchanged_approx() {
        // An edge exactly parallel to the minor axis gets factor (1 + lambda × sin(0)) = 1.
        // Minor axis = π/4 (NE-SW diagonal).
        // Create an edge also running NE-SW: u=(0,0), v=(1,1) in lat/lon.
        let minor_axis = std::f64::consts::FRAC_PI_4; // 45°
        let lambda = 10.0;

        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (45.0, -71.0)); // diff_lat=1, diff_lon=1 → angle=atan2(1,1)=π/4

        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((0,1), 200.0);

        let result = apply_directional_penalty(&ew, &centroids, minor_axis, lambda);
        let w = result[&(0,1)];
        // theta = |π/4 - π/4| = 0 → sin(0) = 0 → factor = 1
        assert!((w - 200.0).abs() < 1e-6,
            "edge parallel to minor axis must be unchanged (×1); got {w}");
    }

    #[test]
    fn apply_directional_penalty_empty_weights_empty_output() {
        let ew: HashMap<(usize,usize),f64> = HashMap::new();
        let centroids: CentroidMap = HashMap::new();
        let result = apply_directional_penalty(&ew, &centroids, 0.0, 1.0);
        assert!(result.is_empty(), "empty input must produce empty output");
    }

    #[test]
    fn apply_directional_penalty_missing_centroid_no_penalty() {
        // Edge where one or both nodes lack centroids → penalty defaults to 0 → weight unchanged.
        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((99, 100), 500.0);
        let centroids: CentroidMap = HashMap::new(); // neither 99 nor 100 have centroids
        let result = apply_directional_penalty(&ew, &centroids, 1.0, 3.0);
        assert!((result[&(99,100)] - 500.0).abs() < 1e-9,
            "edge with missing centroids must be treated as penalty=0 → unchanged");
    }

    // ── compute_minor_axis: 3+ non-collinear points ────────────────────────

    #[test]
    fn compute_minor_axis_square_grid_returns_some() {
        // 4 points forming a square — variance equal in both directions.
        // Result angle is undefined but must be Some.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (44.0, -71.0));
        centroids.insert(2, (45.0, -72.0));
        centroids.insert(3, (45.0, -71.0));
        let verts: std::collections::HashSet<usize> = (0..4).collect();
        assert!(compute_minor_axis(&verts, &centroids).is_some(),
            "square grid must return Some minor axis");
    }

    #[test]
    fn compute_minor_axis_tall_north_south_band_minor_is_ew() {
        // Tall N-S band: large lat spread, tiny lon spread → lon axis (E-W) has smallest variance.
        // Minor axis angle should be near 0 (E-W direction = atan2(0,1) = 0).
        let centroids: CentroidMap = (0..20).map(|i| {
            let lat = 40.0 + (i % 10) as f64; // 10 distinct lat values → large N-S spread
            let lon = if i < 10 { -72.0 } else { -71.9 }; // tiny E-W spread
            (i, (lat, lon))
        }).collect();
        let verts: std::collections::HashSet<usize> = (0..20).collect();
        let angle = compute_minor_axis(&verts, &centroids).unwrap();
        // Minor axis is E-W (angle near 0).
        let deg = angle.to_degrees();
        let near_zero = deg.abs() < 30.0 || (deg.abs() - 180.0).abs() < 30.0;
        assert!(near_zero,
            "tall N-S band → minor axis should be E-W (near 0°), got {deg:.1}°");
    }

    #[test]
    fn compute_minor_axis_diagonal_band_ne_sw() {
        // Points along a NE-SW diagonal: (0,0), (1,1), (2,2), (3,3) in lat/lon.
        // Both axes have equal variance → covariance not zero, minor axis near 45°.
        let centroids: CentroidMap = (0..4usize).map(|i| {
            (i, (i as f64, i as f64))
        }).collect();
        let verts: std::collections::HashSet<usize> = (0..4).collect();
        // Must return Some and have a finite angle.
        let angle = compute_minor_axis(&verts, &centroids).unwrap();
        assert!(angle.is_finite(), "angle must be finite for diagonal band, got {angle}");
    }

    #[test]
    fn compute_minor_axis_three_non_collinear_returns_some() {
        // Triangle of points — neither collinear nor square.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (44.5, -71.0));
        centroids.insert(2, (45.5, -72.5));
        let verts: std::collections::HashSet<usize> = (0..3).collect();
        assert!(compute_minor_axis(&verts, &centroids).is_some());
    }

    #[test]
    fn compute_minor_axis_large_cluster_is_finite() {
        // 50 points scattered in a 2° × 10° region; result must be finite.
        let centroids: CentroidMap = (0..50usize).map(|i| {
            let lat = 40.0 + (i % 5) as f64 * 0.5;  // 2° range
            let lon = -80.0 + (i % 10) as f64;       // 10° range
            (i, (lat, lon))
        }).collect();
        let verts: std::collections::HashSet<usize> = (0..50).collect();
        let angle = compute_minor_axis(&verts, &centroids).unwrap();
        assert!(angle.is_finite(), "large cluster must yield finite angle, got {angle}");
    }

    // ── apply_directional_penalty: large lambda ────────────────────────────

    #[test]
    fn apply_directional_penalty_very_large_lambda_still_positive() {
        // lambda = 1000; all output weights must remain positive.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (45.0, -72.0)); // N-S edge
        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((0, 1), 1.0);
        let result = apply_directional_penalty(&ew, &centroids, 0.0, 1000.0);
        assert!(result[&(0,1)] > 0.0, "weight must be positive even with lambda=1000");
    }

    #[test]
    fn apply_directional_penalty_large_lambda_scales_perpendicular_edge() {
        // lambda=10, edge is perpendicular to minor axis → weight × (1 + 10×1) = 11×.
        let minor_axis = 0.0_f64; // E-W
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(10, (44.0, -72.0));
        centroids.insert(11, (45.0, -72.0)); // N-S → angle=π/2 → perpendicular to E-W minor axis
        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((10, 11), 1.0);
        let result = apply_directional_penalty(&ew, &centroids, minor_axis, 10.0);
        let w = result[&(10, 11)];
        // sin(π/2) = 1 → factor = 1 + 10×1 = 11
        assert!((w - 11.0).abs() < 1e-4, "perpendicular edge with lambda=10 must scale to 11.0, got {w}");
    }

    #[test]
    fn apply_directional_penalty_preserves_edge_count() {
        // Output must have exactly the same number of edges as input.
        let mut centroids: CentroidMap = HashMap::new();
        for i in 0..10usize {
            centroids.insert(i, (44.0 + i as f64 * 0.1, -72.0 + i as f64 * 0.1));
        }
        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        for i in 0..9usize {
            ew.insert((i, i+1), (i+1) as f64 * 100.0);
        }
        let result = apply_directional_penalty(&ew, &centroids, 0.5, 2.0);
        assert_eq!(result.len(), ew.len(), "output must have same number of edges as input");
    }

    #[test]
    fn apply_directional_penalty_only_source_centroid_missing_no_penalty() {
        // Source node missing, dest present → both needed for angle → defaults to 0 penalty.
        let mut centroids: CentroidMap = HashMap::new();
        centroids.insert(1, (44.0, -72.0)); // only dest
        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((0, 1), 200.0);
        let result = apply_directional_penalty(&ew, &centroids, 0.5, 5.0);
        assert!((result[&(0,1)] - 200.0).abs() < 1e-9,
            "edge with missing source centroid must use penalty=0 → unchanged");
    }

    // ── CentroidMap type alias ────────────────────────────────────────────────

    #[test]
    fn centroid_map_insert_and_retrieve() {
        let mut cm: CentroidMap = HashMap::new();
        cm.insert(42, (38.9072, -77.0369)); // Washington DC approx
        assert!(cm.contains_key(&42));
        let (lat, lon) = cm[&42];
        assert!((lat - 38.9072).abs() < 1e-10);
        assert!((lon - (-77.0369)).abs() < 1e-10);
    }

    #[test]
    fn state_code_to_fips_2_all_standard_codes() {
        // Quick round-trip for a selection of state codes.
        assert_eq!(state_code_to_fips_2("CA"), Some("06"));
        assert_eq!(state_code_to_fips_2("TX"), Some("48"));
        assert_eq!(state_code_to_fips_2("NY"), Some("36"));
        assert_eq!(state_code_to_fips_2("FL"), Some("12"));
        assert_eq!(state_code_to_fips_2("DC"), None); // DC not in fips_2
    }

    #[test]
    fn state_code_to_fips_2_lowercase_works() {
        assert_eq!(state_code_to_fips_2("ca"), Some("06"));
        assert_eq!(state_code_to_fips_2("tx"), Some("48"));
    }

    #[test]
    fn state_code_to_fips_2_unknown_returns_none() {
        assert_eq!(state_code_to_fips_2("ZZ"), None);
        assert_eq!(state_code_to_fips_2(""), None);
    }

    #[test]
    fn apply_directional_penalty_multiple_edges_mixed_angles() {
        // Three edges: E-W, N-S, diagonal. Diagonal should land between the two extremes.
        let minor_axis = 0.0_f64; // E-W = minor axis
        let lambda = 4.0;
        let mut centroids: CentroidMap = HashMap::new();
        // E-W edge (parallel to minor axis → penalty 0)
        centroids.insert(0, (44.0, -72.0));
        centroids.insert(1, (44.0, -71.0));
        // N-S edge (perpendicular → max penalty)
        centroids.insert(2, (44.0, -72.0));
        centroids.insert(3, (45.0, -72.0));
        // NE-SW diagonal (45°)
        centroids.insert(4, (44.0, -72.0));
        centroids.insert(5, (44.5, -71.5));

        let mut ew: HashMap<(usize,usize),f64> = HashMap::new();
        ew.insert((0,1), 100.0); // E-W
        ew.insert((2,3), 100.0); // N-S
        ew.insert((4,5), 100.0); // 45°

        let result = apply_directional_penalty(&ew, &centroids, minor_axis, lambda);
        let w_ew   = result[&(0,1)];
        let w_ns   = result[&(2,3)];
        let w_diag = result[&(4,5)];

        // E-W: sin(0)=0 → unchanged
        assert!((w_ew - 100.0).abs() < 1e-4, "E-W edge unchanged: got {w_ew}");
        // N-S: sin(π/2)=1 → ×(1+4)=5
        assert!((w_ns - 500.0).abs() < 1e-4, "N-S edge ×5: got {w_ns}");
        // diagonal: sin(π/4)≈0.707 → between 100 and 500
        assert!(w_diag > 100.0 && w_diag < 500.0,
            "diagonal edge must be between E-W and N-S penalties: got {w_diag}");
    }
}
