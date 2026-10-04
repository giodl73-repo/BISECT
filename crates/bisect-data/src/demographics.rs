//! In-memory demographic import for native callers and browsers. Counts are
//! retained; search fractions do not imply count-weighted district evaluation.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DemographicCounts {
    pub total: f64,
    pub minority: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemographicCsvImport {
    pub schema_version: &'static str,
    pub state: String,
    pub year: String,
    pub basis: String,
    pub source_label: String,
    pub minority_fractions: BTreeMap<String, f64>,
    pub counts: BTreeMap<String, DemographicCounts>,
}

/// Strict counterpart to the CLI's historical lenient CSV readers. The same
/// columns and short-GEOID padding are supported, without silent zero filling,
/// clamping impossible counts, or replacing duplicate tracts.
pub fn import_demographic_csv(
    bytes: &[u8],
    state: &str,
    year: &str,
    basis: &str,
    source_label: &str,
) -> Result<DemographicCsvImport, String> {
    const STATES: &str = "AL01 AK02 AZ04 AR05 CA06 CO08 CT09 DE10 DC11 FL12 GA13 HI15 ID16 IL17 IN18 IA19 KS20 KY21 LA22 ME23 MD24 MA25 MI26 MN27 MS28 MO29 MT30 NE31 NV32 NH33 NJ34 NM35 NY36 NC37 ND38 OH39 OK40 OR41 PA42 RI44 SC45 SD46 TN47 TX48 UT49 VT50 VA51 WA53 WV54 WI55 WY56";
    let fips = STATES
        .split_whitespace()
        .find(|entry| &entry[..2] == state)
        .map(|entry| &entry[2..])
        .ok_or("Select a supported state.")?;
    if year.len() != 4
        || !year.bytes().all(|b| b.is_ascii_digit())
        || source_label.trim().is_empty()
        || source_label.len() > 200
    {
        return Err("Supply a year and source label of 1–200 bytes.".into());
    }
    if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
        return Err("Select a nonempty demographic CSV of at most 8 MiB.".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "Demographic CSV is not valid UTF-8.")?
        .trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let headers = reader.headers().map_err(|e| e.to_string())?.clone();
    let mut seen = HashSet::new();
    for header in &headers {
        if header.is_empty() || !seen.insert(header) {
            return Err("Demographic CSV has empty or duplicate headers.".into());
        }
    }
    let column = |names: &[&str]| -> Result<usize, String> {
        let matches: Vec<_> = headers
            .iter()
            .enumerate()
            .filter(|(_, h)| names.contains(h))
            .map(|(i, _)| i)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Supply exactly one demographic column from {names:?}."
            ));
        }
        Ok(matches[0])
    };
    let geoid = column(&["GEOID", "geoid"])?;
    let (total, minority, white) = match basis {
        "total-population" => (
            column(&["total_pop"])?,
            column(&["white_non_hispanic"])?,
            true,
        ),
        "voting-age-population" => (
            column(&["total_vap", "vap"])?,
            column(&["minority_vap"])?,
            false,
        ),
        "citizen-voting-age-population" => (
            column(&["total_vap", "vap", "cvap"])?,
            column(&["minority_vap"])?,
            false,
        ),
        _ => return Err("Select total population, VAP or CVAP basis.".into()),
    };
    let mut result = DemographicCsvImport {
        schema_version: "bisect-demographic-counts-v2",
        state: state.into(),
        year: year.into(),
        basis: basis.into(),
        source_label: source_label.into(),
        minority_fractions: BTreeMap::new(),
        counts: BTreeMap::new(),
    };
    for (index, row) in reader.records().enumerate() {
        let row = row.map_err(|e| format!("Demographic CSV row {}: {e}", index + 2))?;
        let raw = &row[geoid];
        if raw.is_empty() || raw.len() > 11 || !raw.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("Invalid tract GEOID at CSV row {}.", index + 2));
        }
        let id = format!("{raw:0>11}");
        if !id.starts_with(fips) {
            return Err(format!("Tract {id} does not belong to {state}."));
        }
        let number = |i: usize| -> Result<f64, String> {
            let value = row[i]
                .parse::<f64>()
                .map_err(|_| format!("Invalid demographic count for tract {id}."))?;
            if !value.is_finite() || value.is_sign_negative() || value > 9_007_199_254_740_991.0 {
                return Err(format!("Invalid demographic count for tract {id}."));
            }
            Ok(value)
        };
        let t = number(total)?;
        let m = number(minority)?;
        if m > t {
            return Err(format!(
                "Minority/white count exceeds total for tract {id}."
            ));
        }
        let m = if white { t - m } else { m };
        if result
            .counts
            .insert(
                id.clone(),
                DemographicCounts {
                    total: t,
                    minority: m,
                },
            )
            .is_some()
        {
            return Err(format!("Duplicate tract {id}."));
        }
        result
            .minority_fractions
            .insert(id, if t == 0.0 { 0.0 } else { m / t });
        if result.counts.len() > 100_000 {
            return Err("Demographic CSV exceeds 100,000 tracts.".into());
        }
    }
    if result.counts.is_empty() {
        return Err("Demographic CSV has no tract records.".into());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn read(text: &str, basis: &str) -> Result<DemographicCsvImport, String> {
        import_demographic_csv(text.as_bytes(), "AL", "2020", basis, "Synthetic fixture")
    }
    #[test]
    fn native_columns_counts_padding_and_quotes() {
        let total=read("\u{feff}GEOID,total_pop,white_non_hispanic,note\r\n1001020100,1000,800,\"quoted, note\"\r\n01001020200,0,0,\"two\nlines\"\r\n","total-population").unwrap();
        assert_eq!(
            total.counts["01001020100"],
            DemographicCounts {
                total: 1000.0,
                minority: 200.0
            }
        );
        assert_eq!(total.minority_fractions["01001020100"], 0.2);
        assert_eq!(total.minority_fractions["01001020200"], 0.0);
        for column in ["total_vap", "vap", "cvap"] {
            let basis = if column == "cvap" {
                "citizen-voting-age-population"
            } else {
                "voting-age-population"
            };
            let result = read(
                &format!("geoid,{column},minority_vap\n01001020100,800,500\n"),
                basis,
            )
            .unwrap();
            assert_eq!(result.counts["01001020100"].minority, 500.0);
            assert_eq!(result.minority_fractions["01001020100"], 0.625);
        }
    }
    #[test]
    fn rejects_invalid_or_ambiguous_data() {
        for rows in [
            "01001020100,10,11",
            "01001020100,NaN,1",
            "01001020100,10,-0",
            "01001020100,bad,1",
            "01001020100,10,",
            "abc,10,1",
            "44001020100,10,1",
            "01001020100,10,1\n1001020100,10,1",
            "01001020100,10,1,extra",
        ] {
            assert!(
                read(
                    &format!("GEOID,total_pop,white_non_hispanic\n{rows}\n"),
                    "total-population"
                )
                .is_err(),
                "{rows}"
            );
        }
        assert!(read(
            "GEOID,total_pop,total_pop,white_non_hispanic\n01001020100,10,10,1\n",
            "total-population"
        )
        .is_err());
        assert!(read(
            "GEOID,total_vap,vap,minority_vap\n01001020100,10,10,1\n",
            "voting-age-population"
        )
        .is_err());
        assert!(read("GEOID,total_pop,white_non_hispanic\n", "total-population").is_err());
    }
}
