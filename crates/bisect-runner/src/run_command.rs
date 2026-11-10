//! Actual native `bisect run` configuration, with bounded inert inspection.
use crate::run_args::RunArgs;
use crate::state_config::AlgorithmConfig;
use clap::{Parser,CommandFactory,FromArgMatches};
use serde_json::{json,Value};
#[derive(Debug,Clone)]
pub struct RunConfiguration{pub debug:bool,pub reset:bool,pub reprocess:bool,pub algo:AlgorithmConfig}
impl RunConfiguration{
 pub fn from_args(args:&RunArgs)->Self{Self{debug:args.debug,reset:args.reset,reprocess:args.reprocess,algo:AlgorithmConfig::defaults_for_mode(&args.partition_mode)}}
 pub fn prepared_configuration(&self)->crate::states_config::StatesConfiguration{crate::states_config::StatesConfiguration{debug:self.debug,reprocess:self.reprocess,algo:self.algo.clone(),balance_tolerance:None,multiscale_fine:"tract".into(),multiscale_coarse:"county".into(),smc_resample_threshold:0.5}}
}
pub fn years(args:&RunArgs)->Vec<String>{if args.year=="all"{["2020","2010","2000"].into_iter().map(str::to_owned).collect()}else{vec![args.year.clone()]}}
pub fn output_dir(args:&RunArgs)->String{args.output_dir.clone().unwrap_or_else(||format!("outputs/{}",args.version))}
fn bounded(arguments:&[String])->Result<(),String>{if arguments.len()>256||arguments.iter().any(|v|v.len()>4096||v.contains('\0'))||arguments.iter().map(String::len).sum::<usize>()>64*1024{return Err("Run commands allow 256 bounded UTF-8 arguments, 4096 bytes each and 64 KiB total, without NUL bytes.".into());}Ok(())}
pub fn parse(arguments:Vec<String>)->Result<RunArgs,String>{bounded(&arguments)?;RunArgs::try_parse_from(std::iter::once("bisect run".to_owned()).chain(arguments)).map_err(|e|e.to_string())}
pub fn inspect(arguments:Vec<String>)->Result<Value,String>{
 bounded(&arguments)?;
 let command=RunArgs::command().bin_name("bisect run").color(clap::ColorChoice::Never);
 let matches=match command.clone().try_get_matches_from(std::iter::once("bisect run".to_owned()).chain(arguments.iter().cloned())){Ok(m)=>m,Err(e)=>return Ok(json!({"schema_version":"bisect-native-run-command-v1","status":if e.kind()==clap::error::ErrorKind::DisplayHelp{"help"}else{"parse-rejected"},"arguments":arguments,"target_pointer_width":usize::BITS,"parse_error_kind":format!("{:?}",e.kind()),"message":e.to_string(),"execution_performed":false,"filesystem_reads_performed":false}))};
 let args=RunArgs::from_arg_matches(&matches).map_err(|e|e.to_string())?;let config=RunConfiguration::from_args(&args);
 let fields=command.get_arguments().filter(|a|a.get_id().as_str()!="help").map(|arg|{let id=arg.get_id().as_str();json!({"id":id,"long":arg.get_long(),"short":arg.get_short().map(|c|c.to_string()),"help":arg.get_help().map(|v|v.to_string()),"source":matches.value_source(id).map(|s|match s{clap::parser::ValueSource::CommandLine=>"command-line",clap::parser::ValueSource::DefaultValue=>"default",_=>"other"}).unwrap_or("absent"),"values":matches.get_raw(id).map(|v|v.map(|s|s.to_string_lossy().into_owned()).collect::<Vec<_>>()).unwrap_or_default()})}).collect::<Vec<_>>();
 Ok(json!({"schema_version":"bisect-native-run-command-v1","status":"parsed","arguments":arguments,"target_pointer_width":usize::BITS,"fields":fields,"year":args.year,"years":years(&args),"states":args.states,"version":args.version,"output_dir":output_dir(&args),"workers":args.workers.to_string(),"print_only":args.print_only,"native_print_only_applied":false,"native_unused_fields":["dpi","election_year","skip_analysis","skip_political","skip_demographic","stages","print_only","processing_mode","minimum_boundary_length","run_type","experiment_name"],"effective_configuration":{"algorithm_debug":format!("{:?}",config.algo),"seeds_per_split":config.algo.seeds.seed_count().to_string(),"debug":config.debug,"reset":config.reset,"reprocess":config.reprocess,"balance_tolerance_fraction":Value::Null,"multiscale_fine":"tract","multiscale_coarse":"county","smc_resample_threshold":0.5},"native_debug":format!("{args:?}"),"execution_performed":false,"filesystem_reads_performed":false,"scope":"Actual native Run parser and shared default-mode compositor. Native year=all expands in 2020/2010/2000 order; other year strings are accepted by the parser and resolved by later manifest selection. The listed unused fields are parsed but not consumed by the current native Run branch, including print-only. Browser explicit execution separately honors print-only intent. Paths/reset/workers remain intent in this inert inspection. No files, scheduling, acquisition, analysis, deletion or partition occurs."}))
}
