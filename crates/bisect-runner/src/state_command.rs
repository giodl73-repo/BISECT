//! Inspect the actual shared State parser and compositor without reading paths.
use clap::{CommandFactory,FromArgMatches,Parser};
use serde_json::{json,Value};
use crate::state_args::{StateArgs,validate_court_profile};
use crate::state_config::{AlgorithmConfig,SeedCompositor,SplitStrategy};

fn number(value:f64)->Value{
 if value.is_finite()&&!(value==0.0&&value.is_sign_negative()){json!(value)}else{json!({"numeric_class":if value.is_nan(){"nan"}else if value==0.0{"negative-zero"}else if value>0.0{"positive-infinity"}else{"negative-infinity"},"display":value.to_string()})}
}
fn count(value:usize)->Value{if value as u128>9_007_199_254_740_991{json!(value.to_string())}else{json!(value)}}
fn search_name(search:&SeedCompositor)->&'static str{match search{
 SeedCompositor::Single=>"single",SeedCompositor::Multi{..}=>"multi",SeedCompositor::ConvergenceSweep{..}=>"convergence",SeedCompositor::Percentile{..}=>"percentile",SeedCompositor::BisectionEnsemble{..}=>"bisection-ensemble",SeedCompositor::Flip{..}=>"flip",SeedCompositor::ShortBurst{..}=>"short-burst",SeedCompositor::ShortBurstForest{..}=>"short-burst-forest",SeedCompositor::ShortBurstMergeSplit{..}=>"short-burst-merge-split",SeedCompositor::ForestRecom{..}=>"forest-recom",SeedCompositor::MultiScale{..}=>"multiscale",SeedCompositor::MergeSplit{..}=>"merge-split",SeedCompositor::MultiScaleAdaptive{..}=>"multiscale-adaptive",SeedCompositor::ParallelTempering{..}=>"parallel-tempering",SeedCompositor::VraRecom{..}=>"vra-recom",SeedCompositor::SmcPercentile{..}=>"smc-percentile"}}
pub fn inspect(arguments:Vec<String>)->Result<Value,String>{
 if arguments.len()>256||arguments.iter().any(|v|v.len()>4096||v.contains('\0'))||arguments.iter().map(|v|v.len()).sum::<usize>()>64*1024{return Err("State command inspection allows 256 UTF-8 arguments, 4096 bytes each and 64 KiB total, without NUL bytes.".into());}
 let command=StateArgs::command().bin_name("bisect state").color(clap::ColorChoice::Never);
 let matches=match command.clone().try_get_matches_from(std::iter::once("bisect state".to_owned()).chain(arguments.iter().cloned())){
  Ok(matches)=>matches,Err(error)=>return Ok(json!({"schema_version":"bisect-native-state-command-v1","status":if error.kind()==clap::error::ErrorKind::DisplayHelp{"help"}else{"parse-rejected"},"arguments":arguments,"target_pointer_width":usize::BITS,"parse_error_kind":format!("{:?}",error.kind()),"message":error.to_string(),"execution_performed":false,"filesystem_reads_performed":false}))
 };
 let args=StateArgs::from_arg_matches(&matches).map_err(|e|e.to_string())?;
 let profile=validate_court_profile(args.profile,args.partition_mode,args.structure,args.weights_override,args.search,&args.partisan_shares);
 let algorithm=AlgorithmConfig::from_state_args(&args);let weights=&algorithm.weights;
 let fields=command.get_arguments().filter(|arg|arg.get_id().as_str()!="help").map(|arg|{let id=arg.get_id().as_str();json!({"id":id,"long":arg.get_long(),"short":arg.get_short().map(|v|v.to_string()),"help":arg.get_help().map(|v|v.to_string()),"source":matches.value_source(id).map(|s|match s{clap::parser::ValueSource::CommandLine=>"command-line",clap::parser::ValueSource::DefaultValue=>"default",_=>"other"}).unwrap_or("absent"),"values":matches.get_raw(id).map(|values|values.map(|v|v.to_string_lossy().into_owned()).collect::<Vec<_>>()).unwrap_or_default()})}).collect::<Vec<_>>();
 let dispatch=match &algorithm.split{SplitStrategy::Bisect=>"recursive-bisection-search",SplitStrategy::GeoSection|SplitStrategy::AreaSection{..}|SplitStrategy::VraSection{..}|SplitStrategy::CompactBisect{..}|SplitStrategy::ProportionalSection{..}=>"seed-compositor-count-per-split",SplitStrategy::ApportionRegions=>"native-prime-factor-dispatch",_=>"structure-does-not-dispatch-seed-search"};
 let engine=match algorithm.metis.engine{bisect_apportion::split::MetisEngine::CFfi=>"c-ffi",bisect_apportion::split::MetisEngine::RedistMetis=>"metis-core",bisect_apportion::split::MetisEngine::Gpmetis=>"gpmetis"};
 Ok(json!({"schema_version":"bisect-native-state-command-v1","status":"parsed","arguments":arguments,"target_pointer_width":usize::BITS,"state":args.state,"year":args.year,"resolution":args.resolution.to_string(),"fields":fields,"profile_filter":{"allowed":profile.is_ok(),"error":profile.err()},"effective_algorithm":{"mode":algorithm.mode_name(),"structure":algorithm.split.mode_name(),"search":search_name(&algorithm.seeds),"seed_compositor_budget":count(algorithm.seeds.seed_count()),"dispatch_interpretation":dispatch,"metis":{"engine":engine,"ufactor":algorithm.metis.ufactor,"niter":algorithm.metis.niter,"seed":algorithm.metis.seed.map(|v|v.to_string())},"weights":{"geographic":weights.geographic,"minority_weighting":weights.minority_weighting,"partisan_source":weights.partisan_shares.as_ref().map(|p|p.to_string_lossy().into_owned()),"dem_threshold":number(weights.dem_threshold),"rep_threshold":number(weights.rep_threshold),"alpha_county":number(weights.alpha_county),"alpha_mcd":number(weights.alpha_mcd),"alpha_place":number(weights.alpha_place),"alpha_vtd":number(weights.alpha_vtd),"directional_lambda":number(weights.directional_lambda),"economic_character":weights.economic_character,"housing_character":weights.housing_character,"econ_alpha":number(weights.econ_alpha),"zone_membership":weights.zone_membership,"zone_alpha":number(weights.zone_alpha)},"vertex_constraints":algorithm.vertex_constraints.iter().map(|v|format!("{v:?}")).collect::<Vec<_>>()},"native_debug":{"parsed":format!("{args:?}"),"algorithm":format!("{algorithm:?}")},"execution_performed":false,"filesystem_reads_performed":false,"scope":"Shared native State argument parser and exact compositor conversion, including defaults and overrides. This is configuration inspection, not execution: no paths are read, reset/force/output flags cause no filesystem operations, and no graph, data, resource or plan validation occurs. Native usize ranges follow the loaded target pointer width. The profile filter reports the existing native flag policy; it is not plan certification. Configured seed/search layers may be unused or used only for their count by the chosen structure."}))
}

