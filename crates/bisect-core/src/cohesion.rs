use std::collections::{BTreeSet, HashMap, VecDeque};

use thiserror::Error;

use crate::Graph;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum CohesionError {
    #[error("[INPUT] adjacency edge references missing vertex {vertex}")]
    InvalidNeighbor { vertex: usize },
    #[error("[INPUT] negative population {population} at vertex {vertex}")]
    NegativePopulation { vertex: usize, population: i64 },
    #[error("[INPUT] non-finite boundary weight {weight} for edge ({u}, {v})")]
    NonFiniteBoundaryWeight { u: usize, v: usize, weight: f64 },
    #[error("[INPUT] negative boundary weight {weight} for edge ({u}, {v})")]
    NegativeBoundaryWeight { u: usize, v: usize, weight: f64 },
    #[error("[INPUT] mass clamp min {min_mass} exceeds max {max_mass}")]
    InvalidMassClamp { min_mass: f64, max_mass: f64 },
    #[error("[INPUT] non-finite cohesion parameter {name}={value}")]
    NonFiniteParameter { name: &'static str, value: f64 },
    #[error("[INPUT] invalid geography term {term}={value} for edge ({u}, {v})")]
    InvalidGeographyTerm {
        term: &'static str,
        u: usize,
        v: usize,
        value: f64,
    },
    #[error("[NUMERIC] cohesion weight produced non-finite value for edge ({u}, {v})")]
    NonFiniteCohesionWeight { u: usize, v: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohesionParams {
    pub alpha_cycle: f64,
    pub alpha_module: f64,
    pub alpha_geo: f64,
    pub alpha_bridge: f64,
    pub alpha_barrier: f64,
    pub min_mass: f64,
    pub max_mass: f64,
    pub max_cycle_depth: usize,
}

impl Default for CohesionParams {
    fn default() -> Self {
        Self {
            alpha_cycle: 0.50,
            alpha_module: 0.25,
            alpha_geo: 0.0,
            alpha_bridge: 0.25,
            alpha_barrier: 0.0,
            min_mass: 0.50,
            max_mass: 2.00,
            max_cycle_depth: 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CohesionEdgeTerms {
    pub u: usize,
    pub v: usize,
    pub boundary_weight: f64,
    pub cycle_support: f64,
    pub normalized_cycle_support: f64,
    pub bridge_likeness: f64,
    pub local_mass: f64,
    pub mass_factor: f64,
    pub geo_affinity: f64,
    pub barrier_penalty: f64,
    pub cohesion_weight: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CohesionGeography {
    pub geo_affinity: HashMap<(usize, usize), f64>,
    pub barrier_penalty: HashMap<(usize, usize), f64>,
}

pub fn cohesion_edge_terms(
    graph: &Graph,
    boundary_weights: &HashMap<(usize, usize), f64>,
    params: CohesionParams,
) -> Result<Vec<CohesionEdgeTerms>, CohesionError> {
    cohesion_edge_terms_with_geography(
        graph,
        boundary_weights,
        &CohesionGeography::default(),
        params,
    )
}

pub fn cohesion_edge_terms_with_geography(
    graph: &Graph,
    boundary_weights: &HashMap<(usize, usize), f64>,
    geography: &CohesionGeography,
    params: CohesionParams,
) -> Result<Vec<CohesionEdgeTerms>, CohesionError> {
    validate_params(params)?;
    validate_populations(graph)?;

    let edges = canonical_edges(graph)?;
    let mut raw_terms = Vec::with_capacity(edges.len());
    let mut max_cycle_support = 0.0_f64;
    let mut log_masses = Vec::with_capacity(edges.len());

    for &(u, v) in &edges {
        let boundary_weight = boundary_weight(boundary_weights, u, v)?;
        let cycle_support = alternate_cycle_support(graph, u, v, params.max_cycle_depth)?;
        let local_mass = common_neighborhood_mass(graph, u, v)?;
        max_cycle_support = max_cycle_support.max(cycle_support);
        log_masses.push(local_mass.ln_1p());
        raw_terms.push((u, v, boundary_weight, cycle_support, local_mass));
    }

    let median_log_mass = median(&mut log_masses).max(1.0);
    let mut terms = Vec::with_capacity(raw_terms.len());

    for (u, v, boundary_weight, cycle_support, local_mass) in raw_terms {
        let normalized_cycle_support = if max_cycle_support > 0.0 {
            cycle_support / max_cycle_support
        } else {
            0.0
        };
        let bridge_likeness = 1.0 - normalized_cycle_support;
        let mass_factor =
            (local_mass.ln_1p() / median_log_mass).clamp(params.min_mass, params.max_mass);
        let cycle_factor = 1.0 + params.alpha_cycle * normalized_cycle_support;
        let module_factor = 1.0;
        let geo_affinity = geography_term(&geography.geo_affinity, "geo_affinity", u, v)?;
        let barrier_penalty = geography_term(&geography.barrier_penalty, "barrier_penalty", u, v)?;
        let geo_factor = 1.0 + params.alpha_geo * geo_affinity;
        let bridge_factor = 1.0 - params.alpha_bridge * bridge_likeness;
        let barrier_factor = 1.0 - params.alpha_barrier * barrier_penalty;
        let cohesion_weight = boundary_weight
            * cycle_factor
            * mass_factor
            * module_factor
            * geo_factor
            * bridge_factor
            * barrier_factor;

        if !cohesion_weight.is_finite() {
            return Err(CohesionError::NonFiniteCohesionWeight { u, v });
        }

        terms.push(CohesionEdgeTerms {
            u,
            v,
            boundary_weight,
            cycle_support,
            normalized_cycle_support,
            bridge_likeness,
            local_mass,
            mass_factor,
            geo_affinity,
            barrier_penalty,
            cohesion_weight,
        });
    }

    Ok(terms)
}

fn geography_term(
    values: &HashMap<(usize, usize), f64>,
    term: &'static str,
    u: usize,
    v: usize,
) -> Result<f64, CohesionError> {
    let (a, b) = canonical_edge(u, v);
    let value = values
        .get(&(a, b))
        .or_else(|| values.get(&(b, a)))
        .copied()
        .unwrap_or(0.0);
    if !value.is_finite() || value < 0.0 {
        return Err(CohesionError::InvalidGeographyTerm {
            term,
            u: a,
            v: b,
            value,
        });
    }
    Ok(value)
}

fn validate_params(params: CohesionParams) -> Result<(), CohesionError> {
    for (name, value) in [
        ("alpha_cycle", params.alpha_cycle),
        ("alpha_module", params.alpha_module),
        ("alpha_geo", params.alpha_geo),
        ("alpha_bridge", params.alpha_bridge),
        ("alpha_barrier", params.alpha_barrier),
        ("min_mass", params.min_mass),
        ("max_mass", params.max_mass),
    ] {
        if !value.is_finite() {
            return Err(CohesionError::NonFiniteParameter { name, value });
        }
    }
    if params.min_mass > params.max_mass {
        return Err(CohesionError::InvalidMassClamp {
            min_mass: params.min_mass,
            max_mass: params.max_mass,
        });
    }
    Ok(())
}

fn validate_populations(graph: &Graph) -> Result<(), CohesionError> {
    for (vertex, &population) in graph.vertex_weights.iter().enumerate() {
        if population < 0 {
            return Err(CohesionError::NegativePopulation { vertex, population });
        }
    }
    Ok(())
}

fn canonical_edges(graph: &Graph) -> Result<Vec<(usize, usize)>, CohesionError> {
    let mut edges = BTreeSet::new();
    for (u, neighbors) in graph.adjacency.iter().enumerate() {
        for &v in neighbors {
            if v >= graph.n_vertices {
                return Err(CohesionError::InvalidNeighbor { vertex: v });
            }
            if u != v {
                edges.insert(canonical_edge(u, v));
            }
        }
    }
    Ok(edges.into_iter().collect())
}

fn canonical_edge(u: usize, v: usize) -> (usize, usize) {
    if u < v {
        (u, v)
    } else {
        (v, u)
    }
}

fn boundary_weight(
    boundary_weights: &HashMap<(usize, usize), f64>,
    u: usize,
    v: usize,
) -> Result<f64, CohesionError> {
    let (a, b) = canonical_edge(u, v);
    let weight = boundary_weights.get(&(a, b)).copied().unwrap_or(1.0);
    if !weight.is_finite() {
        return Err(CohesionError::NonFiniteBoundaryWeight { u: a, v: b, weight });
    }
    if weight < 0.0 {
        return Err(CohesionError::NegativeBoundaryWeight { u: a, v: b, weight });
    }
    Ok(weight)
}

fn alternate_cycle_support(
    graph: &Graph,
    u: usize,
    v: usize,
    max_depth: usize,
) -> Result<f64, CohesionError> {
    match bounded_alternate_path_len(graph, u, v, max_depth)? {
        Some(path_len) => Ok(1.0 / (path_len as f64 + 1.0)),
        None => Ok(0.0),
    }
}

fn bounded_alternate_path_len(
    graph: &Graph,
    start: usize,
    target: usize,
    max_depth: usize,
) -> Result<Option<usize>, CohesionError> {
    if max_depth == 0 {
        return Ok(None);
    }

    let mut seen = vec![false; graph.n_vertices];
    let mut queue = VecDeque::new();
    seen[start] = true;
    queue.push_back((start, 0usize));

    while let Some((node, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        for &next in &graph.adjacency[node] {
            if next >= graph.n_vertices {
                return Err(CohesionError::InvalidNeighbor { vertex: next });
            }
            if canonical_edge(node, next) == canonical_edge(start, target) {
                continue;
            }
            let next_depth = depth + 1;
            if next == target {
                return Ok(Some(next_depth));
            }
            if !seen[next] {
                seen[next] = true;
                queue.push_back((next, next_depth));
            }
        }
    }

    Ok(None)
}

fn common_neighborhood_mass(graph: &Graph, u: usize, v: usize) -> Result<f64, CohesionError> {
    let mut vertices = BTreeSet::new();
    vertices.insert(u);
    vertices.insert(v);

    let u_neighbors: BTreeSet<usize> = graph.adjacency[u].iter().copied().collect();
    for &neighbor in &graph.adjacency[v] {
        if neighbor >= graph.n_vertices {
            return Err(CohesionError::InvalidNeighbor { vertex: neighbor });
        }
        if u_neighbors.contains(&neighbor) {
            vertices.insert(neighbor);
        }
    }

    let mut mass = 0.0;
    for vertex in vertices {
        let population = graph.vertex_weights[vertex];
        if population < 0 {
            return Err(CohesionError::NegativePopulation { vertex, population });
        }
        mass += population as f64;
    }
    Ok(mass)
}

fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        values[mid]
    } else {
        (values[mid - 1] + values[mid]) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_boundary_weights() -> HashMap<(usize, usize), f64> {
        HashMap::new()
    }

    fn square_mesh() -> Graph {
        Graph::new(
            vec![
                vec![1, 3],
                vec![0, 2, 4],
                vec![1, 5],
                vec![0, 4, 6],
                vec![1, 3, 5, 7],
                vec![2, 4, 8],
                vec![3, 7],
                vec![4, 6, 8],
                vec![5, 7],
            ],
            vec![100; 9],
        )
        .unwrap()
    }

    fn two_meshes_with_bridge() -> Graph {
        Graph::new(
            vec![
                vec![1, 2],
                vec![0, 2],
                vec![0, 1, 3],
                vec![2, 4, 5],
                vec![3, 5],
                vec![3, 4],
            ],
            vec![100; 6],
        )
        .unwrap()
    }

    fn permute_graph(graph: &Graph, old_to_new: &[usize]) -> Graph {
        let mut adjacency = vec![Vec::new(); graph.n_vertices];
        let mut vertex_weights = vec![0; graph.n_vertices];

        for old in 0..graph.n_vertices {
            let new = old_to_new[old];
            vertex_weights[new] = graph.vertex_weights[old];
            adjacency[new] = graph.adjacency[old]
                .iter()
                .copied()
                .map(|neighbor| old_to_new[neighbor])
                .collect();
            adjacency[new].sort_unstable();
        }

        Graph::new(adjacency, vertex_weights).unwrap()
    }

    fn weighted_edges(graph: &Graph) -> HashMap<(usize, usize), f64> {
        canonical_edges(graph)
            .unwrap()
            .into_iter()
            .map(|(u, v)| ((u, v), 1.0 + ((u + v) as f64 / 10.0)))
            .collect()
    }

    fn permute_edge_weights(
        weights: &HashMap<(usize, usize), f64>,
        old_to_new: &[usize],
    ) -> HashMap<(usize, usize), f64> {
        weights
            .iter()
            .map(|(&(u, v), &weight)| (canonical_edge(old_to_new[u], old_to_new[v]), weight))
            .collect()
    }

    fn assert_close(left: f64, right: f64) {
        assert!(
            (left - right).abs() < 1e-9,
            "expected {left} and {right} to match"
        );
    }

    #[test]
    fn cohesion_cycle_support_square_mesh_has_supported_edges() {
        let terms = cohesion_edge_terms(
            &square_mesh(),
            &no_boundary_weights(),
            CohesionParams::default(),
        )
        .unwrap();

        assert!(terms.iter().any(|term| term.cycle_support > 0.0));
        assert!(terms.iter().all(|term| term.cohesion_weight.is_finite()));
    }

    #[test]
    fn cohesion_bridge_edge_has_zero_cycle_support() {
        let terms = cohesion_edge_terms(
            &two_meshes_with_bridge(),
            &no_boundary_weights(),
            CohesionParams::default(),
        )
        .unwrap();

        let bridge = terms
            .iter()
            .find(|term| (term.u, term.v) == (2, 3))
            .expect("bridge edge terms");
        assert_eq!(bridge.cycle_support, 0.0);
        assert_eq!(bridge.bridge_likeness, 1.0);
    }

    #[test]
    fn cohesion_mass_factor_is_clamped() {
        let graph = Graph::new(vec![vec![1], vec![0, 2], vec![1]], vec![1, 1_000_000, 1]).unwrap();
        let params = CohesionParams {
            min_mass: 0.75,
            max_mass: 1.25,
            ..CohesionParams::default()
        };

        let terms = cohesion_edge_terms(&graph, &no_boundary_weights(), params).unwrap();

        assert!(terms
            .iter()
            .all(|term| term.mass_factor >= 0.75 && term.mass_factor <= 1.25));
    }

    #[test]
    fn cohesion_weight_is_symmetric_over_reversed_adjacency() {
        let graph = Graph::new(vec![vec![1], vec![0]], vec![100, 200]).unwrap();
        let mut weights = HashMap::new();
        weights.insert((0, 1), 3.0);
        weights.insert((1, 0), 99.0);

        let terms = cohesion_edge_terms(&graph, &weights, CohesionParams::default()).unwrap();

        assert_eq!(terms.len(), 1);
        assert_eq!((terms[0].u, terms[0].v), (0, 1));
        assert!(terms[0].cohesion_weight.is_finite());
        assert!(terms[0].cohesion_weight < 3.0);
    }

    #[test]
    fn cohesion_terms_are_invariant_under_vertex_relabeling() {
        let graph = square_mesh();
        let boundary_weights = weighted_edges(&graph);
        let params = CohesionParams::default();
        let original_terms = cohesion_edge_terms(&graph, &boundary_weights, params).unwrap();

        let old_to_new = vec![4, 2, 7, 0, 8, 1, 6, 3, 5];
        let mut new_to_old = vec![0usize; old_to_new.len()];
        for (old, &new) in old_to_new.iter().enumerate() {
            new_to_old[new] = old;
        }

        let relabeled_graph = permute_graph(&graph, &old_to_new);
        let relabeled_boundary_weights = permute_edge_weights(&boundary_weights, &old_to_new);
        let relabeled_terms =
            cohesion_edge_terms(&relabeled_graph, &relabeled_boundary_weights, params).unwrap();
        let relabeled_by_original_edge: HashMap<(usize, usize), CohesionEdgeTerms> =
            relabeled_terms
                .into_iter()
                .map(|term| (canonical_edge(new_to_old[term.u], new_to_old[term.v]), term))
                .collect();

        for original in original_terms {
            let relabeled = relabeled_by_original_edge
                .get(&(original.u, original.v))
                .expect("matching relabeled edge terms");
            assert_close(original.boundary_weight, relabeled.boundary_weight);
            assert_close(original.cycle_support, relabeled.cycle_support);
            assert_close(
                original.normalized_cycle_support,
                relabeled.normalized_cycle_support,
            );
            assert_close(original.bridge_likeness, relabeled.bridge_likeness);
            assert_close(original.local_mass, relabeled.local_mass);
            assert_close(original.mass_factor, relabeled.mass_factor);
            assert_close(original.cohesion_weight, relabeled.cohesion_weight);
        }
    }

    #[test]
    fn cohesion_default_physical_geography_terms_are_disabled() {
        let terms = cohesion_edge_terms(
            &square_mesh(),
            &no_boundary_weights(),
            CohesionParams::default(),
        )
        .unwrap();

        assert!(terms.iter().all(|term| term.geo_affinity == 0.0));
        assert!(terms.iter().all(|term| term.barrier_penalty == 0.0));
    }

    #[test]
    fn cohesion_rejects_non_finite_geography_parameters() {
        let params = CohesionParams {
            alpha_geo: f64::NAN,
            ..CohesionParams::default()
        };

        match cohesion_edge_terms(&square_mesh(), &no_boundary_weights(), params) {
            Err(CohesionError::NonFiniteParameter { name, value }) => {
                assert_eq!(name, "alpha_geo");
                assert!(value.is_nan());
            }
            other => panic!("expected alpha_geo non-finite parameter error, got {other:?}"),
        }
    }

    #[test]
    fn cohesion_geography_affinity_boosts_declared_corridor_edge() {
        let graph = Graph::new(vec![vec![1], vec![0, 2], vec![1]], vec![100; 3]).unwrap();
        let mut geography = CohesionGeography::default();
        geography.geo_affinity.insert((0, 1), 1.0);
        let params = CohesionParams {
            alpha_cycle: 0.0,
            alpha_bridge: 0.0,
            alpha_geo: 0.50,
            ..CohesionParams::default()
        };

        let terms =
            cohesion_edge_terms_with_geography(&graph, &no_boundary_weights(), &geography, params)
                .unwrap();

        let corridor = terms
            .iter()
            .find(|term| (term.u, term.v) == (0, 1))
            .unwrap();
        let plain = terms
            .iter()
            .find(|term| (term.u, term.v) == (1, 2))
            .unwrap();
        assert_eq!(corridor.geo_affinity, 1.0);
        assert!(corridor.cohesion_weight > plain.cohesion_weight);
    }

    #[test]
    fn cohesion_barrier_penalty_lowers_declared_barrier_edge() {
        let graph = Graph::new(vec![vec![1], vec![0, 2], vec![1]], vec![100; 3]).unwrap();
        let mut geography = CohesionGeography::default();
        geography.barrier_penalty.insert((1, 2), 1.0);
        let params = CohesionParams {
            alpha_cycle: 0.0,
            alpha_bridge: 0.0,
            alpha_barrier: 0.50,
            ..CohesionParams::default()
        };

        let terms =
            cohesion_edge_terms_with_geography(&graph, &no_boundary_weights(), &geography, params)
                .unwrap();

        let plain = terms
            .iter()
            .find(|term| (term.u, term.v) == (0, 1))
            .unwrap();
        let barrier = terms
            .iter()
            .find(|term| (term.u, term.v) == (1, 2))
            .unwrap();
        assert_eq!(barrier.barrier_penalty, 1.0);
        assert!(barrier.cohesion_weight < plain.cohesion_weight);
    }

    #[test]
    fn cohesion_geography_terms_accept_reversed_edge_keys() {
        let graph = Graph::new(vec![vec![1], vec![0]], vec![100, 100]).unwrap();
        let mut geography = CohesionGeography::default();
        geography.geo_affinity.insert((1, 0), 1.0);

        let terms = cohesion_edge_terms_with_geography(
            &graph,
            &no_boundary_weights(),
            &geography,
            CohesionParams {
                alpha_geo: 1.0,
                alpha_bridge: 0.0,
                ..CohesionParams::default()
            },
        )
        .unwrap();

        assert_eq!(terms[0].geo_affinity, 1.0);
    }

    #[test]
    fn cohesion_rejects_invalid_geography_terms() {
        let graph = Graph::new(vec![vec![1], vec![0]], vec![100, 100]).unwrap();
        let mut geography = CohesionGeography::default();
        geography.barrier_penalty.insert((0, 1), -0.1);

        assert_eq!(
            cohesion_edge_terms_with_geography(
                &graph,
                &no_boundary_weights(),
                &geography,
                CohesionParams::default()
            ),
            Err(CohesionError::InvalidGeographyTerm {
                term: "barrier_penalty",
                u: 0,
                v: 1,
                value: -0.1
            })
        );
    }

    #[test]
    fn cohesion_negative_population_is_rejected() {
        let graph = Graph::new(vec![vec![1], vec![0]], vec![100, -1]).unwrap();

        assert_eq!(
            cohesion_edge_terms(&graph, &no_boundary_weights(), CohesionParams::default()),
            Err(CohesionError::NegativePopulation {
                vertex: 1,
                population: -1
            })
        );
    }
}
