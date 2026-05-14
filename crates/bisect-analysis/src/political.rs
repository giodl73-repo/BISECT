use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::analyzer::{Analyzer, AnalyzerContext};

#[derive(Debug, Deserialize)]
struct PoliticalRow {
    geoid: String,
    dem_votes: f64,
    rep_votes: f64,
    #[allow(dead_code)]
    lib_votes: Option<f64>,
    #[allow(dead_code)]
    grn_votes: Option<f64>,
    #[allow(dead_code)]
    oth_votes: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PoliticalDistrict {
    pub district: usize,
    pub total_votes: f64,
    pub dem_votes: f64,
    pub rep_votes: f64,
    pub dem_pct: f64,
    pub rep_pct: f64,
    pub margin: f64, // D - R (positive = Dem)
    pub lean_dem: bool,
    pub is_uncontested: bool, // either party = 0 votes
}

#[derive(Debug, Clone, Serialize)]
pub struct PoliticalResult {
    pub analyzer: &'static str,
    pub available: bool,
    pub districts: Vec<PoliticalDistrict>,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum PoliticalError {
    #[error("[INPUT] non-finite {field} for geoid {geoid}: {value}")]
    NonFiniteVote {
        geoid: String,
        field: &'static str,
        value: f64,
    },
    #[error("[INPUT] negative {field} for geoid {geoid}: {value}")]
    NegativeVote {
        geoid: String,
        field: &'static str,
        value: f64,
    },
    #[error("[NUMERIC] {operation} produced non-finite value {value}")]
    NonFiniteAggregate { operation: &'static str, value: f64 },
}

/// Aggregate political rows into per-district results.
pub fn aggregate_political(
    rows: &[PoliticalRow],
    assignments: &HashMap<String, usize>,
    num_districts: usize,
) -> PoliticalResult {
    try_aggregate_political(rows, assignments, num_districts).expect("political inputs are valid")
}

fn try_aggregate_political(
    rows: &[PoliticalRow],
    assignments: &HashMap<String, usize>,
    num_districts: usize,
) -> Result<PoliticalResult, PoliticalError> {
    let mut totals: HashMap<usize, (f64, f64)> = HashMap::new();
    for d in 1..=num_districts {
        totals.insert(d, (0.0, 0.0));
    }

    let mut unmatched = 0usize;
    for row in rows {
        validate_vote(&row.geoid, "dem_votes", row.dem_votes)?;
        validate_vote(&row.geoid, "rep_votes", row.rep_votes)?;
        if let Some(&district) = assignments.get(&row.geoid) {
            let e = totals.entry(district).or_insert((0.0, 0.0));
            e.0 += row.dem_votes;
            validate_finite("district Democratic votes", e.0)?;
            e.1 += row.rep_votes;
            validate_finite("district Republican votes", e.1)?;
        } else {
            unmatched += 1;
        }
    }

    if unmatched > 0 {
        eprintln!("WARNING: {unmatched} political rows had no assignment match");
    }

    let mut districts: Vec<PoliticalDistrict> = totals
        .into_iter()
        .map(|(district, (dem, rep))| {
            let total = dem + rep;
            validate_finite("district two-party votes", total)?;
            let (dem_pct, rep_pct) = if total == 0.0 {
                (0.0, 0.0)
            } else {
                let dem_pct = dem / total;
                let rep_pct = rep / total;
                validate_finite("district Democratic vote share", dem_pct)?;
                validate_finite("district Republican vote share", rep_pct)?;
                (dem_pct, rep_pct)
            };
            let margin = dem_pct - rep_pct;
            validate_finite("district political margin", margin)?;
            Ok(PoliticalDistrict {
                district,
                total_votes: total,
                dem_votes: dem,
                rep_votes: rep,
                dem_pct,
                rep_pct,
                margin,
                lean_dem: margin >= 0.0,
                is_uncontested: dem == 0.0 || rep == 0.0,
            })
        })
        .collect::<Result<Vec<_>, PoliticalError>>()?;
    districts.sort_by_key(|d| d.district);

    Ok(PoliticalResult {
        analyzer: "political",
        available: true,
        districts,
    })
}

fn validate_vote(geoid: &str, field: &'static str, value: f64) -> Result<(), PoliticalError> {
    if !value.is_finite() {
        return Err(PoliticalError::NonFiniteVote {
            geoid: geoid.to_string(),
            field,
            value,
        });
    }
    if value < 0.0 {
        return Err(PoliticalError::NegativeVote {
            geoid: geoid.to_string(),
            field,
            value,
        });
    }
    Ok(())
}

fn validate_finite(operation: &'static str, value: f64) -> Result<(), PoliticalError> {
    if !value.is_finite() {
        return Err(PoliticalError::NonFiniteAggregate { operation, value });
    }
    Ok(())
}

pub struct PoliticalAnalyzer;

impl Analyzer for PoliticalAnalyzer {
    type Output = PoliticalResult;

    fn name() -> &'static str {
        "political"
    }

    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output> {
        // CSV: data/{year}/elections/presidential_by_tract.csv
        let csv_path = ctx
            .data_root
            .join(ctx.year)
            .join("elections")
            .join("presidential_by_tract.csv");

        if !csv_path.exists() {
            eprintln!(
                "WARNING: political data not found at {}",
                csv_path.display()
            );
            return Ok(PoliticalResult {
                analyzer: "political",
                available: false,
                districts: vec![],
            });
        }

        let mut rdr = csv::Reader::from_path(&csv_path)
            .with_context(|| format!("cannot open political CSV: {}", csv_path.display()))?;

        let rows: Vec<PoliticalRow> = rdr
            .deserialize()
            .collect::<Result<Vec<_>, _>>()
            .context("failed to parse political CSV rows")?;

        Ok(try_aggregate_political(
            &rows,
            ctx.assignments,
            ctx.num_districts,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pol_row(geoid: &str, dem: f64, rep: f64) -> PoliticalRow {
        PoliticalRow {
            geoid: geoid.to_string(),
            dem_votes: dem,
            rep_votes: rep,
            lib_votes: None,
            grn_votes: None,
            oth_votes: None,
        }
    }

    fn hashmap(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_political_dem_district() {
        let rows = vec![make_pol_row("50001", 700.0, 300.0)];
        let assignments = hashmap(&[("50001", 1)]);
        let r = aggregate_political(&rows, &assignments, 1);
        let d = &r.districts[0];
        assert!((d.dem_pct - 0.7).abs() < 1e-6);
        assert!((d.margin - 0.4).abs() < 1e-6);
        assert!(d.lean_dem);
    }

    #[test]
    fn test_political_rep_district() {
        let rows = vec![make_pol_row("50001", 300.0, 700.0)];
        let assignments = hashmap(&[("50001", 1)]);
        let r = aggregate_political(&rows, &assignments, 1);
        assert!(!r.districts[0].lean_dem);
        assert!((r.districts[0].margin - (-0.4)).abs() < 1e-6);
    }

    #[test]
    fn test_political_uncontested_flagged() {
        let rows = vec![make_pol_row("50001", 1000.0, 0.0)];
        let assignments = hashmap(&[("50001", 1)]);
        let r = aggregate_political(&rows, &assignments, 1);
        assert!(r.districts[0].is_uncontested);
    }

    #[test]
    fn test_try_political_rejects_non_finite_votes() {
        let rows = vec![make_pol_row("50001", f64::NAN, 100.0)];
        let assignments = hashmap(&[("50001", 1)]);

        match try_aggregate_political(&rows, &assignments, 1) {
            Err(PoliticalError::NonFiniteVote {
                geoid,
                field,
                value,
            }) => {
                assert_eq!(geoid, "50001");
                assert_eq!(field, "dem_votes");
                assert!(value.is_nan());
            }
            other => panic!("expected NonFiniteVote, got {other:?}"),
        }
    }

    #[test]
    fn test_try_political_rejects_negative_votes() {
        let rows = vec![make_pol_row("50001", 100.0, -1.0)];
        let assignments = hashmap(&[("50001", 1)]);

        match try_aggregate_political(&rows, &assignments, 1) {
            Err(PoliticalError::NegativeVote {
                geoid,
                field,
                value,
            }) => {
                assert_eq!(geoid, "50001");
                assert_eq!(field, "rep_votes");
                assert_eq!(value, -1.0);
            }
            other => panic!("expected NegativeVote, got {other:?}"),
        }
    }

    #[test]
    fn test_try_political_rejects_overflowed_district_votes() {
        let rows = vec![
            make_pol_row("50001", f64::MAX, 0.0),
            make_pol_row("50002", f64::MAX, 0.0),
        ];
        let assignments = hashmap(&[("50001", 1), ("50002", 1)]);

        match try_aggregate_political(&rows, &assignments, 1) {
            Err(PoliticalError::NonFiniteAggregate { operation, value }) => {
                assert_eq!(operation, "district Democratic votes");
                assert!(value.is_infinite());
            }
            other => panic!("expected NonFiniteAggregate, got {other:?}"),
        }
    }

    #[test]
    fn test_political_missing_csv_returns_unavailable() {
        use std::path::PathBuf;
        let assignments = hashmap(&[("50001", 1)]);
        let nonexistent = PathBuf::from("/nonexistent/path/that/does/not/exist");
        let ctx = AnalyzerContext {
            assignments: &assignments,
            state_name: "vermont",
            state_code: "VT",
            year: "2020",
            version: "v1",
            num_districts: 1,
            data_root: &nonexistent,
            output_root: &nonexistent,
            balance_tolerance: 0.005,
        };
        let result = PoliticalAnalyzer::run(&ctx);
        assert!(result.is_ok());
        let pr = result.unwrap();
        assert!(!pr.available);
        assert!(pr.districts.is_empty());
    }
}
