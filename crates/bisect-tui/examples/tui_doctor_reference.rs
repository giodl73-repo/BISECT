fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let checks = bisect_tui::screens::doctor::run_checks(&args[0], &args[1], &args[2]);
    println!("{}", serde_json::to_string(&checks)?);
    Ok(())
}
