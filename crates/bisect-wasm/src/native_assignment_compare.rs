//! Direct assignment comparison using the desktop formatter and metric kernel.
use bisect_analysis::comparison::{
    format_comparison_csv, format_comparison_json, format_comparison_table,
    parse_direct_assignment_object, try_compare_plans,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};

const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
// The narrative path retains a separate limit on distinct districts.
const MAX_DISTRICTS_PER_SIDE: usize = 500;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    plan_a: Source,
    plan_b: Source,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    label: String,
    text: String,
}

impl Source {
    fn parse(&self) -> Result<HashMap<String, u64>, String> {
        if self.label.is_empty() || self.label.len() > 1024 || self.label.chars().any(char::is_control) {
            return Err("Use a nonempty source label without control characters (at most 1024 bytes).".into());
        }
        if self.text.len() > MAX_SOURCE_BYTES {
            return Err("Each assignment file must be at most 16 MiB.".into());
        }
        let value: Value = serde_json::from_str(&self.text)
            .map_err(|error| format!("[INPUT] cannot parse {}: {error}", self.label))?;
        if self.label.ends_with(".rplan") {
            let assignments = value.get("assignments").and_then(Value::as_object).ok_or_else(|| {
                format!(".rplan file '{}' missing 'assignments' field", self.label)
            })?;
            return self.parse_object(assignments);
        }
        let object = value.as_object().ok_or_else(|| {
            format!("[INPUT] assignment file {} must be a JSON object", self.label)
        })?;
        if let Some(nested) = object.get("assignment") {
            let assignment = nested.as_object().ok_or_else(|| {
                format!("[INPUT] assignment field in {} must be an object", self.label)
            })?;
            if !assignment.is_empty() {
                return self.parse_object(assignment);
            }
        }
        self.parse_object(object)
    }

    fn parse_object(&self, object: &serde_json::Map<String, Value>) -> Result<HashMap<String, u64>, String> {
        parse_direct_assignment_object(&self.label, object)
    }
}

fn ordered_labels(assignments: &HashMap<String, u64>) -> Vec<u64> {
    assignments.values().copied().collect::<BTreeSet<_>>().into_iter().collect()
}

// Narrative has separate analysis/formatting work and retains its existing limit.
pub(crate) fn ranks(assignments: &HashMap<String, u64>) -> Result<Vec<u64>, String> {
    let ordered = ordered_labels(assignments);
    if ordered.len() > MAX_DISTRICTS_PER_SIDE {
        return Err("Assignment file exceeds the 500-district comparison limit.".into());
    }
    Ok(ordered)
}

pub(crate) fn comparison_ids(assignments: HashMap<String, u64>, ordered: &[u64]) -> HashMap<String, usize> {
    // Preserve exact grouping and the native iteration order, including IDs
    // beyond 32-bit and JS safe-integer ranges. Native formats do not emit IDs.
    assignments.into_iter().map(|(unit, district)| {
        (unit, ordered.binary_search(&district).expect("ranked district") + 1)
    }).collect()
}

pub(crate) fn run(input: Input) -> Result<Value, String> {
    let a = input.plan_a.parse()?;
    let b = input.plan_b.parse()?;
    // Source bytes remain bounded. The shared comparison now visits only
    // observed district pairs, so the old count guards are unnecessary here.
    let ids_a = ordered_labels(&a);
    let ids_b = ordered_labels(&b);
    let common_units = a.keys().filter(|unit| b.contains_key(*unit)).count();
    let a_only = a.len() - common_units;
    let b_only = b.len() - common_units;
    let mut comparison = try_compare_plans(&comparison_ids(a, &ids_a), &comparison_ids(b, &ids_b))
        .map_err(|error| error.to_string())?;
    comparison.plan_a.label = input.plan_a.label;
    comparison.plan_b.label = input.plan_b.label;
    Ok(json!({
        "schema_version": "bisect-native-direct-assignment-comparison-v1",
        "table": format_comparison_table(&comparison),
        "json": format_comparison_json(&comparison),
        "csv": format_comparison_csv(&comparison),
        "source_sha256": {
            "plan_a": format!("{:x}", Sha256::digest(input.plan_a.text.as_bytes())),
            "plan_b": format!("{:x}", Sha256::digest(input.plan_b.text.as_bytes())),
        },
        "units": {"common":common_units,"only_a":a_only,"only_b":b_only},
        "scope": "Native direct-file assignment comparison. The native population output uses unit counts; compactness zeros are placeholders. Unit IDs are compared literally, without geography vintage validation or a crosswalk.",
    }))
}
