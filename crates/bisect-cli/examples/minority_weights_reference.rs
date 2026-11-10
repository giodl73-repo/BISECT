//! Actual native composable minority override oracle.
use bisect_cli::edge_weights::{EdgeWeighter, MinorityOverrideWeighter};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path=std::env::args().nth(1).ok_or("Request JSON required")?;
    let v:serde_json::Value=serde_json::from_slice(&std::fs::read(path)?)?;
    let ids:Vec<String>=serde_json::from_value(v["graph"]["geoids"].clone())?;
    let edges:Vec<(usize,usize,f64)>=serde_json::from_value(v["graph"]["edges"].clone())?;
    let fractions=ids.iter().map(|id|v["demographics"]["minority_fractions"][id].as_f64().ok_or("Missing fraction")).collect::<Result<Vec<_>,_>>()?;
    let weights=MinorityOverrideWeighter::new(edges.iter().map(|&(u,w,_)|(u,w)).collect(),fractions,0.40).apply(edges.iter().map(|&(u,w,length)|((u,w),length)).collect());
    let rows=edges.iter().map(|&(u,w,_)|(u,w,weights.get(&(u,w)).copied().unwrap_or(1.0))).collect::<Vec<_>>();
    println!("{}",serde_json::to_string(&rows)?);
    Ok(())
}
