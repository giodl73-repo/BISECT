//! Rebuild the native experimental tract profile from retained source bytes.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::io::Cursor;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
 #[serde(default)]geometry_profile:GeometryProfile,
 state:String,tiger:crate::tiger_source::Input,pl94:crate::pl94_source::Input,
 projection:crate::tiger_source::Source,
}
#[derive(Deserialize,Default,PartialEq)]
#[serde(rename_all="kebab-case")]
enum GeometryProfile {#[default]DisplaySimplified,DisplayTopology,SourceExact,SourceExactLarge}
pub(crate) fn prepare(input:Input)->Result<Value,String>{prepare_impl(input,false)}
pub(crate) fn prepare_census(input:Input)->Result<Value,String>{prepare_impl(input,true)}
fn prepare_impl(input:Input,census:bool)->Result<Value,String>{
 use bisect_data::tract_preparation::CensusResolution as Resolution;
 let resolution=match input.tiger.kind {crate::tiger_source::Kind::Tracts=>Resolution::Tract,crate::tiger_source::Kind::BlockGroups if census=>Resolution::BlockGroup,crate::tiger_source::Kind::Blocks if census=>Resolution::Block,crate::tiger_source::Kind::Counties if census=>Resolution::County,_=>return Err("Select Census polygon geometry for the requested preparation operation".into())};
 if input.tiger.year!=input.pl94.year||![2000,2010,2020].contains(&input.pl94.year){return Err("Select tract geometry and matching Census population vintage".into());}
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e|e.to_string())?;
 let state=manifest.states.get(&input.state).ok_or("Select a known state code")?;
 if input.projection.base64.len()>((65536+2)/3)*4{return Err("Select NAD83 PRJ up to 64 KiB".into());}
 let projection=crate::tiger_source::bytes(&input.projection)?;
 let prj=std::str::from_utf8(&projection).map_err(|_|"PRJ must be UTF-8")?;
 if projection.len()>65536||!prj.starts_with("GEOGCS[")||prj.contains("PROJCS[")||!prj.contains("North_American_1983")||!prj.contains("UNIT[\"Degree\""){return Err("Geometry must declare geographic NAD83 in its selected PRJ".into());}
 let source_limit=if matches!(resolution,Resolution::County){96*1024*1024}else{64*1024*1024};
 let read=|s:&crate::tiger_source::Source|crate::tiger_source::bytes_with_limit(s,source_limit);
 let shp_raw=read(&input.tiger.shp)?;let dbf_raw=read(&input.tiger.dbf)?;let shx_raw=input.tiger.shx.as_ref().map(read).transpose()?;
 let shp=crate::tiger_source::decoded_part(&shp_raw)?;let dbf=crate::tiger_source::decoded_part(&dbf_raw)?;let shx=shx_raw.as_deref().map(crate::tiger_source::decoded_part).transpose()?;
 crate::tiger_source::preflight(&shp,shx.as_deref())?;
 let mut records=if census{bisect_data::tiger::read_tiger_census_units_from_streams(Cursor::new(shp.as_ref()),Cursor::new(dbf.as_ref()),shx.as_deref().map(Cursor::new),resolution.width())}else{bisect_data::tiger::read_tiger_units_from_streams(Cursor::new(shp.as_ref()),Cursor::new(dbf.as_ref()),shx.as_deref().map(Cursor::new),false)}.map_err(|e|e.to_string())?;
 let source_geometry_units=records.len();
 if matches!(resolution,Resolution::County){records.retain(|r|r.geoid.starts_with(&state.fips));}
 let ignored_geometry_units=source_geometry_units-records.len();
 let mut ids=std::collections::BTreeSet::new();
 if records.is_empty()||records.len()>100000||records.iter().any(|r|r.geoid.len()!=resolution.width()||!r.geoid.bytes().all(|b|b.is_ascii_digit())||!r.geoid.starts_with(&state.fips)||!ids.insert(r.geoid.clone())){return Err("Tract universe must be nonempty, unique, numeric and in the selected state".into());}
 let topology_display=input.geometry_profile==GeometryProfile::DisplayTopology;
 let large_source=input.geometry_profile==GeometryProfile::SourceExactLarge||topology_display;
 let exact_geometry_json=if input.geometry_profile!=GeometryProfile::DisplaySimplified{
  let mut geometry=if topology_display{bisect_data::tract_preparation::census_topology_display_geometry(&records,&input.state,&input.pl94.year.to_string())?}else{bisect_data::tract_preparation::census_source_geometry_bounded(&records,&input.state,&input.pl94.year.to_string(),if large_source{bisect_analysis::district_geometry::MAX_UNIT_GEOMETRY_POSITIONS}else{500_000})?};
  if large_source&&!topology_display{geometry["geometry_profile"]=json!("source-exact-large");}
  let text=serde_json::to_string(&geometry).map_err(|e|e.to_string())?;if text.len()>(if large_source{64*1024*1024}else{16*1024*1024}){return Err(if large_source{"Exact source geometry exceeds 64 MiB"}else{"Exact source geometry exceeds 16 MiB"}.into());}Some(text)
 }else{None};
 let geo=crate::pl94_source::bytes(&input.pl94.geography)?;let pop=input.pl94.population.as_ref().map(crate::pl94_source::bytes).transpose()?;
 let mut geo_reader=crate::pl94_source::reader(&geo);let mut pop_reader=pop.as_deref().map(crate::pl94_source::reader);
 let blocks=bisect_data::pl94::read_pl94_block_populations_from_readers(&mut geo_reader,||pop_reader.as_mut().ok_or_else(||std::io::Error::new(std::io::ErrorKind::NotFound,"Select PL94 population segment 1")),input.pl94.year).map_err(|e|e.to_string())?;
 if blocks.iter().any(|r|!r.geoid.starts_with(&state.fips)){return Err("PL94 contains foreign-state blocks".into());}
 let prepared=if census{bisect_data::tract_preparation::prepare_census_graph(&records,&blocks,resolution)}else{bisect_data::tract_preparation::prepare_tract_graph(&records,&blocks)}?;
 if prepared.graph.vertex_weights.iter().any(|p|*p<0||*p>i32::MAX as i64){return Err("Joined tract populations must fit nonnegative native engine weights".into());}
 let graph=crate::engine::PreparedGraph{native_source_identity:false,native_geoid_index:None,county_ids:None,unit_kind:if census{Some(match resolution{Resolution::County=>rplan_core::UnitKind::County,Resolution::Tract=>rplan_core::UnitKind::Tract,Resolution::BlockGroup=>rplan_core::UnitKind::BlockGroup,Resolution::Block=>rplan_core::UnitKind::Block})}else{None},schema_version:1,state:input.state.clone(),year:input.pl94.year.to_string(),geoids:prepared.geoids.clone(),population:prepared.graph.vertex_weights,adjacency:prepared.graph.adjacency,edges:{let mut edges:Vec<_>=prepared.graph.edge_weights.into_iter().map(|((u,v),w)|(u,v,w)).collect();edges.sort_by_key(|&(u,v,_)|(u,v));edges},areas:prepared.graph.vertex_areas,exterior_perimeters:prepared.graph.vertex_ext_perimeters,centroids:prepared.geographic_centroids};
 let graph_json=serde_json::to_string(&graph).map_err(|e|e.to_string())?;
 if graph_json.len()>8*1024*1024{return Err("Prepared graph exceeds the configured-run 8 MiB input bound".into());}
 let geometry_json=if let Some(exact)=exact_geometry_json{exact}else{let geometry=bisect_data::tract_preparation::tract_display_geometry(&records,&input.state,&graph.year)?;let text=serde_json::to_string(&geometry).map_err(|e|e.to_string())?;if text.len()>16*1024*1024{return Err("Prepared geometry exceeds 16 MiB".into());}text};
 let identity=|name:&str,bytes:&[u8]|json!({"name":name,"byte_length":bytes.len(),"sha256":format!("{:x}",Sha256::digest(bytes))});
 let sources=json!({"shp":identity(&input.tiger.shp.name,&shp_raw),"dbf":identity(&input.tiger.dbf.name,&dbf_raw),"shx":input.tiger.shx.as_ref().zip(shx_raw.as_ref()).map(|(s,b)|identity(&s.name,b)),"shp_decoded":crate::tiger_source::decoded_identity(&shp_raw,&shp),"dbf_decoded":crate::tiger_source::decoded_identity(&dbf_raw,&dbf),"shx_decoded":shx_raw.as_ref().zip(shx.as_ref()).map(|(raw,decoded)|crate::tiger_source::decoded_identity(raw,decoded)),"prj":identity(&input.projection.name,&projection),"geography":identity(&input.pl94.geography.name,&geo),"geography_decoded":crate::pl94_source::decoded_identity(&geo,&geo_reader),"population":input.pl94.population.as_ref().zip(pop.as_ref()).map(|(s,b)|identity(&s.name,b)),"population_decoded":if input.pl94.year==2000{None}else{pop.as_ref().zip(pop_reader.as_ref()).map(|(b,r)|crate::pl94_source::decoded_identity(b,r))}});
 let mut output=json!({"schema_version":if census{"bisect-census-input-preparation-v1"}else{"bisect-tract-input-preparation-v1"},"status":"prepared","profile":if census{"lab-census-v1"}else{"lab-tract-v1"},"state":input.state,"year":graph.year,"units":graph.geoids.len(),"population":prepared.input_population.to_string(),"edges":graph.edges.len(),"graph_json":graph_json,"graph_sha256":format!("{:x}",Sha256::digest(graph_json.as_bytes())),"geometry_json":geometry_json,"geometry_sha256":format!("{:x}",Sha256::digest(geometry_json.as_bytes())),"sources":sources,"minimum_shared_boundary_m":10,"bridges":prepared.bridges.iter().map(|&(u,v)|json!({"from":prepared.geoids[u],"to":prepared.geoids[v],"cost":prepared.bridge_weight})).collect::<Vec<_>>(),"suggested_districts":state.districts.get(&graph.year),"execution_performed":false,"scope":"Shared native experimental lab-tract-v1 preparation: PL blocks grouped by tract, full-state population-retention check, EPSG:5070 geometry, native 10m boundary threshold and median retained edge-cost island bridges (fallback 1m). Unmatched TIGER tracts receive native zero population. Checked sums reject integer overflow. Browser additionally requires selected-state unique numeric tract IDs, matching vintages, geographic NAD83 PRJ and engine population bounds. Display geometry uses the native 0.0002-degree simplification. Bridge costs are derived, not measured boundaries. Source hashes bind selected bytes, not publisher authenticity. No certified NRS claim, disk installation or engine run occurred. Native/WASM floating-point geometry can differ slightly; each graph hash binds its own actual bytes. Large source persistence, other resolutions and full Fetch/state orchestration remain unfinished."});
 if census {
  output["unit_kind"]=serde_json::to_value(resolution).map_err(|e|e.to_string())?;
  output["source_geometry_units"]=json!(source_geometry_units);output["ignored_geometry_units"]=json!(ignored_geometry_units);
  output["scope"]=json!("Shared native experimental lab-census-v1 preparation: selected geographic NAD83 Census polygons, checked PL block populations grouped by the exact selected county/tract/block-group/block prefix, full-state population retention, unsimplified EPSG:5070 topology, 10m boundary threshold, native median retained-edge island bridges and 0.0002-degree display simplification. National county geometry is explicitly filtered to the selected state and ignored counts are recorded. Other unit kinds reject foreign-state geometry. Unmatched zero-population units remain explicit; missing populated units, duplicates, wrong resolution, overflow or invalid sources fail. Bridge weights are derived costs, not measured boundaries. Selected/decoded source and output hashes bind bytes, not publisher truth. Existing source/graph/display/workbench limits remain in force; large-block capacity and full native Fetch/State parity remain unverified. No certified NRS claim, disk installation or engine execution occurred.");
 }
 if input.geometry_profile!=GeometryProfile::DisplaySimplified {
  output["geometry_profile"]=json!(if topology_display{"display-topology"}else if large_source{"source-exact-large"}else{"source-exact"});
  output["recommended_dissolve_method"]=json!("batch");
  if topology_display{output["scope"]=json!("Shared native lab-census-v1 graph/population construction with conservative shared-boundary display simplification. Original analytic WKB, populations, adjacency and partition settings are unchanged. Shared arcs retain identical source vertices; indexed guards reject shortcuts crossing original or prior accepted boundaries or sweeping over a foreign source vertex. Display coordinates approximate the source at 0.0002 degrees; downstream topology/non-overlap validation remains required. This is not exact-source geometry, provenance authentication or legal certification.");}
  if !topology_display{let scope=output["scope"].as_str().unwrap_or("").replace("Display geometry uses the native 0.0002-degree simplification.","Geometry retains the exact decoded geographic source coordinates without simplification, snapping or topology repair.").replace("and 0.0002-degree display simplification.","and exact decoded geographic source coordinates without simplification, snapping or topology repair.");
  output["scope"]=json!(format!("{scope} Exact-coordinate geometry is bounded by 16 MiB/500,000 positions. Topology is checked by the separate native analysis/renderer; source coordinates do not guarantee valid polygons. Batch dissolve is recommended to avoid repeated sequential union work."));}
 }
 if large_source{output["scope"]=json!(output["scope"].as_str().unwrap_or("").replace("16 MiB/500,000 positions","64 MiB/2,000,000 positions"));}
 output["projection_math"]=json!(bisect_data::projection::PROJECTION_MATH);
 output["boundary_length_math"]=json!(bisect_data::adjacency::BOUNDARY_LENGTH_MATH);
 let scope=output["scope"].as_str().unwrap_or("").replace("Native/WASM floating-point geometry can differ slightly; each graph hash binds its own actual bytes.","");
 output["scope"]=json!(format!("{scope} EPSG:5070 projection uses shared libm-f64-v1 math; boundary and ring lengths use libm-hypot-f64-v1. Earlier platform-math builds may have different graph hashes; original geographic source coordinates are not rounded or changed."));
 Ok(output)
}
