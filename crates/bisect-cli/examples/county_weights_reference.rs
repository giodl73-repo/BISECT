//! Real native SubdivisionWeighter oracle, independent of the WASM edge-weight helper.
use bisect_cli::edge_weights::{EdgeWeighter, SubdivisionWeighter};
use std::collections::HashMap;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("Request JSON required")?;
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let ids: Vec<String> = serde_json::from_value(v["graph"]["geoids"].clone())?;
    let edges: Vec<(usize, usize, f64)> = serde_json::from_value(v["graph"]["edges"].clone())?;
    let index = ids.iter().cloned().enumerate().collect::<HashMap<_, _>>();
    let weights = SubdivisionWeighter::county_only(
        &index,
        ids.len(),
        v["options"]["alpha_county"]
            .as_f64()
            .ok_or("Alpha required")?,
    )
    .apply(
        edges
            .iter()
            .map(|&(u, w, length)| ((u, w), length))
            .collect(),
    );
    let rows = edges
        .iter()
        .map(|&(u, w, _)| (u, w, weights[&(u, w)]))
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&rows)?);
    Ok(())
}
