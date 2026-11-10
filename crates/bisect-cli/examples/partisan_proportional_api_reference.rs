//! Native kernel API oracle; this deliberately does not claim a State preset.
use std::collections::{HashMap,BTreeMap};
use bisect_cli::edge_weights::{EdgeWeighter,PartisanOverrideWeighter};
fn main()->Result<(),Box<dyn std::error::Error>>{
 if cfg!(feature="c-ffi-engine"){return Err("Build portable API reference with --no-default-features".into());}
 let v:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request required")?)?)?;
 let graph=&v["graph"];let a:Vec<Vec<usize>>=serde_json::from_value(graph["adjacency"].clone())?;let p:Vec<i64>=serde_json::from_value(graph["population"].clone())?;let ids:Vec<String>=serde_json::from_value(graph["geoids"].clone())?;let physical:Vec<(usize,usize,f64)>=serde_json::from_value(graph["edges"].clone())?;
 let index:HashMap<_,_>=ids.iter().cloned().enumerate().collect();
 let shares=bisect_cli::partisan_shares::load_partisan_shares(std::path::Path::new(v["shares_path"].as_str().ok_or("Shares TSV required")?),&index,ids.len())?;
 let o=&v["options"];let mut e=PartisanOverrideWeighter::new(physical.iter().map(|&(u,v,_)|(u,v)).collect(),shares,o["dem_threshold"].as_f64().ok_or("Democratic threshold required")?,o["rep_threshold"].as_f64().ok_or("Republican threshold required")?).apply(HashMap::new());
 if let Some(path)=v["coi_path"].as_str(){e=bisect_cli::runner::apply_coi_weights(e,std::path::Path::new(path),&index)?;}
 let (dem,_)=bisect_cli::partisan_shares::load_dem_vote_counts(std::path::Path::new(v["election_path"].as_str().ok_or("Election CSV required")?),&index,ids.len())?;
 let k=o["districts"].as_u64().ok_or("District count required")? as usize;
 let seed=if o["seed"].is_null(){None}else{Some(o["seed"].as_u64().or_else(||o["seed"].as_str().and_then(|s|s.parse().ok())).ok_or("Exact u64 seed required")?)};
 let (plan,splits)=bisect_runner::bisection_runner::run_all_splits_proportional_recorded(&a,&p,&e,&dem,k,o["balance_tolerance"].as_f64().ok_or("Tolerance required")?/100.,o["iterations"].as_u64().ok_or("Iterations required")? as u32,seed,None)?;
 let weighted:f64=physical.iter().filter(|&&(u,v,_)|plan[&u]!=plan[&v]).map(|&(u,v,_)|e.get(&(u,v)).copied().unwrap_or(1.)).sum();
 let assignments:BTreeMap<_,_>=ids.into_iter().enumerate().map(|(i,id)|(id,plan[&i])).collect();
 println!("{}",serde_json::json!({"assignments":assignments,"splits":splits,"weighted_boundary":weighted}));Ok(())
}
