use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

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
    pub pct_minority: f64,          // = 1 - pct_white
    pub is_majority_minority: bool, // pct_minority >= 0.50
    pub pop_basis: &'static str,    // always "total_population"
}

#[derive(Debug, Clone, Serialize)]
pub struct DemographicResult {
    pub analyzer: &'static str,
    pub districts: Vec<DemographicDistrict>,
    pub pop_basis: &'static str,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum DemographicError {
    #[error("[INPUT] negative {field} for geoid {geoid}: {value}")]
    NegativeCount {
        geoid: String,
        field: &'static str,
        value: i64,
    },
    #[error("[INPUT] demographic subgroup total exceeds total_pop for geoid {geoid}: subgroups={subgroups}, total_pop={total_pop}")]
    SubgroupsExceedTotal {
        geoid: String,
        subgroups: i64,
        total_pop: i64,
    },
    #[error("[NUMERIC] {operation} overflowed for district {district}")]
    AggregateOverflow {
        district: usize,
        operation: &'static str,
    },
    #[error("[NUMERIC] {operation} produced non-finite value {value}")]
    NonFiniteResult { operation: &'static str, value: f64 },
}

fn validate_columns(headers: &csv::StringRecord) -> anyhow::Result<()> {
    let required = [
        "GEOID",
        "total_pop",
        "white_non_hispanic",
        "black_non_hispanic",
        "asian_non_hispanic",
        "hispanic",
        "other",
    ];
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
    try_aggregate_demographic(rows, assignments, num_districts)
        .expect("demographic inputs are valid")
}

fn try_aggregate_demographic(
    rows: &[DemographicRow],
    assignments: &HashMap<String, usize>,
    num_districts: usize,
) -> Result<DemographicResult, DemographicError> {
    let mut totals: HashMap<usize, (i64, i64, i64, i64, i64, i64)> = HashMap::new();
    for d in 1..=num_districts {
        totals.insert(d, (0, 0, 0, 0, 0, 0));
    }

    let mut unmatched = 0usize;
    for row in rows {
        validate_row(row)?;
        if let Some(&district) = assignments.get(&row.geoid) {
            let e = totals.entry(district).or_insert((0, 0, 0, 0, 0, 0));
            e.0 = checked_add(e.0, row.total_pop, district, "total_pop")?;
            e.1 = checked_add(e.1, row.white_non_hispanic, district, "white_non_hispanic")?;
            e.2 = checked_add(e.2, row.black_non_hispanic, district, "black_non_hispanic")?;
            e.3 = checked_add(e.3, row.asian_non_hispanic, district, "asian_non_hispanic")?;
            e.4 = checked_add(e.4, row.hispanic, district, "hispanic")?;
            e.5 = checked_add(e.5, row.other, district, "other")?;
        } else {
            unmatched += 1;
        }
    }

    if unmatched > 0 {
        eprintln!("WARNING: {unmatched} tract rows had no assignment match — possible census vintage mismatch");
    }

    let mut districts: Vec<DemographicDistrict> = totals
        .into_iter()
        .map(
            |(district, (total_pop, white, black, asian, hisp, other))| {
                let pct = |v: i64, operation: &'static str| -> Result<f64, DemographicError> {
                    if total_pop == 0 {
                        Ok(0.0)
                    } else {
                        let value = v as f64 / total_pop as f64;
                        validate_finite(operation, value)?;
                        Ok(value)
                    }
                };
                let pct_white = pct(white, "district white percentage")?;
                let pct_minority = 1.0 - pct_white;
                validate_finite("district minority percentage", pct_minority)?;
                Ok(DemographicDistrict {
                    district,
                    total_pop,
                    pct_white,
                    pct_black: pct(black, "district Black percentage")?,
                    pct_asian: pct(asian, "district Asian percentage")?,
                    pct_hispanic: pct(hisp, "district Hispanic percentage")?,
                    pct_other: pct(other, "district other percentage")?,
                    pct_minority,
                    is_majority_minority: pct_minority >= 0.50,
                    pop_basis: "total_population",
                })
            },
        )
        .collect::<Result<Vec<_>, DemographicError>>()?;
    districts.sort_by_key(|d| d.district);

    Ok(DemographicResult {
        analyzer: "demographic",
        districts,
        pop_basis: "total_population",
    })
}

fn validate_row(row: &DemographicRow) -> Result<(), DemographicError> {
    validate_non_negative(&row.geoid, "total_pop", row.total_pop)?;
    validate_non_negative(&row.geoid, "white_non_hispanic", row.white_non_hispanic)?;
    validate_non_negative(&row.geoid, "black_non_hispanic", row.black_non_hispanic)?;
    validate_non_negative(&row.geoid, "asian_non_hispanic", row.asian_non_hispanic)?;
    validate_non_negative(&row.geoid, "hispanic", row.hispanic)?;
    validate_non_negative(&row.geoid, "other", row.other)?;
    let subgroups = row
        .white_non_hispanic
        .checked_add(row.black_non_hispanic)
        .and_then(|v| v.checked_add(row.asian_non_hispanic))
        .and_then(|v| v.checked_add(row.hispanic))
        .and_then(|v| v.checked_add(row.other))
        .ok_or(DemographicError::AggregateOverflow {
            district: 0,
            operation: "tract demographic subgroups",
        })?;
    if subgroups > row.total_pop {
        return Err(DemographicError::SubgroupsExceedTotal {
            geoid: row.geoid.clone(),
            subgroups,
            total_pop: row.total_pop,
        });
    }
    Ok(())
}

