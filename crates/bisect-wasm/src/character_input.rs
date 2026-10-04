//! Strict adapters for native tract character CSVs. No missing-tract defaults.
use bisect_core::{
    economic_character::{derive_economic_character, EconChar, LodesWacRaw},
    housing_character::HousingChar,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterInput {
    pub schema_version: String,
    pub state: String,
    /// Census geometry vintage; distinct from the observations' year.
    pub year: String,
    pub data_year: String,
    pub source_label: String,
    pub data: CharacterData,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CharacterData {
    Economic {
        raw_counts: BTreeMap<String, LodesWacRaw>,
        characters: BTreeMap<String, EconChar>,
    },
    /// Native ACS housing export already contains derived fractions/proxies.
    Housing {
        characters: BTreeMap<String, HousingChar>,
    },
}

#[derive(Serialize)]
pub struct CharacterWeights {
    pub schema_version: &'static str,
    pub kind: &'static str,
    pub alpha: f64,
    pub character_hash: String,
    pub edges: Vec<(usize,usize,f64)>,
    pub formula: &'static str,
    pub zero_policy: &'static str,
}

pub fn import_csv(
    bytes: &[u8],
    state: &str,
    year: &str,
    data_year: &str,
    source_label: &str,
    kind: &str,
) -> Result<CharacterInput, String> {
    let fips = crate::tract_input::import_scope(state, year, source_label)?;
    if data_year.len() != 4 || !data_year.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Character observation year must contain four digits.".into());
    }
    if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
        return Err("Select a nonempty character CSV of at most 8 MiB.".into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "Character CSV is not valid UTF-8.")?
        .trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| format!("Invalid character CSV header: {e}"))?
        .clone();
    let mut seen = HashSet::new();
    if headers
        .iter()
        .any(|h| h.is_empty() || !seen.insert(h.to_ascii_lowercase()))
    {
        return Err("Character CSV has empty or duplicate column names.".into());
    }
    let column = |names: &[&str]| -> Result<usize, String> {
        let matches: Vec<_> = headers
            .iter()
            .enumerate()
            .filter(|(_, h)| names.iter().any(|name| h.eq_ignore_ascii_case(name)))
            .map(|(i, _)| i)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Character CSV requires exactly one column for {}.",
                names.join("/")
            ));
        }
        Ok(matches[0])
    };
    let geoid = column(&["geoid"])?;
    let names: &[&[&str]] = match kind {
        "economic" => &[
            &["c000"],
            &["cns01"],
            &["cns02"],
            &["cns05"],
            &["cns07"],
            &["cns08"],
            &["cns09"],
            &["cns10"],
            &["cns11"],
        ],
        "housing" => &[
            &["pct_single_family", "pct_sf"],
            &["pct_multifamily", "pct_mf"],
            &["pct_owner"],
            &["housing_vintage"],
        ],
        _ => return Err("Character kind must be economic or housing.".into()),
    };
    let columns = names
        .iter()
        .map(|names| column(names))
        .collect::<Result<Vec<_>, _>>()?;
    let mut economic = BTreeMap::new();
    let mut raw_counts = BTreeMap::new();
    let mut housing = BTreeMap::new();
    let mut ids = HashSet::new();
    for row in reader.records() {
        let row = row.map_err(|e| format!("Invalid character CSV row: {e}"))?;
        if ids.len() >= 100_000 {
            return Err("Character CSV exceeds 100,000 tracts.".into());
        }
        let id = crate::tract_input::tract_id(&row[geoid], fips)?;
        if !ids.insert(id.clone()) {
            return Err("Character CSV has duplicate tract GEOIDs.".into());
        }
        let values=columns.iter().map(|&index|{
            let value=row[index].parse::<f64>().map_err(|_|"Character CSV has an invalid numeric value.")?;
            let limit=if kind=="economic"{9_007_199_254_740_991.0}else{1.0};
            if !value.is_finite()||value.is_sign_negative()||value>limit {return Err("Character CSV values must be finite, nonnegative and within the declared count/fraction range, excluding negative zero.");}
            Ok(value)
        }).collect::<Result<Vec<_>,_>>()?;
        if kind == "economic" {
            let raw = LodesWacRaw {
                c000: values[0],
                cns01: values[1],
                cns02: values[2],
                cns05: values[3],
                cns07: values[4],
                cns08: values[5],
                cns09: values[6],
                cns10: values[7],
                cns11: values[8],
            };
            economic.insert(id.clone(), derive_economic_character(raw));
            raw_counts.insert(id, raw);
        } else {
            housing.insert(
                id,
                HousingChar {
                    pct_single_family: values[0],
                    pct_multifamily: values[1],
                    pct_owner: values[2],
                    housing_vintage: values[3],
                },
            );
        }
    }
    if ids.is_empty() {
        return Err("Character CSV must contain tract observations.".into());
    }
    let data = if kind == "economic" {
        CharacterData::Economic {
            raw_counts,
            characters: economic,
        }
    } else {
        CharacterData::Housing {
            characters: housing,
        }
    };
    Ok(CharacterInput {
        schema_version: "bisect-character-v1".into(),
        state: state.into(),
        year: year.into(),
        data_year: data_year.into(),
        source_label: source_label.into(),
        data,
    })
}

