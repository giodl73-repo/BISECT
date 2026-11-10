//! Native CLI parser/compositor oracle. Build before shared-source extraction.
use bisect_cli::args::{Cli,Commands};
use bisect_cli::runner::AlgorithmConfig;
use clap::Parser;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let argv:Vec<String>=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Argument JSON path required")?)?)?;
 match Cli::try_parse_from([vec!["bisect".into(),"state".into()],argv].concat()){
  Ok(cli)=>match cli.command{Commands::State(args)=>{let profile=bisect_cli::args::validate_court_profile(args.profile,args.partition_mode,args.structure,args.weights_override,args.search,&args.partisan_shares);let algo=AlgorithmConfig::from_state_args(&args);println!("{}",serde_json::json!({"parsed":format!("{args:?}"),"algorithm":format!("{algo:?}"),"mode":algo.mode_name(),"profile_error":profile.err()}));},_=>return Err("State required".into())},
  Err(error)=>println!("{}",serde_json::json!({"parse_error_kind":format!("{:?}",error.kind())}))
 };Ok(())
}
