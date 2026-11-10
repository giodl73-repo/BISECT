//! Native label comparison checks registry claims and emits instructions only.
use crate::label_registry::LabelEntry;
use serde_json::{json, Value};
use std::collections::BTreeMap;
pub fn validate_labels(a: &str, b: &str) -> Result<(), String> {
    crate::label_rename::validate_name(a)?;
    crate::label_rename::validate_name(b)
}
pub fn check(
    registry: &BTreeMap<String, LabelEntry>,
    a: &str,
    b: &str,
    year: &str,
) -> Result<(), String> {
    validate_labels(a, b)?;
    for label in [a, b] {
        if !registry
            .get(label)
            .is_some_and(|e| e.built.iter().any(|y| y == year))
        {
            return Err(format!("[CONFIG] analyze: '{label}' has not been built for year {year}.\nRun: bisect build {label} --year {year}\n  (label-compare requires both plans to be built for year {year})"));
        }
    }
    for label in [a, b] {
        if !registry
            .get(label)
            .is_some_and(|e| e.analyzed.iter().any(|y| y == year))
        {
            return Err(format!("[CONFIG] report: '{label}' has not been analyzed for year {year}.\nRun: bisect analyze {label} --year {year}\n  (label-compare requires both plans to be analyzed for year {year})"));
        }
    }
    Ok(())
}
pub fn summary(a: &str, b: &str, year: &str) -> Value {
    json!({"label_a":a,"label_b":b,"year":year,"analysis_a":format!("analysis/{a}/{year}"),"analysis_b":format!("analysis/{b}/{year}"),"status":"delegated_to_compare","next_step":format!("bisect compare --plan-a {a} --plan-b {b} --year {year} --format json")})
}
pub fn render(a: &str, b: &str, year: &str, json_output: bool) -> Result<String, String> {
    if json_output {
        return serde_json::to_string_pretty(&summary(a, b, year))
            .map_err(|e| format!("[INTERNAL] label-compare: serialize failed: {e}"));
    }
    Ok(format!("label-compare: {a} vs {b} (year {year})\nanalysis/{a}/{year}/ vs analysis/{b}/{year}/\n\nBoth plans are built and analyzed for year {year}.\n\nTo compare:  bisect compare --plan-a {a} --plan-b {b} --year {year}\nJSON output: bisect compare --plan-a {a} --plan-b {b} --year {year} --format json\nNarrative:   bisect compare --plan-a {a} --plan-b {b} --year {year} --format narrative"))
}