fn validate_non_negative(
    geoid: &str,
    field: &'static str,
    value: i64,
) -> Result<(), DemographicError> {
    if value < 0 {
        return Err(DemographicError::NegativeCount {
            geoid: geoid.to_string(),
            field,
            value,
        });
    }
    Ok(())
}

fn checked_add(
    current: i64,
    addend: i64,
    district: usize,
    operation: &'static str,
) -> Result<i64, DemographicError> {
    current
        .checked_add(addend)
        .ok_or(DemographicError::AggregateOverflow {
            district,
            operation,
        })
}

fn validate_finite(operation: &'static str, value: f64) -> Result<(), DemographicError> {
    if !value.is_finite() {
        return Err(DemographicError::NonFiniteResult { operation, value });
    }
    Ok(())
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
    let rows: Vec<DemographicRow> = rdr
        .deserialize()
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse demographics CSV")?;
    Ok(try_aggregate_demographic(
        &rows,
        assignments,
        num_districts,
    )?)
}

pub struct DemographicAnalyzer;

impl Analyzer for DemographicAnalyzer {
    type Output = DemographicResult;

    fn name() -> &'static str {
        "demographic"
    }

    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output> {
        // Demographics CSV may be named by state_name (vermont_demographics_2020.csv)
        // or state_code_lower (vt_demographics_2020.csv). Try both.
        let state_code_lower = ctx.state_code.to_lowercase();
        let state_name_lower = ctx.state_name.replace(' ', "_");
        let demo_dir = ctx.data_root.join(ctx.year).join("demographics");
        let candidates = [
            demo_dir.join(format!("{state_name_lower}_demographics_{}.csv", ctx.year)),
            demo_dir.join(format!("{state_code_lower}_demographics_{}.csv", ctx.year)),
        ];
        let csv_path = candidates.iter().find(|p| p.exists()).ok_or_else(|| {
            anyhow::anyhow!(
                "demographics CSV not found for {} — tried {:?}",
                ctx.state_name,
                candidates
            )
        })?;

        let mut rdr = csv::Reader::from_path(csv_path)
            .with_context(|| format!("cannot open demographics CSV: {}", csv_path.display()))?;

        let headers = rdr.headers()?.clone();
        validate_columns(&headers)?;

        let rows: Vec<DemographicRow> = rdr
            .deserialize()
            .collect::<Result<Vec<_>, _>>()
            .context("failed to parse demographics CSV rows")?;

        Ok(try_aggregate_demographic(
            &rows,
            ctx.assignments,
            ctx.num_districts,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_demo_row(
        geoid: &str,
        total_pop: i64,
        white: i64,
        black: i64,
        asian: i64,
        hisp: i64,
        other: i64,
    ) -> DemographicRow {
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
    fn test_try_demographic_rejects_negative_counts() {
        let rows = vec![make_demo_row("50001", 1000, -1, 300, 100, 200, 0)];
        let assignments = hashmap(&[("50001", 1)]);

        match try_aggregate_demographic(&rows, &assignments, 1) {
            Err(DemographicError::NegativeCount {
                geoid,
                field,
                value,
            }) => {
                assert_eq!(geoid, "50001");
                assert_eq!(field, "white_non_hispanic");
                assert_eq!(value, -1);
            }
            other => panic!("expected NegativeCount, got {other:?}"),
        }
    }

    #[test]
    fn test_try_demographic_rejects_subgroups_exceeding_total() {
        let rows = vec![make_demo_row("50001", 1000, 800, 300, 0, 0, 0)];
        let assignments = hashmap(&[("50001", 1)]);

        match try_aggregate_demographic(&rows, &assignments, 1) {
            Err(DemographicError::SubgroupsExceedTotal {
                geoid,
                subgroups,
                total_pop,
            }) => {
                assert_eq!(geoid, "50001");
                assert_eq!(subgroups, 1100);
                assert_eq!(total_pop, 1000);
            }
            other => panic!("expected SubgroupsExceedTotal, got {other:?}"),
        }
    }

    #[test]
    fn test_try_demographic_rejects_overflowed_district_counts() {
        let rows = vec![
            make_demo_row("50001", i64::MAX, i64::MAX, 0, 0, 0, 0),
            make_demo_row("50002", i64::MAX, i64::MAX, 0, 0, 0, 0),
        ];
        let assignments = hashmap(&[("50001", 1), ("50002", 1)]);

        match try_aggregate_demographic(&rows, &assignments, 1) {
            Err(DemographicError::AggregateOverflow {
                district,
                operation,
            }) => {
                assert_eq!(district, 1);
                assert_eq!(operation, "total_pop");
            }
            other => panic!("expected AggregateOverflow, got {other:?}"),
        }
    }

    #[test]
    fn test_validate_columns_missing_column_returns_error() {
        // CSV with only "GEOID,total_pop" → Err containing "missing required column"
        let csv_content = "GEOID,total_pop\n50001,1000\n";
        let assignments = hashmap(&[]);
        let result = aggregate_demographic_from_str(csv_content, &assignments, 1);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing required column"));
    }
}