/// Evaluate the native blend with complete explicit tract data. This is shared
/// by the portable adapter and future engine routing; no native fallback fills.
pub fn build_weights(
    input: &CharacterInput,
    geoids: &[String],
    edges: &[(usize, usize, f64)],
    alpha: f64,
) -> Result<CharacterWeights, String> {
    let fips = crate::tract_input::import_scope(&input.state, &input.year, &input.source_label)?;
    if input.schema_version != "bisect-character-v1"
        || input.data_year.len() != 4
        || !input.data_year.bytes().all(|b| b.is_ascii_digit())
        || geoids.is_empty()
        || geoids.len() > 100_000
        || edges.len() > 1_000_000
        || !alpha.is_finite()
        || alpha.is_sign_negative()
        || alpha > 1.
    {
        return Err(
            "Invalid character schema, observation year, graph size or blend alpha.".into(),
        );
    }
    let mut ids = HashSet::new();
    for id in geoids {
        if id.len() != 11 || crate::tract_input::tract_id(id, fips)? != *id || !ids.insert(id) {
            return Err("Character weights require unique canonical tract GEOIDs.".into());
        }
    }
    let mut digest = Sha256::new();
    digest.update(b"BISECT_CHARACTER_V1\0");
    for value in [&input.state, &input.year, &input.data_year] {
        digest.update(value.as_bytes());
        digest.update([0]);
    }
    let kind = match &input.data {
        CharacterData::Economic {
            raw_counts,
            characters,
        } => {
            digest.update(b"economic\0");
            if characters.len() != ids.len() || raw_counts.len() != ids.len() {
                return Err(
                    "Economic character requires complete raw and derived tract coverage.".into(),
                );
            }
            for (id, character) in characters {
                if !ids.contains(id) {
                    return Err("Economic character has an unexpected tract.".into());
                }
                let raw = raw_counts
                    .get(id)
                    .ok_or("Economic raw count coverage mismatch.")?;
                if [
                    raw.c000, raw.cns01, raw.cns02, raw.cns05, raw.cns07, raw.cns08, raw.cns09,
                    raw.cns10, raw.cns11,
                ]
                .iter()
                .any(|v| !v.is_finite() || v.is_sign_negative() || *v > 9_007_199_254_740_991.0)
                    || *character != derive_economic_character(*raw)
                {
                    return Err(
                        "Economic raw counts and derived character disagree or are invalid.".into(),
                    );
                }
                digest.update(id.as_bytes());
                for value in [
                    character.commercial_intensity,
                    character.industrial_fraction,
                    character.jobs_per_resident,
                ] {
                    if value.is_sign_negative() {
                        return Err("Character fractions cannot contain negative zero.".into());
                    }
                    digest.update(value.to_le_bytes());
                }
            }
            "economic"
        }
        CharacterData::Housing { characters } => {
            digest.update(b"housing\0");
            if characters.len() != ids.len() {
                return Err("Housing character requires complete tract coverage.".into());
            }
            for (id, character) in characters {
                if !ids.contains(id) {
                    return Err("Housing character has an unexpected tract.".into());
                }
                digest.update(id.as_bytes());
                for value in [
                    character.pct_single_family,
                    character.pct_multifamily,
                    character.pct_owner,
                    character.housing_vintage,
                ] {
                    if !value.is_finite() || value.is_sign_negative() || value > 1. {
                        return Err("Housing character must contain finite fractions in [0,1], excluding negative zero.".into());
                    }
                    digest.update(value.to_le_bytes());
                }
            }
            "housing"
        }
    };
    let mut seen = HashSet::new();
    let weighted = edges
        .iter()
        .map(|&(u, v, weight)| {
            if u >= v
                || v >= geoids.len()
                || !seen.insert((u, v))
                || !weight.is_finite()
                || weight <= 0.
            {
                return Err("Character weights require unique canonical positive boundary edges.");
            }
            let blended = match &input.data {
                CharacterData::Economic { characters, .. } => {
                    bisect_core::economic_character::blend_edge(
                        weight,
                        &characters[&geoids[u]],
                        &characters[&geoids[v]],
                        alpha,
                    )
                }
                CharacterData::Housing { characters } => {
                    bisect_core::housing_character::blend_edge(
                        weight,
                        &characters[&geoids[u]],
                        &characters[&geoids[v]],
                        alpha,
                    )
                }
            };
            Ok((u, v, blended))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CharacterWeights{schema_version:"bisect-character-weights-v1",kind,alpha,character_hash:format!("{:x}",digest.finalize()),edges:weighted,formula:"geographic-times-alpha-plus-one-minus-alpha-times-cosine",zero_policy:if kind=="economic"{"both-zero-one;one-zero-half"}else{"any-zero-one"}})
}
