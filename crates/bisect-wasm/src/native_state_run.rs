//! Bind the real native State compositor to explicitly selected in-memory inputs.
use bisect_runner::state_args::Resolution;
use bisect_runner::state_config::{AlgorithmConfig, SeedCompositor as Search, SplitStrategy as Split};
use serde::Deserialize;
use bisect_runner::native_replay_recipe::RecursiveBackend;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input { pub arguments:Vec<String>, pub prepared:crate::prepared_engine::Input }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphInput {
 pub arguments:Vec<String>, pub graph_json:String, pub graph_source_name:String,
 #[serde(default)]pub auxiliary_json:Option<String>,
 #[serde(default)]pub auxiliary_source_name:Option<String>,
 #[serde(default)]pub context_source:Option<crate::context_engine_input::Input>,
 pub label:String, pub chamber:String, pub created_at:String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdjacencyInput {
 pub arguments:Vec<String>,pub source:crate::native_adjacency::PrepareInput,
 #[serde(default)]pub auxiliary_json:Option<String>,
 #[serde(default)]pub auxiliary_source_name:Option<String>,
 pub label:String,pub chamber:String,pub created_at:String,
}
/// Bind the actual selected binary reader to the native CLI's adjacency bypass.
/// This entry point never discovers or opens the path named in the flags.
pub(crate) fn prepare_adjacency(input:AdjacencyInput)->Result<Value,String>{
 let review=bisect_runner::state_command::inspect(input.arguments.clone())?;
 if review["status"]!="parsed"||review["profile_filter"]["allowed"]!=true{return Err("Review valid native State flags/profile before selected adjacency preparation.".into());}
 validate_orchestration_flags(&review,true)?;
 let args=bisect_runner::state_command::parse(input.arguments.clone())?;
 let path=args.adjacency.as_deref().ok_or("Selected adjacency State preparation requires explicit --adjacency; paths are retained intent only.")?;
 if path.rsplit(['/', '\\']).next()!=Some(input.source.adjacency_name()){return Err("Select the exact basename named by --adjacency; no path is opened.".into());}
 let (state,year)=input.source.scope();
 if args.state.to_uppercase()!=state||args.year!=year||args.chamber!=input.chamber{return Err("Native adjacency flags must match the selected source state/year and export chamber.".into());}
 let mut source=crate::native_adjacency::prepare(input.source)?;
 let graph_json=source["graph_json"].as_str().ok_or("Missing native adjacency graph")?.to_owned();
 let mut result=prepare_graph_with_override::<usize>(GraphInput{arguments:input.arguments,graph_json,graph_source_name:"selected-native-adjacency-graph.json".into(),auxiliary_json:input.auxiliary_json,auxiliary_source_name:input.auxiliary_source_name,context_source:None,label:input.label,chamber:input.chamber,created_at:input.created_at},None,true,None,None)?;
 source.as_object_mut().ok_or("Invalid native source receipt")?.remove("graph_json");
 result["native_adjacency_source"]=source;
 result["adjacency_override"]=json!({"path_accessed":false,"resolution_lookup_applied":false,"census_year_policy_applied":false,"scope":"Selected binary and companions read in memory through the shared native reader. Native --adjacency bypasses manifest source/year/resolution lookup; selected graph state/year and export chamber must match the flags. Other explicit orchestration flags remain refused. No partition, discovery, disk installation or certification occurred."});
 Ok(result)
}

pub(crate) fn validate_orchestration_flags(review:&Value,adjacency_override:bool)->Result<(),String>{
 let unsupported=["version","output_dir","dpi","position","print_only","debug","reset","manifest","force","adjacency","state_name","population_source","label","time_partition"];
 let explicit=review["fields"].as_array().ok_or("Missing native flag inventory")?;
 let refused=explicit.iter().filter(|f|f["source"]=="command-line"&&unsupported.contains(&f["id"].as_str().unwrap_or(""))&&!(adjacency_override&&f["id"]=="adjacency")).map(|f|format!("--{}",f["long"].as_str().unwrap_or(""))).collect::<Vec<_>>();
 if !refused.is_empty(){return Err(format!("Prepared State execution does not perform these native orchestration flags: {}. Select prepared sources and export settings separately; use inspection to review a full command.",refused.join(", ")));}Ok(())
}

/// Construct only a typed source envelope; the actual State compositor supplies
/// every algorithm option through `prepare`. No user-authored request template
/// or JavaScript numeric roundtrip is required.
pub fn prepare_graph(input:GraphInput)->Result<Value,String>{prepare_graph_resolved(input,None)}
/// Internal resolved recipe route. The caller has checked selected bytes and
/// constructs the flags from recorded scope; no imported output controls enter.
pub(crate) fn prepare_recorded_graph(input:GraphInput,recipe:&bisect_runner::native_replay_recipe::ReadOnlyRecipe,native_version:&str)->Result<Value,String>{
 let s=&recipe.state;
 let mut algorithm=s.algorithm.clone();algorithm.metis.engine=recipe.resolve_replay_backend(native_version)?.engine;
 let config=bisect_runner::states_config::StatesConfiguration{debug:false,reprocess:false,algo:algorithm,balance_tolerance:Some(s.balance_tolerance_fraction),multiscale_fine:s.multiscale_fine.clone(),multiscale_coarse:s.multiscale_coarse.clone(),smc_resample_threshold:s.smc_resample_threshold};
 let mut result=prepare_graph_with_override(input,Some(&config),true,Some(s.balance_tolerance_fraction),None)?;
 result["native_command_role"]=json!("synthetic-scope-validation-only; not original argv");
 result["resolved_algorithm_json"]=json!(serde_json::to_string(&config.algo).map_err(|e|e.to_string())?);
 Ok(result)
}
pub(crate) fn prepare_recorded_multiscale(input:Input,recipe:&bisect_runner::native_replay_recipe::ReadOnlyRecipe,native_version:&str)->Result<Value,String>{
 let s=&recipe.state;let mut algorithm=s.algorithm.clone();algorithm.metis.engine=recipe.resolve_replay_backend(native_version)?.engine;
 let config=bisect_runner::states_config::StatesConfiguration{debug:false,reprocess:false,algo:algorithm,balance_tolerance:Some(s.balance_tolerance_fraction),multiscale_fine:s.multiscale_fine.clone(),multiscale_coarse:s.multiscale_coarse.clone(),smc_resample_threshold:s.smc_resample_threshold};
 let mut result=prepare_resolved_with_override(input,Some(&config),true,Some(s.balance_tolerance_fraction),None)?;
 result["native_command_role"]=json!("synthetic-scope-validation-only; not original argv");result["resolved_algorithm_json"]=json!(serde_json::to_string(&config.algo).map_err(|e|e.to_string())?);Ok(result)
}
fn prepare_graph_resolved(input:GraphInput,bulk:Option<&bisect_runner::states_config::StatesConfiguration>)->Result<Value,String>{
 prepare_graph_with_override(input,bulk,false,None,None)
}
fn prepare_graph_with_override<C:bisect_runner::native_replay_recipe::CountDomain>(input:GraphInput,bulk:Option<&bisect_runner::states_config::StatesConfiguration<C>>,adjacency_override:bool,exact_fraction:Option<f64>,native_profile:Option<RecursiveBackend>)->Result<Value,String>{
 let named=|name:&str|!name.is_empty()&&name.len()<=200&&!name.chars().any(char::is_control);
 if !named(&input.graph_source_name)||input.graph_json.is_empty()||input.graph_json.len()+input.auxiliary_json.as_ref().map_or(0,String::len)>25*1024*1024{return Err("Select named graph and optional auxiliary JSON totaling at most 25 MiB.".into());}
 if input.auxiliary_source_name.as_ref().is_some_and(|name|!named(name))||input.auxiliary_source_name.is_some()!=input.auxiliary_json.is_some(){return Err("Supply auxiliary JSON and its source name together, or neither.".into());}
 let graph:crate::engine::PreparedGraph=serde_json::from_str(&input.graph_json).map_err(|e|format!("Selected State graph: {e}"))?;
 let auxiliary:serde_json::Map<String,Value>=serde_json::from_str(input.auxiliary_json.as_deref().unwrap_or("{}")).map_err(|e|format!("Selected auxiliary input object: {e}"))?;
 const ALLOWED:&[&str]=&["demographics","partisan","elections","character","native_character_source","coi","native_auxiliary","native_weighting"];
 if auxiliary.keys().any(|key|!ALLOWED.contains(&key.as_str())){return Err("Auxiliary JSON supports only demographics, partisan, elections, character, native_character_source and coi. Algorithm options belong to State flags; multiscale uses the full prepared request path.".into());}
 let mut request=json!({"graph":graph,"options":{"structure":"standard-bisect","search":"single","weights":"geographic","districts":1,"seed":null,"seeds":1,"steps":200,"percentile":0.0,"alpha_county":0.0,"balance_tolerance":0.5,"area_swing":1.10,"iterations":100}});
 for (key,value) in auxiliary{request.as_object_mut().unwrap().insert(key,value);}
 let source=json!({"schema_version":"bisect-native-state-graph-source-v1","graph_source_name":input.graph_source_name,"graph_sha256":format!("{:x}",Sha256::digest(input.graph_json.as_bytes())),"auxiliary_source_name":input.auxiliary_source_name,"auxiliary_sha256":input.auxiliary_json.as_ref().map(|text|format!("{:x}",Sha256::digest(text.as_bytes()))),"state":graph.state,"year":graph.year,"unit_kind":graph.effective_unit_kind()});
 let prepared=crate::prepared_engine::Input{request_json:serde_json::to_string(&request).map_err(|e|e.to_string())?,inspect_only:true,label:input.label,chamber:input.chamber,created_at:input.created_at,context_source:input.context_source};
 let mut result=prepare_resolved_with_override(Input{arguments:input.arguments,prepared},bulk,adjacency_override,exact_fraction,native_profile)?;
 result["state_graph_source"]=source;
 Ok(result)
}

/// Preparation is deliberately inert. The resulting exact request is run through
/// the existing prepared engine boundary, including its retry/export machinery.
pub fn prepare(input:Input)->Result<Value,String>{prepare_resolved(input,None)}
fn prepare_resolved(input:Input,bulk:Option<&bisect_runner::states_config::StatesConfiguration>)->Result<Value,String>{
 prepare_resolved_with_override(input,bulk,false,None,None)
}
fn prepare_resolved_with_override<C:bisect_runner::native_replay_recipe::CountDomain>(input:Input,bulk:Option<&bisect_runner::states_config::StatesConfiguration<C>>,adjacency_override:bool,exact_fraction:Option<f64>,native_profile:Option<RecursiveBackend>)->Result<Value,String>{
 let review=bisect_runner::state_command::inspect(input.arguments.clone())?;
 if review["status"]!="parsed" {return Err(review["message"].as_str().unwrap_or("State arguments did not parse.").into());}
 if review["profile_filter"]["allowed"]!=true {return Err(review["profile_filter"]["error"].as_str().unwrap_or("Native profile rejected these flags.").into());}
 let args=bisect_runner::state_command::parse(input.arguments)?;
 // These flags name orchestration work outside the selected prepared graph.
 // Refuse an explicit request rather than silently claiming to perform it.
 validate_orchestration_flags(&review,adjacency_override)?;
 if input.prepared.request_json.len()>25*1024*1024{return Err("Prepared State template exceeds 25 MiB.".into());}
 let mut template:Value=serde_json::from_str(&input.prepared.request_json).map_err(|e|format!("Prepared State template: {e}"))?;
 let operation=template.get("operation").and_then(Value::as_str).map(str::to_owned);
 let multiscale=matches!(operation.as_deref(),Some("run-multiscale"|"run-multiscale-build"));
 let request=match operation.as_deref(){None=>&mut template,Some("run-engine-build")=>template.get_mut("request").ok_or("Missing build request")?,Some("run-multiscale"|"run-multiscale-build")=>template.get_mut("input").and_then(|v|v.get_mut("request")).ok_or("Missing multiscale request")?,_=>return Err("Select ordinary, run-engine-build, run-multiscale or run-multiscale-build prepared request JSON.".into())};
 // Parse the entire original typed request before changing any options.
 let original:crate::engine::Request=serde_json::from_value(request.clone()).map_err(|e|format!("Prepared State source request: {e}"))?;
 let graph=&original.graph;
 let state=args.state.to_uppercase();
 let unit=match args.resolution{Resolution::Tract=>rplan_core::UnitKind::Tract,Resolution::BlockGroup=>rplan_core::UnitKind::BlockGroup,Resolution::Block=>rplan_core::UnitKind::Block};
 if state!=graph.state||args.year!=graph.year||!adjacency_override&&graph.effective_unit_kind()!=unit{return Err("State flags must match the selected graph's state, Census year and resolution.".into());}
 if args.chamber!=input.prepared.chamber{return Err("Native --chamber must match the selected prepared export chamber.".into());}
 let policy:Value=serde_json::from_str(include_str!("../../../data/location_policy.json")).map_err(|e|e.to_string())?;
 if bulk.is_none()&&!adjacency_override{bisect_analysis::location_registry::PolicyRegistry::new(&policy).validate_year(&state,&args.year)?;}
 let count_source=if let Some(count)=args.districts{json!({"districts":count,"allocation_source":"explicit-native-flag"})}else if adjacency_override{
  let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e|e.to_string())?;
  let count=bisect_analysis::location_registry::PolicyRegistry::new(&policy).chamber_districts(&state,&args.chamber,&args.year).or_else(||if args.chamber=="congressional"{manifest.states.get(&state).and_then(|s|s.districts.get(&args.year)).map(|&n|n as u64)}else{None}).ok_or("Native --adjacency requires --districts when the registry has no matching chamber/year count.")?;
  json!({"districts":count,"allocation_source":"native-adjacency-registry","policy_sha256":format!("{:x}",Sha256::digest(include_str!("../../../data/location_policy.json").as_bytes()))})
 }else{crate::chamber_count::inspect(crate::chamber_count::Input{state:state.clone(),year:args.year.clone(),chamber:args.chamber.clone(),source_name:None,policy_json:None,manifest_json:None})?};
 let districts=count_source["districts"].as_u64().ok_or("Native district count unavailable")?;
 let tolerance=args.balance_tolerance.unwrap_or_else(||bisect_analysis::location_registry::chamber_balance_tolerance(&policy,&state,&args.chamber)*100.0);
 let mut algo=if let Some(config)=bulk{config.algo.clone()}else{serde_json::from_value::<AlgorithmConfig<C>>(serde_json::to_value(AlgorithmConfig::from_state_args(&args)).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?};
 // The omitted configured default is also a property of the native build.
 if let Some(profile)=native_profile {if args.metis_engine.is_none(){algo.metis.engine=profile.engine();}}
 let native_profile_algorithm_json=if native_profile.is_some(){Some(serde_json::to_string(&algo).map_err(|e|e.to_string())?)}else{None};
 // Fresh selected-source runs mirror actual native dispatch. These methods
 // and the explicit no-fallback ILP path never call a METIS kernel. Historical
 // prepared/replay entry points keep their existing None-profile contract.
 let fresh_without_metis=native_profile.is_some()&&matches!(&algo.split,Split::BfsGrowth|Split::CentroidalVoronoi{..}|Split::CapacityClustering|Split::Ilp{fallback:bisect_runner::args::IlpFallback::Error,..});
 let uses_metis=!matches!(algo.split,Split::MovingKnife{..}|Split::Spectral{..}|Split::Regionalization|Split::FlowConstruction)&&!fresh_without_metis;
 // Fresh State mirrors the selected native compile profile, separately from
 // the parsed CLI flag. The prime-factor compositor alone forwards that flag.
 // Other prepared and historical replay APIs retain their existing selection.
 let configured_backend_used=matches!(algo.split,Split::ApportionRegions);
 let execution_engine=if !configured_backend_used {native_profile.map(RecursiveBackend::engine).unwrap_or(algo.metis.engine)}else{algo.metis.engine};
 let metis_used=districts>1&&uses_metis;
 if metis_used&&configured_backend_used&&native_profile==Some(RecursiveBackend::MetisCore)&&matches!(execution_engine,bisect_apportion::split::MetisEngine::CFfi){return Err("The selected portable native profile was built without C FFI for prime-factor execution.".into());}
 if metis_used&&!matches!(execution_engine,bisect_apportion::split::MetisEngine::RedistMetis)&&!(matches!(execution_engine,bisect_apportion::split::MetisEngine::CFfi)&&crate::metis_backend::c_available()){return Err("This native State method requires an available explicit METIS backend in WASM.".into());}
 let w=&algo.weights;
 if let Some(path)=&w.partisan_shares{
  if original.partisan.as_ref().map(|p|p.source_label.as_str()).or_else(||original.native_auxiliary.as_ref().and_then(|i|i.partisan.as_ref()).map(|s|s.source_name.as_str()))!=path.to_str(){return Err("Bind --partisan-shares to the exact source_label of the selected prepared partisan input; paths are never read.".into());}
 }else if original.partisan.is_some()||original.native_auxiliary.as_ref().is_some_and(|i|i.partisan.is_some()){return Err("Prepared partisan input requires matching --partisan-shares flags; no source is silently discarded.".into());}
 if args.coi_weights.as_deref()!=original.coi.as_ref().map(|c|c.source_name.as_str()).or_else(||original.native_auxiliary.as_ref().and_then(|i|i.coi.as_ref()).map(|s|s.source_name.as_str())){return Err("Bind --coi-weights to the exact source_name of the selected prepared COI input; paths are never read.".into());}
 if original.zone_membership.is_some(){return Err("Native State currently uses the zone spatial-join fallback. Explicit zone observations use the separate prepared engine operation.".into());}
 if let Some(spec)=&original.native_weighting{if serde_json::to_value(spec).map_err(|e|e.to_string())?!=serde_json::to_value(w).map_err(|e|e.to_string())?{return Err("Recorded native WeightSpec differs from resolved State settings.".into());}}
 let weights=if original.native_weighting.is_some(){"native-composed"}else if w.minority_weighting{"vra-aligned"}else if w.partisan_shares.is_some(){"partisan"}else if w.economic_character{"economic-character"}else if w.housing_character{"housing-character"}else if w.zone_membership{"zone-membership"}else if w.geographic{"geographic"}else{"unweighted"};
 let mut options=json!({"structure":"standard-bisect","search":"single","weights":weights,"districts":districts,"seed":algo.metis.seed.map(|s|s.to_string()),"seeds":1,"steps":200,"percentile":0.0,"alpha_county":0.0,"balance_tolerance":tolerance,"area_swing":1.10,"iterations":algo.metis.niter});
 if let Some(fraction)=exact_fraction{options["balance_tolerance_fraction"]=json!(fraction);}
 if metis_used&&matches!(execution_engine,bisect_apportion::split::MetisEngine::CFfi){options["metis_engine"]=json!("c-ffi");options["metis_runtime"]=json!("msvcrt-v1");}
 if original.native_weighting.is_none()&&w.alpha_county!=0.0{options["county_augmentation"]=json!(w.alpha_county);}
 if weights=="partisan"{options["dem_threshold"]=json!(w.dem_threshold);options["rep_threshold"]=json!(w.rep_threshold);}
 if original.native_weighting.is_none()&&(w.economic_character||w.housing_character){options["character_alpha"]=json!(w.econ_alpha);if original.character.is_none()&&original.native_character_source.is_none(){options["character_absent_file_fallback"]=json!(true);}}
 for key in ["capture_rounds","capture_ilp_reports"]{if let Some(value)=serde_json::to_value(&original.options).map_err(|e|e.to_string())?.get(key){options[key]=value.clone();}}
 let count=algo.seeds.seed_count_u64();
 let counted=matches!(algo.split,Split::GeoSection|Split::AreaSection{..}|Split::VraSection{..}|Split::CompactBisect{..}|Split::ProportionalSection{..});
 match &algo.split{
  Split::Bisect=>match &algo.seeds{
   Search::Single=>{},Search::Multi{seeds}=>{options["search"]=json!("multi");options["seeds"]=json!(seeds);},
   Search::ConvergenceSweep{threshold}=>{options["search"]=json!("convergence");options["seeds"]=json!(threshold);options["convergence_unlimited"]=json!(true);},
   Search::Percentile{p,seeds}=>{options["search"]=json!("percentile");options["percentile"]=json!(p);options["seeds"]=json!(seeds);},
   Search::BisectionEnsemble{p,ensemble_steps}=>{options["search"]=json!("bisection-ensemble");options["percentile"]=json!(p);options["steps"]=json!(ensemble_steps);},
   Search::Flip{p,flip_steps}=>{options["search"]=json!("flip");options["percentile"]=json!(p);options["steps"]=json!(flip_steps);},
   Search::ForestRecom{p,steps}|Search::MergeSplit{p,steps}|Search::VraRecom{p,steps,..}=>{options["search"]=json!(match &algo.seeds{Search::ForestRecom{..}=>"forest-recom",Search::MergeSplit{..}=>"merge-split",_=>"vra-recom"});options["steps"]=json!(steps);options["percentile"]=json!(p);if let Search::VraRecom{vap_threshold,..}=&algo.seeds{options["vra_threshold"]=json!(vap_threshold);}},
   Search::ShortBurst{p,burst_length,n_bursts}|Search::ShortBurstForest{p,burst_length,n_bursts}|Search::ShortBurstMergeSplit{p,burst_length,n_bursts}=>{options["search"]=json!(match &algo.seeds{Search::ShortBurst{..}=>"short-burst",Search::ShortBurstForest{..}=>"short-burst-forest",_=>"short-burst-merge-split"});options["percentile"]=json!(p);options["burst_length"]=json!(burst_length);options["n_bursts"]=json!(n_bursts);},
   Search::ParallelTempering{p,steps,n_replicas,swap_interval,cold_tolerance,hot_tolerance}=>{options["search"]=json!("parallel-tempering");options["percentile"]=json!(p);options["seeds"]=json!(steps);options["pt_replicas"]=json!(n_replicas);options["pt_swap_interval"]=json!(swap_interval);options["pt_cold_tol"]=json!(cold_tolerance);options["pt_hot_tol"]=json!(hot_tolerance);},
   Search::SmcPercentile{p,n_particles}=>{options["search"]=json!("smc-percentile");options["percentile"]=json!(p);options["smc_particles"]=json!(n_particles);options["smc_resample_threshold"]=json!(args.smc_resample_threshold);},
   Search::MultiScale{..}|Search::MultiScaleAdaptive{..}=>{},
  },
  Split::NWay=>{options["structure"]=json!("nway");options["metis_ufactor"]=json!(algo.metis.ufactor);},
  Split::GeoSection=>options["structure"]=json!("ratio-optimal"),
  Split::AreaSection{area_swing,area_section_init}=>{options["structure"]=json!("ratio-optimal-area");options["area_swing"]=json!(area_swing);options["area_init"]=json!(match area_section_init{bisect_runner::runner::AreaSectionInit::RatioOptimal=>"ratio-optimal",bisect_runner::runner::AreaSectionInit::MovingKnife=>"moving-knife"});},
  Split::VraSection{w_vra}=>{options["structure"]=json!("ratio-optimal-vra");options["w_vra"]=json!(w_vra);},
  Split::CompactBisect{epsilon}=>{options["structure"]=json!("compact-polsby");options["compact_epsilon"]=json!(epsilon);options["compact_geometry"]=json!("native-state-circular");},
  Split::ProportionalSection{eta}=>{options["structure"]=json!("proportional-section");options["proportional_eta"]=json!(eta);},
  Split::ApportionRegions=>options["structure"]=json!("prime-factor"),
  Split::SimulatedAnnealing{steps_per_tract,t0_factor,t_final}=>{options["structure"]=json!("simulated-annealing");options["sa_steps_per_tract"]=json!(steps_per_tract);options["sa_t0_factor"]=json!(t0_factor);options["sa_t_final"]=json!(t_final);},
  Split::CentroidalVoronoi{n_iter,metric}=>{options["structure"]=json!("centroidal-voronoi");options["cvd_iters"]=json!(n_iter);options["cvd_metric"]=json!(match metric{bisect_runner::bisection_runner::VoronoiMetric::GraphDistance=>"graph-distance",bisect_runner::bisection_runner::VoronoiMetric::Geographic=>"geographic"});},
  Split::BfsGrowth=>options["structure"]=json!("bfs-growth"),
  Split::Ilp{method,fallback,time_limit_secs,optimality_gap,max_tracts}=>{options["structure"]=json!("ilp");options["ilp_method"]=json!(method.to_string());options["ilp_fallback"]=json!(fallback.to_string());options["ilp_time_limit"]=json!(time_limit_secs.to_string());options["ilp_gap"]=json!(optimality_gap);options["ilp_max_tracts"]=json!(max_tracts);},
  Split::MovingKnife{n_orientations,metric}=>{options["structure"]=json!("moving-knife");options["mka_orientations"]=json!(n_orientations);options["mka_metric"]=json!(if metric=="polsby"{"polsby"}else{"reock"});},
  Split::CapacityClustering=>options["structure"]=json!("capacity-clustering"),Split::Spectral{max_iters}=>{options["structure"]=json!("spectral");options["steps"]=json!(max_iters);},Split::Regionalization=>options["structure"]=json!("regionalization"),Split::FlowConstruction=>options["structure"]=json!("flow-construction"),
 }
 if counted {options["seeds"]=json!(count);options["search"]=json!(if count==1{"single"}else{"multi"});}
 let native_ms=(matches!(algo.split,Split::Bisect)||matches!(algo.split,Split::NWay)&&districts<=1)&&matches!(algo.seeds,Search::MultiScale{..}|Search::MultiScaleAdaptive{..});
 if native_ms{options["structure"]=json!("standard-bisect");options.as_object_mut().unwrap().remove("metis_ufactor");}
 if native_ms!=multiscale{return Err("Native multiscale search requires a selected multiscale template; other native dispatches require an ordinary/build template.".into());}
 if (matches!(algo.split,Split::VraSection{..})||options["search"]=="vra-recom")&&original.demographics.is_none()&&!original.native_auxiliary.as_ref().and_then(|i|i.demographics.as_ref()).is_some_and(|s|s.bytes_base64.is_some()){options["vra_absent_file_fallback"]=json!(true);}
 request["options"]=options;
 request["electoral"]=json!({"seats_per_district":args.seats_per_district,"total_seats":args.total_seats});
 if multiscale {
  let m=&mut template["input"];
  m["fine_level"]=json!(if args.multiscale_fine=="block_group"{"bg"}else{&args.multiscale_fine});m["coarse_level"]=json!(args.multiscale_coarse);
  match algo.seeds{
   Search::MultiScale{total_steps,p,alpha}=>{m["total_steps"]=json!(total_steps);m["percentile"]=json!(p);m["alpha"]=json!(alpha);m["adaptive"]=Value::Null;},
   Search::MultiScaleAdaptive{total_steps,p,target_accept,adapt_interval}=>{m["total_steps"]=json!(total_steps);m["percentile"]=json!(p);m["alpha"]=json!(target_accept);m["adaptive"]=json!({"target_accept":target_accept,"adapt_interval":adapt_interval,"gamma_0":0.10,"coarse_tol_factor":3.0});},_=>unreachable!(),
  }
  // Full multiscale validation is required even though preparation is inert.
  let typed:crate::multiscale_input::MultiscaleRequest=serde_json::from_value(m.clone()).map_err(|e|e.to_string())?;crate::multiscale_input::validate(&typed)?;
  template["operation"]=json!("run-multiscale-build");
 }else if operation.is_none(){template=json!({"operation":"run-engine-build","request":template});}
 let request_json=serde_json::to_string(&template).map_err(|e|e.to_string())?;
 let original_sha=format!("{:x}",Sha256::digest(input.prepared.request_json.as_bytes()));
 let prepared=crate::prepared_engine::Input{request_json:request_json.clone(),inspect_only:true,..input.prepared};
 let validation=crate::prepared_engine::run(prepared)?;
 let mut result=json!({"schema_version":"bisect-native-state-run-input-v1","native_command":review,"original_template_sha256":original_sha,"prepared_request_sha256":format!("{:x}",Sha256::digest(request_json.as_bytes())),"prepared_request_json":request_json,"district_count":count_source,"embedded_policy_sha256":format!("{:x}",Sha256::digest(include_str!("../../../data/location_policy.json").as_bytes())),"prepared_review":validation,"execution_performed":false,"filesystem_reads_performed":false,"scope":"Native State parser/compositor bound to selected prepared graph and auxiliary inputs. Template algorithm options are replaced; capture switches retained. Native State flag profile is checked before translation; the prepared request uses open profile because a second resolved-options filter differs from the native flag filter. Native objtype and target-mm-districts are unused by the native State runner and are not applied. Non-nway ufactor is likewise unused. Multiscale settings follow native State defaults; ordinary and multiscale outputs select native population retries. Preparation performs no engine run, source acquisition, filesystem effects, installation or certification. Execute the compiled request explicitly in Prepared engine Run; existing browser resource and source validation remains in force."});
 if let Some(profile)=native_profile {
  result["native_command_role"]=json!("Loaded-WASM parser/default-compositor inspection; selected native build defaults are resolved separately.");
  result["native_profile_algorithm_json"]=json!(native_profile_algorithm_json.as_deref().ok_or("Missing selected native profile algorithm")?);
  result["native_backend"]=json!({"schema_version":"bisect-fresh-native-backend-profile-v1","recursive_backend":profile,"configured_backend":algo.metis.engine.as_str(),"configured_backend_source":if args.metis_engine.is_some(){"command-line"}else{"native-compile-profile-default"},"metis_used":metis_used,"configuration_role":if !metis_used{"unused"}else if configured_backend_used{"configured-prime-factor"}else{"native-compile-profile"},"effective_backend":if metis_used{Some(execution_engine.as_str())}else{None},"runtime_profile":if metis_used&&matches!(execution_engine,bisect_apportion::split::MetisEngine::CFfi){Some("msvcrt-v1")}else{None},"authority":"Explicit selected execution profile, not an observed or authenticated desktop binary."});
 }
 Ok(result)
}

struct BulkBinding { arguments:Vec<String>,config:bisect_runner::states_config::StatesConfiguration,review:Value,allocation:Value }
fn bulk_binding(arguments:Vec<String>,graph:&crate::engine::PreparedGraph,chamber:&str)->Result<BulkBinding,String>{
 let review=bisect_runner::states_config::inspect(arguments.clone())?;
 if review["status"]!="parsed"{return Err(review["message"].as_str().unwrap_or("States arguments did not parse.").into());}
 for key in ["profile_filter","multiscale_levels"]{if review[key]["allowed"]!=true{return Err(review[key]["error"].as_str().unwrap_or("Native States preflight rejected these flags.").into());}}
 let args=bisect_runner::states_config::parse(arguments)?;
 if graph.effective_unit_kind()!=rplan_core::UnitKind::Tract||chamber!="congressional"||graph.year!=args.year.to_string(){return Err("Native States requires a tract graph for its selected Census year and congressional chamber.".into());}
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e|e.to_string())?;
 let (code,slug,districts)=bisect_core::state_catalog::congressional_states(&manifest,&graph.year)?.into_iter().find(|r|r.0==graph.state&&(args.states.is_empty()||args.states.iter().any(|s|s.to_uppercase()==r.0))).ok_or("Selected graph is not an eligible state in this native States selection.")?;
 let mut flags=vec!["--state".into(),code.clone(),"--year".into(),graph.year.clone(),"--resolution".into(),"tract".into(),"--chamber".into(),"congressional".into(),"--metis-engine".into(),"metis-core".into(),"--districts".into(),districts.to_string(),"--total-seats".into(),districts.to_string(),"--smc-resample-threshold".into(),args.smc_resample_threshold.to_string()];
 if let Some(tol)=args.balance_tolerance{flags.extend(["--balance-tolerance".into(),tol.to_string()]);}
 flags.extend(["--multiscale-fine".into(),args.multiscale_fine.clone(),"--multiscale-coarse".into(),args.multiscale_coarse.clone()]);
 let config=bisect_runner::states_config::StatesConfiguration::from_args(&args);
 Ok(BulkBinding{arguments:flags,config,review,allocation:json!({"districts":districts,"allocation_source":"native-states-manifest","state":code,"state_slug":slug,"year":graph.year})})
}
fn finish_bulk(mut result:Value,binding:BulkBinding)->Value{
 result["native_command"]=binding.review;result["district_count"]=binding.allocation;
 result["scope"]=json!("Actual native States parser and shared bulk configuration compiled onto one explicitly selected eligible tract graph, with native congressional allocation and profile/level preflights. Bulk algorithm defaults and search budgets replace template options; the portable default METIS backend follows this build. Native unused declarations remain unused. Output paths, workers, print-only and analysis flags are retained intent, not filesystem/scheduler execution. Preparation is inert. Review and explicitly run the compiled request through Prepared engine Run; full States source acquisition/scheduling/output orchestration is separate.");result
}
pub fn prepare_states_graph(mut input:GraphInput)->Result<Value,String>{
 if input.graph_json.len()>25*1024*1024{return Err("Selected States graph exceeds 25 MiB.".into());}
 let graph:crate::engine::PreparedGraph=serde_json::from_str(&input.graph_json).map_err(|e|format!("Selected States graph: {e}"))?;
 let binding=bulk_binding(input.arguments,&graph,&input.chamber)?;input.arguments=binding.arguments.clone();
 let result=prepare_graph_resolved(input,Some(&binding.config))?;Ok(finish_bulk(result,binding))
}
pub fn prepare_states(mut input:Input)->Result<Value,String>{
 if input.prepared.request_json.len()>25*1024*1024{return Err("Selected States template exceeds 25 MiB.".into());}
 let value:Value=serde_json::from_str(&input.prepared.request_json).map_err(|e|e.to_string())?;
 let request=match value.get("operation").and_then(Value::as_str){None=>&value,Some("run-engine-build")=>&value["request"],Some("run-multiscale"|"run-multiscale-build")=>&value["input"]["request"],_=>return Err("Select an ordinary/build or multiscale/build States request template.".into())};
 let graph:crate::engine::PreparedGraph=serde_json::from_value(request["graph"].clone()).map_err(|e|e.to_string())?;
 let binding=bulk_binding(input.arguments,&graph,&input.prepared.chamber)?;input.arguments=binding.arguments.clone();
 let result=prepare_resolved(input,Some(&binding.config))?;Ok(finish_bulk(result,binding))
}


