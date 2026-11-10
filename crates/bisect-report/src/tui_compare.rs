//! Native TUI Compare projection over selected source text, without filesystem I/O.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CompareResult {
    pub jaccard: f64,
    pub mean_pp_a: f64,
    pub mean_pp_b: f64,
    pub max_dev_a: f64,
    pub max_dev_b: f64,
    pub splits_a: u64,
    pub splits_b: u64,
    pub contiguous_a: bool,
    pub contiguous_b: bool,
    pub most_changed: Vec<(u64, f64)>,
}
fn json(source: Option<&str>) -> Option<Value> {
    serde_json::from_str(source?).ok()
}
pub fn mean_pp(source: Option<&str>) -> Option<f64> {
    let v = json(source)?;
    let vals: Vec<_> = v["districts"]
        .as_array()?
        .iter()
        .filter_map(|x| x["polsby_popper"].as_f64())
        .collect();
    if vals.is_empty() {
        None
    } else {
        Some(vals.iter().sum::<f64>() / vals.len() as f64)
    }
}
pub fn max_dev(source: Option<&str>) -> Option<f64> {
    json(source)?["max_deviation_pct"].as_f64()
}
pub fn splits(source: Option<&str>) -> Option<u64> {
    json(source)?["split"].as_u64()
}
pub fn contiguous(source: Option<&str>) -> Option<bool> {
    json(source)?["all_contiguous"].as_bool()
}
pub fn assignments(source: Option<&str>) -> HashMap<String, u64> {
    source
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}
pub fn compare(a: &BTreeMap<String, String>, b: &BTreeMap<String, String>) -> CompareResult {
    let source = |files: &BTreeMap<String, String>, name: &str| files.get(name).cloned();
    let aa = assignments(source(a, "data/final_assignments.json").as_deref());
    let bb = assignments(source(b, "data/final_assignments.json").as_deref());
    let matching = aa.iter().filter(|(g, d)| bb.get(*g) == Some(*d)).count();
    CompareResult {
        jaccard: if aa.is_empty() || bb.is_empty() {
            0.0
        } else {
            matching as f64 / aa.len().max(bb.len()) as f64
        },
        mean_pp_a: mean_pp(source(a, "analysis/compactness.json").as_deref()).unwrap_or(0.0),
        mean_pp_b: mean_pp(source(b, "analysis/compactness.json").as_deref()).unwrap_or(0.0),
        max_dev_a: max_dev(source(a, "analysis/summary.json").as_deref()).unwrap_or(0.0),
        max_dev_b: max_dev(source(b, "analysis/summary.json").as_deref()).unwrap_or(0.0),
        splits_a: splits(source(a, "analysis/splits.json").as_deref()).unwrap_or(0),
        splits_b: splits(source(b, "analysis/splits.json").as_deref()).unwrap_or(0),
        contiguous_a: contiguous(source(a, "analysis/contiguity.json").as_deref()).unwrap_or(true),
        contiguous_b: contiguous(source(b, "analysis/contiguity.json").as_deref()).unwrap_or(true),
        most_changed: vec![],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_score_and_fallback() {
        let a = BTreeMap::from([(
            "data/final_assignments.json".into(),
            r#"{"a":1,"b":2}"#.into(),
        )]);
        let b = BTreeMap::from([(
            "data/final_assignments.json".into(),
            r#"{"a":1,"c":2}"#.into(),
        )]);
        let r = compare(&a, &b);
        assert_eq!(r.jaccard, 0.5);
        assert!(r.contiguous_a);
        assert_eq!(r.splits_a, 0);
        assert!(r.most_changed.is_empty());
        assert!(assignments(Some(r#"{"a":1,"b":"bad"}"#)).is_empty());
    }
    #[test]
    fn compare_sources_differ_from_home() {
        assert_eq!(
            mean_pp(Some(
                r#"{"mean_polsby_popper":0.9,"districts":[{"polsby_popper":0.2},{"polsby_popper":0.4},{"polsby_popper":"bad"}]}"#
            )),
            Some(0.30000000000000004)
        );
        assert_eq!(
            splits(Some(r#"{"split_count":9,"split":18446744073709551615}"#)),
            Some(u64::MAX)
        );
    }
}
