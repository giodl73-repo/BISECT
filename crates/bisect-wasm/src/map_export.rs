use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use base64::{engine::general_purpose::STANDARD,Engine};
#[derive(Deserialize,Clone,Copy,Default,PartialEq)]
#[serde(rename_all="kebab-case")]
pub(crate) enum StateAnalysisPolicy {#[default]Validated,NativeState}
#[derive(Deserialize,Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {#[serde(default)]pub(crate) analysis_policy:StateAnalysisPolicy,#[serde(default)]round_assignments_json:Option<String>,#[serde(default="default_basis")]round_key_basis:String,#[serde(default)]round_unit_index_json:Option<String>,#[serde(default="default_type")]pub(crate) map_type:String,#[serde(default)]pub(crate) analysis_json:Option<String>,#[serde(default)]pub(crate) analysis_base64:Option<String>,document_json:String,unit_geometry_json:String,pub(crate) dpi:u32,#[serde(default)]dissolve_method:bisect_analysis::district_geometry::DissolveMethod}
fn default_basis()->String {"unit-id".into()}
fn default_type()->String {"districts".into()}
pub(crate) fn validate_geometry_identity_json(text:&str)->Result<(),String>{
 #[derive(Deserialize)] #[allow(dead_code)] struct Properties {#[serde(rename="GEOID")] upper:Option<String>,#[serde(rename="geoid")] lower:Option<String>}
 #[derive(Deserialize)] #[allow(dead_code)] struct Shape {#[serde(rename="type")] kind:String,coordinates:Value}
 #[derive(Deserialize)] #[allow(dead_code)] struct Feature {id:Option<String>,properties:Option<Properties>,geometry:Shape,#[serde(rename="type")] kind:String}
 #[derive(Deserialize)] #[allow(dead_code)] struct Collection {features:Vec<Feature>,#[serde(rename="type")] kind:String,state:Option<Value>,year:Option<Value>,crs:Option<Value>}
 let _:Collection=serde_json::from_str(text).map_err(|e|e.to_string())?;Ok(())
}
pub(crate) fn prepare(input:&Input)->Result<(bisect_map::renderer::DistrictGeometries,usize,Value,Value),String>{
 if input.analysis_policy==StateAnalysisPolicy::NativeState{return crate::state_map_analysis::prepare(input);}
 if input.analysis_base64.is_some(){return Err("Raw analysis byte sources require the native national analysis policy".into());}
 if ![72,100,150,200,300].contains(&input.dpi)||input.document_json.len()>3*1024*1024||input.unit_geometry_json.len()>bisect_analysis::district_geometry::unit_geometry_byte_limit(&input.unit_geometry_json,16*1024*1024)?{return Err("Select native DPI 72/100/150/200/300, RPLAN up to 3 MiB and geometry within its 16/64 MiB source profile".into());}
 let document=rplan_io::read_rplan_str(&input.document_json).map_err(|e|e.to_string())?;
 let plan=&document.plan;plan.validate().map_err(|e|e.to_string())?;
 if plan.units.unit_ids.len()>100_000||plan.k>500{return Err("Native browser maps support up to 100,000 units and 500 districts".into());}
 if !["districts","political","demographic","compactness","rounds"].contains(&input.map_type.as_str()){return Err("Unsupported native state map type".into());}
 let mut fallback_districts=vec![];
 let analysis=if input.map_type=="districts"||input.map_type=="rounds" {
  if input.analysis_json.is_some(){return Err("Clear analysis source for a districts map".into());}Value::Null
 }else{
  let text=input.analysis_json.as_ref().ok_or("Select native analysis JSON for the chosen map type")?;
  if text.len()>3*1024*1024{return Err("Native map analysis exceeds 3 MiB".into());}
  let value=crate::national::parse(text)?;
  if !value.is_object(){return Err("Native map analysis must be an object".into());}
  if let Some(analyzer)=value.get("analyzer").filter(|v|!v.is_null()){if analyzer.as_str()!=Some(input.map_type.as_str()){return Err("Analysis analyzer does not match map type".into());}}
  if let Some(state)=value.get("state").filter(|v|!v.is_null()){if state.as_str()!=plan.units.state.as_deref(){return Err("Analysis state does not match plan".into());}}
  if let Some(year)=value.get("year").filter(|v|!v.is_null()){
   let expected=plan.units.year.ok_or("Analysis declares a year absent from the plan")?;
   if year.as_u64()!=Some(expected as u64)&&year.as_str()!=Some(expected.to_string().as_str()){return Err("Analysis year does not match plan".into());}
  }
  if let Some(hash)=value.get("plan_hash").filter(|v|!v.is_null()){if hash.as_str()!=Some(plan.plan_hash().map_err(|e|e.to_string())?.as_str()){return Err("Analysis plan hash does not match plan".into());}}
  let rows=value["districts"].as_array().ok_or("Native map analysis requires a districts array")?;
  let field=match input.map_type.as_str(){"political"=>"dem_pct","demographic"=>"pct_minority",_=>"polsby_popper"};
  let mut measured=std::collections::HashSet::new();let mut ids=std::collections::HashSet::new();
  for row in rows{
   let id=row["district"].as_u64().filter(|id|*id>0&&*id<=plan.k as u64).ok_or("Analysis district IDs must match one-based plan districts")?;
   if !row.is_object()||!ids.insert(id){return Err("Duplicate or invalid analysis district".into());}
   if row[field].as_f64().is_some(){measured.insert(id);}
  }
  fallback_districts=(1..=plan.k as u64).filter(|id|!measured.contains(id)).collect();value
 };
 if input.map_type!="rounds"&&(input.round_assignments_json.is_some()||input.round_unit_index_json.is_some()||input.round_key_basis!="unit-id"){return Err("Clear round sources/settings for another map type".into());}
 let mut round_regions=None;
 let round_assignments=if input.map_type=="rounds" {
  let text=input.round_assignments_json.as_ref().ok_or("Select round assignments JSON")?;
  if text.len()>3*1024*1024{return Err("Round assignments exceed 3 MiB".into());}
  let raw=crate::national::parse(text)?;let raw=raw.as_object().ok_or("Round assignments must be a JSON object")?;
  if raw.len()>plan.units.unit_ids.len(){return Err("Round assignments must stay within the plan universe".into());}
  let units:std::collections::HashSet<_>=plan.units.unit_ids.iter().cloned().collect();
  let index=match input.round_key_basis.as_str(){
   "unit-id"|"plan-order-index"=>{if input.round_unit_index_json.is_some(){return Err("Clear index companion for this round key basis".into());}None},
   "native-index-map"=>{
    let text=input.round_unit_index_json.as_ref().ok_or("Select native index-to-unit JSON companion")?;
    if text.len()>3*1024*1024{return Err("Round index companion exceeds 3 MiB".into());}
    let value=crate::national::parse(text)?;let map=value.as_object().ok_or("Round index companion must be an object")?;
    if map.len()!=units.len(){return Err("Round index companion must cover the plan universe".into());}
    let mut mapped=std::collections::HashSet::new();let mut index=std::collections::HashMap::new();
    for (key,value) in map {
     let i=key.parse::<usize>().map_err(|_|"Invalid native round index")?;
     if i>=units.len()||i.to_string()!=*key{return Err("Round indices must be canonical 0..N-1".into());}
     let unit=value.as_str().filter(|id|units.contains(*id)).ok_or("Round index companion contains unrelated unit IDs")?.to_owned();
     if !mapped.insert(unit.clone()){return Err("Round index companion repeats a unit ID".into());}index.insert(key.clone(),unit);
    }Some(index)
   },_=>return Err("Select round key basis unit-id, plan-order-index or native-index-map".into()),
  };
  let mut resolved=std::collections::HashMap::new();let mut regions=std::collections::HashSet::new();
  for (key,value) in raw {
   let unit=match input.round_key_basis.as_str(){
    "unit-id"=>key.clone(),"native-index-map"=>index.as_ref().and_then(|m|m.get(key)).ok_or("Round index absent from companion")?.clone(),
    _=>{let i=key.parse::<usize>().map_err(|_|"Invalid plan-order round index")?;if i.to_string()!=*key{return Err("Round index is not canonical".into());}plan.units.unit_ids.get(i).ok_or("Round index outside plan order")?.clone()},
   };
   if !units.contains(&unit){return Err("Round assignment contains unrelated unit IDs".into());}
   let region=value.as_u64().filter(|v|*v<=u32::MAX as u64).ok_or("Round region IDs must be unsigned integers up to u32 max")? as usize;
   resolved.insert(unit,region);regions.insert(region);
  }
  if regions.len()>500{return Err("Round map exceeds 500 regions".into());}round_regions=Some(regions.len());Some(resolved)
 }else{None};
 validate_geometry_identity_json(&input.unit_geometry_json)?;
 let mut geometry:Value=serde_json::from_str(&input.unit_geometry_json).map_err(|e|e.to_string())?;
 if !geometry["crs"].is_null(){return Err("Map source must be supplied WGS84 coordinates without a CRS override".into());}
 if let Some(state)=geometry.get("state").filter(|v|!v.is_null()){if state.as_str()!=plan.units.state.as_deref(){return Err("Map state does not match plan".into());}}
 if let Some(year)=geometry.get("year").filter(|v|!v.is_null()){
  let expected=plan.units.year.ok_or("Map declares a year absent from the plan")?;
  if year.as_u64()!=Some(expected as u64)&&year.as_str()!=Some(expected.to_string().as_str()){return Err("Map year does not match plan".into());}
 }

 let features=geometry["features"].as_array().ok_or("Map requires unit FeatureCollection")?;
 if features.len()!=plan.units.unit_ids.len(){return Err("Map geometry must cover exactly the plan's unit universe".into());}
 let supplied_round_regions=round_regions;
 let mut assignments=round_assignments.unwrap_or_else(||plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect::<std::collections::HashMap<_,_>>());
 let full_units:std::collections::HashSet<_>=plan.units.unit_ids.iter().cloned().collect();let mut seen_units=std::collections::HashSet::new();
 // Normalize unambiguous identity aliases without changing source coordinates.
 for feature in geometry["features"].as_array_mut().ok_or("Missing unit features")? {
  let ids=[feature["properties"]["GEOID"].as_str(),feature["properties"]["geoid"].as_str(),feature["id"].as_str()].into_iter().flatten().collect::<Vec<_>>();
  let id=ids.first().ok_or("Map feature requires GEOID, geoid or string feature.id")?.to_string();
  if ids.iter().any(|value|*value!=id){return Err("Map feature has conflicting unit identity aliases".into());}
  if !full_units.contains(&id)||!seen_units.insert(id.clone()){return Err("Map geometry must uniquely cover the selected plan universe".into());}
  if feature["properties"].is_null(){feature["properties"]=json!({});}
  if !feature["properties"].is_object(){return Err("Map feature properties must be an object".into());}
  feature["properties"]["GEOID"]=json!(id);
  // The shared reader rejects duplicate semantic fields, even equal aliases.
  feature["properties"].as_object_mut().unwrap().remove("geoid");
 }
 fn positions(v:&Value)->usize {match v {Value::Array(a) if a.first().is_some_and(Value::is_number)=>1,Value::Array(a)=>a.iter().map(positions).sum(),_=>0}}
 let source_positions=geometry["features"].as_array().unwrap().iter().map(|f|positions(&f["geometry"]["coordinates"])).sum::<usize>();
 let assigned_units=assignments.len();
 if input.map_type=="rounds"{for id in &plan.units.unit_ids{assignments.entry(id.clone()).or_insert(0);}let count=assignments.values().copied().collect::<std::collections::HashSet<_>>().len();if count>500{return Err("Round map exceeds 500 rendered regions including native zero fill".into());}round_regions=Some(count);}
 let rendered_positions=geometry["features"].as_array().unwrap().iter().map(|f|positions(&f["geometry"]["coordinates"])).sum::<usize>();
 let normalized=serde_json::to_string(&geometry).map_err(|e|e.to_string())?;
 let (districts,extra)=bisect_analysis::district_geometry::read_unit_geometries_with_method(&normalized,&assignments,bisect_analysis::district_geometry::CoordinateBasis::Wgs84Degrees,input.dissolve_method)?;
 if extra!=0{return Err("Map geometry contains unrelated unit IDs".into());}
 let hash=|bytes:&[u8]|format!("{:x}",Sha256::digest(bytes));
 let metadata=json!({"schema_version":"bisect-native-district-map-v1","status":"rendered","map_type":input.map_type,"fallback_districts":fallback_districts,"round_regions":round_regions,"round_supplied_regions":supplied_round_regions,"round_unassigned_rendering":if input.map_type=="rounds"{Some("native-zero")}else{None},"round_assigned_units":if input.map_type=="rounds"{Some(assigned_units)}else{None},"round_unassigned_units":if input.map_type=="rounds"{Some(plan.units.unit_ids.len()-assigned_units)}else{None},"round_complete_universe":if input.map_type=="rounds"{Some(assigned_units==plan.units.unit_ids.len())}else{None},"rendered_positions":rendered_positions,"round_key_basis":input.round_key_basis,"round_source_sha256":input.round_assignments_json.as_ref().map(|s|hash(s.as_bytes())),"round_index_source_sha256":input.round_unit_index_json.as_ref().map(|s|hash(s.as_bytes())),"analysis_source_sha256":input.analysis_json.as_ref().map(|s|hash(s.as_bytes())),"dpi":input.dpi,"dissolve_method":input.dissolve_method,"source_positions":source_positions,"state":plan.units.state,"year":plan.units.year,"units":plan.units.unit_ids.len(),"districts":round_regions.unwrap_or(plan.k),"plan_hash":plan.plan_hash().map_err(|e|e.to_string())?,"plan_source_sha256":hash(input.document_json.as_bytes()),"geometry_source_sha256":hash(input.unit_geometry_json.as_bytes()),"geometry_reader_sha256":hash(normalized.as_bytes()),"execution_performed":false,"scope":"Shared native district renderer with explicitly selected native dissolve method, plate-carree display projection, 8-inch long edge, legacy native palette/number labels and embedded-font Rust SVG/PNG rasterization. Unambiguous GEOID/geoid/string feature.id aliases normalized with original source identity retained. Exact complete unit geometry matched to validated RPLAN; supplied boundaries and WGS84 declarations remain source claims. Native one-based labels correspond to zero-based plan assignments. This renders supplied assignments; it does not generate or certify a plan, authenticate geometry, or measure areas from projected pixels. Political, demographic and compactness colors/stat labels use supplied analysis, with native defaults for fallback_districts (0.5/0.0/0.3 respectively); source identity and matching metadata do not authenticate analysis or bind absent plan hashes. Round maps preserve native unsigned region labels, including label zero. As in native CLI geometry loading, units absent from supplied assignments render in region zero, merged with any explicitly supplied zero region; round_supplied_regions counts supplied labels while round_regions counts rendered labels including this fill. Full source geometry identities remain bound to the original plan universe. They render supplied intermediate regions, preserving native categorical colors and the native global current-region-count annotation (not a verified descendant count). Explicit unit-ID, declared RPLAN order or native index companion resolves keys. Final plan_hash binds the original final RPLAN, not rendered intermediate assignments; original round source/index hashes bind their sources. Engine capture, round-sequence/lineage verification and directory orchestration remain separate."});
 Ok((districts,round_regions.unwrap_or(plan.k),analysis,metadata))
}

fn outputs_staged(metadata:Value,svg:String,w:u32,h:u32,max_pixels:u32)->Result<Value,(&'static str,String)>{
 outputs_bounded(metadata,svg,w,h,max_pixels,16*1024*1024)
}
fn outputs_bounded(mut metadata:Value,svg:String,w:u32,h:u32,max_pixels:u32,max_svg_bytes:usize)->Result<Value,(&'static str,String)>{
 if svg.len()>max_svg_bytes||w==0||h==0||w>max_pixels||h>max_pixels{return Err(("output-limit","Native map output exceeds browser bounds".into()));}
 let png=bisect_map::svg_to_png(&svg,&bisect_map::default_font_db()).map_err(|e|("rasterize",e.to_string()))?;
 if png.len()>32*1024*1024{return Err(("output-limit","Native PNG output exceeds 32 MiB".into()));}
 metadata["width"]=json!(w);metadata["height"]=json!(h);metadata["svg_sha256"]=json!(format!("{:x}",Sha256::digest(svg.as_bytes())));metadata["png_sha256"]=json!(format!("{:x}",Sha256::digest(&png)));metadata["svg_text"]=json!(svg);metadata["png_base64"]=json!(STANDARD.encode(png));Ok(metadata)
}
pub(crate) fn outputs_native_original(metadata:Value,svg:String,w:u32,h:u32)->Result<Value,String>{
 outputs_bounded(metadata,svg,w,h,4800,128*1024*1024).map_err(|(_,message)|message)
}
pub(crate) fn outputs(metadata:Value,svg:String,w:u32,h:u32,max_pixels:u32)->Result<Value,String>{
 outputs_staged(metadata,svg,w,h,max_pixels).map_err(|(_,message)|message)
}
// Batch orchestration needs the actual Rust failure stage, not string matching.
// Existing single-map operations retain their exact result and error contracts.
fn svg_staged(districts:bisect_map::renderer::DistrictGeometries,k:usize,dpi:u32,map_type:&str,analysis:&Value)->Result<(String,u32,u32),(&'static str,String)>{
 bisect_map::renderer::state_map_svg(districts,k,dpi,map_type,analysis).map_err(|e|("render",e.to_string()))
}
pub(crate) fn render_step(input:Input)->Result<Value,String>{
 let failed=|stage:&str,error:String|json!({"schema_version":"bisect-native-map-step-v1","status":"failed","failure_stage":stage,"error":error,"map":Value::Null});
 let (districts,k,analysis,metadata)=match prepare(&input){Ok(v)=>v,Err(error)=>return Ok(failed("geometry",error))};
 let (svg,w,h)=match svg_staged(districts,k,input.dpi,&input.map_type,&analysis){Ok(v)=>v,Err((stage,error))=>return Ok(failed(stage,error))};
 match outputs_staged(metadata,svg,w,h,2400){
  Ok(map)=>Ok(json!({"schema_version":"bisect-native-map-step-v1","status":"rendered","failure_stage":Value::Null,"error":Value::Null,"map":map})),
  Err((stage,error))=>Ok(failed(stage,error)),
 }
}
pub(crate) fn render(input:Input)->Result<Value,String>{
 let (districts,k,analysis,metadata)=prepare(&input)?;
 let (svg,w,h)=bisect_map::renderer::state_map_svg(districts,k,input.dpi,&input.map_type,&analysis).map_err(|e|e.to_string())?;
 outputs(metadata,svg,w,h,2400)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NationalInput {dpi:u32,map_type:String,states:std::collections::BTreeMap<String,Input>}
pub(crate) fn render_national(input:NationalInput)->Result<Value,String>{
 if !["districts","political","demographic","compactness"].contains(&input.map_type.as_str()){return Err("Native national maps do not render rounds".into());}
 if ![72,100,150,200,300].contains(&input.dpi)||input.states.is_empty()||input.states.len()>50{return Err("Select native DPI and 1–50 supplied state maps".into());}
 let total_bytes=input.states.values().map(|s|s.document_json.len().saturating_add(s.unit_geometry_json.len()).saturating_add(s.analysis_json.as_ref().map_or(0,String::len))).sum::<usize>();
 if total_bytes>64*1024*1024{return Err("National map sources exceed 64 MiB combined".into());}
 let mut districts=vec![];let mut provenance=vec![];let mut year=None;let mut units=0u64;let mut positions=0u64;
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e|e.to_string())?;
 let mut states:Vec<_>=input.states.into_iter().collect();
 states.sort_by_key(|(code,_)|manifest.states.get(code).map(|s|s.name.to_lowercase().replace(' ',"_")).unwrap_or_else(||code.clone()));
 for (code,source) in states {
  if !"AL AK AZ AR CA CO CT DE FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY".split_whitespace().any(|v|v==code){return Err("National map keys require unique uppercase state codes".into());}
  if source.dpi!=input.dpi||source.map_type!=input.map_type{return Err("State map DPI/type differs from national settings".into());}
  if source.analysis_policy!=StateAnalysisPolicy::Validated{return Err("State-only analysis policy is not permitted in a national bundle".into());}
  let (geometries,k,analysis,mut metadata)=prepare(&source)?;
  if metadata["state"].as_str()!=Some(code.as_str())||metadata["year"].is_null(){return Err("State plan must declare matching state and year".into());}
  if year.as_ref().is_some_and(|y|y!=&metadata["year"]){return Err("National state plans must share a year".into());}year=Some(metadata["year"].clone());
  positions+=metadata["source_positions"].as_u64().unwrap_or(0);if positions>500_000{return Err("National map exceeds 500,000 source positions".into());}
  units+=metadata["units"].as_u64().unwrap_or(0);if units>100_000{return Err("National map exceeds 100,000 units".into());}
  if districts.len()+k>500{return Err("National map exceeds 500 districts".into());}
  districts.extend(bisect_map::renderer::state_map_items(geometries,k,&input.map_type,&analysis).map_err(|e|e.to_string())?);if let Some(fields)=metadata.as_object_mut(){for key in ["schema_version","status","scope"]{fields.remove(key);}}provenance.push(metadata);
 }
 let (w,h)=bisect_map::canvas_size_from_dpi(input.dpi,16.0,1.6);let projection=bisect_map::InsetProjection::us_national(w,h);
 let svg=bisect_map::renderer::build_national_svg(&districts,&projection,w,h);
 outputs(json!({"schema_version":"bisect-native-district-map-v1","status":"rendered","map_scope":"national","map_type":input.map_type,"dpi":input.dpi,"year":year,"units":units,"districts":districts.len(),"provided_states":provenance,"execution_performed":false,"scope":"Supplied-state national display using shared native 16-inch 1.6-aspect canvas and continental/Alaska/Hawaii inset projection. States sorted by native state-name slug, then one-based district ID; district IDs repeat within states. Native national paths draw exteriors only, ignore holes and stat sublabels, and omit labels too small to fit. Coverage describes supplied states, not a complete national plan or certification. Original source hashes and native defaults are recorded per state. Directory discovery, missing-state orchestration and complete all-state raw-geometry capacity remain unfinished."}),svg,w,h,4800)
}

#[cfg(test)]
mod stage_tests {
 use super::*;
 #[test]fn empty_native_svg_is_a_render_failure(){
  let error=svg_staged(std::collections::HashMap::new(),0,72,"rounds",&Value::Null).unwrap_err();assert_eq!(error.0,"render");assert!(error.1.contains("empty geometry"));
 }
 #[test]fn invalid_svg_is_a_rasterization_failure(){
  let error=outputs_staged(json!({}),"invalid SVG".into(),1,1,2400).unwrap_err();assert_eq!(error.0,"rasterize");
 }
 #[test]fn output_bounds_are_distinct_from_rasterization(){
  let error=outputs_staged(json!({}),"invalid SVG".into(),2401,1,2400).unwrap_err();assert_eq!(error.0,"output-limit");assert!(error.1.contains("bounds"));
 }
 #[test]fn native_original_svg_capacity_does_not_relax_standard_outputs(){
  let svg=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1\" height=\"1\"><!--{}--></svg>","x".repeat(16*1024*1024));
  assert_eq!(outputs_staged(json!({}),svg.clone(),1,1,2400).unwrap_err().0,"output-limit");
  let output=outputs_native_original(json!({}),svg,1,1).unwrap();assert!(output["png_base64"].as_str().unwrap().len()>10);
 }

}
