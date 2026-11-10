//! Test/reference writer for deliberately changed typed contexts.
//! Recomputes existing native hashes; it does not authenticate supplied data.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("context JSON path required")?;
    let mut context: rplan_core::RplanContext = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    context.units.unit_universe_hash = context.units.compute_unit_universe_hash()?;
    print!("{}", rplan_io::write_rctx_string(&context)?);
    Ok(())
}
