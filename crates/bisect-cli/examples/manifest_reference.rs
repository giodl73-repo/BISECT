fn main()->anyhow::Result<()>{
 let path=std::env::args().nth(1).ok_or_else(||anyhow::anyhow!("Expected manifest path"))?;
 let status=bisect_cli::doctor::run_verify_manifest(&path);std::process::exit(status);
}
