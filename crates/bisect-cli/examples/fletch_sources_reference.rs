fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(args.get(1).ok_or("request path")?)?)?;
    let items = value
        .as_array()
        .ok_or("source items array")?
        .iter()
        .map(|r| {
            let text = |k: &str| {
                r[k].as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("Missing {k}"))
            };
            Ok(bisect_cli::fetch::FetchItem {
                state_code: text("state_code")?,
                year: text("year")?,
                kind: text("kind")?,
                url: r["url"].as_str().map(str::to_owned),
                local_path: text("local_path")?.into(),
                done_marker: text("done_marker")?.into(),
                available_locally: false,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let registry = bisect_cli::fletch::fletch_registry_from_items(&items);
    let report = bisect_cli::fletch::fletch_source_handoff_report(&registry);
    bisect_cli::fletch::write_fletch_source_handoff(
        args.get(2).ok_or("CSV path")?.as_ref(),
        &report,
    )?;
    println!(
        "{}",
        serde_json::to_string(
            &serde_json::json!({"report":report,"gate_failures":bisect_cli::fletch::fletch_source_handoff_gate_failures(&report)})
        )?
    );
    Ok(())
}
