fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let r = bisect_tui::screens::compare::compute_compare_result(
        std::path::Path::new(&args[0]),
        std::path::Path::new(&args[1]),
        "a",
        "b",
    );
    let mut value = serde_json::to_value(&r)?;
    value["splits_a"] = serde_json::json!(r.splits_a.to_string());
    value["splits_b"] = serde_json::json!(r.splits_b.to_string());
    println!("{value}");
    Ok(())
}
