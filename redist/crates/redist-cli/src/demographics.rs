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
    use std::io::Write;

    fn write_tmp_csv(content: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("demographics_test_{}_{}.csv", std::process::id(), id));
        let mut f = std::fs::File::create(&tmp).expect("create tmp");
        f.write_all(content.as_bytes()).expect("write tmp");
        tmp
    }

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

    // ── Additional demographics.rs tests ─────────────────────────────────────

    #[test]
    fn test_load_demographics_basic_parse() {
        let csv = "GEOID,total_pop,white_non_hispanic\n01001020100,1000,800\n01001020200,500,100\n";
        let path = write_tmp_csv(csv);
        let demo = load_demographics(&path).expect("should parse valid CSV");
        assert_eq!(demo.len(), 2);
        // minority frac = (1000-800)/1000 = 0.20
        assert!((demo["01001020100"] - 0.20).abs() < 1e-9);
        // minority frac = (500-100)/500 = 0.80
        assert!((demo["01001020200"] - 0.80).abs() < 1e-9);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_zero_pop_returns_zero_frac() {
        let csv = "GEOID,total_pop,white_non_hispanic\n01001020100,0,0\n";
        let path = write_tmp_csv(csv);
        let demo = load_demographics(&path).expect("should parse");
        assert_eq!(demo["01001020100"], 0.0,
            "zero total_pop must produce 0.0 minority fraction");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_all_white_returns_zero_frac() {
        let csv = "GEOID,total_pop,white_non_hispanic\n01001020100,500,500\n";
        let path = write_tmp_csv(csv);
        let demo = load_demographics(&path).expect("should parse");
        assert!((demo["01001020100"] - 0.0).abs() < 1e-9,
            "all-white tract must have 0% minority fraction");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_all_minority_returns_one() {
        let csv = "GEOID,total_pop,white_non_hispanic\n01001020100,400,0\n";
        let path = write_tmp_csv(csv);
        let demo = load_demographics(&path).expect("should parse");
        assert!((demo["01001020100"] - 1.0).abs() < 1e-9,
            "all-minority tract must have 100% minority fraction");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_pads_geoid_leading_zeros() {
        // Short GEOID (10 chars) must be padded to 11.
        let csv = "GEOID,total_pop,white_non_hispanic\n1001020100,200,50\n";
        let path = write_tmp_csv(csv);
        let demo = load_demographics(&path).expect("should parse");
        assert!(demo.contains_key("01001020100"),
            "GEOID must be padded to 11 chars; got keys: {:?}", demo.keys().collect::<Vec<_>>());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_missing_column_errors() {
        // CSV missing the 'white_non_hispanic' column
        let csv = "GEOID,total_pop\n01001020100,1000\n";
        let path = write_tmp_csv(csv);
        let result = load_demographics(&path);
        assert!(result.is_err(), "missing white_non_hispanic column must produce Err");
        let msg = result.unwrap_err();
        assert!(msg.contains("white_non_hispanic"), "error must name the missing column: {msg}");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_skips_blank_lines() {
        let csv = "GEOID,total_pop,white_non_hispanic\n01001020100,1000,600\n\n01001020200,500,300\n";
        let path = write_tmp_csv(csv);
        let demo = load_demographics(&path).expect("should parse with blank line");
        assert_eq!(demo.len(), 2, "blank lines must be silently skipped");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_minority_frac_clamped_to_1() {
        // white_non_hispanic > total_pop (data error) → clamped to 1.0 (not negative)
        let csv = "GEOID,total_pop,white_non_hispanic\n01001020100,100,200\n";
        let path = write_tmp_csv(csv);
        let demo = load_demographics(&path).expect("should parse");
        // (100-200)/100 = -1.0 → clamped to 0.0 by .clamp(0.0, 1.0)
        assert!((demo["01001020100"] - 0.0).abs() < 1e-9,
            "negative minority frac must clamp to 0.0; got {}", demo["01001020100"]);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_empty_file_errors() {
        let path = write_tmp_csv("");
        let result = load_demographics(&path);
        assert!(result.is_err(), "empty file must return Err (no header)");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_align_demographics_empty_map_all_zero() {
        let demo: HashMap<String, f64> = HashMap::new();
        let mut idx = HashMap::new();
        idx.insert(0usize, "50005957100".to_string());
        idx.insert(1usize, "50005957400".to_string());
        let result = align_demographics_to_adjacency(&demo, &idx, 2);
        assert_eq!(result, vec![0.0, 0.0],
            "empty demo map must produce all-zero alignment");
    }

    #[test]
    fn test_align_demographics_zero_n_tracts_empty_vec() {
        let demo: HashMap<String, f64> = HashMap::new();
        let idx: HashMap<usize, String> = HashMap::new();
        let result = align_demographics_to_adjacency(&demo, &idx, 0);
        assert!(result.is_empty(), "n_tracts=0 must produce empty vector");
    }

    #[test]
    fn test_align_demographics_correct_length() {
        let mut demo = HashMap::new();
        demo.insert("01001020100".to_string(), 0.30);
        let idx: HashMap<usize, String> = HashMap::new(); // empty mapping
        let result = align_demographics_to_adjacency(&demo, &idx, 5);
        assert_eq!(result.len(), 5, "output length must equal n_tracts");
    }

    #[test]
    fn test_load_demographics_multiple_rows_correct_count() {
        let rows: Vec<String> = (0..10)
            .map(|i| format!("0100102{:04},1000,{}", i, 500 + i * 10))
            .collect();
        let csv = format!("GEOID,total_pop,white_non_hispanic\n{}\n", rows.join("\n"));
        let path = write_tmp_csv(&csv);
        let demo = load_demographics(&path).expect("should parse 10 rows");
        assert_eq!(demo.len(), 10, "should load all 10 rows");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_demographics_all_fracs_in_range() {
        let rows: Vec<String> = (0..5)
            .map(|i| format!("0100102{:04},{},{}", i, 1000, i * 100))
            .collect();
        let csv = format!("GEOID,total_pop,white_non_hispanic\n{}\n", rows.join("\n"));
        let path = write_tmp_csv(&csv);
        let demo = load_demographics(&path).expect("should parse");
        for (&ref geoid, &frac) in &demo {
            assert!(frac >= 0.0 && frac <= 1.0,
                "GEOID {geoid} minority frac {frac} out of [0,1]");
        }
        std::fs::remove_file(&path).ok();
    }
}
