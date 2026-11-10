//! Native selected-file COI adjustment independent of the browser adapter.
use std::collections::HashMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let path=std::env::args().nth(1).ok_or("Request path required")?;let value:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
 let ids:Vec<String>=serde_json::from_value(value["geoids"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(value["edges"].clone())?;
 let adjusted=bisect_cli::runner::apply_coi_weights(edges.into_iter().map(|(u,v,w)|((u,v),w)).collect(),std::path::Path::new(value["coi_path"].as_str().ok_or("COI path required")?),&ids.into_iter().enumerate().collect::<HashMap<_,_>>())?;
 let mut rows:Vec<_>=adjusted.into_iter().map(|((u,v),w)|(u,v,w)).collect();rows.sort_by_key(|&(u,v,_)|(u,v));println!("{}",serde_json::to_string(&rows)?);Ok(())
}
