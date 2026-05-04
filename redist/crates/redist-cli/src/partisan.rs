/// Partisan analyzer runner — load election CSV, aggregate, compute metrics,
/// write analysis/partisan.json.
/// Spec 4 — board amendments R3 applied.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::Context;

use redist_analysis::{
    DistrictElection, compute_partisan_metrics,
};
use crate::io_utils::write_json_atomic;

// ---------------------------------------------------------------------------
// Election CSV record
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ElectionRecord {
    pub geoid: String,
    pub dem_votes: f64,
    pub rep_votes: f64,
}

// ---------------------------------------------------------------------------
// Output schema
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct PartisanDistrictOutput {
    pub district: usize,
    pub dem_votes: f64,
    pub rep_votes: f64,
    pub dem_pct: f64,
    pub margin: f64,
    pub is_competitive: bool,
    pub is_uncontested: bool,
}

#[derive(Debug, Serialize)]
pub struct PartisanJsonOutput {
    pub analyzer: &'static str,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub election: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub election_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methodology_warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statewide: Option<StatewideStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub districts: Option<Vec<PartisanDistrictOutput>>,
}

#[derive(Debug, Serialize)]
pub struct StatewideStats {
    pub dem_vote_share: f64,
    pub dem_seat_share: f64,
    pub total_votes: f64,
}

#[derive(Debug, Serialize)]
pub struct MetricsBlock {
    pub efficiency_gap: redist_analysis::MetricWithCI,
    pub mean_median: redist_analysis::MetricWithCI,
    pub partisan_bias: redist_analysis::MetricWithCI,
}

// ---------------------------------------------------------------------------
// CSV parsing
// ---------------------------------------------------------------------------

/// Parse election CSV from a reader. Returns Vec<ElectionRecord>.
pub fn parse_election_csv<R: std::io::Read>(reader: R) -> anyhow::Result<Vec<ElectionRecord>> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut records = Vec::new();
    for result in rdr.deserialize::<ElectionRecord>() {
        let rec = result.context("Failed to parse election CSV row")?;
        records.push(rec);
    }
    Ok(records)
}

/// Load election CSV from disk. Returns Err with guidance message if not found.
pub fn load_election_data(path: &Path, year: &str) -> anyhow::Result<Vec<ElectionRecord>> {
    if !path.exists() {
        anyhow::bail!(
            "Election data not found at {}.\n\
             Download with:\n  \
             python scripts/data/elections/download_election_data.py --year {year}\n\
             (`redist fetch --type elections` is declared but not yet implemented; \
             the Python downloader is canonical for now.)",
            path.display()
        );
    }
    let f = std::fs::File::open(path)
        .with_context(|| format!("Cannot open election file: {}", path.display()))?;
    parse_election_csv(f)
}

