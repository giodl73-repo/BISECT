/// Demographics CSV reader for VRA mode.
///
/// Reads `data/{year}/demographics/{state_name}_demographics_{year}.csv`
/// and returns per-tract minority fractions, aligned to adjacency index order.
///
/// Minority fraction = (total_pop - white_non_hispanic) / total_pop
/// This matches Python vra_utils.py which uses "all non-white as minority"
/// to support coalition districts (Black + Hispanic + Asian).
use std::collections::HashMap;
use std::path::Path;

/// Load minority fraction per GEOID from a demographics CSV.
/// Returns HashMap<GEOID (11-char), pct_minority>.
pub fn load_demographics(csv_path: &Path) -> Result<HashMap<String, f64>, String> {
    let content = std::fs::read_to_string(csv_path)
        .map_err(|e| format!("cannot read {}: {e}", csv_path.display()))?;

    let mut lines = content.lines();
    let header = lines.next().ok_or("empty demographics file")?;
    let cols: Vec<&str> = header.split(',').collect();

    let col = |name: &str| -> Result<usize, String> {
        cols.iter().position(|&c| c.trim() == name)
            .ok_or_else(|| format!("column '{name}' not found in header: {}", header))
    };

    let i_geoid = col("GEOID")?;
    let i_total = col("total_pop")?;
    let i_white = col("white_non_hispanic")?;

    let mut result = HashMap::new();
    for (line_num, line) in lines.enumerate() {
        if line.trim().is_empty() { continue; }
        let fields: Vec<&str> = line.split(',').collect();

        let geoid = format!("{:0>11}", fields.get(i_geoid).unwrap_or(&"").trim());
        let total: f64 = fields.get(i_total).and_then(|f| f.trim().parse().ok()).unwrap_or(0.0);
        let white: f64 = fields.get(i_white).and_then(|f| f.trim().parse().ok()).unwrap_or(0.0);

        if total <= 0.0 {
            result.insert(geoid, 0.0);
            continue;
        }

        let minority_frac = ((total - white) / total).clamp(0.0, 1.0);
        result.insert(geoid, minority_frac);
    }

    Ok(result)
}

/// Align per-GEOID demographics to adjacency vertex index order.
/// Returns Vec<f64> of length n_tracts; index matches adjacency vertex index.
/// Tracts with no demographic data get 0.0 (conservative: won't trigger VRA boost).
pub fn align_demographics_to_adjacency(
    demo: &HashMap<String, f64>,
    index_to_geoid: &HashMap<usize, String>,
    n_tracts: usize,
) -> Vec<f64> {
    (0..n_tracts)
        .map(|i| {
            index_to_geoid.get(&i)
                .and_then(|geoid| demo.get(geoid))
                .copied()
                .unwrap_or(0.0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demographics_vermont() {
        let path = std::path::Path::new("data/2020/demographics/vermont_demographics_2020.csv");
        if !path.exists() { return; }
        let demo = load_demographics(path).expect("should load VT demographics");
        assert_eq!(demo.len(), 193, "VT has 193 tracts");
        for (geoid, frac) in &demo {
            assert!(*frac >= 0.0 && *frac <= 1.0, "GEOID {geoid} frac {frac} out of range");
            assert_eq!(geoid.len(), 11, "GEOID must be 11 chars: {geoid}");
        }
    }

    #[test]
    fn test_demographics_missing_file_errors() {
        let result = load_demographics(std::path::Path::new("/nonexistent.csv"));
        assert!(result.is_err());
    }

    #[test]
    fn test_align_demographics_to_adjacency() {
        let mut demo = HashMap::new();
        demo.insert("50005957100".to_string(), 0.25);
        demo.insert("50005957400".to_string(), 0.60);

        let mut idx = HashMap::new();
        idx.insert(0usize, "50005957100".to_string());
        idx.insert(1usize, "50005957400".to_string());
        idx.insert(2usize, "50005999999".to_string()); // not in demo

        let result = align_demographics_to_adjacency(&demo, &idx, 3);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 0.25).abs() < 1e-9);
        assert!((result[1] - 0.60).abs() < 1e-9);
        assert_eq!(result[2], 0.0, "missing geoid should default to 0.0");
    }
}
