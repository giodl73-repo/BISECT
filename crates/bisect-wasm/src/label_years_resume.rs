//! Saved checkpoints are evidence to verify, never trusted cached measurements.
use serde_json::Value;
pub(crate) fn prefix(
    saved: &Value,
    schema: &str,
    label: &str,
    requested: &[String],
) -> Result<usize, String> {
    let years = saved["years"]
        .as_array()
        .ok_or("Resume requires a saved year checkpoint.")?;
    if saved["schema_version"] != schema
        || saved["label"] != label
        || saved["status"] != "in-progress"
        || saved["requested_years"] != serde_json::json!(requested)
        || years.is_empty()
        || years.len() >= requested.len()
        || years.iter().enumerate().any(|(i, row)| {
            row["year"] != requested[i]
                || !["completed", "failed"].contains(&row["status"].as_str().unwrap_or(""))
        })
    {
        return Err(
            "Resume requires an incomplete matching prefix of the requested label years.".into(),
        );
    }
    if serde_json::to_vec(saved).map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
        return Err("Saved year checkpoint exceeds 64 MiB.".into());
    }
    Ok(years.len())
}
pub(crate) fn verify(saved: &Value, fresh: &Value) -> Result<(), String> {
    for (key, value) in fresh
        .as_object()
        .ok_or("Missing freshly verified checkpoint.")?
    {
        if !saved.get(key).is_some_and(|old| equivalent(old, value)) {
            return Err(format!("Saved year checkpoint differs from fresh native output: {key}. Run the full request again."));
        }
    }
    Ok(())
}
// JavaScript serializes integral f64 metadata such as 1.0 as 1. Native file
// strings and all integers beyond the exact JavaScript range remain exact.
fn equivalent(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            x == y
                || x.as_f64()
                    .zip(y.as_f64())
                    .is_some_and(|(x, y)| x == y && x.abs() <= 9_007_199_254_740_991.0)
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| equivalent(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| equivalent(v, w)))
        }
        _ => a == b,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_integral_metadata_preserves_exact_sources() {
        assert!(equivalent(&serde_json::json!(1), &serde_json::json!(1.0)));
        assert!(equivalent(&serde_json::json!(0), &serde_json::json!(-0.0)));
        assert!(!equivalent(&serde_json::json!(1), &serde_json::json!(1.01)));
        assert!(!equivalent(
            &serde_json::json!(9_007_199_254_740_993u64),
            &serde_json::json!(9_007_199_254_740_992.0)
        ));
        assert!(!equivalent(
            &serde_json::json!("1.0"),
            &serde_json::json!("1")
        ));
    }
}