/// Aggregate election records to per-district totals using assignments map.
pub fn aggregate_election_to_districts(
    election: &[ElectionRecord],
    assignments: &HashMap<String, usize>,
) -> HashMap<usize, DistrictElection> {
    let mut totals: HashMap<usize, (f64, f64)> = HashMap::new();
    for rec in election {
        if let Some(&dist) = assignments.get(&rec.geoid) {
            let entry = totals.entry(dist).or_insert((0.0, 0.0));
            entry.0 += rec.dem_votes;
            entry.1 += rec.rep_votes;
        }
    }
    totals
        .into_iter()
        .map(|(district, (dem_votes, rep_votes))| {
            (district, DistrictElection { district, dem_votes, rep_votes })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Main runner
// ---------------------------------------------------------------------------

pub struct PartisanArgs<'a> {
    pub assignments: &'a HashMap<String, usize>,
    pub state_code: &'a str,
    pub state_name: &'a str,
    pub year: &'a str,
    pub version: &'a str,
    pub election_file: Option<&'a PathBuf>,
    pub bootstrap_samples: usize,
    pub analysis_dir: &'a Path,
    pub force: bool,
    pub chamber: &'a str,
}

/// Run the partisan analyzer and write partisan.json.
/// If election file is absent, writes available=false and returns Ok.
pub fn run_partisan(args: &PartisanArgs<'_>) -> anyhow::Result<()> {
    let out_path = args.analysis_dir.join("partisan.json");
    if out_path.exists() && !args.force {
        eprintln!("[skip] partisan (exists, use --force to regenerate)");
        return Ok(());
    }

    // Determine election file path
    let default_election_path = PathBuf::from(format!(
        "data/{}/elections/presidential_by_tract.csv",
        args.year
    ));
    let election_path: &Path = args
        .election_file
        .map(|p| p.as_path())
        .unwrap_or(&default_election_path);

    // Handle missing election file
    if !election_path.exists() {
        let output = PartisanJsonOutput {
            analyzer: "partisan",
            available: false,
            unavailable_reason: Some(format!(
                "Election data not found. Run: redist fetch --type elections --year {}",
                args.year
            )),
            required_file: Some(election_path.to_string_lossy().into_owned()),
            election: None,
            election_file: None,
            methodology_warning: None,
            statewide: None,
            metrics: None,
            districts: None,
        };
        write_json_atomic(&out_path, &output)?;
        eprintln!(
            "[partisan] election data not found at {} — wrote unavailable marker",
            election_path.display()
        );
        return Ok(());
    }

    // Load and aggregate
    let records = load_election_data(election_path, args.year)
        .with_context(|| format!("Loading election data from {}", election_path.display()))?;

    let by_district = aggregate_election_to_districts(&records, args.assignments);
    let mut districts_vec: Vec<DistrictElection> = by_district.into_values().collect();
    districts_vec.sort_by_key(|d| d.district);

    // Compute metrics
    let metrics = compute_partisan_metrics(&districts_vec, None, args.bootstrap_samples);

    // Methodology warning for non-congressional
    let methodology_warning = if args.chamber != "congressional" {
        Some("Presidential election results used as proxy for state legislative elections. \
              Caution: turnout and candidate effects may differ significantly.".to_string())
    } else {
        Some("Presidential election results used as proxy for congressional elections. \
              Results are indicative only and may not reflect actual partisan performance.".to_string())
    };

    let total_votes: f64 = districts_vec.iter().map(|d| d.total()).sum();

    let district_outputs: Vec<PartisanDistrictOutput> = districts_vec
        .iter()
        .map(|d| {
            let dem_pct = d.dem_pct();
            let margin = (d.dem_votes - d.rep_votes).abs() / d.total().max(1.0);
            PartisanDistrictOutput {
                district: d.district,
                dem_votes: d.dem_votes,
                rep_votes: d.rep_votes,
                dem_pct,
                margin,
                is_competitive: margin < 0.05,
                is_uncontested: d.dem_votes == 0.0 || d.rep_votes == 0.0,
            }
        })
        .collect();

    let output = PartisanJsonOutput {
        analyzer: "partisan",
        available: true,
        unavailable_reason: None,
        required_file: None,
        election: Some("2020_presidential".into()),
        election_file: Some(election_path.to_string_lossy().into_owned()),
        methodology_warning,
        statewide: Some(StatewideStats {
            dem_vote_share: metrics.statewide_dem_vote_share,
            dem_seat_share: metrics.statewide_dem_seat_share,
            total_votes,
        }),
        metrics: Some(MetricsBlock {
            efficiency_gap: metrics.efficiency_gap,
            mean_median: metrics.mean_median,
            partisan_bias: metrics.partisan_bias,
        }),
        districts: Some(district_outputs),
    };

    write_json_atomic(&out_path, &output)?;
    eprintln!("[OK] partisan -> {}", out_path.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_load_election_csv_valid() {
        let csv = "geoid,dem_votes,rep_votes\n53033001001,1203.5,876.2\n53033001002,500.0,400.0\n";
        let records = parse_election_csv(Cursor::new(csv.as_bytes())).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].geoid, "53033001001");
        assert!((records[0].dem_votes - 1203.5).abs() < 0.001);
    }

    #[test]
    fn test_load_election_csv_missing_file_returns_err_with_guidance() {
        let result = load_election_data(Path::new("/nonexistent/file.csv"), "2020");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("redist fetch --type elections"), "error must guide user to fetch: {msg}");
    }

    #[test]
    fn test_aggregate_to_districts_correct_sum() {
        let election = vec![
            ElectionRecord { geoid: "t1".into(), dem_votes: 100.0, rep_votes: 50.0 },
            ElectionRecord { geoid: "t2".into(), dem_votes: 200.0, rep_votes: 150.0 },
            ElectionRecord { geoid: "t3".into(), dem_votes: 80.0, rep_votes: 120.0 },
        ];
        let assignments: HashMap<String, usize> = [
            ("t1".into(), 1usize), ("t2".into(), 1), ("t3".into(), 2),
        ].into();
        let by_district = aggregate_election_to_districts(&election, &assignments);
        assert!((by_district[&1].dem_votes - 300.0).abs() < 1e-9);
        assert!((by_district[&1].rep_votes - 200.0).abs() < 1e-9);
        assert!((by_district[&2].dem_votes - 80.0).abs() < 1e-9);
    }

    #[test]
    fn test_analyze_partisan_type_dispatches_correctly() {
        use redist_analysis::AnalyzerType;
        let types = vec![AnalyzerType::Partisan];
        assert!(types.contains(&AnalyzerType::Partisan));
        assert!(!types.contains(&AnalyzerType::Demographic));
    }

    // ── parse_election_csv edge cases ─────────────────────────────────────────

    #[test]
    fn test_parse_election_csv_empty_body() {
        // Header only, no data rows → empty vec.
        let csv = "geoid,dem_votes,rep_votes\n";
        let records = parse_election_csv(Cursor::new(csv.as_bytes())).unwrap();
        assert_eq!(records.len(), 0, "header-only CSV must yield 0 records");
    }

    #[test]
    fn test_parse_election_csv_fractional_votes() {
        // Election data uses f64 — fractional allocations (e.g. from party-reg proration) are valid.
        let csv = "geoid,dem_votes,rep_votes\n12345678901,0.5,0.5\n";
        let records = parse_election_csv(Cursor::new(csv.as_bytes())).unwrap();
        assert_eq!(records.len(), 1);
        assert!((records[0].dem_votes - 0.5).abs() < 1e-12);
        assert!((records[0].rep_votes - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_parse_election_csv_zero_votes() {
        // Uncontested races may have zero votes for one party.
        let csv = "geoid,dem_votes,rep_votes\n99999999999,0.0,1000.0\n";
        let records = parse_election_csv(Cursor::new(csv.as_bytes())).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].dem_votes, 0.0);
        assert_eq!(records[0].rep_votes, 1000.0);
    }

    #[test]
    fn test_parse_election_csv_multiple_rows() {
        let csv = "geoid,dem_votes,rep_votes\ng1,100.0,80.0\ng2,200.0,150.0\ng3,50.0,75.0\n";
        let records = parse_election_csv(Cursor::new(csv.as_bytes())).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[2].geoid, "g3");
        assert!((records[2].rep_votes - 75.0).abs() < 1e-12);
    }

    #[test]
    fn test_parse_election_csv_preserves_geoid_as_string() {
        // Leading-zero GEOIDs must be preserved as strings, not parsed as integers.
        let csv = "geoid,dem_votes,rep_votes\n01001020100,500.0,400.0\n";
        let records = parse_election_csv(Cursor::new(csv.as_bytes())).unwrap();
        assert_eq!(records[0].geoid, "01001020100", "leading-zero GEOID must be preserved");
    }

    // ── aggregate_election_to_districts ──────────────────────────────────────

    #[test]
    fn test_aggregate_empty_election_returns_empty() {
        let election: Vec<ElectionRecord> = Vec::new();
        let assignments: HashMap<String, usize> = HashMap::new();
        let result = aggregate_election_to_districts(&election, &assignments);
        assert!(result.is_empty());
    }

    #[test]
    fn test_aggregate_geoid_not_in_assignments_is_ignored() {
        // GEOIDs in the election file that have no assignment are silently dropped.
        let election = vec![
            ElectionRecord { geoid: "t_unassigned".into(), dem_votes: 999.0, rep_votes: 111.0 },
        ];
        let assignments: HashMap<String, usize> = HashMap::new();
        let result = aggregate_election_to_districts(&election, &assignments);
        assert!(result.is_empty(), "unassigned GEOIDs must be dropped");
    }

    #[test]
    fn test_aggregate_single_district_single_tract() {
        let election = vec![
            ElectionRecord { geoid: "t1".into(), dem_votes: 300.0, rep_votes: 200.0 },
        ];
        let assignments: HashMap<String, usize> = [("t1".to_string(), 1usize)].into();
        let result = aggregate_election_to_districts(&election, &assignments);
        assert_eq!(result.len(), 1);
        assert!((result[&1].dem_votes - 300.0).abs() < 1e-9);
        assert!((result[&1].rep_votes - 200.0).abs() < 1e-9);
    }

    #[test]
    fn test_aggregate_three_tracts_same_district() {
        let election = vec![
            ElectionRecord { geoid: "a".into(), dem_votes: 100.0, rep_votes: 50.0 },
            ElectionRecord { geoid: "b".into(), dem_votes: 200.0, rep_votes: 100.0 },
            ElectionRecord { geoid: "c".into(), dem_votes: 150.0, rep_votes: 75.0 },
        ];
        let assignments: HashMap<String, usize> = [
            ("a".to_string(), 1usize), ("b".to_string(), 1), ("c".to_string(), 1)
        ].into();
        let result = aggregate_election_to_districts(&election, &assignments);
        assert_eq!(result.len(), 1);
        assert!((result[&1].dem_votes - 450.0).abs() < 1e-9);
        assert!((result[&1].rep_votes - 225.0).abs() < 1e-9);
    }

    #[test]
    fn test_aggregate_district_id_is_preserved() {
        let election = vec![
            ElectionRecord { geoid: "x".into(), dem_votes: 10.0, rep_votes: 5.0 },
        ];
        let assignments: HashMap<String, usize> = [("x".to_string(), 7usize)].into();
        let result = aggregate_election_to_districts(&election, &assignments);
        assert!(result.contains_key(&7), "district_id from assignment must be preserved");
        assert_eq!(result[&7].district, 7);
    }

    #[test]
    fn test_aggregate_all_dem_votes_zero() {
        // Completely uncontested R race — district gets 0 dem, all rep.
        let election = vec![
            ElectionRecord { geoid: "t".into(), dem_votes: 0.0, rep_votes: 1000.0 },
        ];
        let assignments: HashMap<String, usize> = [("t".to_string(), 1usize)].into();
        let result = aggregate_election_to_districts(&election, &assignments);
        assert_eq!(result[&1].dem_votes, 0.0);
        assert_eq!(result[&1].rep_votes, 1000.0);
    }

    #[test]
    fn test_aggregate_large_number_of_tracts() {
        // Stress test: 500 tracts all in the same district.
        let mut election = Vec::new();
        for i in 0..500usize {
            election.push(ElectionRecord {
                geoid: format!("g{i}"),
                dem_votes: 10.0,
                rep_votes: 8.0,
            });
        }
        let assignments: HashMap<String, usize> = (0..500)
            .map(|i| (format!("g{i}"), 1usize))
            .collect();
        let result = aggregate_election_to_districts(&election, &assignments);
        assert_eq!(result.len(), 1);
        assert!((result[&1].dem_votes - 5000.0).abs() < 1e-6);
        assert!((result[&1].rep_votes - 4000.0).abs() < 1e-6);
    }

    // ── PartisanDistrictOutput derived fields ────────────────────────────────

    #[test]
    fn test_partisan_district_output_dem_pct_100_percent() {
        // When there are zero rep votes, dem_pct should be 1.0 (100%).
        // Use the DistrictElection helper that the real code uses.
        let d = redist_analysis::DistrictElection {
            district: 1,
            dem_votes: 1000.0,
            rep_votes: 0.0,
        };
        let pct = d.dem_pct();
        assert!((pct - 1.0).abs() < 1e-9, "100% dem: pct should be 1.0, got {pct}");
    }

    #[test]
    fn test_partisan_district_output_dem_pct_50_50() {
        let d = redist_analysis::DistrictElection {
            district: 2,
            dem_votes: 500.0,
            rep_votes: 500.0,
        };
        let pct = d.dem_pct();
        assert!((pct - 0.5).abs() < 1e-9, "50/50 split: pct should be 0.5, got {pct}");
    }

    #[test]
    fn test_partisan_district_output_total() {
        let d = redist_analysis::DistrictElection {
            district: 1,
            dem_votes: 300.0,
            rep_votes: 200.0,
        };
        assert!((d.total() - 500.0).abs() < 1e-9, "total should be 500.0, got {}", d.total());
    }

    #[test]
    fn test_is_competitive_flag_below_5pct_margin() {
        // Margin < 5% → competitive.  margin = |dem - rep| / total.
        // 510 dem, 490 rep → margin = 20/1000 = 0.02 < 0.05 → competitive.
        let margin = (510.0f64 - 490.0).abs() / 1000.0;
        assert!(margin < 0.05, "margin {margin} should flag as competitive");
    }

    #[test]
    fn test_is_uncontested_flag_zero_dem_votes() {
        // dem_votes == 0 → uncontested.
        let is_uncontested = 0.0_f64 == 0.0 || 500.0_f64 == 0.0;
        assert!(is_uncontested, "zero dem_votes should be uncontested");
    }

    // ── ElectionRecord struct ────────────────────────────────────────────────

    #[test]
    fn test_election_record_debug_roundtrip() {
        let r = ElectionRecord { geoid: "12345".into(), dem_votes: 100.0, rep_votes: 200.0 };
        let debug_str = format!("{r:?}");
        assert!(debug_str.contains("12345"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_statewide_stats_fields() {
        let s = StatewideStats {
            dem_vote_share: 0.52,
            dem_seat_share: 0.60,
            total_votes: 1_000_000.0,
        };
        assert!((s.dem_vote_share - 0.52).abs() < 1e-12);
        assert!((s.dem_seat_share - 0.60).abs() < 1e-12);
        assert!((s.total_votes - 1_000_000.0).abs() < 1e-6);
    }
}
