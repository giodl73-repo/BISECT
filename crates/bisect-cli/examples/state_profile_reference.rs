//! Parse native State/States arguments and evaluate only the native run filter.
use clap::Parser;
fn main(){
 let file=std::env::args().nth(1).expect("request path");
 let args:Vec<String>=serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
 let cli=bisect_cli::args::Cli::try_parse_from(std::iter::once("bisect".to_owned()).chain(args)).unwrap();
 let result=match cli.command {
  bisect_cli::args::Commands::State(a)=>bisect_cli::args::validate_court_profile(a.profile,a.partition_mode,a.structure,a.weights_override,a.search,&a.partisan_shares),
  bisect_cli::args::Commands::States(a)=>bisect_cli::args::validate_court_profile(a.profile,a.partition_mode,None,None,a.search,&None),
  _=>panic!("Expected state or states")
 };
 println!("{}",serde_json::json!({"allowed":result.is_ok(),"error":result.err()}));
}
