//! Explicit source-preserving preparation for the integer certified split model.
use std::collections::BTreeMap;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::hash;
#[derive(Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum WeightPolicy { SourceInteger, UnweightedEdgeCount }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareRequest { pub operation:String, pub context_json:String, pub districts:usize, pub weight_policy:WeightPolicy }
pub fn prepare(request:PrepareRequest)->Result<Value,String>{inner(request).map_err(|e|e.to_string())}
fn inner(request:PrepareRequest)->Result<Value,Box<dyn std::error::Error>>{
 if request.operation!="prepare-native-certified-model"||request.context_json.len()>8*1024*1024||!(2..=1024).contains(&request.districts){return Err("Invalid model preparation operation, context limit or seat count".into());}
 let original=rplan_io::read_rctx_str(&request.context_json)?;
 let populations=original.populations.as_ref().ok_or("Model preparation requires populations")?;
 let graph=original.graph.as_ref().ok_or("Model preparation requires a graph")?;
 if original.units.unit_ids.len()>100000{return Err("Model preparation exceeds 100000 units".into());}
 let mut context=original.clone();let mut edges=BTreeMap::new();let mut land=vec![Vec::new();populations.len()];
 for (left,row) in graph.adjacency.iter().enumerate(){for edge in row{
  let right=edge.to as usize;if !matches!(edge.kind,rplan_core::EdgeKind::Bridge){land[left].push(right);}
  if left>=right{continue;}
  let weight=match request.weight_policy{
   WeightPolicy::UnweightedEdgeCount=>1,
   WeightPolicy::SourceInteger=>match edge.weight{None=>1,Some(value) if value.is_finite()&&value>0.0&&value.fract()==0.0&&value< u64::MAX as f64=>value as u64,_=>return Err("Source-integer policy requires positive integer edge weights; source weights are never rounded".into())}
  };
  edges.insert((left,right),bisect_ilp::ExactEdge{left,right,weight});
 }}
 if edges.len()>200000{return Err("Model preparation exceeds 200000 edges".into());}
 let (k_left,k_right)=bisect_ilp::canonical_seat_split(request.districts)?;
 let root=bisect_ilp::CertifiedSplitInstance{schema_version:bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.into(),model_id:bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.into(),node_path:String::new(),parent_certificate_id:None,unit_universe_hash:bisect_ilp::certified_split_unit_universe_hash(&original.units.unit_ids)?,unit_ids:original.units.unit_ids.clone(),populations:populations.clone(),edges:edges.into_values().collect(),k_parent:request.districts,k_left,k_right,orientation_rule:bisect_ilp::canonical_orientation_rule(k_left,k_right)};root.validate()?;
 let policy=match request.weight_policy{WeightPolicy::SourceInteger=>"source-integer",WeightPolicy::UnweightedEdgeCount=>"unweighted-edge-count"};
 let changed=matches!(request.weight_policy,WeightPolicy::UnweightedEdgeCount);
 if changed{
  for row in &mut context.graph.as_mut().unwrap().adjacency{for edge in row{edge.weight=Some(1.0);}}
  context.source_hashes.entries.insert("bisect.certified-model.original-context".into(),format!("sha256:{}",hash(request.context_json.as_bytes())));
  context.source_hashes.entries.insert("bisect.certified-model.weight-policy".into(),format!("sha256:{}",hash(policy.as_bytes())));
 }
 context.context_hash=context.compute_context_hash()?;
 let context_json=if changed{rplan_io::write_rctx_string(&context)?}else{request.context_json.clone()};
 // Re-read the native serialization before handing it to later proof/audit tools.
 let prepared=rplan_io::read_rctx_str(&context_json)?;
 let root_json=serde_json::to_string_pretty(&root)?;
 if root_json.len()>crate::MAX_SOURCE{return Err("Prepared split source exceeds 3 MiB".into());}
 Ok(json!({"schema":"bisect-native-certified-model-preparation-v1","status":"model-prepared-unproved","weight_policy":policy,"objective_changed":changed,"districts":request.districts,"original_context_json":request.context_json,"original_context_source_sha256":hash(request.context_json.as_bytes()),"original_context_hash":original.context_hash,"model_context_json":context_json,"model_context_hash":prepared.context_hash,"root_instance_json":root_json,"root_instance_hash":root.hash()?,"land_adjacency":land,"scope":"Explicit native integer proof model preparation. Original context bytes, unit ordering, populations, graph topology and edge kinds are retained. Unweighted-edge-count replaces model edge weights with one and changes the cut objective; the original context is kept separately. No sorting, rounding, discovery, proof, source authentication or optimality claim occurs."}))
}
