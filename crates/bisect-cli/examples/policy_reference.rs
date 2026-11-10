//! Exercise the actual native policy command for browser parity.
fn main()->anyhow::Result<()> {
    let args:Vec<String>=std::env::args().skip(1).collect();
    if args.len()<2 {anyhow::bail!("Expected query and format, optionally override path.");}
    std::env::remove_var("BISECT_LOCATION_POLICY");std::env::remove_var("BISECT_STATE_POLICY");
    if let Some(path)=args.get(2){std::env::set_var("BISECT_LOCATION_POLICY",path);}
    bisect_cli::policy::run_policy(&bisect_cli::args::PolicyArgs{state:args[0].clone(),format:args[1].clone()})
}
