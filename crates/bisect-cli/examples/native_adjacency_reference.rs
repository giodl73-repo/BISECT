use serde_json::json;
#[derive(serde::Serialize)]
struct EngineGraph {schema_version:u32,state:String,year:String,geoids:Vec<String>,adjacency:Vec<Vec<usize>>,population:Vec<i64>,edges:Vec<(usize,usize,f64)>,areas:Vec<f64>,exterior_perimeters:Vec<f64>,centroids:Vec<(f64,f64)>}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let path=std::env::args().nth(1).ok_or("Expected native binary path")?;
 let loaded=bisect_runner::adjacency_loader::load_adjacency_pkl(std::path::Path::new(&path))?;
 let mut embedded=bisect_data::deserialize_adjacency(&std::fs::read(&path)?)?;
 let mut edges=loaded.edge_weights.iter().map(|(&(u,v),&w)|(u,v,w)).collect::<Vec<_>>();edges.sort_by_key(|e|(e.0,e.1));
 let mapping=loaded.index_to_geoid.iter().map(|(&i,g)|(i,g.clone())).collect::<std::collections::BTreeMap<_,_>>();
 if let Some(state)=std::env::args().nth(2){
  let year=std::env::args().nth(3).ok_or("Expected Census year")?;
  let geoids=(0..loaded.n_vertices).map(|i|mapping.get(&i).cloned().ok_or("Missing GEOID")).collect::<Result<Vec<_>,_>>()?;
  if let Some(path)=std::env::args().nth(4) {
   let data:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
   if !embedded.vertex_areas.is_empty()||!embedded.vertex_ext_perimeters.is_empty(){return Err("Cannot override native geometry".into());}
   if data["schema_version"]!=1||data["state"]!=state||data["year"]!=year||data["area_units"]!="m2"||data["length_units"]!="m"||data["geoids"]!=serde_json::to_value(&geoids)?{return Err("Invalid supplemental scope".into());}
   embedded.vertex_areas=serde_json::from_value(data["areas"].clone())?;embedded.vertex_ext_perimeters=serde_json::from_value(data["exterior_perimeters"].clone())?;
  }
  print!("{}",serde_json::to_string(&EngineGraph {schema_version:1,state,year,geoids,adjacency:loaded.adjacency,population:loaded.vertex_weights,edges,areas:embedded.vertex_areas,exterior_perimeters:embedded.vertex_ext_perimeters,centroids:loaded.tract_centroids})?);return Ok(());
 }
 print!("{}",serde_json::to_string_pretty(&json!({"adjacency":loaded.adjacency,"vertex_weights":loaded.vertex_weights,"edges":edges,"index_to_geoid":mapping,"n_vertices":loaded.n_vertices,"n_edges":loaded.n_edges,"tract_centroids":loaded.tract_centroids,"vertex_areas":embedded.vertex_areas,"vertex_ext_perimeters":embedded.vertex_ext_perimeters}))?);Ok(())
}
