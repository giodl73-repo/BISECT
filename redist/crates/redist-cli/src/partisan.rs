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
            "Election data not found at {}. \
             Run: redist fetch --type elections --year {year}",
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

fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let tmp = path.with_extension("tmp.json");
    std::fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    std::fs::rename(&tmp, path)?;
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
}
