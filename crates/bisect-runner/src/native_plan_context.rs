//! Shared native State output identities and physical context assembly.
//! No filesystem reads, source authentication, partitioning or imported-context authority.
use std::collections::HashMap;

pub fn unit_index(
    state_code: &str,
    year: &str,
    plan_resolution: &str,
    graph: &crate::adjacency_loader::LoadedGraph,
) -> Result<rplan_core::PlanUnitIndex, String> {
    let unit_kind = match plan_resolution {
        "bg" => rplan_core::UnitKind::BlockGroup,
        "county" => rplan_core::UnitKind::County,
        _ => rplan_core::UnitKind::Tract,
    };
    let mut unit_ids = Vec::with_capacity(graph.n_vertices);
    for idx in 0..graph.n_vertices {
        unit_ids.push(
            graph
                .index_to_geoid
                .get(&idx)
                .cloned()
                .unwrap_or_else(|| idx.to_string()),
        );
    }
    let mut units = rplan_core::PlanUnitIndex {
        unit_kind,
        state: Some(state_code.to_owned()),
        year: year.parse().ok(),
        canonical_order: rplan_core::CanonicalOrder::ExplicitUnitIds,
        unit_ids,
        unit_universe_hash: String::new(),
        source_id: Some(format!("bisect-adjacency-{}-{}", state_code, year)),
    };
    if units.validate().is_err() {
        units.unit_kind = rplan_core::UnitKind::Imported;
    }
    units.unit_universe_hash = units
        .compute_unit_universe_hash()
        .map_err(|e| format!("unit universe hash failed: {e}"))?;
    units
        .validate()
        .map_err(|e| format!("RPLAN unit validation failed: {e}"))?;
    Ok(units)
}

pub fn assignment(
    assignments: &HashMap<usize, usize>,
    n_vertices: usize,
    k: usize,
) -> Result<Vec<u32>, String> {
    let mut out = Vec::with_capacity(n_vertices);
    for idx in 0..n_vertices {
        let one_based = assignments
            .get(&idx)
            .copied()
            .ok_or_else(|| format!("missing final assignment for unit index {idx}"))?;
        if one_based == 0 || one_based > k {
            return Err(format!(
                "final assignment for unit index {idx} has district {one_based}, expected 1..={k}"
            ));
        }
        out.push((one_based - 1) as u32);
    }
    Ok(out)
}

pub fn physical_graph(
    graph: &crate::adjacency_loader::LoadedGraph,
) -> rplan_core::UnitGraph {
    let adjacency = graph
        .adjacency
        .iter()
        .enumerate()
        .map(|(from, neighbors)| {
            neighbors
                .iter()
                .copied()
                .map(|to| {
                    let key = if from < to { (from, to) } else { (to, from) };
                    rplan_core::UnitEdge {
                        to: to as u32,
                        kind: rplan_core::EdgeKind::Boundary,
                        weight: graph.edge_weights.get(&key).copied(),
                    }
                })
                .collect()
        })
        .collect();
    rplan_core::UnitGraph {
        edge_semantics: rplan_core::EdgeSemantics::Undirected,
        adjacency,
    }
}

pub fn subdivisions(
    graph: &crate::adjacency_loader::LoadedGraph,
) -> Option<rplan_core::SubdivisionContext> {
    let mut county_ids = Vec::with_capacity(graph.n_vertices);
    for idx in 0..graph.n_vertices {
        let county_id = graph.index_to_geoid.get(&idx)
            .and_then(|geoid| geoid.get(..5))
            .filter(|prefix| prefix.bytes().all(|byte| byte.is_ascii_digit()))
            .map(str::to_owned);
        county_ids.push(county_id);
    }

    county_ids
        .iter()
        .any(Option::is_some)
        .then_some(rplan_core::SubdivisionContext {
            county_ids: Some(county_ids),
            municipal_ids: None,
        })
}

