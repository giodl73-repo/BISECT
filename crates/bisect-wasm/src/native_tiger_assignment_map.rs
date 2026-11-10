//! Explicit native CLI map loading: original TIGER WKB directly into native
//! input-order dissolve, without GeoJSON materialization or analysis admission.
use serde::Deserialize;
use serde_json::{json,Value};
use std::{collections::{HashMap,HashSet,BTreeSet},io::Cursor};
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Input {
 pub state:String,pub tiger:crate::tiger_source::Input,pub assignments_base64:String,
 #[serde(default)]pub unit_index_base64:Option<String>,pub key_basis:String,
 pub dpi:u32,pub map_type:String,#[serde(default)]pub analysis_base64:Option<String>,
}
pub(crate) fn prepare(input:&Input)->Result<(bisect_map::renderer::DistrictGeometries,usize,Value,Value),String>{prepare_mode(input,true)}
fn prepare_mode(input:&Input,national:bool)->Result<(bisect_map::renderer::DistrictGeometries,usize,Value,Value),String>{
 use crate::{tiger_source as tiger,native_assignment_map::{source,sha,join_assignments}};
 let fips=bisect_core::state_code_to_fips(&input.state).ok_or("Select a known uppercase state code")?;
 let block_groups=matches!(input.tiger.kind,tiger::Kind::BlockGroups);
 if !matches!(input.tiger.kind,tiger::Kind::Tracts|tiger::Kind::BlockGroups)||![2000,2010,2020].contains(&input.tiger.year)||![72,100,150,200,300].contains(&input.dpi)||(!["districts","rounds","political","demographic","compactness"].contains(&input.map_type.as_str())||national&&input.map_type=="rounds"){return Err("Native TIGER maps require tracts or block groups, Census year and supported national map type/DPI".into());}
 let raw_bytes=source(&input.assignments_base64)?;
 let raw:HashMap<String,usize>=serde_json::from_slice(&raw_bytes).map_err(|e|e.to_string())?;
 if raw.len()>100_000||raw.values().any(|v|*v>500){return Err("Native browser maps support 100,000 assignment keys and labels 0..500".into());}
 let requested_k=raw.values().copied().max().unwrap_or(1);
 let state_analysis=if !national&&["political","demographic","compactness"].contains(&input.map_type.as_str()){
  let bytes=source(input.analysis_base64.as_deref().ok_or("Select native analysis JSON for this map type")?)?;
  let text=std::str::from_utf8(&bytes).map_err(|e|e.to_string())?;let value:Value=serde_json::from_str(text).map_err(|e|e.to_string())?;Some((bytes,value))
 }else{None};
 let shp_raw=tiger::bytes(&input.tiger.shp)?;let dbf_raw=tiger::bytes(&input.tiger.dbf)?;let shx_raw=input.tiger.shx.as_ref().map(tiger::bytes).transpose()?;
 if shp_raw.len()+dbf_raw.len()+shx_raw.as_ref().map_or(0,Vec::len)>90*1024*1024{return Err("Native TIGER state parts exceed 90 MiB combined".into());}
 let shp=tiger::decoded_part(&shp_raw)?;let dbf=tiger::decoded_part(&dbf_raw)?;let shx=shx_raw.as_deref().map(tiger::decoded_part).transpose()?;
 tiger::preflight(&shp,shx.as_deref())?;
 let records=bisect_data::tiger::read_tiger_units_from_streams(Cursor::new(shp.as_ref()),Cursor::new(dbf.as_ref()),shx.as_deref().map(Cursor::new),block_groups).map_err(|e|e.to_string())?;
 let mut ids=HashSet::new();
 if records.is_empty()||records.len()>100_000||records.iter().any(|r|r.geoid.len()!=if block_groups{12}else{11}||!r.geoid.bytes().all(|b|b.is_ascii_digit())||!r.geoid.starts_with(fips)||!ids.insert(r.geoid.clone())){return Err("Native tract universe must be nonempty, unique, numeric and in the selected state".into());}
 let index_bytes=input.unit_index_base64.as_ref().map(|s|source(s)).transpose()?;
 let (assignments,assigned,ignored)=join_assignments(&raw,index_bytes.as_deref(),&input.key_basis,&ids)?;
 let mut positions=0usize;let mut geometries=Vec::with_capacity(records.len());let mut labels=Vec::with_capacity(records.len());
 for record in &records{
  let geometry=bisect_map::wkb_to_geometry(&record.geometry_wkb).map_err(|e|e.to_string())?;
  // Count exact original coordinates, without topology admission or repair.
  let count=|poly:&geo_types::Polygon<f64>|poly.exterior().0.len()+poly.interiors().iter().map(|r|r.0.len()).sum::<usize>();
  let n=match &geometry{geo_types::Geometry::Polygon(p)=>count(p),geo_types::Geometry::MultiPolygon(mp)=>mp.0.iter().map(count).sum(),_=>return Err("Native TIGER map requires Polygon or MultiPolygon records".into())};
  positions=positions.checked_add(n).ok_or("Native coordinate count overflow")?;if positions>4_000_000{return Err("Native TIGER map state exceeds four million source positions".into());}
  geometries.push(geometry);labels.push(assignments[&record.geoid]);
 }
 // Exactly the same native primitive and record/part order as CLI geometry.rs.
 // Native loading intentionally does not run the separate strict unit analyzer.
 let districts=bisect_map::group_dissolve(&geometries,&labels,requested_k);
 let rendered=districts.keys().map(|id|*id as u64).collect::<BTreeSet<_>>();
 let analysis_bytes=if let Some((bytes,_))=&state_analysis{Some(bytes.clone())}else{input.analysis_base64.as_ref().map(|s|source(s)).transpose()?};
 let (analysis,fallback,observation)=if ["districts","rounds"].contains(&input.map_type.as_str()){(Value::Null,vec![],json!({"status":"not-used"}))}else if let Some((_,value))=&state_analysis{crate::state_map_analysis::statistics_for_labels(value,&input.map_type,&rendered)}else{crate::national_map_analysis::statistics_for_labels(analysis_bytes.as_deref(),&input.map_type,&rendered)};
 let identity=|source:&tiger::Source,bytes:&[u8]|json!({"name":source.name,"byte_length":bytes.len(),"sha256":sha(bytes)});
 let mut metadata=json!({"schema_version":"bisect-native-district-map-v1","status":"rendered","state":input.state,"year":input.tiger.year,"map_source":"native-assignments","source_geometry_mode":"native-cli-tiger","topology_validation":"native-cli-unchecked","geometry_materialization":"original-wkb-direct","map_type":input.map_type,"dpi":input.dpi,"dissolve_method":"sequential","units":records.len(),"source_positions":positions,"districts":rendered.len(),"rendered_region_labels":rendered,"native_requested_region_count":requested_k,"supplied_assignment_keys":raw.len(),"matched_assignment_units":assigned,"unassigned_units_rendered_as_zero":ids.len()-assigned,"ignored_assignment_keys":ignored,"key_basis":input.key_basis,"assignment_source_sha256":sha(&raw_bytes),"unit_index_source_sha256":index_bytes.as_ref().map(|b|sha(b)),"tiger_sources":{"shp":identity(&input.tiger.shp,&shp_raw),"dbf":identity(&input.tiger.dbf,&dbf_raw),"shx":input.tiger.shx.as_ref().zip(shx_raw.as_ref()).map(|(s,b)|identity(s,b)),"shp_decoded":tiger::decoded_identity(&shp_raw,&shp),"dbf_decoded":tiger::decoded_identity(&dbf_raw,&dbf),"shx_decoded":shx_raw.as_ref().zip(shx.as_ref()).map(|(r,d)|tiger::decoded_identity(r,d))},"analysis_semantics":"native-national","analysis_source_sha256":analysis_bytes.as_ref().map(|b|sha(b)),"analysis_observation":observation,"fallback_districts":fallback,"plan_hash":Value::Null,"execution_performed":false,"scope":"Explicit native CLI TIGER loading. Original WKB is decoded in native record order and grouped with the identical native input-order dissolve primitive. No GeoJSON intermediate, coordinate simplification, population join, generated plan or RPLAN is created. Native unit topology is unchecked; strict analysis geometry admission/work estimates are not applied. Selected file and source-position limits remain; native unions can be long-running and worker cancellation is available. State/year/geographic coordinates remain source claims, not authentication, legal certification or full national coverage."});
 if !national{metadata["analysis_semantics"]=json!("native-state");metadata["scope"]=json!("Original TIGER state map using native record-order WKB decoding and input-order dissolve. No intermediate GeoJSON, simplification, population join, generated plan or RPLAN identity. State choropleths require readable UTF-8 JSON and use native last-row/default semantics. Original/decoded source hashes and explicit key mapping are recorded. Native unit topology is unchecked; this display does not certify accepted round sequence, lineage, source authenticity or legal compliance. Existing source, position and output bounds remain.");}
 if block_groups {metadata["unit_kind"]=json!("block-group");metadata["scope"]=json!("Explicit selected block-group TIGER geometry, native twelve-digit reader/order and native dissolve/rendering. Original assignments are not regenerated; input coordinates and source hashes are retained. This display does not certify topology, generation, source authenticity, legal validity or complete fine-plan export/workflow support.");}
 let k=if input.map_type=="rounds"{districts.len()}else{requested_k};Ok((districts,k,analysis,metadata))
}

pub(crate) fn render(input:Input)->Result<Value,String>{
 let (districts,k,analysis,metadata)=prepare_mode(&input,false)?;
 let (svg,w,h)=bisect_map::renderer::state_map_svg(districts,k,input.dpi,&input.map_type,&analysis).map_err(|e|e.to_string())?;
 crate::map_export::outputs(metadata,svg,w,h,2400)
}
