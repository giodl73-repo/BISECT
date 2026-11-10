fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let base = fletch_core::read_cache_manifest_json(&args[0])?;
    let entries: Vec<fletch_core::CacheEntry> = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let merged = fletch_core::upsert_cache_manifest_entries(base, entries)?;
    print!("{}", serde_json::to_string_pretty(&merged)?);
    Ok(())
}
