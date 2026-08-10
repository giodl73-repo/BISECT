//! Validated adapter from certified block RCTX and NRS baseline assignments
//! into the arrays consumed by the ReCom kernels.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RctxUnits {
    unit_kind: String,
    state: String,
    year: u16,
    canonical_order: String,
    unit_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RctxEdge {
    to: usize,
    weight: f64,
}

#[derive(Debug, Deserialize)]
struct RctxGraph {
    edge_semantics: String,
    adjacency: Vec<Vec<RctxEdge>>,
}

#[derive(Debug, Deserialize)]
struct BlockRctx {
    rctx_version: String,
    units: RctxUnits,
    graph: RctxGraph,
    populations: Vec<i64>,
}

#[derive(Debug, Deserialize)]
struct BaselineAssignments {
    schema_version: String,
    canonical_order: String,
    label_base: u32,
    assignments: BTreeMap<String, u32>,
}

/// Fully audited input for a block-level ensemble chain.
#[derive(Debug, Clone)]
pub struct BlockEnsembleInput {
    pub state: String,
    pub year: u16,
    pub unit_ids: Vec<String>,
    pub adjacency: Vec<Vec<u32>>,
    /// Parallel to `adjacency`; weights retain the RCTX boundary/bridge signal.
    pub edge_weights: Vec<Vec<f64>>,
    pub populations: Vec<i64>,
    pub assignment: Vec<u32>,
    pub districts: u32,
}

impl BlockEnsembleInput {
    pub fn max_population_deviation(&self) -> f64 {
        let mut totals = vec![0_i64; self.districts as usize];
        for (&population, &district) in self.populations.iter().zip(&self.assignment) {
            totals[(district - 1) as usize] += population;
        }
        let ideal = self.populations.iter().sum::<i64>() as f64 / self.districts as f64;
        totals
            .into_iter()
            .map(|total| (total as f64 - ideal).abs() / ideal)
            .fold(0.0, f64::max)
    }
}

/// Load and audit a certified block context plus its governed NRS assignment.
pub fn load_block_ensemble_input(
    rctx_path: &Path,
    assignments_path: &Path,
    expected_state: &str,
    expected_year: u16,
    expected_districts: u32,
    tolerance: f64,
) -> Result<BlockEnsembleInput> {
    let rctx_bytes = fs::read(rctx_path)
        .with_context(|| format!("cannot read block RCTX {}", rctx_path.display()))?;
    let assignment_bytes = fs::read(assignments_path).with_context(|| {
        format!(
            "cannot read baseline assignments {}",
            assignments_path.display()
        )
    })?;
    parse_block_ensemble_input(
        &rctx_bytes,
        &assignment_bytes,
        expected_state,
        expected_year,
        expected_districts,
        tolerance,
    )
}

fn parse_block_ensemble_input(
    rctx_bytes: &[u8],
    assignment_bytes: &[u8],
    expected_state: &str,
    expected_year: u16,
    expected_districts: u32,
    tolerance: f64,
) -> Result<BlockEnsembleInput> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        bail!("population tolerance must be finite and nonnegative");
    }
    let rctx: BlockRctx = serde_json::from_slice(rctx_bytes).context("invalid block RCTX JSON")?;
    let baseline: BaselineAssignments =
        serde_json::from_slice(assignment_bytes).context("invalid baseline assignment JSON")?;
    let state = expected_state.to_uppercase();
    if rctx.rctx_version != "0.1" {
        bail!("unsupported RCTX version {}", rctx.rctx_version);
    }
    if rctx.units.unit_kind != "block"
        || rctx.units.state != state
        || rctx.units.year != expected_year
        || rctx.units.canonical_order != "sorted-geoid"
    {
        bail!("RCTX unit identity does not match the frozen block contract");
    }
    if rctx.graph.edge_semantics != "undirected" {
        bail!("RCTX graph must declare undirected edge semantics");
    }
    if baseline.label_base != 1
        || baseline.canonical_order != "sorted-geoid"
        || !baseline
            .schema_version
            .starts_with("nrs-baseline-assignments-")
    {
        bail!("baseline assignment metadata does not match the NRS contract");
    }

    let n = rctx.units.unit_ids.len();
    if n == 0 || rctx.populations.len() != n || rctx.graph.adjacency.len() != n {
        bail!("RCTX unit, population, and adjacency lengths must agree and be nonzero");
    }
    if rctx
        .units
        .unit_ids
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
    {
        bail!("RCTX unit IDs must be unique and strictly sorted");
    }
    if rctx.populations.iter().any(|&population| population < 0) {
        bail!("RCTX populations must be nonnegative");
    }

    let unit_set: BTreeSet<&str> = rctx.units.unit_ids.iter().map(String::as_str).collect();
    if baseline.assignments.len() != n
        || baseline
            .assignments
            .keys()
            .any(|unit| !unit_set.contains(unit.as_str()))
    {
        bail!("baseline assignment universe differs from the RCTX universe");
    }
    let assignment: Vec<u32> = rctx
        .units
        .unit_ids
        .iter()
        .map(|unit| {
            baseline
                .assignments
                .get(unit)
                .copied()
                .with_context(|| format!("missing assignment for block {unit}"))
        })
        .collect::<Result<_>>()?;
    let labels: BTreeSet<u32> = assignment.iter().copied().collect();
    let expected_labels: BTreeSet<u32> = (1..=expected_districts).collect();
    if labels != expected_labels {
        bail!("baseline labels must be consecutive 1..{expected_districts}");
    }

    let mut adjacency = Vec::with_capacity(n);
    let mut edge_weights = Vec::with_capacity(n);
    for (node, edges) in rctx.graph.adjacency.iter().enumerate() {
        if edges.windows(2).any(|pair| pair[0].to >= pair[1].to) {
            bail!("adjacency for node {node} must be unique and strictly sorted");
        }
        let mut neighbors = Vec::with_capacity(edges.len());
        let mut weights = Vec::with_capacity(edges.len());
        for edge in edges {
            if edge.to >= n || edge.to == node {
                bail!("invalid adjacency endpoint {} from node {node}", edge.to);
            }
            if !edge.weight.is_finite() || edge.weight < 0.0 {
                bail!("edge weights must be finite and nonnegative");
            }
            neighbors.push(u32::try_from(edge.to).context("block index exceeds u32")?);
            weights.push(edge.weight);
        }
        adjacency.push(neighbors);
        edge_weights.push(weights);
    }
    for (node, edges) in rctx.graph.adjacency.iter().enumerate() {
        for edge in edges {
            let reverse_edges = &rctx.graph.adjacency[edge.to];
            let Ok(reverse_index) = reverse_edges.binary_search_by_key(&node, |item| item.to)
            else {
                bail!(
                    "RCTX adjacency is not symmetric for edge {node}-{}",
                    edge.to
                );
            };
            if reverse_edges[reverse_index].weight.to_bits() != edge.weight.to_bits() {
                bail!("RCTX edge weight is asymmetric for edge {node}-{}", edge.to);
            }
        }
    }

    let input = BlockEnsembleInput {
        state,
        year: expected_year,
        unit_ids: rctx.units.unit_ids,
        adjacency,
        edge_weights,
        populations: rctx.populations,
        assignment,
        districts: expected_districts,
    };
    audit_district_contiguity(&input)?;
    let deviation = input.max_population_deviation();
    if deviation > tolerance {
        bail!("baseline population deviation {deviation:.12} exceeds tolerance {tolerance:.12}");
    }
    Ok(input)
}

