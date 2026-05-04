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
}
