//! Independent selected-file Census geometry/population preparation oracle.
use bisect_data::tract_preparation::{CensusResolution,prepare_census_graph,tract_display_geometry};
fn main()->Result<(),String>{
 let a=std::env::args().collect::<Vec<_>>();if a.len()!=7{return Err("Expected state, year, kind, SHP, geography and population paths".into());}
 let resolution:CensusResolution=serde_json::from_value(serde_json::Value::String(a[3].clone())).map_err(|e|e.to_string())?;
 let year:u16=a[2].parse().map_err(|_|"Invalid year")?;let path=std::path::Path::new(&a[4]);
 let mut records=bisect_data::tiger::read_tiger_census_units_from_streams(std::fs::File::open(path).map_err(|e|e.to_string())?,std::fs::File::open(path.with_extension("dbf")).map_err(|e|e.to_string())?,Some(std::fs::File::open(path.with_extension("shx")).map_err(|e|e.to_string())?),resolution.width()).map_err(|e|e.to_string())?;
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e|e.to_string())?;let state=manifest.states.get(&a[1]).ok_or("Invalid state")?;
 let source_units=records.len();if matches!(resolution,CensusResolution::County){records.retain(|r|r.geoid.starts_with(&state.fips));}let ignored=source_units-records.len();
 let blocks=bisect_data::read_pl94_block_populations_for_year(&a[5],&a[6],year).map_err(|e|e.to_string())?;let p=prepare_census_graph(&records,&blocks,resolution)?;
 let mut edges=p.graph.edge_weights.iter().map(|(&(u,v),&w)|(u,v,w)).collect::<Vec<_>>();edges.sort_by_key(|&(u,v,_)|(u,v));
 let graph=serde_json::json!({"schema_version":1,"unit_kind":resolution,"state":a[1],"year":a[2],"geoids":p.geoids,"population":p.graph.vertex_weights,"adjacency":p.graph.adjacency,"edges":edges,"areas":p.graph.vertex_areas,"exterior_perimeters":p.graph.vertex_ext_perimeters,"centroids":p.geographic_centroids});
 println!("{}",serde_json::json!({"graph":graph,"geometry":tract_display_geometry(&records,&a[1],&a[2])?,"population":p.input_population.to_string(),"ignored_geometry_units":ignored,"source_geometry_units":source_units,"bridges":p.bridges.iter().map(|&(u,v)|serde_json::json!({"from":p.geoids[u],"to":p.geoids[v],"cost":p.bridge_weight})).collect::<Vec<_>>() }));Ok(())
}
