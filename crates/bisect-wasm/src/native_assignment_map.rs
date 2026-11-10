//! Render original native assignment snapshots without inventing an RPLAN.
use serde::Deserialize;
use serde_json::{json,Value};
use std::collections::{HashMap,HashSet,BTreeSet};
use base64::{engine::general_purpose::STANDARD,Engine};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Input{
 pub state:String,pub year:u16,pub assignments_base64:String,pub unit_geometry_json:String,
 #[serde(default)]pub unit_index_base64:Option<String>,pub key_basis:String,
 pub map_type:String,pub dpi:u32,#[serde(default)]pub dissolve_method:bisect_analysis::district_geometry::DissolveMethod,
 #[serde(default)]pub analysis_base64:Option<String>,
}
pub(crate) fn source(encoded:&str)->Result<Vec<u8>,String>{if encoded.len()>4*1024*1024{return Err("Native map byte source exceeds 3 MiB".into());}let bytes=STANDARD.decode(encoded).map_err(|_|"Invalid native map Base64 source")?;if bytes.len()>3*1024*1024{return Err("Native map byte source exceeds 3 MiB".into());}Ok(bytes)}
pub(crate) fn sha(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
pub(crate) fn join_assignments(raw:&HashMap<String,usize>,index_bytes:Option<&[u8]>,key_basis:&str,ids:&HashSet<String>)->Result<(HashMap<String,usize>,usize,usize),String>{
 let index=if key_basis=="native-index-map"{
  let bytes=index_bytes.ok_or("Select an explicit native index companion")?;let index:HashMap<String,String>=serde_json::from_slice(bytes).map_err(|e|e.to_string())?;let mut seen=HashSet::new();
  if index.len()!=ids.len(){return Err("Native index companion must cover exactly the supplied geometry universe".into());}
  for (key,id)in &index{let n=key.parse::<usize>().map_err(|_|"Invalid native index key")?;if n>=ids.len()||n.to_string()!=*key||!ids.contains(id)||!seen.insert(id){return Err("Native index companion must uniquely map canonical 0..N-1 keys to supplied unit IDs".into());}}Some(index)
 }else if key_basis=="unit-id"{if index_bytes.is_some(){return Err("Clear native index companion for unit-ID assignments".into());}None}else{return Err("Select explicit unit-id or native-index-map assignment key basis".into());};
 let mut assignments=HashMap::new();let mut ignored=0usize;
 for (key,label)in raw{let id=if let Some(index)=&index{index.get(key)}else{Some(key)};if let Some(id)=id.filter(|id|ids.contains(*id)){assignments.insert(id.clone(),*label);}else{ignored+=1;}}
 let assigned=assignments.len();for id in ids{assignments.entry(id.clone()).or_insert(0);}
 Ok((assignments,assigned,ignored))
}
pub(crate) fn prepare(input:&Input)->Result<(bisect_map::renderer::DistrictGeometries,usize,Value,Value),String>{prepare_mode(input,false)}
pub(crate) fn prepare_national(input:&Input)->Result<(bisect_map::renderer::DistrictGeometries,usize,Value,Value),String>{
 if input.map_type=="rounds"{return Err("National raw assignment maps do not render rounds".into());}prepare_mode(input,true)
}
fn prepare_mode(input:&Input,national:bool)->Result<(bisect_map::renderer::DistrictGeometries,usize,Value,Value),String>{
 if input.state.len()!=2||!input.state.bytes().all(|b|b.is_ascii_uppercase())||bisect_core::state_code_to_fips(&input.state).is_none()||![2000,2010,2020].contains(&input.year){return Err("Select an uppercase native state code and Census year 2000/2010/2020".into());}
 if ![72,100,150,200,300].contains(&input.dpi)||!["districts","rounds","political","demographic","compactness"].contains(&input.map_type.as_str()){return Err("Select a supported native map type and DPI".into());}
 if input.unit_geometry_json.len()>bisect_analysis::district_geometry::unit_geometry_byte_limit(&input.unit_geometry_json,16*1024*1024)?{return Err("Native map geometry exceeds its 16/64 MiB source profile".into());}
 let raw_bytes=source(&input.assignments_base64)?;let text=std::str::from_utf8(&raw_bytes).map_err(|e|e.to_string())?;
 let raw:HashMap<String,usize>=serde_json::from_str(text).map_err(|e|e.to_string())?;
 if raw.len()>100_000||raw.values().any(|v|*v>500){return Err("Native browser assignment maps support up to 100,000 keys and region labels 0..500".into());}
 let requested_k=raw.values().copied().max().unwrap_or(1);
 crate::map_export::validate_geometry_identity_json(&input.unit_geometry_json)?;
 let mut geometry:Value=serde_json::from_str(&input.unit_geometry_json).map_err(|e|e.to_string())?;
 if !geometry["crs"].is_null(){return Err("Native map geometry requires supplied geographic coordinates without a CRS override".into());}
 if geometry.get("state").is_some_and(|v|!v.is_null()&&v.as_str()!=Some(&input.state)){return Err("Native map geometry state differs from selected state".into());}
 if geometry.get("year").is_some_and(|v|!v.is_null()&&v.as_u64()!=Some(input.year as u64)&&v.as_str()!=Some(input.year.to_string().as_str())){return Err("Native map geometry year differs from selected year".into());}
 let features=geometry["features"].as_array_mut().ok_or("Native map geometry requires features")?;
 if features.is_empty()||features.len()>100_000{return Err("Native assignment maps require 1–100,000 supplied units".into());}
 let mut ids=HashSet::new();
 for f in features{
  let aliases=[f["properties"]["GEOID"].as_str(),f["properties"]["geoid"].as_str(),f["id"].as_str()].into_iter().flatten().collect::<Vec<_>>();let id=aliases.first().ok_or("Native map feature requires a string unit ID")?.to_string();
  if id.is_empty()||id.len()>200||aliases.iter().any(|a|*a!=id)||!ids.insert(id.clone()){return Err("Native map unit IDs must be unique, bounded and have consistent aliases".into());}
  if f["properties"].is_null(){f["properties"]=json!({});}let properties=f["properties"].as_object_mut().ok_or("Native map properties must be an object")?;properties.remove("geoid");properties.insert("GEOID".into(),json!(id));
 }
 let index_bytes=input.unit_index_base64.as_ref().map(|encoded|source(encoded)).transpose()?;
 let (assignments,assigned,ignored)=join_assignments(&raw,index_bytes.as_deref(),&input.key_basis,&ids)?;
 let normalized=serde_json::to_string(&geometry).map_err(|e|e.to_string())?;
 let (districts,extra)=bisect_analysis::district_geometry::read_unit_geometries_with_method(&normalized,&assignments,bisect_analysis::district_geometry::CoordinateBasis::Wgs84Degrees,input.dissolve_method)?;if extra!=0{return Err("Unexpected unmatched native geometry identities".into());}
 let labels=districts.keys().map(|id|*id as u64).collect::<BTreeSet<_>>();
 let analysis_bytes=input.analysis_base64.as_ref().map(|encoded|source(encoded)).transpose()?;
 let (analysis,fallback,observation)=if national&&["political","demographic","compactness"].contains(&input.map_type.as_str()){crate::national_map_analysis::statistics_for_labels(analysis_bytes.as_deref(),&input.map_type,&labels)}else if ["political","demographic","compactness"].contains(&input.map_type.as_str()){
  let bytes=analysis_bytes.as_ref().ok_or("Select native analysis JSON for this map type")?;let text=std::str::from_utf8(bytes).map_err(|e|e.to_string())?;let value:Value=serde_json::from_str(text).map_err(|e|e.to_string())?;crate::state_map_analysis::statistics_for_labels(&value,&input.map_type,&labels)
 }else{(Value::Null,vec![],json!({"status":"not-used"}))};
 let k=if input.map_type=="rounds"{districts.len()}else{requested_k};
 fn positions(value:&Value)->usize{match value{Value::Array(a)if a.first().is_some_and(Value::is_number)=>1,Value::Array(a)=>a.iter().map(positions).sum(),_=>0}}
 let source_positions=geometry["features"].as_array().unwrap().iter().map(|f|positions(&f["geometry"]["coordinates"])).sum::<usize>();
 let mut metadata=json!({"schema_version":"bisect-native-district-map-v1","status":"rendered","map_source":"native-assignments","map_type":input.map_type,"dpi":input.dpi,"dissolve_method":input.dissolve_method,"state":input.state,"year":input.year,"units":ids.len(),"source_positions":source_positions,"districts":labels.len(),"rendered_region_labels":labels,"native_requested_region_count":requested_k,"supplied_assignment_keys":raw.len(),"matched_assignment_units":assigned,"unassigned_units_rendered_as_zero":ids.len()-assigned,"ignored_assignment_keys":ignored,"key_basis":input.key_basis,"assignment_source_sha256":sha(&raw_bytes),"unit_index_source_sha256":index_bytes.as_ref().map(|b|sha(b)),"geometry_source_sha256":sha(input.unit_geometry_json.as_bytes()),"geometry_reader_sha256":sha(normalized.as_bytes()),"analysis_source_sha256":analysis_bytes.as_ref().map(|b|sha(b)),"analysis_observation":observation,"fallback_districts":fallback,"plan_hash":Value::Null,"execution_performed":false,"scope":"Original native assignment snapshot rendered without creating or claiming an RPLAN. State/year and supplied geometry are source declarations, not authenticated official coverage. Explicit unit-ID or native index companion joins assignments; unmatched assignment keys are counted and ignored, and unassigned geometry units render as region zero like native CLI loading. Labels 0..500 are accepted; categorical size uses the original maximum raw label, rounds use rendered region count. Choropleths use native state last-row/default semantics and unchecked metadata; missing/unreadable/invalid analysis remains fatal. Source bytes are hashed; geometry/source/work/output limits remain. No plan hash, generated plan, metric certification, lineage or filesystem discovery is claimed."});
 if national{metadata["analysis_semantics"]=json!("native-national");metadata["scope"]=json!("Native national raw assignment source: no RPLAN is created or authenticated. Missing/invalid analysis uses national defaults and last-valid numeric row semantics, including region zero. Full supplied geometry, explicit key mapping, source hashes and source/work limits remain.");}
 Ok((districts,k,analysis,metadata))
}

pub(crate) fn render(input:Input)->Result<Value,String>{
 let (districts,k,analysis,metadata)=prepare(&input)?;let (svg,w,h)=bisect_map::renderer::state_map_svg(districts,k,input.dpi,&input.map_type,&analysis).map_err(|e|e.to_string())?;crate::map_export::outputs(metadata,svg,w,h,2400)
}

#[cfg(test)]mod tests{
 #[test]fn duplicate_geometry_identity_is_rejected_before_normalization(){
  let source=r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"GEOID":"a","GEOID":"b"},"geometry":{"type":"Polygon","coordinates":[]}}]}"#;
  let error=crate::map_export::validate_geometry_identity_json(source).unwrap_err();assert!(error.contains("duplicate field"));
 }
}
