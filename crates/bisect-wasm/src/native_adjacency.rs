use base64::{engine::general_purpose::STANDARD,Engine};
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Clone,Deserialize,serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {name:String,base64:String}
#[derive(Clone,Deserialize,serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {adjacency:Source,geoids:Option<Source>,centroids:Option<Source>,#[serde(default)]geometry:Option<Source>,#[serde(default)]county_ids:Option<Source>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Geometry {schema_version:u32,state:String,year:String,area_units:String,length_units:String,geoids:Vec<String>,areas:Vec<f64>,exterior_perimeters:Vec<f64>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PrepareInput {sources:Input,state:String,year:String,#[serde(default)]unit_kind:Option<rplan_core::UnitKind>,#[serde(default="geometry_required")]require_geometry:bool}
impl PrepareInput {
 pub(crate) fn scope(&self)->(&str,&str){(&self.state,&self.year)}
 pub(crate) fn adjacency_name(&self)->&str{&self.sources.adjacency.name}
}
fn geometry_required()->bool{true}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CountyCompanion {schema_version:u32,state:String,year:String,unit_ids:Vec<String>,county_ids:Vec<String>}
fn bytes(source:&Source,limit:usize)->Result<Vec<u8>,String>{
 if source.name.is_empty()||source.name.len()>200||source.base64.len()>limit.div_ceil(3)*4{return Err("Invalid bounded native adjacency source".into());}
 let bytes=STANDARD.decode(&source.base64).map_err(|_|"Invalid native adjacency base64")?;
 if bytes.is_empty()||bytes.len()>limit||STANDARD.encode(&bytes)!=source.base64{return Err("Invalid canonical native adjacency source".into());}Ok(bytes)
}
pub(crate) fn prepare(input:PrepareInput)->Result<Value,String>{
 let custom=matches!(input.unit_kind,Some(rplan_core::UnitKind::Precinct|rplan_core::UnitKind::Imported));
 if if custom {input.year.len()!=4||!input.year.bytes().all(|b|b.is_ascii_digit())}else{!["2000","2010","2020"].contains(&input.year.as_str())}{return Err("Select Census year 2000, 2010 or 2020, or an explicit four-digit year for precinct/imported inputs".into());}
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e|e.to_string())?;
 let state=manifest.states.get(&input.state);
 if !custom&&state.is_none(){return Err("Select a known state code".into());}
 if custom&&(input.state.len()!=2||!input.state.bytes().all(|b|b.is_ascii_uppercase())){return Err("Custom graphs require an explicit two-letter uppercase jurisdiction code.".into());}
 let width=match input.unit_kind.unwrap_or(rplan_core::UnitKind::Tract){rplan_core::UnitKind::County=>Some(5),rplan_core::UnitKind::Tract=>Some(11),rplan_core::UnitKind::BlockGroup=>Some(12),rplan_core::UnitKind::Block=>Some(15),rplan_core::UnitKind::Precinct|rplan_core::UnitKind::Imported=>None};
 // Retain native typed values directly: an intermediate JSON round trip can
 // perturb floating-point geometry/centroids and would invalidate exact parity.
 let mut loaded=load(&input.sources)?;
 let n=loaded.graph.n_vertices;
 let mapping=&loaded.graph.index_to_geoid;
 if mapping.len()!=n{return Err("Engine preparation requires exactly one GEOID for every vertex".into());}
 let mut geoids=Vec::with_capacity(n);
 for i in 0..n {
  let geoid=mapping.get(&i).ok_or("GEOID companion must cover every vertex index")?;
  if width.is_some_and(|width|geoid.len()!=width||!geoid.bytes().all(|b|b.is_ascii_digit())||!geoid.starts_with(state.map(|s|s.fips.as_str()).unwrap_or(""))){return Err(if input.unit_kind.is_none(){"Engine preparation requires tract GEOIDs belonging to the selected state".into()}else{"Engine preparation requires complete GEOIDs of the selected unit kind belonging to the selected state".into()});}
  geoids.push(geoid.clone());
 }
 let county_ids=if custom {
  let source=input.sources.county_ids.as_ref().ok_or("Precinct/imported binary preparation requires an explicit county companion; county IDs are never inferred from custom unit IDs.")?;
  let companion:CountyCompanion=serde_json::from_str(text(&bytes(source,16*1024*1024)?)?).map_err(|e|format!("Invalid county companion: {e}"))?;
  if companion.schema_version!=1||companion.state!=input.state||companion.year!=input.year||companion.unit_ids!=geoids {return Err("County companion must match the selected state, year and complete ordered unit IDs.".into());}
  Some(companion.county_ids)
 }else{if input.sources.county_ids.is_some(){return Err("Census graphs use canonical county prefixes; clear the custom county companion.".into());}None};
 let geometry_origin=if let Some(source)=&input.sources.geometry {
  if !loaded.areas.is_empty()||!loaded.perimeters.is_empty(){return Err("Supplemental geometry cannot override embedded native geometry; clear the supplemental source".into());}
  let geometry:Geometry=serde_json::from_str(text(&bytes(source,16*1024*1024)?)?).map_err(|e|format!("Invalid supplemental geometry: {e}"))?;
  if geometry.schema_version!=1||geometry.state!=input.state||geometry.year!=input.year||geometry.area_units!="m2"||geometry.length_units!="m"||geometry.geoids!=geoids{return Err("Supplemental geometry must match state, year, complete ordered GEOIDs and m2/m units".into());}
  loaded.areas=geometry.areas;loaded.perimeters=geometry.exterior_perimeters;"supplemental"
 }else{"embedded"};
 if input.require_geometry&&(loaded.areas.len()!=n||loaded.perimeters.len()!=n){return Err("Engine preparation requires complete embedded or explicit supplemental geometry attributes".into());}
 if input.require_geometry&&loaded.graph.tract_centroids.len()!=n{return Err("Engine preparation requires a complete WGS84 centroid companion".into());}
 let mut edges=loaded.graph.edge_weights.iter().map(|(&(u,v),&w)|(u,v,w)).collect::<Vec<_>>();edges.sort_by_key(|e|(e.0,e.1));
 let graph=crate::engine::PreparedGraph {native_source_identity:false,native_geoid_index:None,county_ids,unit_kind:input.unit_kind,schema_version:1,state:input.state,year:input.year,geoids,adjacency:loaded.graph.adjacency,population:loaded.graph.vertex_weights,edges,areas:loaded.areas,exterior_perimeters:loaded.perimeters,centroids:loaded.graph.tract_centroids};
 let options:crate::engine::Options=serde_json::from_value(json!({"structure":"standard-bisect","weights":"geographic","search":"single","districts":1,"seed":42,"seeds":1,"steps":1,"percentile":0.0,"alpha_county":0.0,"balance_tolerance":0.5,"area_swing":1.0,"iterations":10})).map_err(|e|e.to_string())?;
 crate::engine::validate(&graph,&options)?;
 let graph_json=serde_json::to_string(&graph).map_err(|e|e.to_string())?;
 if graph_json.len()>8*1024*1024{return Err("Prepared native engine graph exceeds 8 MiB".into());}
 let mut result=json!({"schema_version":"bisect-native-engine-input-v1","status":"prepared","execution_performed":false,"state":graph.state,"year":graph.year,"geometry_origin":geometry_origin,"units":n,"population_text":graph.population.iter().sum::<i64>().to_string(),"suggested_districts":state.and_then(|s|s.districts.get(&graph.year)),"graph_sha256":format!("{:x}",Sha256::digest(graph_json.as_bytes())),"graph_json":graph_json,"sources":loaded.sources,"format_version":loaded.version,"scope":"Native tract graph validated for engine input. State matches tract GEOIDs; year and geometry units remain source-owner claims (areas square metres, perimeter/edge metres, centroids WGS84). No polygon display map is derived from adjacency. Select and review YAML options, district count and seed before a separate run. No execution or certification."});
 if let Some(kind)=graph.unit_kind{result["unit_kind"]=serde_json::to_value(kind).map_err(|e|e.to_string())?;result["scope"]=json!("Native Census unit graph validated for engine input. Unit kind and state match complete ordered GEOIDs; year and physical geometry units remain source-owner claims (areas square metres, perimeter/edge metres, centroids WGS84). Select and review YAML, district count and seed before a separate run. No execution, polygon display map or certification.");}
 if custom {result["county_identity"]=json!("explicit-companion");result["scope"]=json!("Native precinct/imported binary graph validated with exact selected unit order and explicit aligned county IDs. State/year, county assignments and measurement units remain source-owner claims. No Census GEOID or county-prefix inference, polygon map, execution, source authentication or certification. Select and review YAML, district count and seed before a separate Run.");}
 if !input.require_geometry {
  result["require_geometry"]=json!(false);
  result["input_availability"]=json!({"areas_m2":!graph.areas.is_empty(),"exterior_perimeters_m":!graph.exterior_perimeters.is_empty(),"centroids_wgs84":!graph.centroids.is_empty()});
  if graph.areas.is_empty()&&graph.exterior_perimeters.is_empty(){result["geometry_origin"]=json!("unavailable");}
  let scope=result["scope"].as_str().unwrap_or("").to_owned();result["scope"]=json!(format!("{scope} Optional geometry mode preserves missing attributes as empty arrays. Each selected algorithm checks its own requirements on Run; no coordinates, land area or perimeter values are fabricated."));
 }
 Ok(result)
}
fn identity(source:&Source,bytes:&[u8])->Value{json!({"name":source.name,"sha256":format!("{:x}",Sha256::digest(bytes)),"bytes":bytes.len()})}
fn text(b:&[u8])->Result<&str,String>{std::str::from_utf8(b).map_err(|_|"Native adjacency companions require UTF-8".to_string())}
pub(crate) fn load_selected_graph(binary:&[u8],geoids:Option<&str>,centroids:Option<&str>)->Result<bisect_runner::adjacency_loader::LoadedGraph,String>{
 if binary.len()<16||binary.len()>64*1024*1024||u32::from_le_bytes(binary[8..12].try_into().unwrap())>1_000_000||u32::from_le_bytes(binary[12..16].try_into().unwrap())>4_000_000{return Err("Native adjacency inspection exceeds vertex/edge/byte bounds".into());}
 bisect_runner::adjacency_loader::load_adjacency_bytes(binary,geoids,centroids)
}
struct Loaded {graph:bisect_runner::adjacency_loader::LoadedGraph,areas:Vec<f64>,perimeters:Vec<f64>,sources:Value,version:u32}
fn load(input:&Input)->Result<Loaded,String>{
 let binary=bytes(&input.adjacency,64*1024*1024)?;
 if binary.len()<16||u32::from_le_bytes(binary[8..12].try_into().unwrap())>1_000_000||u32::from_le_bytes(binary[12..16].try_into().unwrap())>4_000_000{return Err("Native adjacency inspection exceeds vertex/edge bounds".into());}
 let geoids=input.geoids.as_ref().map(|s|bytes(s,16*1024*1024)).transpose()?;
 let centroids=input.centroids.as_ref().map(|s|bytes(s,16*1024*1024)).transpose()?;
 let geoid_text=geoids.as_deref().map(text).transpose()?;let centroid_text=centroids.as_deref().map(text).transpose()?;
 let loaded=load_selected_graph(&binary,geoid_text,centroid_text)?;
 let embedded=bisect_data::deserialize_adjacency(&binary).map_err(|e|e.to_string())?;
 let geometry=input.geometry.as_ref().map(|s|bytes(s,16*1024*1024)).transpose()?;
 if let Some(b)=&geometry{text(b)?;}
 let sources=json!({"adjacency":identity(&input.adjacency,&binary),"geoids":input.geoids.as_ref().zip(geoids.as_ref()).map(|(s,b)|identity(s,b)),"centroids":input.centroids.as_ref().zip(centroids.as_ref()).map(|(s,b)|identity(s,b))});
 let mut sources=sources;
 if let Some((source,b))=input.geometry.as_ref().zip(geometry.as_ref()){sources["geometry"]=identity(source,b);}
 if let Some(source)=&input.county_ids {let b=bytes(source,16*1024*1024)?;text(&b)?;sources["county_ids"]=identity(source,&b);}
 Ok(Loaded {graph:loaded,areas:embedded.vertex_areas,perimeters:embedded.vertex_ext_perimeters,sources,version:u32::from_le_bytes(binary[4..8].try_into().unwrap())})
}
pub(crate) fn inspect(input:Input)->Result<Value,String>{
 let selected=load(&input)?;let loaded=selected.graph;
 let mut edges=loaded.edge_weights.iter().map(|(&(u,v),&w)|(u,v,w)).collect::<Vec<_>>();edges.sort_by_key(|e|(e.0,e.1));
 let mapping=loaded.index_to_geoid.iter().map(|(&i,g)|(i,g.clone())).collect::<std::collections::BTreeMap<_,_>>();
 // Keep native i64 populations inside exact text; JS never reserializes integers.
 let document=json!({"adjacency":loaded.adjacency,"vertex_weights":loaded.vertex_weights,"edges":edges,"index_to_geoid":mapping,"n_vertices":loaded.n_vertices,"n_edges":loaded.n_edges,"tract_centroids":loaded.tract_centroids,"vertex_areas":selected.areas,"vertex_ext_perimeters":selected.perimeters});
 let loaded_graph_json=serde_json::to_string_pretty(&document).map_err(|e|e.to_string())?;
 if loaded_graph_json.len()>64*1024*1024{return Err("Native adjacency inspection output exceeds 64 MiB".into());}
 Ok(json!({"schema_version":"bisect-native-adjacency-inspection-v1","status":"inspected","execution_performed":false,"n_vertices":loaded.n_vertices,"n_edges":loaded.n_edges,"format_version":selected.version,"loaded_graph_json":loaded_graph_json,"loaded_graph_sha256":format!("{:x}",Sha256::digest(loaded_graph_json.as_bytes())),"sources":selected.sources,"scope":"Selected native RADJ bytes and optional GEOID/centroid companions parsed through the shared native loader. Paths are names only; no files or Python pickle are opened. Exact native populations remain in JSON text. Supplemental geometry is retained as a source claim and is not applied to this inspection. This inspects data; engine identity, geometry and topology admissibility require separate validation. No plan execution or certification."}))
}

