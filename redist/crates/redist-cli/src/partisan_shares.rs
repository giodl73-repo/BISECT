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