pub fn context(
    units: rplan_core::PlanUnitIndex,
    graph: &crate::adjacency_loader::LoadedGraph,
    source_hashes: rplan_core::SourceHashes,
    demographics: Option<rplan_core::DemographicContext>,
    geometry: Option<rplan_core::GeometryContext>,
) -> Result<rplan_core::RplanContext, String> {
    let mut context = rplan_core::RplanContext {
        rctx_version: rplan_core::RCTX_VERSION.to_string(),
        context_hash: String::new(), units,
        graph: Some(physical_graph(graph)),
        populations: Some(graph.vertex_weights.clone()),
        subdivisions: subdivisions(graph), demographics, geometry, source_hashes,
    };
    context.context_hash = context.compute_context_hash()
        .map_err(|e| format!("context hash failed: {e}"))?;
    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::path::Path;

    fn originals() -> (Vec<Value>, Value) {
        let corpus = include_str!("../../../scripts/web/fixtures/native-imported-recorded-replay.json");
        let manifest: Value = serde_json::from_str(include_str!("../../../scripts/web/fixtures/native-plan-context-physical-v1/manifest.json")).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(corpus.as_bytes())), manifest["source_corpus_sha256"].as_str().unwrap());
        (serde_json::from_str(corpus).unwrap(), manifest)
    }

    fn loaded(name: &str, manifest: &Value) -> crate::adjacency_loader::LoadedGraph {
        let case = manifest["cases"].as_array().unwrap().iter().find(|r| r["name"] == name).unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/web/fixtures/native-plan-context-physical-v1");
        let blob = |role: &str| -> Option<Vec<u8>> {
            let entry = &case["sources"][role];
            if entry.is_null() { return None; }
            let relative = entry["path"].as_str().unwrap();
            assert_eq!(Path::new(relative).file_name().unwrap().to_str().unwrap(), relative);
            let bytes = std::fs::read(root.join(relative)).unwrap();
            assert_eq!(bytes.len() as u64, entry["bytes"].as_u64().unwrap());
            assert_eq!(format!("{:x}", Sha256::digest(&bytes)), entry["sha256"].as_str().unwrap());
            Some(bytes)
        };
        let adjacency = blob("adjacency-binary").unwrap();
        let geoids = blob("geoid-companion");
        let centroids = blob("centroid-companion");
        crate::adjacency_loader::load_adjacency_bytes(
            &adjacency,
            geoids.as_deref().map(|b| std::str::from_utf8(b).unwrap()),
            centroids.as_deref().map(|b| std::str::from_utf8(b).unwrap()),
        ).unwrap()
    }

    #[test]
    fn native_plan_context_matches_twelve_actual_originals() {
        let (corpus, manifest) = originals();
        assert_eq!(corpus.len(), 12);
        for row in corpus {
            let name = row["name"].as_str().unwrap();
            let graph = loaded(name, &manifest);
            let expected: rplan_core::RplanContext = serde_json::from_str(row["context_json"].as_str().unwrap()).unwrap();
            let metadata: Value = serde_json::from_str(row["request"]["input"]["sources"]["prechecks"]["manifest_json"].as_str().unwrap()).unwrap();
            let resolution = metadata["native_replay_recipe"]["state"]["plan_resolution"].as_str().unwrap();
            let units = unit_index(expected.units.state.as_deref().unwrap(), &expected.units.year.unwrap().to_string(), resolution, &graph).unwrap();
            assert_eq!(serde_json::to_value(&units).unwrap(), serde_json::to_value(&expected.units).unwrap(), "{name}: unit identity");
            assert_eq!(serde_json::to_value(physical_graph(&graph)).unwrap(), serde_json::to_value(&expected.graph).unwrap(), "{name}: original physical graph");
            assert_eq!(serde_json::to_value(subdivisions(&graph)).unwrap(), serde_json::to_value(&expected.subdivisions).unwrap(), "{name}: report counties");
            assert_eq!(serde_json::to_value(legacy_cli_subdivisions(&graph)).unwrap(), serde_json::to_value(subdivisions(&graph)).unwrap(), "{name}: unchanged native ASCII behavior");
            let built = context(units, &graph, expected.source_hashes.clone(), expected.demographics.clone(), expected.geometry.clone()).unwrap();
            assert_eq!(serde_json::to_value(&built).unwrap(), serde_json::to_value(&expected).unwrap(), "{name}: canonical native context assembly");
            let plan: Value = serde_json::from_str(row["document_json"].as_str().unwrap()).unwrap();
            let native: Vec<u32> = serde_json::from_value(plan["plan"]["assignment"].clone()).unwrap();
            let input: HashMap<_, _> = native.iter().enumerate().map(|(i, &d)| (i, d as usize + 1)).collect();
            assert_eq!(assignment(&input, graph.n_vertices, plan["plan"]["k"].as_u64().unwrap() as usize).unwrap(), native, "{name}: ordered assignment");
        }
    }

    #[test]
    fn native_plan_context_assignment_refuses_missing_zero_and_out_of_range() {
        let mut input: HashMap<_, _> = [(0, 1), (1, 1), (2, 2), (3, 2)].into_iter().collect();
        assert_eq!(assignment(&input, 4, 2).unwrap(), [0, 0, 1, 1]);
        input.remove(&3); assert!(assignment(&input, 4, 2).unwrap_err().contains("missing final assignment"));
        input.insert(3, 0); assert!(assignment(&input, 4, 2).unwrap_err().contains("expected 1..=2"));
        input.insert(3, 3); assert!(assignment(&input, 4, 2).unwrap_err().contains("expected 1..=2"));
    }

    #[test]
    fn native_plan_context_preserves_final_bg_output_convention() {
        let (_, manifest) = originals();
        let graph = loaded("block-group-native-spelling", &manifest);
        assert_eq!(unit_index("RI", "2020", "bg", &graph).unwrap().unit_kind, rplan_core::UnitKind::BlockGroup);
        assert_eq!(unit_index("RI", "2020", "block_group", &graph).unwrap().unit_kind, rplan_core::UnitKind::Imported);
    }

    #[test]
    fn native_plan_context_unicode_counties_are_absent_without_panicking() {
        let (_, manifest) = originals();
        let mut graph = loaded("tract", &manifest);
        graph.index_to_geoid = (0..4).map(|i| (i, format!("é😀{i}"))).collect();
        assert!(std::panic::catch_unwind(|| legacy_cli_subdivisions(&graph)).is_err(), "Retained exact pre-fix native helper must reproduce the UTF-8 slice panic");
        assert_eq!(unit_index("RI", "2020", "tract", &graph).unwrap().unit_kind, rplan_core::UnitKind::Imported);
        assert!(subdivisions(&graph).is_none());
    }

    fn legacy_cli_subdivisions(
    graph: &crate::adjacency_loader::LoadedGraph,
) -> Option<rplan_core::SubdivisionContext> {
    let mut county_ids = Vec::with_capacity(graph.n_vertices);
    for idx in 0..graph.n_vertices {
        let county_id = graph.index_to_geoid.get(&idx).and_then(|geoid| {
            (geoid.len() >= 5 && geoid[..5].bytes().all(|byte| byte.is_ascii_digit()))
                .then(|| geoid[..5].to_string())
        });
        county_ids.push(county_id);
    }

    county_ids
        .iter()
        .any(Option::is_some)
        .then_some(rplan_core::SubdivisionContext {
            county_ids: Some(county_ids),
            municipal_ids: None,
        })
}
}
