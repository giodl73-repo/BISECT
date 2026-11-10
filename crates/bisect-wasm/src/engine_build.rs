//! Native final-plan population retries over portable engine requests.
use serde_json::{json,Value};
pub(crate) enum BuildFailure {Other(String),Population{tolerance:f64,last_error:String}}
impl From<String> for BuildFailure {fn from(error:String)->Self{Self::Other(error)}}
impl From<&str> for BuildFailure {fn from(error:&str)->Self{Self::Other(error.into())}}
impl BuildFailure {
 pub(crate) fn message(&self)->String{match self{Self::Other(error)=>error.clone(),Self::Population{tolerance,last_error}=>format!("Native population balance retry policy exhausted after {} attempts (tolerance {}%): {}",bisect_runner::balance_retry::MAX_BALANCE_RETRIES+1,tolerance,last_error)}}
 pub(crate) fn native_label_error(&self,state:&str)->String{let error=match self{Self::Other(error)=>error.clone(),Self::Population{tolerance,last_error}=>bisect_runner::balance_retry::population_failure(tolerance/100.0,last_error)};bisect_runner::balance_retry::state_failure(state,&error)}
 pub(crate) fn details(&self)->Value{match self{Self::Other(_)=>json!({"kind":"generation-or-export","message":self.message()}),Self::Population{tolerance,..}=>json!({"kind":"population-exhausted","attempts":bisect_runner::balance_retry::MAX_BALANCE_RETRIES+1,"tolerance_percent":tolerance,"message":self.message()})}}
}
pub(crate) fn population_check(result:&Value, ids:&[String], population:&[i64], k:usize, tolerance:f64)->Result<Result<(),String>,String>{
    let assignments:std::collections::HashMap<usize,usize>=ids.iter().enumerate().map(|(i,id)| {
        result["assignments"][id].as_u64().map(|d|(i,d as usize)).ok_or_else(||"Missing generated assignment.".to_string())
    }).collect::<Result<_,_>>()?;
    Ok(bisect_core::partition::Partition::from_assignments(assignments)
        .assert_balanced(population,k,tolerance).map_err(|e|e.to_string()))
}
fn attempts_detailed(base:Option<u64>,tolerance:f64,mut execute:impl FnMut(Option<u64>)->Result<(Value,Result<(),String>,f64),String>)->Result<Value,BuildFailure>{
    use bisect_runner::balance_retry::{MAX_BALANCE_RETRIES,balance_retry_seed};
    let mut attempts=Vec::new();let mut last_error=String::new();
    for attempt in 0..=MAX_BALANCE_RETRIES {
        let seed=balance_retry_seed(base,attempt);
        // Generation errors abort immediately, as in native runner dispatch.
        let (mut result,balanced,deviation)=execute(seed)?;
        attempts.push(json!({"attempt":attempt,"seed":seed.map(|s|s.to_string()),
            "balanced":balanced.is_ok(),"max_deviation_percent":deviation}));
        match balanced {
            Ok(())=>{
                if result["ilp_report_capture"].is_object(){result["ilp_report_capture"]["accepted_attempt"]=json!(attempt);}
                result["build_report"]=json!({"schema_version":"bisect-balance-build-v1","accepted_attempt":attempt,
                    "max_balance_retries":MAX_BALANCE_RETRIES,"attempts":attempts,
                    "scope":"First population-passing attempt under native retry order; no best-of selection, legal certification or optimality proof."});
                return Ok(result);
            },
            Err(e)=>last_error=e,
        }
    }
    Err(BuildFailure::Population{tolerance,last_error})
}
fn attempts(base:Option<u64>,tolerance:f64,execute:impl FnMut(Option<u64>)->Result<(Value,Result<(),String>,f64),String>)->Result<Value,String>{attempts_detailed(base,tolerance,execute).map_err(|error|error.message())}
pub(crate) fn run(request:crate::engine::Request)->Result<Value,String>{run_detailed(request).map_err(|error|error.message())}
pub(crate) fn run_detailed(mut request:crate::engine::Request)->Result<Value,BuildFailure>{
    attempts_detailed(request.options.seed,request.options.balance_tolerance,|seed|{
        request.options.seed=seed;
        let result=crate::engine::execute(request.clone())?;
        let balanced=population_check(&result,&request.graph.geoids,&request.graph.population,request.options.districts,request.options.effective_balance_tolerance())?;
        let deviation=result["metrics"]["max_deviation_percent"].as_f64().ok_or("Missing population deviation.")?;
        Ok((result,balanced,deviation))
    })
}
pub(crate) fn run_multiscale(mut input:crate::multiscale_input::MultiscaleRequest)->Result<Value,String>{
    attempts(input.request.options.seed,input.request.options.balance_tolerance,|seed|{
        input.request.options.seed=seed;
        let result=crate::multiscale_input::execute_for_build(input.clone())?;
        let g=&input.request.graph;let o=&input.request.options;
        // The supplied fine graph is the output universe, including block groups.
        let ids=input.block_groups.as_ref().map_or(&g.geoids,|b|&b.geoids);
        let population=input.block_groups.as_ref().map_or(&g.population,|b|&b.population);
        let balanced=population_check(&result,ids,population,o.districts,o.effective_balance_tolerance())?;
        if balanced.is_ok(){crate::multiscale_input::validate_native_final(&input,&result,false)?;}
        let deviation=result["max_deviation_percent"].as_f64().ok_or("Missing fine population deviation.")?;
        Ok((result,balanced,deviation))
    })
}


#[cfg(test)]
mod failure_tests {
 use super::*;
 #[test] fn detailed_retry_context_and_native_index_wording_stay_distinct(){
  let failure=BuildFailure::Population{tolerance:25.0,last_error:"district 1 (tolerance ±0.250)".into()};
  assert_eq!(failure.native_label_error("IA"),"IA: population balance violation (tolerance 25.0%): district 1 (tolerance ?0.250)");
  assert!(failure.message().contains("exhausted after 51 attempts"));
  assert!(failure.message().contains("±0.250"));assert_eq!(failure.details()["attempts"],51);
 }
 #[test] fn typed_failure_preserves_all_native_retry_seeds(){
  let mut seen=Vec::new();let failure=attempts_detailed(Some(u64::MAX),25.0,|seed|{seen.push(seed);Ok((json!({}),Err("not balanced".into()),100.0))}).unwrap_err();
  assert_eq!(seen.len(),51);assert_eq!(seen[0],Some(u64::MAX));assert_eq!(seen[50],Some(49));assert_eq!(failure.details()["kind"],"population-exhausted");
 }
}
