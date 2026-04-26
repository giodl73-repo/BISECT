use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Context;

use crate::analyzer::{Analyzer, AnalyzerContext};

#[derive(Debug, Deserialize)]
struct DemographicRow {
    #[serde(rename = "GEOID")]
    geoid: String,
    total_pop: i64,
    white_non_hispanic: i64,
    black_non_hispanic: i64,
    asian_non_hispanic: i64,
    hispanic: i64,
    other: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DemographicDistrict {
    pub district: usize,
    pub total_pop: i64,
    pub pct_white: f64,
    pub pct_black: f64,
    pub pct_asian: f64,
    pub pct_hispanic: f64,
    pub pct_other: f64,
    pub pct_minority: f64,       // = 1 - pct_white
    pub is_majority_minority: bool,  // pct_minority >= 0.50
    pub pop_basis: &'static str,    // always "total_population"
}

#[derive(Debug, Clone, Serialize)]
pub struct DemographicResult {
    pub analyzer: &'static str,
    pub districts: Vec<DemographicDistrict>,
    pub pop_basis: &'static str,
}

fn validate_columns(headers: &csv::StringRecord) -> anyhow::Result<()> {
    let required = ["GEOID", "total_pop", "white_non_hispanic", "black_non_hispanic",
                    "asian_non_hispanic", "hispanic", "other"];
    for col in required {
        if !headers.iter().any(|h| h == col) {
            anyhow::bail!("demographics CSV missing required column: {col}");
        }
    }
    Ok(())
}

/// Aggregate demographic rows into per-district results.
/// Unknown GEOIDs (not in assignments) are silently skipped but counted.
pub fn aggregate_demographic(
    rows: &[DemographicRow],
    assignments: &HashMap<String, usize>,
    num_districts: usize,
) -> DemographicResult {
    let mut totals: HashMap<usize, (i64, i64, i64, i64, i64, i64)> = HashMap::new();
    for d in 1..=num_districts {
        totals.insert(d, (0, 0, 0, 0, 0, 0));
    }

    let mut unmatched = 0usize;
    for row in rows {
        if let Some(&district) = assignments.get(&row.geoid) {
            let e = totals.entry(district).or_insert((0,0,0,0,0,0));
            e.0 += row.total_pop;
            e.1 += row.white_non_hispanic;
            e.2 += row.black_non_hispanic;
            e.3 += row.asian_non_hispanic;
            e.4 += row.hispanic;
            e.5 += row.other;
        } else {
            unmatched += 1;
        }
    }

    if unmatched > 0 {
        eprintln!("WARNING: {unmatched} tract rows had no assignment match — possible census vintage mismatch");
    }

    let mut districts: Vec<DemographicDistrict> = totals.into_iter().map(|(district, (total_pop, white, black, asian, hisp, other))| {
        let pct = |v: i64| -> f64 {
            if total_pop == 0 { 0.0 } else { v as f64 / total_pop as f64 }
        };
        let pct_white = pct(white);
        let pct_minority = 1.0 - pct_white;
        DemographicDistrict {
            district,
            total_pop,
            pct_white,
            pct_black: pct(black),
            pct_asian: pct(asian),
            pct_hispanic: pct(hisp),
            pct_other: pct(other),
            pct_minority,
            is_majority_minority: pct_minority >= 0.50,
            pop_basis: "total_population",
        }
    }).collect();
    districts.sort_by_key(|d| d.district);

    DemographicResult {
        analyzer: "demographic",
        districts,
        pop_basis: "total_population",
    }
}

/// Parse CSV content string and aggregate. Used in tests.
pub fn aggregate_demographic_from_str(
    csv_content: &str,
    assignments: &HashMap<String, usize>,
    num_districts: usize,
) -> anyhow::Result<DemographicResult> {
    let mut rdr = csv::Reader::from_reader(csv_content.as_bytes());
    let headers = rdr.headers()?.clone();
    validate_columns(&headers)?;
    let rows: Vec<DemographicRow> = rdr.deserialize()
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse demographics CSV")?;
    Ok(aggregate_demographic(&rows, assignments, num_districts))
}

pub struct DemographicAnalyzer;

impl Analyzer for DemographicAnalyzer {
    type Output = DemographicResult;

    fn name() -> &'static str { "demographic" }

    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output> {
        // Locate demographics CSV: data/{year}/demographics/{state_code}_demographics_{year}.csv
        let state_lower = ctx.state_code.to_lowercase();
        let csv_path = ctx.data_root
            .join(ctx.year)
            .join("demographics")
            .join(format!("{state_lower}_demographics_{}.csv", ctx.year));

        let mut rdr = csv::Reader::from_path(&csv_path)
            .with_context(|| format!("cannot open demographics CSV: {}", csv_path.display()))?;

        let headers = rdr.headers()?.clone();
        validate_columns(&headers)?;

        let rows: Vec<DemographicRow> = rdr.deserialize()
            .collect::<Result<Vec<_>, _>>()
            .context("failed to parse demographics CSV rows")?;

        Ok(aggregate_demographic(&rows, ctx.assignments, ctx.num_districts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_demo_row(geoid: &str, total_pop: i64, white: i64, black: i64, asian: i64, hisp: i64, other: i64) -> DemographicRow {
        DemographicRow {
            geoid: geoid.to_string(),
            total_pop,
            white_non_hispanic: white,
            black_non_hispanic: black,
            asian_non_hispanic: asian,
            hispanic: hisp,
            other,
        }
    }

    fn hashmap(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_demographic_aggregation_two_tracts_one_district() {
        let rows = vec![
            make_demo_row("50001", 1000, 800, 100, 0, 100, 0),
            make_demo_row("50002", 500, 400, 50, 0, 50, 0),
        ];
        let assignments = hashmap(&[("50001", 1), ("50002", 1)]);
        let result = aggregate_demographic(&rows, &assignments, 1);
        let d = &result.districts[0];
        assert_eq!(d.total_pop, 1500);
        assert!((d.pct_white - 0.8).abs() < 1e-6);
        assert!((d.pct_minority - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_demographic_unmatched_geoid_ignored() {
        let rows = vec![make_demo_row("99999", 1000, 800, 200, 0, 0, 0)];
        let assignments = hashmap(&[("50001", 1)]);
        let result = aggregate_demographic(&rows, &assignments, 1);
        assert_eq!(result.districts[0].total_pop, 0);
    }

    #[test]
    fn test_demographic_two_districts() {
        let rows = vec![
            make_demo_row("50001", 600, 600, 0, 0, 0, 0),
            make_demo_row("50002", 400, 0, 400, 0, 0, 0),
        ];
        let assignments = hashmap(&[("50001", 1), ("50002", 2)]);
        let result = aggregate_demographic(&rows, &assignments, 2);
        assert!((result.districts[0].pct_white - 1.0).abs() < 1e-6);
        assert!((result.districts[1].pct_black - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_majority_minority_flagged() {
        // 60% non-white → is_majority_minority=true
        let rows = vec![make_demo_row("50001", 1000, 400, 300, 100, 200, 0)];
        let assignments = hashmap(&[("50001", 1)]);
        let result = aggregate_demographic(&rows, &assignments, 1);
        assert!(result.districts[0].is_majority_minority);
    }

    #[test]
    fn test_validate_columns_missing_column_returns_error() {
        // CSV with only "GEOID,total_pop" → Err containing "missing required column"
        let csv_content = "GEOID,total_pop\n50001,1000\n";
        let assignments = hashmap(&[]);
        let result = aggregate_demographic_from_str(csv_content, &assignments, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing required column"));
    }
}
