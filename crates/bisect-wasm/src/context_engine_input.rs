//! Explicit, hash-checked RCTX to prepared engine graph boundary. No inferred metadata.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,HashMap};
#[derive(Clone,Deserialize,serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub context_json:String,pub source_name:String,pub physical_edge_weights_metres:bool,#[serde(default,skip_serializing_if="Option::is_none")]pub attributes_source:Option<AttributeSource>}
#[derive(Clone,Deserialize,serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttributeSource {pub source_name:String,pub attributes_json:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Attributes {schema_version:u32,context_hash:String,state:String,year:String,unit_kind:rplan_core::UnitKind,unit_ids:Vec<String>,area_units:String,length_units:String,centroid_crs:String,areas:Vec<f64>,exterior_perimeters:Vec<f64>,centroids:Vec<(f64,f64)>}
pub(crate) fn prepare(input:Input)->Result<Value,String>{
 if input.context_json.len()>8*1024*1024||input.source_name.is_empty()||input.source_name.len()>200{return Err("Select an RCTX up to8MiB with a source name of1–200 UTF-8 bytes.".into());}
 if !input.physical_edge_weights_metres{return Err("Explicitly assert that every supplied RCTX edge weight is a physical length in metres. Generic RCTX weights have no declared length unit.".into());}
 let context=rplan_io::read_rctx_str(&input.context_json).map_err(|e|e.to_string())?;
 if context.units.unit_universe_hash!=context.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("RCTX unit-universe hash mismatch.".into());}
 let state=context.units.state.clone().ok_or("RCTX requires an explicit state.")?;
 let year=context.units.year.ok_or("RCTX requires an explicit unit year.")?.to_string();
 let kind=context.units.unit_kind;let ids=context.units.unit_ids.clone();let n=ids.len();
 if n==0||n>1_000_000{return Err("Context engine preparation requires1–1,000,000 units.".into());}
 let native=context.graph.as_ref().ok_or("RCTX requires its supplied graph; geometry hashes cannot reconstruct it.")?;
 if native.adjacency.iter().map(Vec::len).sum::<usize>()>8_000_000{return Err("RCTX exceeds8,000,000 directed arcs.".into());}
 let mut arcs=HashMap::new();
 for (u,row) in native.adjacency.iter().enumerate(){for edge in row{
  let w=edge.weight.ok_or("Every RCTX edge requires an explicit physical weight; absent weights are not filled.")?;
  arcs.insert((u,edge.to as usize),(w,&edge.kind));
 }}
 let mut edges=Vec::new();let mut counts=BTreeMap::<String,usize>::new();
 for (u,row) in native.adjacency.iter().enumerate(){for edge in row{
  let v=edge.to as usize;let (w,kind)=arcs[&(u,v)];
  if arcs.get(&(v,u))!=Some(&(w,kind)){return Err("Reciprocal RCTX edge weights and kinds must agree exactly; no directed value is selected or averaged.".into());}
  if u<v{edges.push((u,v,w));let key=serde_json::to_value(kind).map_err(|e|e.to_string())?.as_str().ok_or("Invalid edge kind")?.to_owned();*counts.entry(key).or_default()+=1;}
 }}
 edges.sort_by_key(|e|(e.0,e.1));
 let supplied=context.subdivisions.as_ref().and_then(|s|s.county_ids.as_ref());
 let county_ids=if matches!(kind,rplan_core::UnitKind::Precinct|rplan_core::UnitKind::Imported){
  Some(supplied.ok_or("Custom RCTX units require complete explicit county IDs.")?.iter().cloned().collect::<Option<Vec<_>>>().ok_or("Custom RCTX county IDs cannot be missing.")?)
 }else{
  if supplied.is_some_and(|values|values.iter().zip(&ids).any(|(county,id)|county.as_deref().is_some_and(|county|id.get(..5)!=Some(county)))){return Err("Supplied Census county IDs disagree with canonical unit prefixes.".into());}None
 };
 let (areas,exterior_perimeters,centroids)=if let Some(source)=&input.attributes_source {
  if source.source_name.is_empty()||source.source_name.len()>200||source.attributes_json.len()>8*1024*1024{return Err("Select an attributes source up to8MiB and a name of1–200 UTF-8 bytes.".into());}
  let attrs:Attributes=serde_json::from_str(&source.attributes_json).map_err(|e|format!("Invalid context attributes: {e}"))?;
  if attrs.schema_version!=1||attrs.context_hash!=context.context_hash||attrs.state!=state||attrs.year!=year||attrs.unit_kind!=kind||attrs.unit_ids!=ids||attrs.area_units!="m2"||attrs.length_units!="m"||attrs.centroid_crs!="WGS84"{return Err("Attributes must match the original context hash, state, year, kind and ordered unit IDs, with m2/m units and WGS84 longitude/latitude centroids.".into());}
  if attrs.areas.is_empty()&&attrs.exterior_perimeters.is_empty()&&attrs.centroids.is_empty(){return Err("Supply at least one complete attribute vector; unknown attributes remain empty arrays.".into());}
  (attrs.areas,attrs.exterior_perimeters,attrs.centroids)
 }else{(vec![],vec![],vec![])};
 let graph=crate::engine::PreparedGraph {native_source_identity:false,native_geoid_index:None,schema_version:1,state,year,unit_kind:Some(kind),county_ids,geoids:ids,adjacency:native.adjacency.iter().map(|row|row.iter().map(|e|e.to as usize).collect()).collect(),population:context.populations.clone().ok_or("RCTX requires explicit unit populations.")?,edges,areas,exterior_perimeters,centroids};
 let options=serde_json::from_value(json!({"structure":"standard-bisect","weights":"geographic","search":"single","districts":1,"seed":42,"seeds":1,"steps":1,"percentile":0,"alpha_county":0,"balance_tolerance":0.5,"area_swing":1,"iterations":10})).map_err(|e|e.to_string())?;
 crate::engine::validate(&graph,&options)?;
 let graph_json=serde_json::to_string(&graph).map_err(|e|e.to_string())?;
 if graph_json.len()>8*1024*1024{return Err("Prepared RCTX engine graph exceeds8MiB.".into());}
 let hash=|b:&[u8]|format!("{:x}",Sha256::digest(b));
 let mut result=json!({"schema_version":"bisect-native-engine-input-v1","status":"prepared","execution_performed":false,"state":graph.state,"year":graph.year,"unit_kind":kind,"units":n,"population_text":graph.population.iter().sum::<i64>().to_string(),"suggested_districts":null,"geometry_origin":"unavailable","input_availability":{"areas_m2":!graph.areas.is_empty(),"exterior_perimeters_m":!graph.exterior_perimeters.is_empty(),"centroids_wgs84":!graph.centroids.is_empty()},"graph_sha256":hash(graph_json.as_bytes()),"graph_json":graph_json,"source_context":{"schema_version":"bisect-context-engine-origin-v1","source_name":input.source_name,"source_sha256":hash(input.context_json.as_bytes()),"context_hash":context.context_hash,"unit_universe_hash":context.units.unit_universe_hash,"graph_sha256":hash(graph_json.as_bytes()),"physical_edge_weights_metres":true,"edge_policy":"all-context-edges","edge_kind_counts":counts},"scope":"Hash-checked selected RCTX graph/populations/unit order prepared for a separate Run. Every supplied edge kind, including bridges, remains in topology. Reciprocal weights/kinds must agree. Edge weights as physical metres are an explicit source-owner claim, not inferred from the RCTX schema. Areas, exterior perimeters and centroids are unavailable; geometry hashes are not coordinates. Municipal IDs, demographics, geometry references and original edge-kind semantics remain only in the retained original RCTX, not separate engine inputs. Choose auxiliary inputs explicitly for methods requiring them. Context identity/byte integrity do not establish source truth, earlier execution, legality or optimality; no plan is executed or certified."});
 if let Some(source)=&input.attributes_source {
  result["geometry_origin"]=json!("supplemental-attributes");
  result["source_context"]["attributes_source"]=json!({"source_name":source.source_name,"source_sha256":hash(source.attributes_json.as_bytes()),"bytes":source.attributes_json.len()});
  let scope=result["scope"].as_str().unwrap().replace("Areas, exterior perimeters and centroids are unavailable; geometry hashes are not coordinates.","Attribute vectors come from a separate selected owner document bound to this context and ordered units. Empty vectors remain unavailable. No coordinates, areas or perimeters are inferred from RCTX references.");result["scope"]=json!(scope);
 }
 Ok(result)
}
