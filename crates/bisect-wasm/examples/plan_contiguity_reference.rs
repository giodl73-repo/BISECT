//! Independent invocation of the same native CLI contiguity analyzer.
fn main()->Result<(),Box<dyn std::error::Error>>{
 let path=std::env::args().nth(1).ok_or("request path required")?;
 let mut req:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
 if std::env::args().nth(2).as_deref()==Some("normalize"){
  let ctx:rplan_core::RplanContext=serde_json::from_str(req["context_json"].as_str().ok_or("context required")?)?;
  req["context_json"]=serde_json::json!(rplan_io::write_rctx_string(&ctx)?);
  println!("{}",serde_json::to_string(&req)?);return Ok(());
 }
 let plan=rplan_io::read_rplan_str(req["document_json"].as_str().ok_or("plan required")?)?.plan;
 let ctx=rplan_io::read_rctx_str(req["context_json"].as_str().ok_or("context required")?)?;
 rplan_audit::measurements::measure_plan(&plan,Some(&ctx))?;
 let graph=ctx.graph.as_ref().ok_or("graph required")?;
 let adjacency=graph.adjacency.iter().map(|r|r.iter().map(|e|e.to as usize).collect()).collect::<Vec<Vec<_>>>();
 let assignments=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect();
 let report=bisect_analysis::contiguity::try_check_contiguity(&assignments,&adjacency,&plan.units.unit_ids,plan.k)?;
 print!("{}",serde_json::to_string_pretty(&report)?);Ok(())
}
