use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const RCTX_VERSION: &str = "0.1";
pub const RCTX_CROSSWALK_HASH_PREFIX: &[u8] = b"RCTX_CROSSWALK_V1\0";
pub const RCTX_CROSSWALK_SET_HASH_PREFIX: &[u8] = b"RCTX_CROSSWALK_SET_V1\0";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RctxCoreError {
    #[error("context {context_hash} has invalid sha256 hash")]
    InvalidContextHash { context_hash: String },
    #[error("context {context_hash} has no units")]
    EmptyContextUnits { context_hash: String },
    #[error("context {context_hash} has duplicate unit id: {unit_id}")]
    DuplicateContextUnit {
        context_hash: String,
        unit_id: String,
    },
    #[error("duplicate source id: {source_id}")]
    DuplicateSourceId { source_id: String },
    #[error("source {source_id} has invalid sha256 hash: {sha256}")]
    InvalidSourceHash { source_id: String, sha256: String },
    #[error("crosswalk id is empty")]
    EmptyCrosswalkId,
    #[error("crosswalk {crosswalk_id} references missing context: {context_hash}")]
    MissingCrosswalkContext {
        crosswalk_id: String,
        context_hash: String,
    },
    #[error(
        "crosswalk {crosswalk_id} references missing unit {unit_id} in context {context_hash}"
    )]
    MissingCrosswalkUnit {
        crosswalk_id: String,
        context_hash: String,
        unit_id: String,
    },
    #[error("crosswalk {crosswalk_id} has invalid rational weight")]
    InvalidCrosswalkWeight { crosswalk_id: String },
    #[error(
        "duplicate crosswalk row for {crosswalk_id} {from_unit_id}->{to_unit_id} {weight_kind:?}"
    )]
    DuplicateCrosswalkRow {
        crosswalk_id: String,
        from_unit_id: String,
        to_unit_id: String,
        weight_kind: CrosswalkWeightKind,
    },
    #[error("crosswalk {crosswalk_id} exhaustive weights for {from_unit_id} do not sum to 1")]
    CrosswalkWeightSum {
        crosswalk_id: String,
        from_unit_id: String,
    },
    #[error("crosswalk {crosswalk_id} references missing source: {source_id}")]
    MissingSourceRef {
        crosswalk_id: String,
        source_id: String,
    },
    #[error("canonical JSON error: {0}")]
    CanonicalJson(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextUnitIndex {
    pub context_hash: String,
    pub unit_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIndexEntry {
    pub source_id: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrosswalkWeightKind {
    Population,
    Area,
    Ballots,
    RegisteredVoters,
    Manual,
    UnitCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalWeight {
    pub num: i64,
    pub den: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosswalkRecord {
    pub crosswalk_id: String,
    pub from_context_hash: String,
    pub to_context_hash: String,
    pub from_unit_id: String,
    pub to_unit_id: String,
    pub weight: RationalWeight,
    pub weight_kind: CrosswalkWeightKind,
    pub exhaustive: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosswalkVerificationInput {
    #[serde(default)]
    pub contexts: Vec<ContextUnitIndex>,
    #[serde(default)]
    pub sources: Vec<SourceIndexEntry>,
    #[serde(default)]
    pub crosswalks: Vec<CrosswalkRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    pub check_id: &'static str,
}

pub fn verify_crosswalk_input(
    input: &CrosswalkVerificationInput,
) -> Result<Vec<VerificationReport>, RctxCoreError> {
    let contexts = context_unit_sets(&input.contexts)?;
    let sources = source_ids(&input.sources)?;
    verify_crosswalk_records(&input.crosswalks, &contexts, &sources)?;
    let mut reports = vec![report("context_unit_index")];
    if !input.sources.is_empty() {
        reports.push(report("source_index"));
    }
    if !input.crosswalks.is_empty() {
        reports.push(report("crosswalk_unit_refs"));
        reports.push(report("crosswalk_weight_sum"));
        reports.push(report("crosswalk_source_refs"));
    }
    Ok(reports)
}

pub fn crosswalk_record_hash(record: &CrosswalkRecord) -> Result<String, RctxCoreError> {
    let value = serde_json::to_value(record)
        .map_err(|err| RctxCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(RCTX_CROSSWALK_HASH_PREFIX, &value)
}

pub fn crosswalk_set_hash(records: &[CrosswalkRecord]) -> Result<String, RctxCoreError> {
    let value = serde_json::to_value(records)
        .map_err(|err| RctxCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(RCTX_CROSSWALK_SET_HASH_PREFIX, &value)
}

pub fn canonical_hash(prefix: &[u8], value: &Value) -> Result<String, RctxCoreError> {
    let canonical = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| RctxCoreError::CanonicalJson(err.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(prefix);
    hasher.update(bytes);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

pub fn is_sha256_hash(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit()))
}

fn context_unit_sets(
    contexts: &[ContextUnitIndex],
) -> Result<BTreeMap<String, BTreeSet<String>>, RctxCoreError> {
    let mut map = BTreeMap::new();
    for context in contexts {
        if !is_sha256_hash(&context.context_hash) {
            return Err(RctxCoreError::InvalidContextHash {
                context_hash: context.context_hash.clone(),
            });
        }
        if context.unit_ids.is_empty() {
            return Err(RctxCoreError::EmptyContextUnits {
                context_hash: context.context_hash.clone(),
            });
        }
        let mut units = BTreeSet::new();
        for unit_id in &context.unit_ids {
            if !units.insert(unit_id.clone()) {
                return Err(RctxCoreError::DuplicateContextUnit {
                    context_hash: context.context_hash.clone(),
                    unit_id: unit_id.clone(),
                });
            }
        }
        map.insert(context.context_hash.clone(), units);
    }
    Ok(map)
}

fn source_ids(sources: &[SourceIndexEntry]) -> Result<BTreeSet<String>, RctxCoreError> {
    let mut ids = BTreeSet::new();
    for source in sources {
        if !ids.insert(source.source_id.clone()) {
            return Err(RctxCoreError::DuplicateSourceId {
                source_id: source.source_id.clone(),
            });
        }
        if !is_sha256_hash(&source.sha256) {
            return Err(RctxCoreError::InvalidSourceHash {
                source_id: source.source_id.clone(),
                sha256: source.sha256.clone(),
            });
        }
    }
    Ok(ids)
}

fn verify_crosswalk_records(
    crosswalks: &[CrosswalkRecord],
    contexts: &BTreeMap<String, BTreeSet<String>>,
    sources: &BTreeSet<String>,
) -> Result<(), RctxCoreError> {
    let mut seen = BTreeSet::new();
    let mut sums: BTreeMap<(String, String, CrosswalkWeightKind), RationalSum> = BTreeMap::new();
    for crosswalk in crosswalks {
        if crosswalk.crosswalk_id.trim().is_empty() {
            return Err(RctxCoreError::EmptyCrosswalkId);
        }
        ensure_context_unit(
            &crosswalk.crosswalk_id,
            &crosswalk.from_context_hash,
            &crosswalk.from_unit_id,
            contexts,
        )?;
        ensure_context_unit(
            &crosswalk.crosswalk_id,
            &crosswalk.to_context_hash,
            &crosswalk.to_unit_id,
            contexts,
        )?;
        if crosswalk.weight.den <= 0 || crosswalk.weight.num < 0 {
            return Err(RctxCoreError::InvalidCrosswalkWeight {
                crosswalk_id: crosswalk.crosswalk_id.clone(),
            });
        }
        if !seen.insert((
            crosswalk.crosswalk_id.as_str(),
            crosswalk.from_unit_id.as_str(),
            crosswalk.to_unit_id.as_str(),
            crosswalk.weight_kind,
        )) {
            return Err(RctxCoreError::DuplicateCrosswalkRow {
                crosswalk_id: crosswalk.crosswalk_id.clone(),
                from_unit_id: crosswalk.from_unit_id.clone(),
                to_unit_id: crosswalk.to_unit_id.clone(),
                weight_kind: crosswalk.weight_kind,
            });
        }
        for source_id in &crosswalk.source_refs {
            if !sources.contains(source_id) {
                return Err(RctxCoreError::MissingSourceRef {
                    crosswalk_id: crosswalk.crosswalk_id.clone(),
                    source_id: source_id.clone(),
                });
            }
        }
        if crosswalk.exhaustive {
            sums.entry((
                crosswalk.crosswalk_id.clone(),
                crosswalk.from_unit_id.clone(),
                crosswalk.weight_kind,
            ))
            .or_default()
            .add(crosswalk.weight);
        }
    }

    for ((crosswalk_id, from_unit_id, _), sum) in sums {
        if !sum.is_one() {
            return Err(RctxCoreError::CrosswalkWeightSum {
                crosswalk_id,
                from_unit_id,
            });
        }
    }
    Ok(())
}

fn ensure_context_unit(
    crosswalk_id: &str,
    context_hash: &str,
    unit_id: &str,
    contexts: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), RctxCoreError> {
    let units =
        contexts
            .get(context_hash)
            .ok_or_else(|| RctxCoreError::MissingCrosswalkContext {
                crosswalk_id: crosswalk_id.to_string(),
                context_hash: context_hash.to_string(),
            })?;
    if !units.contains(unit_id) {
        return Err(RctxCoreError::MissingCrosswalkUnit {
            crosswalk_id: crosswalk_id.to_string(),
            context_hash: context_hash.to_string(),
            unit_id: unit_id.to_string(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
struct RationalSum {
    num: i128,
    den: i128,
}

impl RationalSum {
    fn add(&mut self, value: RationalWeight) {
        if self.den == 0 {
            self.num = value.num as i128;
            self.den = value.den as i128;
            return;
        }
        self.num = self.num * value.den as i128 + value.num as i128 * self.den;
        self.den *= value.den as i128;
        let divisor = gcd(self.num.unsigned_abs(), self.den.unsigned_abs()) as i128;
        if divisor > 1 {
            self.num /= divisor;
            self.den /= divisor;
        }
    }

    fn is_one(self) -> bool {
        self.den != 0 && self.num == self.den
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn report(check_id: &'static str) -> VerificationReport {
    VerificationReport { check_id }
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_value).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), canonicalize_value(&map[key]));
            }
            Value::Object(sorted)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROM_HASH: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const TO_HASH: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
    const SOURCE_HASH: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn valid_input() -> CrosswalkVerificationInput {
        CrosswalkVerificationInput {
            contexts: vec![
                ContextUnitIndex {
                    context_hash: FROM_HASH.to_string(),
                    unit_ids: vec!["P-001".to_string(), "P-002".to_string()],
                },
                ContextUnitIndex {
                    context_hash: TO_HASH.to_string(),
                    unit_ids: vec!["D-01".to_string(), "D-02".to_string()],
                },
            ],
            sources: vec![SourceIndexEntry {
                source_id: "source:crosswalk".to_string(),
                sha256: SOURCE_HASH.to_string(),
            }],
            crosswalks: vec![
                CrosswalkRecord {
                    crosswalk_id: "cw-precinct-district".to_string(),
                    from_context_hash: FROM_HASH.to_string(),
                    to_context_hash: TO_HASH.to_string(),
                    from_unit_id: "P-001".to_string(),
                    to_unit_id: "D-01".to_string(),
                    weight: RationalWeight { num: 1, den: 3 },
                    weight_kind: CrosswalkWeightKind::Population,
                    exhaustive: true,
                    source_refs: vec!["source:crosswalk".to_string()],
                },
                CrosswalkRecord {
                    crosswalk_id: "cw-precinct-district".to_string(),
                    from_context_hash: FROM_HASH.to_string(),
                    to_context_hash: TO_HASH.to_string(),
                    from_unit_id: "P-001".to_string(),
                    to_unit_id: "D-02".to_string(),
                    weight: RationalWeight { num: 2, den: 3 },
                    weight_kind: CrosswalkWeightKind::Population,
                    exhaustive: true,
                    source_refs: vec!["source:crosswalk".to_string()],
                },
            ],
        }
    }

    #[test]
    fn verifies_exhaustive_crosswalk_weight_sum() {
        let reports = verify_crosswalk_input(&valid_input()).expect("valid crosswalk verifies");
        assert!(reports
            .iter()
            .any(|report| report.check_id == "crosswalk_weight_sum"));
    }

    #[test]
    fn rejects_crosswalk_weight_sum_drift() {
        let mut input = valid_input();
        input.crosswalks[1].weight = RationalWeight { num: 1, den: 3 };

        assert!(matches!(
            verify_crosswalk_input(&input),
            Err(RctxCoreError::CrosswalkWeightSum { .. })
        ));
    }

    #[test]
    fn rejects_missing_endpoint_unit() {
        let mut input = valid_input();
        input.crosswalks[0].to_unit_id = "D-99".to_string();

        assert!(matches!(
            verify_crosswalk_input(&input),
            Err(RctxCoreError::MissingCrosswalkUnit { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_crosswalk_row() {
        let mut input = valid_input();
        input.crosswalks[1].to_unit_id = "D-01".to_string();

        assert!(matches!(
            verify_crosswalk_input(&input),
            Err(RctxCoreError::DuplicateCrosswalkRow { .. })
        ));
    }

    #[test]
    fn rejects_missing_source_ref() {
        let mut input = valid_input();
        input.crosswalks[0].source_refs = vec!["source:missing".to_string()];

        assert!(matches!(
            verify_crosswalk_input(&input),
            Err(RctxCoreError::MissingSourceRef { .. })
        ));
    }

    #[test]
    fn crosswalk_hash_is_domain_prefixed_and_stable() {
        let input = valid_input();
        let first = crosswalk_record_hash(&input.crosswalks[0]).unwrap();
        let second = crosswalk_record_hash(&input.crosswalks[0]).unwrap();
        assert_eq!(first, second);
        assert!(is_sha256_hash(&first));
    }

    #[test]
    fn crosswalk_set_hash_is_stable() {
        let input = valid_input();
        let first = crosswalk_set_hash(&input.crosswalks).unwrap();
        let second = crosswalk_set_hash(&input.crosswalks).unwrap();
        assert_eq!(first, second);
        assert!(is_sha256_hash(&first));
    }
}
