//! Native filesystem adapter for the shared laboratory tract preparation profile.
use serde_json::json;
fn main()->Result<(),String>{
 let args:Vec<_>=std::env::args().collect();if args.len()!=6{return Err("Expected state, year, SHP, geography and population paths".into());}
 let year:u16=args[2].parse().map_err(|_|"Invalid year")?;
 let records=bisect_data::read_tiger_tracts(&args[3]).map_err(|e|e.to_string())?;
 let blocks=bisect_data::read_pl94_block_populations_for_year(&args[4],&args[5],year).map_err(|e|e.to_string())?;
 let p=bisect_data::tract_preparation::prepare_tract_graph(&records,&blocks)?;
 let mut edges:Vec<_>=p.graph.edge_weights.iter().map(|(&(u,v),&w)|(u,v,w)).collect();edges.sort_by_key(|&(u,v,_)|(u,v));
 let graph=json!({"schema_version":1,"state":args[1],"year":args[2],"geoids":p.geoids,"population":p.graph.vertex_weights,"adjacency":p.graph.adjacency,"edges":edges,"areas":p.graph.vertex_areas,"exterior_perimeters":p.graph.vertex_ext_perimeters,"centroids":p.geographic_centroids});
 let geometry=bisect_data::tract_preparation::tract_display_geometry(&records,&args[1],&args[2])?;
 println!("{}",json!({"graph":graph,"geometry":geometry,"population":p.input_population.to_string(),"bridges":p.bridges.iter().map(|&(u,v)|json!({"from":p.geoids[u],"to":p.geoids[v],"cost":p.bridge_weight})).collect::<Vec<_>>() }));Ok(())
}