/// Bounded typed State parser for in-memory execution adapters. No paths are read.
pub fn parse(arguments:Vec<String>)->Result<StateArgs,String>{
 if arguments.len()>256||arguments.iter().any(|v|v.len()>4096||v.contains('\0'))||arguments.iter().map(|v|v.len()).sum::<usize>()>64*1024{return Err("State command exceeds bounded argument limits.".into());}
 StateArgs::try_parse_from(std::iter::once("bisect state".to_owned()).chain(arguments)).map_err(|e|e.to_string())
}

/// Change only the scope used by a congressional source job. Canonical long
/// flags come from the actual clap declaration; values never pass through JS
/// numbers or shell tokenization. Native defaults remain native defaults.
pub fn scope(arguments:Vec<String>,state:String,year:String,resolution:String,districts:usize)->Result<Value,String>{
 if state.len()!=2||!state.bytes().all(|b|b.is_ascii_uppercase())||!["2000","2010","2020"].contains(&year.as_str())||!["tract","block_group","block"].contains(&resolution.as_str())||!(1..=500).contains(&districts){return Err("Source scope requires a two-letter state, Census vintage, native resolution and 1-500 congressional districts.".into());}
 let before=parse(arguments.clone())?;
 let command=StateArgs::command().color(clap::ColorChoice::Never);
 let matches=command.clone().try_get_matches_from(std::iter::once("bisect state".to_owned()).chain(arguments.iter().cloned())).map_err(|e|e.to_string())?;
 let mut scoped=vec!["--state".into(),state.clone(),"--year".into(),year.clone(),"--resolution".into(),resolution.clone(),"--chamber".into(),"congressional".into(),"--districts".into(),districts.to_string()];
 for arg in command.get_arguments(){
  let id=arg.get_id().as_str();
  if ["help","state","year","resolution","chamber","districts"].contains(&id)||matches.value_source(id)!=Some(clap::parser::ValueSource::CommandLine){continue;}
  let name=arg.get_long().ok_or_else(||format!("Native State flag {id} has no canonical long form."))?;
  match arg.get_action(){
   clap::ArgAction::Set|clap::ArgAction::Append=>{for raw in matches.get_raw(id).into_iter().flatten(){scoped.push(format!("--{name}={}",raw.to_str().ok_or("State flag is not UTF-8.")?));}},
   clap::ArgAction::SetTrue|clap::ArgAction::SetFalse=>scoped.push(format!("--{name}")),
   _=>return Err(format!("State flag {name} needs an explicit source-scope adapter for its clap action.")),
  }
 }
 let after=parse(scoped.clone())?;
 let algorithm=format!("{:?}",AlgorithmConfig::from_state_args(&before));
 if algorithm!=format!("{:?}",AlgorithmConfig::from_state_args(&after)){return Err("Source scope unexpectedly changed native algorithm configuration.".into());}
 Ok(json!({"schema_version":"bisect-native-state-scoped-command-v1","source_arguments":arguments,"arguments":scoped,"scope":{"state":state,"year":year,"resolution":resolution,"chamber":"congressional","districts":districts},"algorithm_unchanged":true,"native_algorithm_debug":algorithm,"execution_performed":false,"filesystem_reads_performed":false,"scope_policy":"Only state, year, resolution, chamber and district allocation are replaced. All other explicitly supplied flags retain native values through the shared clap parser; omitted options retain native defaults. No command, path, graph preparation or partition is executed."}))
}
