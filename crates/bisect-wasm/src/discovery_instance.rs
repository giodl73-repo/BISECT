//! Shared native discovery on an exact split instance, including recursive children.
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub seed: String,
    pub refinement: bisect_runner::certified_discovery::DiscoveryRefinementArg,
    pub land_adjacency: Option<Vec<Vec<usize>>>,
}
pub(crate) fn solve(text: &str, settings: Settings) -> Result<Value, String> { inner(text, settings).map_err(|e| e.to_string()) }
fn inner(text: &str, settings: Settings) -> Result<Value, Box<dyn std::error::Error>> {
    if text.len() > 3 * 1024 * 1024 || settings.seed.is_empty() || settings.seed.len() > 20 || !settings.seed.bytes().all(|byte| byte.is_ascii_digit()) || (settings.seed.len() > 1 && settings.seed.starts_with('0')) { return Err("Discovery instance or canonical decimal seed exceeds its boundary".into()); }
    let seed = settings.seed.parse::<u64>()?;
    let instance: bisect_ilp::CertifiedSplitInstance = serde_json::from_str(text)?;
    instance.validate()?;
    if instance.unit_ids.len() > 100000 || instance.edges.len() > 200000 { return Err("Discovery instance graph exceeds its boundary".into()); }
    let mut adjacency = vec![Vec::new(); instance.unit_ids.len()];
    for edge in &instance.edges { adjacency[edge.left].push(edge.right); adjacency[edge.right].push(edge.left); }
    for row in &mut adjacency { row.sort_unstable(); }
    let land = settings.land_adjacency.as_ref().unwrap_or(&adjacency);
    if land.len() != adjacency.len() || land.iter().enumerate().any(|(from, row)| {
        let unique = row.iter().copied().collect::<std::collections::BTreeSet<_>>();
        unique.len() != row.len() || row.iter().any(|&to| to >= land.len() || !adjacency[from].contains(&to) || !land[to].contains(&from))
    }) { return Err("Land adjacency must be a symmetric subset of the exact split graph".into()); }
    let discovery = bisect_runner::certified_discovery::discover(&instance, &adjacency, land, seed, settings.refinement)?;
    Ok(json!({"schema_version":"bisect-certified-discovery-instance-v1", "status":"unproved-incumbent", "instance_hash":instance.hash()?, "discovery_id":discovery.discovery_id, "instance_json":text, "discovery_json":serde_json::to_string_pretty(&discovery)?, "seed":settings.seed, "refinement":settings.refinement, "land_adjacency_supplied":settings.land_adjacency.is_some(), "scope":"Shared native discovery and refinement for this exact supplied split instance. No optimality proof, tree completion or legal compliance is claimed."}))
}