fn audit_district_contiguity(input: &BlockEnsembleInput) -> Result<()> {
    for district in 1..=input.districts {
        let Some(start) = input.assignment.iter().position(|&label| label == district) else {
            bail!("district {district} is empty");
        };
        let expected = input
            .assignment
            .iter()
            .filter(|&&label| label == district)
            .count();
        let mut seen = vec![false; input.assignment.len()];
        let mut queue = VecDeque::from([start]);
        seen[start] = true;
        let mut reached = 0usize;
        while let Some(node) = queue.pop_front() {
            reached += 1;
            for &neighbor in &input.adjacency[node] {
                let neighbor = neighbor as usize;
                if !seen[neighbor] && input.assignment[neighbor] == district {
                    seen[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        if reached != expected {
            bail!("district {district} is disconnected: reached {reached} of {expected}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> (Vec<u8>, Vec<u8>) {
        let rctx = json!({
            "rctx_version": "0.1",
            "units": {
                "unit_kind": "block", "state": "RI", "year": 2020,
                "canonical_order": "sorted-geoid", "unit_ids": ["1", "2", "3", "4"]
            },
            "graph": {
                "edge_semantics": "undirected",
                "adjacency": [
                    [{"to": 1, "weight": 1.0}],
                    [{"to": 0, "weight": 1.0}, {"to": 2, "weight": 2.0}],
                    [{"to": 1, "weight": 2.0}, {"to": 3, "weight": 1.0}],
                    [{"to": 2, "weight": 1.0}]
                ]
            },
            "populations": [10, 10, 10, 10]
        });
        let assignments = json!({
            "schema_version": "nrs-baseline-assignments-v0.3-v1",
            "canonical_order": "sorted-geoid", "label_base": 1,
            "assignments": {"1": 1, "2": 1, "3": 2, "4": 2}
        });
        (
            serde_json::to_vec(&rctx).unwrap(),
            serde_json::to_vec(&assignments).unwrap(),
        )
    }

    #[test]
    fn valid_block_input_passes_all_audits() {
        let (rctx, assignments) = fixture();
        let input = parse_block_ensemble_input(&rctx, &assignments, "ri", 2020, 2, 0.005).unwrap();
        assert_eq!(input.state, "RI");
        assert_eq!(input.assignment, vec![1, 1, 2, 2]);
        assert_eq!(input.max_population_deviation(), 0.0);
    }

    #[test]
    fn asymmetric_graph_is_rejected() {
        let (rctx, assignments) = fixture();
        let mut value: serde_json::Value = serde_json::from_slice(&rctx).unwrap();
        value["graph"]["adjacency"][1] = json!([{"to": 2, "weight": 2.0}]);
        let error = parse_block_ensemble_input(
            &serde_json::to_vec(&value).unwrap(),
            &assignments,
            "RI",
            2020,
            2,
            0.005,
        )
        .unwrap_err();
        assert!(error.to_string().contains("not symmetric"));
    }

    #[test]
    fn extra_assignment_unit_is_rejected() {
        let (rctx, assignments) = fixture();
        let mut value: serde_json::Value = serde_json::from_slice(&assignments).unwrap();
        value["assignments"]["5"] = json!(1);
        let error = parse_block_ensemble_input(
            &rctx,
            &serde_json::to_vec(&value).unwrap(),
            "RI",
            2020,
            2,
            0.005,
        )
        .unwrap_err();
        assert!(error.to_string().contains("universe differs"));
    }
}
