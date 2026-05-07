/// Partisan shares TSV loader for partisan-weighted bisection mode.
///
/// Reads a TSV file with header `geoid<TAB>dem_share` and returns
/// per-tract Democratic vote share aligned to adjacency vertex index order.
///
/// Format spec (`docs/file-formats/partisan-shares.md`):
///   - UTF-8 encoded TSV
///   - Header row required (column names not enforced; first non-blank row is data)
///   - GEOID: 11-character TIGER FIPS string with leading zeros (e.g., "01001020100")
///   - dem_share: float in [0.0, 1.0]
///   - Lines starting with `#` are comments
///
/// Plan 03 (Callais 2026-04-29) — see docs/legal/CALLAIS_REFERENCE.md.
use std::collections::HashMap;
use std::path::Path;

/// Load per-GEOID Democratic vote share from a TSV file.
/// Returns HashMap<GEOID (11-char), dem_share>.
pub fn load_partisan_shares_map(tsv_path: &Path) -> Result<HashMap<String, f64>, String> {
    let content = std::fs::read_to_string(tsv_path)
        .map_err(|e| format!("cannot read {}: {e}", tsv_path.display()))?;

    let mut result = HashMap::new();
    let mut data_started = false;
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            return Err(format!(
                "{}:{}: expected 2 tab-separated columns, got {}",
                tsv_path.display(), line_num + 1, parts.len()
            ));
        }

        // Skip header row (first data line that doesn't parse as a float in column 2)
        if !data_started {
            if parts[1].trim().parse::<f64>().is_err() {
                data_started = true;
                continue;
            }
            data_started = true;
        }

        let geoid = format!("{:0>11}", parts[0].trim());
        let share: f64 = parts[1].trim().parse()
            .map_err(|e| format!(
                "{}:{}: cannot parse dem_share '{}': {e}",
                tsv_path.display(), line_num + 1, parts[1]
            ))?;

        if !(0.0..=1.0).contains(&share) {
            return Err(format!(
                "{}:{}: dem_share {} for {} not in [0.0, 1.0]",
                tsv_path.display(), line_num + 1, share, geoid
            ));
        }

        result.insert(geoid, share);
    }

    Ok(result)
}

