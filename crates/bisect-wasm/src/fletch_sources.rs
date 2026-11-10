use serde_json::{json, Value};
pub(crate) fn inspect(input: crate::fetch_plan::Input) -> Result<Value, String> {
    let plan = crate::fetch_plan::plan(input)?;
    let items = plan["items"]
        .as_array()
        .ok_or("Missing source plan items")?
        .iter()
        .map(|r| {
            let text = |k: &str| {
                r[k].as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("Missing source field {k}"))
            };
            Ok(bisect_core::fletch_sources::GeneratedFetchSource {
                state_code: text("state_code")?,
                year: text("year")?,
                kind: text("kind")?,
                url: r["url"].as_str().map(str::to_owned),
                cache_targets: text("local_path")?,
                done_marker: text("done_marker")?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let report = bisect_core::fletch_sources::generated_source_handoff(&items);
    let csv = bisect_core::fletch_sources::source_handoff_csv(&report)?;
    let failures = bisect_core::fletch_sources::fletch_source_handoff_gate_failures(&report);
    Ok(
        json!({"schema_version":"bisect-fletch-source-handoff-v1","report":report,"report_csv":csv,"gate_passed":failures.is_empty(),"gate_failures":failures,"source_plan":plan,"execution_performed":false,
        "scope":"Native BISECT generated source registry handoff. Portable target paths use forward slashes. HTTP readiness and adapter-required rows describe handoff metadata, not successful acquisition. No URL is requested, no files are opened or installed, and no engine or admissibility check is run. Download/cache success does not validate a district plan."}),
    )
}
