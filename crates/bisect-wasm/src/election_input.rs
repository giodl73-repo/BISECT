//! Explicit scoped election counts; no filesystem loading or missing-tract fill.
use crate::engine::{Options, PreparedGraph};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ElectionCounts {
    pub democratic: f64,
    pub two_party: f64,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ElectionInput {
    pub schema_version: String,
    pub state: String,
    pub year: String,
    pub election_year: String,
    pub source_label: String,
    pub counts: BTreeMap<String, ElectionCounts>,
}
pub(crate) struct ElectionData {
    pub dem: Vec<f64>,
    pub two_party: Vec<f64>,
    pub total_dem: f64,
    pub total_two_party: f64,
    pub sha256: String,
}
pub(crate) fn validate(
    g: &PreparedGraph,
    o: &Options,
    input: &ElectionInput,
) -> Result<ElectionData, String> {
    if input.schema_version != "bisect-election-counts-v1"
        || input.state != g.state
        || input.year != g.year
        || input.state.len() != 2
        || !input.state.bytes().all(|b| b.is_ascii_uppercase())
        || !["2000", "2010", "2020"].contains(&input.year.as_str())
        || input.election_year.len() != 4
        || !input.election_year.bytes().all(|b| b.is_ascii_digit())
        || input.source_label.trim().is_empty()
        || input.source_label.len() > 200
        || input.counts.len() != g.geoids.len()
        || input.counts.values().any(|c| {
            [c.democratic, c.two_party]
                .iter()
                .any(|v| !v.is_finite() || v.is_sign_negative() || *v > 9_007_199_254_740_991.0)
                || c.democratic > c.two_party
        })
    {
        return Err(
            "Invalid election schema, scope, label, counts or complete tract coverage.".into(),
        );
    }
    let aligned = g
        .geoids
        .iter()
        .map(|id| {
            input
                .counts
                .get(id)
                .ok_or_else(|| "Election input omits a graph tract.".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let dem: Vec<_> = aligned.iter().map(|c| c.democratic).collect();
    let two_party: Vec<_> = aligned.iter().map(|c| c.two_party).collect();
    let total_dem: f64 = dem.iter().sum();
    let total_two_party: f64 = two_party.iter().sum();
    if !total_dem.is_finite()
        || !total_two_party.is_finite()
        || total_two_party <= 0.0
        || total_two_party > 9_007_199_254_740_991.0
        || (o.structure == "proportional-section" && total_dem <= 0.0)
        || (o.structure == "proportional-bisect"
            && dem.iter().zip(&g.population).any(|(&d, &p)| d > p as f64))
    {
        return Err("Election totals must be positive finite two-party counts; Section requires positive Democratic counts; recursive votes cannot exceed census population.".into());
    }
    let mut h = Sha256::new();
    h.update(b"BISECT_ELECTION_COUNTS_V1\0");
    for field in [&input.state, &input.year, &input.election_year] {
        h.update(field.as_bytes());
        h.update([0]);
    }
    for (id, c) in &input.counts {
        h.update(id.as_bytes());
        h.update(c.democratic.to_le_bytes());
        h.update(c.two_party.to_le_bytes());
    }
    Ok(ElectionData {
        dem,
        two_party,
        total_dem,
        total_two_party,
        sha256: format!("{:x}", h.finalize()),
    })
}

/// Strict import of the native presidential tract-count CSV format. No missing
/// values, duplicate replacement, foreign-state filtering, or zero filling.
pub fn import_csv(
    bytes: &[u8],
    state: &str,
    year: &str,
    election_year: &str,
    source_label: &str,
) -> Result<ElectionInput, String> {
    let fips = crate::tract_input::import_scope(state, year, source_label)?;
    if election_year.len() != 4
        || !election_year.bytes().all(|b| b.is_ascii_digit())
    {
        return Err("Supply a supported Census year, four-digit election year and source label of 1–200 bytes.".into());
    }
    if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
        return Err("Select a nonempty election CSV of at most 8 MiB.".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "Election CSV is not valid UTF-8.")?
        .trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let headers = reader.headers().map_err(|e| e.to_string())?.clone();
    let mut seen = std::collections::HashSet::new();
    if headers.iter().any(|h| h.is_empty() || !seen.insert(h)) {
        return Err("Election CSV has empty or duplicate headers.".into());
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
                "Supply exactly one election CSV column from {names:?}."
            ));
        }
        Ok(matches[0])
    };
    let geoid = column(&["geoid", "GEOID"])?;
    let dem = column(&["dem_votes"])?;
    let rep = column(&["rep_votes"])?;
    let mut counts = BTreeMap::new();
    let mut total = 0.0;
    for record in reader.records() {
        let record = record.map_err(|e| format!("Invalid election CSV row: {e}"))?;
        if counts.len() >= 100_000 {
            return Err("Election CSV exceeds 100,000 tracts.".into());
        }
        let id = crate::tract_input::tract_id(&record[geoid], fips)?;
        let number = |i: usize| -> Result<f64, String> {
            let v = record[i]
                .parse::<f64>()
                .map_err(|_| "Election CSV has an invalid vote count.")?;
            if !v.is_finite() || v.is_sign_negative() || v > 9_007_199_254_740_991.0 {
                return Err("Election CSV requires finite nonnegative vote counts within the browser's exact integer range.".into());
            }
            Ok(v)
        };
        let democratic = number(dem)?;
        let two_party = democratic + number(rep)?;
        total += two_party;
        if !total.is_finite() || total > 9_007_199_254_740_991.0 {
            return Err("Election CSV total exceeds the browser's exact integer range.".into());
        }
        if counts
            .insert(
                id,
                ElectionCounts {
                    democratic,
                    two_party,
                },
            )
            .is_some()
        {
            return Err("Election CSV contains duplicate tract GEOIDs.".into());
        }
    }
    if counts.is_empty() || total <= 0.0 {
        return Err("Election CSV must contain tracts and positive two-party votes.".into());
    }
    Ok(ElectionInput {
        schema_version: "bisect-election-counts-v1".into(),
        state: state.into(),
        year: year.into(),
        election_year: election_year.into(),
        source_label: source_label.into(),
        counts,
    })
}
