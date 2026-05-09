/// LODES (Longitudinal Employer-Household Dynamics) WAC (Workplace Area Characteristics)
/// data loader for economic character edge weights (B.27/M.1).
///
/// CSV format: `data/{year}/lodes/{state_name}_wac_tract.csv`
/// Columns used: geoid, c000 (total jobs), cns07, cns09, cns10, cns11 (commercial),
///               cns01, cns02, cns05, cns08 (industrial).
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// EconChar struct
// ---------------------------------------------------------------------------

/// Economic character summary for a single census tract.
///
/// All fields are fractions of total jobs (`c000`).
/// Zero-job tracts (purely residential) have all fields = 0.0.
#[derive(Debug, Clone, PartialEq)]
pub struct EconChar {
    /// Commercial intensity: (CNS07+CNS09+CNS10+CNS11) / C000
    pub commercial_intensity: f64,
    /// Industrial fraction: (CNS01+CNS02+CNS05+CNS08) / C000
    pub industrial_fraction: f64,
    /// Jobs-per-resident proxy: C000 / tract_population, capped at 10.0.
    /// Because we use C000 as both numerator and denominator proxy, this
    /// is computed as the raw C000 count normalised to [0, 10].
    pub jobs_per_resident: f64,
}

impl EconChar {
    /// The zero vector — represents a pure residential (zero-job) tract.
    pub fn zero() -> Self {
        Self {
            commercial_intensity: 0.0,
            industrial_fraction: 0.0,
            jobs_per_resident: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// cosine_similarity
// ---------------------------------------------------------------------------

/// Cosine similarity between two `EconChar` vectors, in [0.0, 1.0].
///
/// Special cases:
/// - Both zero (both residential) → 1.0 (maximally similar)
/// - One zero, one non-zero → 0.5 (neutral — one residential, one not)
/// - Otherwise: dot / (|a| * |b|), clamped to [0.0, 1.0]
pub fn cosine_similarity(a: &EconChar, b: &EconChar) -> f64 {
    let dot = a.commercial_intensity * b.commercial_intensity
        + a.industrial_fraction * b.industrial_fraction
        + a.jobs_per_resident * b.jobs_per_resident;

    let mag_a = (a.commercial_intensity.powi(2)
        + a.industrial_fraction.powi(2)
        + a.jobs_per_resident.powi(2))
    .sqrt();

    let mag_b = (b.commercial_intensity.powi(2)
        + b.industrial_fraction.powi(2)
        + b.jobs_per_resident.powi(2))
    .sqrt();

    if mag_a < 1e-15 && mag_b < 1e-15 {
        // Both zero → both residential → maximally similar
        1.0
    } else if mag_a < 1e-15 || mag_b < 1e-15 {
        // One residential, one not → neutral
        0.5
    } else {
        (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// CSV loader
// ---------------------------------------------------------------------------

/// Load LODES WAC tract data for a state/year.
///
/// Reads `data/{year}/lodes/{state_name}_wac_tract.csv`.
/// Returns `Ok(empty map)` when the file does not exist (LODES not downloaded yet).
/// Returns `Err` only on genuine parse failures.
pub fn load_lodes_wac_tract(
    state_name: &str,
    year: &str,
) -> Result<HashMap<String, EconChar>, String> {
    let path = std::path::Path::new("data")
        .join(year)
        .join("lodes")
        .join(format!("{state_name}_wac_tract.csv"));

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("LODES WAC read error {}: {e}", path.display()))?;

    let mut map = HashMap::new();
    let mut lines = content.lines();

    // Parse header
    let header_line = match lines.next() {
        Some(h) => h,
        None => return Ok(HashMap::new()),
    };
    let headers: Vec<&str> = header_line.split(',').map(str::trim).collect();

    // Locate required column indices
    let col = |name: &str| -> Result<usize, String> {
        headers
            .iter()
            .position(|&h| h.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("LODES WAC: column '{name}' not found in {}", path.display()))
    };

    let idx_geoid = col("geoid")?;
    let idx_c000  = col("c000")?;
    let idx_cns07 = col("cns07")?;
    let idx_cns09 = col("cns09")?;
    let idx_cns10 = col("cns10")?;
    let idx_cns11 = col("cns11")?;
    let idx_cns01 = col("cns01")?;
    let idx_cns02 = col("cns02")?;
    let idx_cns05 = col("cns05")?;
    let idx_cns08 = col("cns08")?;

    for (lineno, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();

        let get = |idx: usize| -> Result<f64, String> {
            fields
                .get(idx)
                .ok_or_else(|| {
                    format!(
                        "LODES WAC {}: row {} has too few columns",
                        path.display(),
                        lineno + 2
                    )
                })?
                .parse::<f64>()
                .map_err(|e| {
                    format!(
                        "LODES WAC {}: row {} column {idx} parse error: {e}",
                        path.display(),
                        lineno + 2
                    )
                })
        };

        let geoid = fields
            .get(idx_geoid)
            .ok_or_else(|| {
                format!(
                    "LODES WAC {}: row {} missing geoid",
                    path.display(),
                    lineno + 2
                )
            })?
            .to_string();

        let c000  = get(idx_c000)?;
        let cns07 = get(idx_cns07)?;
        let cns09 = get(idx_cns09)?;
        let cns10 = get(idx_cns10)?;
        let cns11 = get(idx_cns11)?;
        let cns01 = get(idx_cns01)?;
        let cns02 = get(idx_cns02)?;
        let cns05 = get(idx_cns05)?;
        let cns08 = get(idx_cns08)?;

        let (commercial_intensity, industrial_fraction, jobs_per_resident) = if c000 < 1e-10 {
            // Zero-job tract — pure residential
            (0.0, 0.0, 0.0)
        } else {
            let ci  = (cns07 + cns09 + cns10 + cns11) / c000;
            let ind = (cns01 + cns02 + cns05 + cns08) / c000;
            // Use c000 raw as proxy for jobs_per_resident, capped at 10.0
            let jpr = (c000 / 1000.0_f64.max(1.0)).min(10.0);
            (ci, ind, jpr)
        };

        map.insert(
            geoid,
            EconChar {
                commercial_intensity,
                industrial_fraction,
                jobs_per_resident,
            },
        );
    }

    Ok(map)
}

// ---------------------------------------------------------------------------
// Alignment helper
// ---------------------------------------------------------------------------

/// Align a LODES char map (keyed by GEOID string) to the adjacency node indices.
///
/// `index_to_geoid` maps node index → GEOID string (as stored in `LoadedGraph`).
/// `n` is the total number of nodes.
/// Nodes whose GEOID is absent from `chars` receive `EconChar::zero()`.
pub fn align_lodes_to_adjacency(
    chars: &HashMap<String, EconChar>,
    index_to_geoid: &HashMap<usize, String>,
    n: usize,
) -> Vec<EconChar> {
    let mut result = vec![EconChar::zero(); n];
    for (&idx, geoid) in index_to_geoid {
        if idx < n {
            if let Some(ec) = chars.get(geoid) {
                result[idx] = ec.clone();
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// L0 tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Helper to build a simple EconChar
    fn ec(ci: f64, ind: f64, jpr: f64) -> EconChar {
        EconChar {
            commercial_intensity: ci,
            industrial_fraction: ind,
            jobs_per_resident: jpr,
        }
    }

    // ── cosine_similarity invariants ─────────────────────────────────────────

    #[test]
    fn cosine_sim_both_zero_returns_one() {
        let a = EconChar::zero();
        let b = EconChar::zero();
        assert!(
            (cosine_similarity(&a, &b) - 1.0).abs() < 1e-10,
            "both-zero should be 1.0 (both residential)"
        );
    }

    #[test]
    fn cosine_sim_one_zero_returns_half() {
        let a = EconChar::zero();
        let b = ec(0.5, 0.3, 2.0);
        assert!(
            (cosine_similarity(&a, &b) - 0.5).abs() < 1e-10,
            "one zero should return 0.5 (neutral)"
        );
        // Symmetric
        assert!(
            (cosine_similarity(&b, &a) - 0.5).abs() < 1e-10,
            "one zero symmetric should also return 0.5"
        );
    }

    #[test]
    fn cosine_sim_identical_returns_one() {
        let a = ec(0.3, 0.2, 1.5);
        let b = ec(0.3, 0.2, 1.5);
        assert!(
            (cosine_similarity(&a, &b) - 1.0).abs() < 1e-10,
            "identical vectors should give 1.0"
        );
    }

    #[test]
    fn cosine_sim_orthogonal_returns_zero() {
        // Orthogonal in the first two dims: (1,0,0) vs (0,1,0)
        let a = ec(1.0, 0.0, 0.0);
        let b = ec(0.0, 1.0, 0.0);
        assert!(
            cosine_similarity(&a, &b).abs() < 1e-10,
            "orthogonal vectors should give 0.0"
        );
    }

    #[test]
    fn cosine_sim_symmetric() {
        let a = ec(0.4, 0.1, 3.0);
        let b = ec(0.1, 0.6, 1.5);
        let sim_ab = cosine_similarity(&a, &b);
        let sim_ba = cosine_similarity(&b, &a);
        assert!(
            (sim_ab - sim_ba).abs() < 1e-12,
            "cosine similarity must be symmetric: {sim_ab} vs {sim_ba}"
        );
    }

    #[test]
    fn cosine_sim_clamped_to_zero_one() {
        // All reasonable inputs should produce values in [0, 1]
        let pairs = [
            (ec(0.0, 0.0, 0.0), ec(0.0, 0.0, 0.0)),
            (ec(1.0, 0.0, 0.0), ec(0.0, 1.0, 0.0)),
            (ec(0.5, 0.5, 5.0), ec(0.5, 0.5, 5.0)),
            (ec(0.9, 0.0, 0.1), ec(0.1, 0.9, 0.0)),
        ];
        for (a, b) in &pairs {
            let s = cosine_similarity(a, b);
            assert!(s >= 0.0 && s <= 1.0, "similarity {s} out of [0,1]");
        }
    }

    // ── load_lodes_missing_file_returns_empty ─────────────────────────────────

    #[test]
    fn load_lodes_missing_file_returns_empty() {
        // File definitely does not exist in the test environment
        let result = load_lodes_wac_tract("__nonexistent_state__", "9999");
        assert!(result.is_ok(), "missing file should not be an error");
        assert!(result.unwrap().is_empty(), "missing file should return empty map");
    }

    // ── align_lodes_to_adjacency_zero_for_missing ─────────────────────────────

    #[test]
    fn align_lodes_to_adjacency_zero_for_missing() {
        let mut chars: HashMap<String, EconChar> = HashMap::new();
        chars.insert("12345678901".to_string(), ec(0.4, 0.2, 2.0));

        let mut index_to_geoid: HashMap<usize, String> = HashMap::new();
        index_to_geoid.insert(0, "12345678901".to_string()); // present
        index_to_geoid.insert(1, "99999999999".to_string()); // absent

        let result = align_lodes_to_adjacency(&chars, &index_to_geoid, 2);
        assert_eq!(result.len(), 2);

        let found = &result[0];
        assert!((found.commercial_intensity - 0.4).abs() < 1e-10, "node 0 should have ci=0.4");

        let missing = &result[1];
        assert!((missing.commercial_intensity).abs() < 1e-10, "missing node should have zero ci");
        assert!((missing.industrial_fraction).abs() < 1e-10, "missing node should have zero ind");
        assert!((missing.jobs_per_resident).abs() < 1e-10, "missing node should have zero jpr");
    }

    #[test]
    fn align_lodes_out_of_bounds_indices_ignored() {
        let mut chars: HashMap<String, EconChar> = HashMap::new();
        chars.insert("11111111111".to_string(), ec(0.5, 0.1, 1.0));

        let mut index_to_geoid: HashMap<usize, String> = HashMap::new();
        index_to_geoid.insert(0, "11111111111".to_string());
        index_to_geoid.insert(999, "11111111111".to_string()); // out of bounds for n=2

        // Should not panic
        let result = align_lodes_to_adjacency(&chars, &index_to_geoid, 2);
        assert_eq!(result.len(), 2);
        assert!((result[0].commercial_intensity - 0.5).abs() < 1e-10);
    }
}
