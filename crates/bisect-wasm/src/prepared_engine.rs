//! Shared typed prepared-request review/run boundary; raw JSON remains exact.
//! Legacy native-character selection delegates here with one explicitly attached CSV.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use crate::{engine::Request,multiscale_input::MultiscaleRequest,native_character_source};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input{#[serde(default)]pub(crate) context_source:Option<crate::context_engine_input::Input>,#[serde(default="review_default")]pub inspect_only:bool,pub request_json:String,pub label:String,pub chamber:String,pub created_at:String}
fn review_default()->bool{true}
#[derive(Deserialize)]
#[serde(tag="operation",rename_all="kebab-case",deny_unknown_fields)]
enum Operation{RunEngineBuild{request:Request},RunMultiscale{input:MultiscaleRequest,#[serde(default)]fine_edges:Option<Vec<(usize,usize,f64)>>},RunMultiscaleBuild{input:MultiscaleRequest,#[serde(default)]fine_edges:Option<Vec<(usize,usize,f64)>>}}
#[derive(Deserialize)]
#[serde(untagged)]
enum SavedRun{Operation(Operation),Ordinary(Request)}
fn attach(request:&mut Request,source:&native_character_source::Input)->Result<(),String>{
 if request.native_character_source.is_some()||request.character.is_some(){return Err("The engine request must omit native source and explicit character data; choose the original CSV separately.".into());}
 request.native_character_source=Some(source.clone());Ok(())
}
fn accepted_seed(result:&Value,request:&mut Request)->Result<(),String>{
 if let Some(attempts)=result.get("build_report").and_then(|r|r["attempts"].as_array()){
  let seed=attempts.last().ok_or("Missing accepted build attempt.")?.get("seed").ok_or("Missing accepted build seed.")?;
  request.options.seed=if seed.is_null(){None}else{Some(seed.as_str().ok_or("Invalid accepted build seed.")?.parse().map_err(|_|"Invalid accepted build seed.")?)};
 }Ok(())
}
pub fn run(input:Input)->Result<Value,String>{run_with_source(input,None)}
pub(crate) fn run_with_source(input:Input,source:Option<native_character_source::Input>)->Result<Value,String>{
 if input.request_json.len()>25*1024*1024{return Err("Native character engine request exceeds25MiB.".into());}
 if !input.inspect_only{for field in [&input.label,&input.chamber,&input.created_at]{if field.trim().is_empty()||field.len()>200{return Err("Record nonempty export label, chamber and timestamp of at most200 bytes.".into());}}}
 let saved:SavedRun=serde_json::from_str(&input.request_json).map_err(|e|format!("Select an ordinary engine request, run-engine-build, run-multiscale or run-multiscale-build JSON: {e}"))?;
 let (mode,build,mut request)=match saved{
  SavedRun::Ordinary(mut r)=>{if let Some(s)=&source{attach(&mut r,s)?;}("ordinary",false,Run::Ordinary(r))},
  SavedRun::Operation(Operation::RunEngineBuild{mut request})=>{if let Some(s)=&source{attach(&mut request,s)?;}("ordinary",true,Run::Ordinary(request))},
  SavedRun::Operation(Operation::RunMultiscale{input:mut r,fine_edges})=>{if let Some(s)=&source{attach(&mut r.request,s)?;}("multiscale",false,Run::Multiscale(r,fine_edges))},
  SavedRun::Operation(Operation::RunMultiscaleBuild{input:mut r,fine_edges})=>{if let Some(s)=&source{attach(&mut r.request,s)?;}("multiscale",true,Run::Multiscale(r,fine_edges))},
 };
 let context_engine_source=if let Some(context_source)=&input.context_source {
  let prepared=crate::context_engine_input::prepare(context_source.clone())?;
  let graph:crate::engine::PreparedGraph=serde_json::from_str(prepared["graph_json"].as_str().ok_or("Missing prepared context graph")?).map_err(|e|e.to_string())?;
  let actual=match &request{Run::Ordinary(r)=>&r.graph,Run::Multiscale(r,_)=>&r.request.graph};
  if serde_json::to_value(&graph).map_err(|e|e.to_string())?!=serde_json::to_value(actual).map_err(|e|e.to_string())?{return Err("Original RCTX graph differs from the full engine request. Bind the context graph to an empty graph slot before review or execution.".into());}
  Some(json!({"source":context_source,"origin":prepared["source_context"]}))
 }else{None};
 if input.inspect_only{
  let selected=match &request{Run::Ordinary(r)=>r,Run::Multiscale(r,_)=>&r.request};crate::native_composed::validate_request(selected)?;crate::native_auxiliary::validate_selection(selected.native_auxiliary.as_ref(),selected.partisan.is_some(),selected.coi.is_some())?;
  let r=match &request{Run::Ordinary(r)=>r,Run::Multiscale(r,_)=>&r.request};crate::engine::validate_profile(&r.options,r.partisan.is_some()||r.native_auxiliary.as_ref().is_some_and(|i|i.partisan.is_some()),None)?;crate::engine::validate(&r.graph,&r.options)?;
  if let Some(source)=&source{
  if source.state!=r.graph.state||source.year!=r.graph.year||format!("{}-character",source.kind)!=r.options.weights{return Err("Native CSV state, Census year and kind must match the engine request.".into());}
  let bytes=native_character_source::decode(source)?;
  return Ok(json!({"schema_version":"bisect-native-character-engine-request-v1","run_kind":mode,"native_build":build,"state":r.graph.state,"year":r.graph.year,"tracts":r.graph.geoids.len(),"physical_edges":r.graph.edges.len(),"source_name":source.source_name,"source_sha256":format!("{:x}",Sha256::digest(&bytes)),"data_year":source.data_year,"engine_request":{"options":r.options},"execution_performed":false,"csv_loading_performed":false,"scope":"Review of selected typed request, prepared graph/base option checks and selected original CSV byte identity. Does not parse CSV observations, validate every multiscale/data constraint, generate a plan or certify feasibility. Original JSON/CSV remain selected until explicit execution."}));
  }
  let mut review=json!({"schema_version":"bisect-prepared-engine-request-v1","run_kind":mode,"native_build":build,"state":r.graph.state,"year":r.graph.year,"unit_kind":r.graph.effective_unit_kind(),"units":r.graph.geoids.len(),"physical_edges":r.graph.edges.len(),"request_sha256":format!("{:x}",Sha256::digest(input.request_json.as_bytes())),"engine_request":{"options":r.options},"execution_performed":false,"csv_loading_performed":false,"scope":"Typed prepared-request review and prepared graph/base-option validation only. Original JSON retains exact u64 numbers and declared auxiliary inputs. Does not load CSV observations, validate every auxiliary/multiscale constraint, partition or certify feasibility. Select explicit execution to run."});
  if let Some(context)=context_engine_source {review["context_engine_source"]=context;}
  return Ok(review);
 }
 let (result,exported,executed,replay_json)=match &mut request{
  Run::Ordinary(r)=>{
   let result=if build{crate::engine_build::run(r.clone())?}else{crate::engine::execute(r.clone())?};accepted_seed(&result,r)?;
   let assignments=serde_json::from_value(result["assignments"].clone()).map_err(|e|format!("Invalid engine assignment: {e}"))?;
   let exported=crate::plan_export::export(r.clone(),assignments,input.label,input.chamber,input.created_at);
   let executed=serde_json::to_value(&*r).map_err(|e|e.to_string())?;
   let replay=if source.is_none(){serde_json::to_string(&json!({"kind":"engine","request":&executed})).map_err(|e|e.to_string())?}else{String::new()};
   (result,exported,executed,replay)
  },
  Run::Multiscale(r,fine_edges)=>{
   let result=if build{crate::engine_build::run_multiscale(r.clone())?}else{crate::multiscale_input::execute(r.clone())?};accepted_seed(&result,&mut r.request)?;
   let assignments=serde_json::from_value(result["assignments"].clone()).map_err(|e|format!("Invalid fine assignment: {e}"))?;
   // Fine physical boundaries are supplied separately by the multiscale ABI.
   let exported=crate::plan_export::export_multiscale(r.clone(),fine_edges.clone(),assignments,input.label,input.chamber,input.created_at);
   let executed=serde_json::to_value(&*r).map_err(|e|e.to_string())?;
   let replay=if source.is_none(){serde_json::to_string(&json!({"kind":"multiscale","input":&executed,"fine_edges":&*fine_edges})).map_err(|e|e.to_string())?}else{String::new()};
   (result,exported,executed,replay)
  }
 };
 let (exported,export_error)=match exported{Ok(v)=>(Some(v),None),Err(e)=>(None,Some(e))};
 let mut output=json!({"schema_version":if source.is_some(){"bisect-native-character-engine-v1"}else{"bisect-prepared-engine-v1"},"run_kind":mode,"native_build":build,"execution_performed":true,"request_sha256":format!("{:x}",Sha256::digest(input.request_json.as_bytes())),"executed_request":executed,"result":result,"export":exported,"export_error":export_error,"scope":if source.is_some(){"Shared native file reader, exact-ID alignment, character weighter and selected portable engine dispatch. The requested option/search/structure and native retry semantics are preserved. Nonfinite weighted metrics use null with numeric classes, without geographic imputation. Available RPLAN/RCTX exports retain physical graph weights and original CSV input identities. Export failure preserves the generated result with an explicit export_error. Export does not authenticate historical execution, source publication, optimality or legal compliance."}else{"Shared native prepared engine dispatch with original typed graph, options and auxiliary inputs. Native retry semantics and accepted seeds are preserved. Available RPLAN/RCTX exports retain physical boundaries and input identities. Export errors preserve generated assignments. No source authentication, historical execution authentication, optimality or legal certification."}});
 if let Some(context)=context_engine_source {output["context_engine_source"]=context;}
 if source.is_none(){output["replay_artifact_sha256"]=json!(format!("{:x}",Sha256::digest(replay_json.as_bytes())));output["replay_artifact_json"]=json!(replay_json);}
 Ok(output)
}
enum Run{Ordinary(Request),Multiscale(MultiscaleRequest,Option<Vec<(usize,usize,f64)>>)}

/// Explicit timing operation; review stays inert and normal requests stay unchanged.
pub(crate) fn run_with_partition_timing(input:Input)->Result<Value,String>{
 if input.inspect_only{return run(input);}
 let capture=crate::partition_timing::Capture::start()?;
 let mut output=run(input)?;output["partition_timing"]=capture.finish();Ok(output)
}
