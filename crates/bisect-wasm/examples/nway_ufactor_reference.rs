//! Native State's direct n-way dispatch entry point with its separate --ufactor.
use std::collections::{BTreeMap, HashMap};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let file=std::env::args().nth(1).ok_or("request JSON required")?;
 let request:serde_json::Value=serde_json::from_slice(&std::fs::read(file)?)?;
 let graph:bisect_wasm::engine::PreparedGraph=serde_json::from_value(request["graph"].clone())?;
 let options=&request["options"];
 let factor=u32::try_from(options["metis_ufactor"].as_u64().ok_or("ufactor required")?)?;
 let seed=match &options["seed"]{serde_json::Value::String(text)=>Some(text.parse::<u64>()?),serde_json::Value::Number(n)=>Some(n.as_u64().ok_or("invalid seed")?),serde_json::Value::Null=>None,_=>return Err("invalid seed".into())};
 let edges:HashMap<_,_>=if options["weights"]=="unweighted"{HashMap::new()}else{graph.edges.iter().map(|&(u,v,w)|((u,v),w)).collect()};
 let assignments=bisect_runner::bisection_runner::run_nway_partition(&graph.adjacency,&graph.population,&edges,options["districts"].as_u64().ok_or("districts required")? as usize,1.0+factor as f64/1000.0,options["iterations"].as_u64().ok_or("iterations required")? as u32,seed)?;
 let by_id:BTreeMap<_,_>=graph.geoids.into_iter().enumerate().map(|(index,id)|(id,assignments[&index])).collect();println!("{}",serde_json::to_string(&by_id)?);Ok(())
}