fn run_binding(arguments:Vec<String>,graph:&crate::engine::PreparedGraph,chamber:&str)->Result<BulkBinding,String>{
 let review=bisect_runner::run_command::inspect(arguments.clone())?;
 if review["status"]!="parsed"{return Err(review["message"].as_str().unwrap_or("Run arguments did not parse.").into());}
 let args=bisect_runner::run_command::parse(arguments)?;
 if !bisect_runner::run_command::years(&args).contains(&graph.year){return Err("Native Run year selection does not include this graph vintage.".into());}
 if !args.states.is_empty()&&!args.states.iter().any(|state|state.to_uppercase()==graph.state){return Err("Selected graph is not eligible in the native Run state filter.".into());}
 // Metadata values remain values even when prefixed with '-'. Only the already
 // selected graph code enters the internal States scope parser; original filter
 // values remain exact in the native Run review and are never reinterpreted.
 let flags=vec!["--year".into(),graph.year.clone(),format!("--version={}",args.version),format!("--output-dir={}",bisect_runner::run_command::output_dir(&args)),"--states".into(),graph.state.clone()];
 let mut binding=bulk_binding(flags,graph,chamber)?;
 binding.config=bisect_runner::run_command::RunConfiguration::from_args(&args).prepared_configuration();
 for key in ["--multiscale-fine","--multiscale-coarse"]{if let Some(i)=binding.arguments.iter().position(|v|v==key){binding.arguments[i+1]=if key=="--multiscale-fine"{"tract".into()}else{"county".into()};}}
 binding.review=review;binding.allocation["allocation_source"]=json!("native-run-manifest");Ok(binding)
}
fn finish_run(mut result:Value,binding:BulkBinding)->Value{
 result["native_command"]=binding.review;result["district_count"]=binding.allocation;
 result["scope"]=json!("Actual shared native Run parser and default-mode algorithm compiled onto one selected eligible congressional tract graph. Year=all permits each selected native vintage; this compilation does not schedule other years. Native unused flags remain unused in the algorithm. Browser Run separately honors print-only and template inspection. Output path, reset, debug and workers remain recorded intent; no discovery, deletion, installation, analysis or partition occurs during preparation.");result
}
pub fn prepare_run_graph(mut input:GraphInput)->Result<Value,String>{
 if input.graph_json.len()>25*1024*1024{return Err("Selected Run graph exceeds 25 MiB.".into());}
 let graph:crate::engine::PreparedGraph=serde_json::from_str(&input.graph_json).map_err(|e|e.to_string())?;
 let binding=run_binding(input.arguments,&graph,&input.chamber)?;input.arguments=binding.arguments.clone();let result=prepare_graph_resolved(input,Some(&binding.config))?;Ok(finish_run(result,binding))
}
pub fn prepare_run(mut input:Input)->Result<Value,String>{
 if input.prepared.request_json.len()>25*1024*1024{return Err("Selected Run template exceeds 25 MiB.".into());}
 let value:Value=serde_json::from_str(&input.prepared.request_json).map_err(|e|e.to_string())?;
 let request=match value.get("operation").and_then(Value::as_str){None=>&value,Some("run-engine-build")=>&value["request"],Some("run-multiscale"|"run-multiscale-build")=>&value["input"]["request"],_=>return Err("Select an ordinary/build or multiscale/build Run request template.".into())};
 let graph:crate::engine::PreparedGraph=serde_json::from_value(request["graph"].clone()).map_err(|e|e.to_string())?;
 let binding=run_binding(input.arguments,&graph,&input.prepared.chamber)?;input.arguments=binding.arguments.clone();let result=prepare_resolved(input,Some(&binding.config))?;Ok(finish_run(result,binding))
}

pub(crate) fn prepare_fresh_graph(input:GraphInput,profile:RecursiveBackend)->Result<Value,String>{prepare_graph_with_override::<usize>(input,None,true,None,Some(profile))}

pub(crate) fn prepare_fresh_template(input:Input,profile:RecursiveBackend)->Result<Value,String>{prepare_resolved_with_override::<usize>(input,None,true,None,Some(profile))}
