fn main()->Result<(),Box<dyn std::error::Error>>{
 let manifest=fletch_core::read_cache_manifest_json(std::env::args().nth(1).ok_or("manifest path")?)?;
 println!("{}",serde_json::to_string_pretty(&fletch_core::verify_cache_manifest(&manifest)?)?);Ok(())
}