#[derive(Clone,Deserialize,serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FreshPrepareInput {
 pub sources:Input,pub state:String,pub year:String,
 #[serde(default)]pub require_geometry:bool,
}
impl FreshPrepareInput {
 pub(crate) fn scope(&self)->(&str,&str){(&self.state,&self.year)}
 pub(crate) fn adjacency_name(&self)->&str{&self.sources.adjacency.name}
}
pub(crate) struct FreshGraph {
 pub graph:crate::engine::PreparedGraph,
 pub loaded:bisect_runner::adjacency_loader::LoadedGraph,
 pub sources:Value,
}
pub(crate) fn prepare_fresh_state(input:FreshPrepareInput,resolution:&str)->Result<FreshGraph,String>{
 if input.state.len()!=2||!input.state.bytes().all(|b|b.is_ascii_uppercase())||input.year.len()!=4||!input.year.bytes().all(|b|b.is_ascii_digit()){return Err("Fresh native source requires explicit two-letter uppercase state and four-digit year claims.".into());}
 if input.sources.geometry.is_some()||input.sources.county_ids.is_some(){return Err("Fresh native State preserves native raw graph/report conventions. Supplemental geometry and explicit custom county companions use the existing generic source workflow.".into());}
 let selected=load(&input.sources)?;let n=selected.graph.n_vertices;
 if selected.graph.index_to_geoid.keys().any(|&i|i>=n){return Err("Fresh native State has no representation for out-of-range source index entries.".into());}
 if input.require_geometry&&(selected.areas.len()!=n||selected.perimeters.len()!=n||selected.graph.tract_centroids.len()!=n){return Err("Selected fresh native geometry requirements are incomplete.".into());}
 let units=bisect_runner::native_plan_context::unit_index(&input.state,&input.year,resolution,&selected.graph)?;
 let index=if selected.graph.index_to_geoid.len()==n&&(0..n).all(|i|selected.graph.index_to_geoid.contains_key(&i)){None}else{Some(selected.graph.index_to_geoid.iter().map(|(&i,id)|(i.to_string(),id.clone())).collect())};
 let mut edges=selected.graph.edge_weights.iter().map(|(&(u,v),&w)|(u,v,w)).collect::<Vec<_>>();edges.sort_by_key(|e|(e.0,e.1));
 let graph=crate::engine::PreparedGraph{schema_version:1,state:input.state,year:input.year,unit_kind:Some(units.unit_kind),county_ids:None,native_source_identity:true,native_geoid_index:index,geoids:units.unit_ids,adjacency:selected.graph.adjacency.clone(),population:selected.graph.vertex_weights.clone(),edges,areas:selected.areas,exterior_perimeters:selected.perimeters,centroids:selected.graph.tract_centroids.clone()};
 if serde_json::to_string(&graph).map_err(|e|e.to_string())?.len()>8*1024*1024{return Err("Fresh prepared native graph exceeds8MiB.".into());}
 Ok(FreshGraph{graph,loaded:selected.graph,sources:selected.sources})
}
