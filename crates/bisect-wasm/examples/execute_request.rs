//! Native reference for the same typed request executed by the WASM boundary.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("engine request JSON path required")?;
    let request = serde_json::from_slice(&std::fs::read(path)?)?;
    println!(
        "{}",
        serde_json::to_string(&bisect_wasm::engine::execute(request)?)?
    );
    Ok(())
}