/// Align per-GEOID partisan shares to adjacency vertex index order.
/// Tracts with no share data default to 0.5 (swing) so they don't trigger boosting.
pub fn load_partisan_shares(
    tsv_path: &Path,
    index_to_geoid: &HashMap<usize, String>,
    n_tracts: usize,
) -> Result<Vec<f64>, String> {
    let shares = load_partisan_shares_map(tsv_path)?;

    let aligned: Vec<f64> = (0..n_tracts)
        .map(|i| {
            index_to_geoid.get(&i)
                .and_then(|geoid| shares.get(geoid))
                .copied()
                .unwrap_or(0.5)
        })
        .collect();

    Ok(aligned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(content: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("partisan_shares_test_{}_{}.tsv", std::process::id(), id));
        let mut f = std::fs::File::create(&tmp).expect("create tmp");
        f.write_all(content.as_bytes()).expect("write tmp");
        tmp
    }

    #[test]
    fn test_loads_basic_tsv() {
        let path = write_tmp("geoid\tdem_share\n01001020100\t0.42\n01001020200\t0.71\n");
        let shares = load_partisan_shares_map(&path).expect("should load");
        assert_eq!(shares.len(), 2);
        assert!((shares["01001020100"] - 0.42).abs() < 1e-9);
        assert!((shares["01001020200"] - 0.71).abs() < 1e-9);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_pads_geoid_with_leading_zeros() {
        // Caller may pass GEOIDs without leading zeros (Excel strips them);
        // loader normalizes to 11 chars.
        let path = write_tmp("geoid\tdem_share\n1001020100\t0.42\n");
        let shares = load_partisan_shares_map(&path).expect("should load");
        assert!(shares.contains_key("01001020100"), "leading zero must be added");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_rejects_out_of_range_share() {
        let path = write_tmp("geoid\tdem_share\n01001020100\t1.5\n");
        let result = load_partisan_shares_map(&path);
        assert!(result.is_err(), "share > 1.0 must error");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_rejects_unparseable_share() {
        let path = write_tmp("geoid\tdem_share\n01001020100\tabc\n");
        let result = load_partisan_shares_map(&path);
        assert!(result.is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_skips_comments_and_blank_lines() {
        let path = write_tmp("# generated by foo\ngeoid\tdem_share\n\n01001020100\t0.42\n# trailing comment\n");
        let shares = load_partisan_shares_map(&path).expect("should load");
        assert_eq!(shares.len(), 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_align_returns_swing_for_missing_geoid() {
        let path = write_tmp("geoid\tdem_share\n50005957100\t0.42\n");
        let mut idx = HashMap::new();
        idx.insert(0usize, "50005957100".to_string());
        idx.insert(1usize, "50005999999".to_string()); // not in TSV

        let aligned = load_partisan_shares(&path, &idx, 2).expect("should load");
        assert_eq!(aligned.len(), 2);
        assert!((aligned[0] - 0.42).abs() < 1e-9);
        assert!((aligned[1] - 0.5).abs() < 1e-9, "missing geoid → swing default 0.5");
        std::fs::remove_file(&path).ok();
    }
}

/// Load per-GEOID Democratic vote counts from the presidential CSV.
/// Format: geoid,dem_votes,rep_votes,...
/// Returns aligned Vec<f64> of dem_votes and a separate Vec<f64> of (dem+rep) totals.
pub fn load_dem_vote_counts(
    csv_path: &Path,
    index_to_geoid: &HashMap<usize, String>,
    n_tracts: usize,
) -> Result<(Vec<f64>, Vec<f64>), String> {
    let content = std::fs::read_to_string(csv_path)
        .map_err(|e| format!("cannot read {}: {e}", csv_path.display()))?;

    let mut dem_map: HashMap<String, f64> = HashMap::new();
    let mut total_map: HashMap<String, f64> = HashMap::new();
    let mut header_seen = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if !header_seen { header_seen = true; continue; } // skip header
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 3 { continue; }
        let geoid = cols[0].to_string();
        let dem: f64 = cols[1].parse().unwrap_or(0.0);
        let rep: f64 = cols[2].parse().unwrap_or(0.0);
        dem_map.insert(geoid.clone(), dem);
        total_map.insert(geoid, dem + rep);
    }

    let dem_votes: Vec<f64> = (0..n_tracts)
        .map(|i| index_to_geoid.get(&i).and_then(|g| dem_map.get(g)).copied().unwrap_or(0.0))
        .collect();
    let two_party: Vec<f64> = (0..n_tracts)
        .map(|i| index_to_geoid.get(&i).and_then(|g| total_map.get(g)).copied().unwrap_or(1.0))
        .collect();

    Ok((dem_votes, two_party))
}

#[cfg(test)]
mod dem_vote_tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(content: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("dem_vote_test_{}_{}.csv", std::process::id(), id));
        let mut f = std::fs::File::create(&tmp).expect("create tmp");
        f.write_all(content.as_bytes()).expect("write tmp");
        tmp
    }

    fn make_idx(pairs: &[(usize, &str)]) -> HashMap<usize, String> {
        pairs.iter().map(|&(i, g)| (i, g.to_string())).collect()
    }

    #[test]
    fn test_dem_vote_counts_basic() {
        let csv = "geoid,dem_votes,rep_votes\n01001020100,600,400\n01001020200,200,800\n";
        let path = write_tmp(csv);
        let idx = make_idx(&[(0, "01001020100"), (1, "01001020200")]);
        let (dem, total) = load_dem_vote_counts(&path, &idx, 2).expect("should parse");
        assert!((dem[0] - 600.0).abs() < 1e-9, "vertex 0 dem_votes");
        assert!((dem[1] - 200.0).abs() < 1e-9, "vertex 1 dem_votes");
        assert!((total[0] - 1000.0).abs() < 1e-9, "vertex 0 total two-party");
        assert!((total[1] - 1000.0).abs() < 1e-9, "vertex 1 total two-party");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_missing_geoid_defaults() {
        let csv = "geoid,dem_votes,rep_votes\n01001020100,600,400\n";
        let path = write_tmp(csv);
        let idx = make_idx(&[(0, "01001020100"), (1, "99999999999")]); // 1 not in CSV
        let (dem, total) = load_dem_vote_counts(&path, &idx, 2).expect("should parse");
        assert!((dem[1] - 0.0).abs() < 1e-9, "missing geoid → dem_votes=0.0");
        assert!((total[1] - 1.0).abs() < 1e-9, "missing geoid → total=1.0 (avoid div/0)");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_skips_comment_lines() {
        let csv = "# comment\ngeoid,dem_votes,rep_votes\n01001020100,300,700\n";
        let path = write_tmp(csv);
        let idx = make_idx(&[(0, "01001020100")]);
        let (dem, _) = load_dem_vote_counts(&path, &idx, 1).expect("comments must be skipped");
        assert!((dem[0] - 300.0).abs() < 1e-9);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_skips_blank_lines() {
        let csv = "geoid,dem_votes,rep_votes\n\n01001020100,400,600\n";
        let path = write_tmp(csv);
        let idx = make_idx(&[(0, "01001020100")]);
        let (dem, _) = load_dem_vote_counts(&path, &idx, 1).expect("blank lines must be skipped");
        assert!((dem[0] - 400.0).abs() < 1e-9);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_missing_file_errors() {
        let idx: HashMap<usize, String> = HashMap::new();
        let result = load_dem_vote_counts(
            std::path::Path::new("/nonexistent_votes.csv"), &idx, 0
        );
        assert!(result.is_err(), "missing file must return Err");
    }

    #[test]
    fn test_dem_vote_counts_correct_output_length() {
        let csv = "geoid,dem_votes,rep_votes\n01001020100,500,500\n";
        let path = write_tmp(csv);
        let idx: HashMap<usize, String> = HashMap::new(); // empty mapping
        let (dem, total) = load_dem_vote_counts(&path, &idx, 5).expect("should parse");
        assert_eq!(dem.len(), 5, "dem_votes vector length must equal n_tracts");
        assert_eq!(total.len(), 5, "two_party vector length must equal n_tracts");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_zero_n_tracts_empty_vecs() {
        let csv = "geoid,dem_votes,rep_votes\n01001020100,500,500\n";
        let path = write_tmp(csv);
        let idx: HashMap<usize, String> = HashMap::new();
        let (dem, total) = load_dem_vote_counts(&path, &idx, 0).expect("should parse");
        assert!(dem.is_empty());
        assert!(total.is_empty());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_unparseable_votes_default_zero() {
        // If a dem_votes cell is non-numeric, defaults to 0.0
        let csv = "geoid,dem_votes,rep_votes\n01001020100,abc,400\n";
        let path = write_tmp(csv);
        let idx = make_idx(&[(0, "01001020100")]);
        let (dem, total) = load_dem_vote_counts(&path, &idx, 1).expect("should parse");
        assert!((dem[0] - 0.0).abs() < 1e-9, "unparseable dem_votes → 0.0");
        assert!((total[0] - 400.0).abs() < 1e-9, "total = 0 + 400");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_header_always_skipped() {
        // The first non-comment non-blank line is always treated as header.
        // Even if it looks numeric, it must be skipped (first-row-is-header rule).
        let csv = "geoid,dem_votes,rep_votes\n01001020100,700,300\n";
        let path = write_tmp(csv);
        let idx = make_idx(&[(0, "geoid")]); // if header not skipped, "geoid" row would be data
        let (dem, _) = load_dem_vote_counts(&path, &idx, 1).expect("should parse");
        // vertex 0 maps to "geoid" which is NOT in dem_map (header was skipped), so dem=0.0
        assert!((dem[0] - 0.0).abs() < 1e-9,
            "header row must be skipped; should not appear as a data row");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_total_is_dem_plus_rep() {
        let csv = "geoid,dem_votes,rep_votes\n01001020100,350,450\n";
        let path = write_tmp(csv);
        let idx = make_idx(&[(0, "01001020100")]);
        let (dem, total) = load_dem_vote_counts(&path, &idx, 1).expect("should parse");
        assert!((dem[0] - 350.0).abs() < 1e-9);
        assert!((total[0] - 800.0).abs() < 1e-9, "total must be 350+450=800");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dem_vote_counts_multiple_tracts_correct_alignment() {
        let csv = "geoid,dem_votes,rep_votes\n\
            01001020100,600,400\n\
            01001020200,100,900\n\
            01001020300,500,500\n";
        let path = write_tmp(csv);
        let idx = make_idx(&[
            (0, "01001020300"), // note: reversed order
            (1, "01001020100"),
            (2, "01001020200"),
        ]);
        let (dem, total) = load_dem_vote_counts(&path, &idx, 3).expect("should parse");
        // Vertex 0 → "01001020300" → dem=500, total=1000
        assert!((dem[0] - 500.0).abs() < 1e-9);
        assert!((total[0] - 1000.0).abs() < 1e-9);
        // Vertex 1 → "01001020100" → dem=600, total=1000
        assert!((dem[1] - 600.0).abs() < 1e-9);
        // Vertex 2 → "01001020200" → dem=100, total=1000
        assert!((dem[2] - 100.0).abs() < 1e-9);
        std::fs::remove_file(&path).ok();
    }
}
