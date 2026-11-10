fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let v = bisect_report::tui_verify::parse_output(
        &args[0],
        &std::fs::read_to_string(&args[1])?,
        &std::fs::read_to_string(&args[2])?,
    );
    println!("{}", serde_json::to_string(&v)?);
    Ok(())
}
