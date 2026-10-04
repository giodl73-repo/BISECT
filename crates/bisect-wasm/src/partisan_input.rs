use crate::{
    engine::PartisanInput,
    tract_input::{import_scope, tract_id},
};
use std::collections::BTreeMap;

/// Headered or headerless native partisan-share TSV, with strict values and
/// duplicate rejection. The engine subsequently enforces complete graph coverage.
pub fn import_tsv(
    bytes: &[u8],
    state: &str,
    year: &str,
    source_label: &str,
) -> Result<PartisanInput, String> {
    let fips = import_scope(state, year, source_label)?;
    if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
        return Err("Select a nonempty partisan TSV of at most 8 MiB.".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "Partisan TSV is not valid UTF-8.")?
        .trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .trim(csv::Trim::All)
        .comment(Some(b'#'))
        .from_reader(text.as_bytes());
    let mut shares = BTreeMap::new();
    let mut first = true;
    for row in reader.records() {
        let row = row.map_err(|e| format!("Invalid partisan TSV row: {e}"))?;
        if row.len() != 2 {
            return Err(
                "Partisan TSV requires exactly two tab-separated columns: geoid and dem_share."
                    .into(),
            );
        }
        if first {
            first = false;
            if ["geoid", "GEOID"].contains(&&row[0]) && row[1] == *"dem_share" {
                continue;
            }
        }
        if shares.len() >= 100_000 {
            return Err("Partisan TSV exceeds 100,000 tracts.".into());
        }
        let id = tract_id(&row[0], fips)?;
        let value = row[1]
            .parse::<f64>()
            .map_err(|_| "Partisan TSV has an invalid Democratic share.")?;
        if !value.is_finite() || value.is_sign_negative() || value > 1.0 {
            return Err("Partisan shares must be finite values from zero through one, excluding negative zero.".into());
        }
        if shares.insert(id, value).is_some() {
            return Err("Partisan TSV contains duplicate tract GEOIDs.".into());
        }
    }
    if shares.is_empty() {
        return Err("Partisan TSV must contain tract shares.".into());
    }
    Ok(PartisanInput {
        schema_version: "bisect-partisan-shares-v1".into(),
        state: state.into(),
        year: year.into(),
        source_label: source_label.into(),
        dem_shares: shares,
    })
}
