use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
const PATHS: [&str; 5] = [
    "data/final_assignments.json",
    "analysis/compactness.json",
    "analysis/summary.json",
    "analysis/splits.json",
    "analysis/contiguity.json",
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Side {
    pub label: String,
    pub sources: BTreeMap<String, String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub a: Side,
    pub b: Side,
}
pub(crate) fn compare(input: Input) -> Result<Value, String> {
    let mut total = 0;
    let mut hashes = Vec::new();
    let mut defaults = Vec::new();
    for (name, side) in [("a", &input.a), ("b", &input.b)] {
        if side.label.len() > 200
            || side.label.chars().any(char::is_control)
            || side
                .sources
                .keys()
                .any(|key| !PATHS.contains(&key.as_str()))
        {
            return Err("Compare labels support 200 bytes without controls; only five native source paths are accepted.".into());
        }
        for (path, text) in &side.sources {
            if text.len() > 8 * 1024 * 1024 {
                return Err("Compare sources are limited to 8 MiB each.".into());
            }
            total += text.len();
            hashes.push(json!({"side":name,"path":path,"bytes":text.len(),"sha256":format!("{:x}",Sha256::digest(text.as_bytes()))}));
        }
        let get = |path: &str| side.sources.get(path).map(String::as_str);
        for (path, known, fallback) in [
            (
                PATHS[0],
                !bisect_report::tui_compare::assignments(get(PATHS[0])).is_empty(),
                "empty assignment map; native score becomes zero if either side is empty",
            ),
            (
                PATHS[1],
                bisect_report::tui_compare::mean_pp(get(PATHS[1])).is_some(),
                "compactness = 0",
            ),
            (
                PATHS[2],
                bisect_report::tui_compare::max_dev(get(PATHS[2])).is_some(),
                "deviation = 0",
            ),
            (
                PATHS[3],
                bisect_report::tui_compare::splits(get(PATHS[3])).is_some(),
                "splits = 0",
            ),
            (
                PATHS[4],
                bisect_report::tui_compare::contiguous(get(PATHS[4])).is_some(),
                "contiguous = true",
            ),
        ] {
            if !known {
                defaults.push(json!({"side":name,"path":path,"fallback":fallback}));
            }
        }
    }
    if total > 16 * 1024 * 1024 {
        return Err("Compare sources exceed 16 MiB total.".into());
    }
    let native = bisect_report::tui_compare::compare(&input.a.sources, &input.b.sources);
    if [
        native.mean_pp_a,
        native.mean_pp_b,
        native.max_dev_a,
        native.max_dev_b,
    ]
    .iter()
    .any(|n| !n.is_finite())
    {
        return Err("Native metric computation produced a nonfinite value.".into());
    }
    let mut metrics = serde_json::to_value(&native).map_err(|e| e.to_string())?;
    metrics["splits_a"] = json!(native.splits_a.to_string());
    metrics["splits_b"] = json!(native.splits_b.to_string());
    Ok(
        json!({"schema_version":"bisect-tui-comparison-v1","status":"compared","label_a":input.a.label,"label_b":input.b.label,"metrics":metrics,"native_defaults":defaults,"source_hashes":hashes,"execution_performed":false,"scope":"Shared native TUI Compare calculation over five explicitly selected source texts per plan. Its score is exact matching tract labels divided by the larger assignment count; it is not set-union Jaccard or label-invariant similarity. Missing/malformed metrics use native zero/true defaults listed below; these are not verified measurements. Native Compare reads districts[].polsby_popper and split, unlike Home summary fields. most_changed remains empty as in the native implementation. No paths are opened, plans executed or sources authenticated. Project Open is inert."}),
    )
}
