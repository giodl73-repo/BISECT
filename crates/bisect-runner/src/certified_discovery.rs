//! Shared native discovery incumbent and refinement computation (no filesystem or threads).
use anyhow::{anyhow,Context};
use std::collections::{BTreeMap,BinaryHeap,HashMap,HashSet,VecDeque};
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum DiscoveryRefinementArg {
    /// Emit the raw METIS candidate after exact validation.
    Metis,
    /// Apply articulation-safe population repair only.
    Population,
    /// NRS v0.1 repair order with the profile-mandated 100 METIS iterations.
    #[value(name = "nrs-v0-1")]
    #[serde(rename="nrs-v0-1")]
    NrsV01,
    /// NRS v0.2 with canonical multi-root fallback after a v0.1 tolerance miss.
    #[value(name = "nrs-v0-2")]
    #[serde(rename="nrs-v0-2")]
    NrsV02,
    /// NRS v0.3 with bridge-aware land-component fallback after a v0.2 miss.
    #[value(name = "nrs-v0-3")]
    #[serde(rename="nrs-v0-3")]
    NrsV03,
    /// Apply population repair and bounded 0-, 1-, and 1-to-2 boundary moves.
    Fast,
    /// Apply all deterministic neighborhoods, including bounded 2-to-2 search.
    #[default]
    Full,
}

pub fn discover(root: &bisect_ilp::CertifiedSplitInstance, adjacency: &[Vec<usize>], land_adjacency: &[Vec<usize>], seed:u64, refinement:DiscoveryRefinementArg) -> anyhow::Result<bisect_ilp::CertifiedSplitDiscovery> {
    root.validate()?;
    if adjacency.len()!=root.unit_ids.len() || land_adjacency.len()!=adjacency.len() || adjacency.iter().chain(land_adjacency).flatten().any(|&to|to>=adjacency.len()) { return Err(anyhow!("Discovery adjacency does not match the unit universe")); }
    let populations=&root.populations;
    let edge_weights = root
        .edges
        .iter()
        .map(|edge| ((edge.left, edge.right), edge.weight as f64))
        .collect::<HashMap<_, _>>();
    let units = (0..root.unit_ids.len()).collect::<HashSet<_>>();
    let niter = if matches!(
        refinement,
        DiscoveryRefinementArg::NrsV01
            | DiscoveryRefinementArg::NrsV02
            | DiscoveryRefinementArg::NrsV03
    ) {
        100
    } else {
        10
    };
    let nrs_profile = matches!(
        refinement,
        DiscoveryRefinementArg::NrsV01
            | DiscoveryRefinementArg::NrsV02
            | DiscoveryRefinementArg::NrsV03
    );
    let mut assignment = if nrs_profile {
        let tpwgts = Some(vec![
            root.k_left as f32 / root.k_parent as f32,
            root.k_right as f32 / root.k_parent as f32,
        ]);
        let (left, _right) = crate::bisection_runner::split_subgraph_nrs_v0_1(
            &adjacency,
            populations,
            1,
            &edge_weights,
            &units,
            1.005,
            niter,
            Some(seed),
            tpwgts,
            None,
        )
        .map_err(|error| anyhow!("certified discovery METIS split failed: {error}"))?;
        (0..root.unit_ids.len())
            .map(|unit| if left.contains(&unit) { 0_u8 } else { 1_u8 })
            .collect::<Vec<_>>()
    } else if root.k_left == root.k_right {
        let partition = crate::bisection_runner::run_nway_partition(
            &adjacency,
            populations,
            &edge_weights,
            2,
            1.005,
            niter,
            Some(seed),
        )
        .map_err(|error| anyhow!("certified discovery METIS split failed: {error}"))?;
        (0..root.unit_ids.len())
            .map(|unit| if partition[&unit] == 1 { 0_u8 } else { 1_u8 })
            .collect::<Vec<_>>()
    } else {
        let tpwgts = Some(vec![
            root.k_left as f32 / root.k_parent as f32,
            root.k_right as f32 / root.k_parent as f32,
        ]);
        let (left, _right) = crate::bisection_runner::split_subgraph(
            &adjacency,
            populations,
            1,
            &edge_weights,
            &units,
            1.005,
            niter,
            Some(seed),
            tpwgts,
            None,
        )
        .map_err(|error| anyhow!("certified discovery METIS split failed: {error}"))?;
        (0..root.unit_ids.len())
            .map(|unit| if left.contains(&unit) { 0_u8 } else { 1_u8 })
            .collect::<Vec<_>>()
    };
    let original_assignment = assignment.clone();
    let mut initial_dfs_minimum_deviation_candidates = 0;
    let mut initial_dfs_minimum_deviation_cut_candidates = 0;
    let mut initial_dfs_minimum_deviation_cut_partitions = 0;
    if nrs_profile {
        let candidate = nrs_dfs_tree_cut_candidate(&root, &adjacency, &assignment, 0)?;
        initial_dfs_minimum_deviation_candidates = candidate.minimum_deviation_candidate_count;
        initial_dfs_minimum_deviation_cut_candidates =
            candidate.minimum_deviation_cut_candidate_count;
        initial_dfs_minimum_deviation_cut_partitions =
            candidate.minimum_deviation_cut_partition_count;
        assignment = candidate.assignment;
    } else {
        let left = assignment
            .iter()
            .enumerate()
            .filter_map(|(unit, &label)| (label == 0).then_some(unit))
            .collect::<HashSet<_>>();
        let right = assignment
            .iter()
            .enumerate()
            .filter_map(|(unit, &label)| (label == 1).then_some(unit))
            .collect::<HashSet<_>>();
        let (left, _) =
            crate::bisection_runner::repair_bisection_contiguity(&adjacency, left, right);
        for (unit, label) in assignment.iter_mut().enumerate() {
            *label = if left.contains(&unit) { 0 } else { 1 };
        }
    }
    let contiguity_repair_moves = assignment
        .iter()
        .zip(&original_assignment)
        .filter(|(after, before)| after != before)
        .count();
    if root.k_left == root.k_right && assignment[0] == 1 {
        for label in &mut assignment {
            *label = 1 - *label;
        }
    }
    let mut population_improvement_operations = 0;
    let mut population_improvement_units = 0;
    let mut nrs_v0_2_fallback_activated = false;
    let mut nrs_v0_3_fallback_activated = false;
    let mut nrs_v0_2_fallback_diagnostics = NrsFallbackDiagnostics::default();
    let mut nrs_v0_3_fallback_diagnostics = NrsFallbackDiagnostics::default();
    let mut zero_population_cut_moves = 0;
    let mut same_population_swap_moves = 0;
    let mut one_to_two_swap_moves = 0;
    let mut two_to_two_swap_moves = 0;
    if refinement != DiscoveryRefinementArg::Metis {
        (
            population_improvement_operations,
            population_improvement_units,
        ) = improve_discovery_population(
            &root,
            &adjacency,
            &mut assignment,
            nrs_profile.then(|| nrs_population_tolerance_scaled_bound(&root)),
        )?;
        if matches!(
            refinement,
            DiscoveryRefinementArg::NrsV02 | DiscoveryRefinementArg::NrsV03
        ) && nrs_population_deviation_scaled(&root, &assignment)
            > nrs_population_tolerance_scaled_bound(&root)
        {
            nrs_v0_2_fallback_activated = true;
            let fallback = nrs_v0_2_fallback_candidate(
                &root,
                &adjacency,
                &original_assignment,
                &assignment,
                population_improvement_operations,
                population_improvement_units,
            )?;
            assignment = fallback.assignment;
            population_improvement_operations = fallback.operations;
            population_improvement_units = fallback.moved_units;
            nrs_v0_2_fallback_diagnostics = fallback.diagnostics;
        }
        if refinement == DiscoveryRefinementArg::NrsV03
            && nrs_population_deviation_scaled(&root, &assignment)
                > nrs_population_tolerance_scaled_bound(&root)
        {
            nrs_v0_3_fallback_activated = true;
            let fallback = nrs_v0_3_bridge_aware_component_candidate(
                &root,
                &adjacency,
                &land_adjacency,
                &original_assignment,
                &assignment,
                population_improvement_operations,
                population_improvement_units,
            )?;
            assignment = fallback.assignment;
            population_improvement_operations = fallback.operations;
            population_improvement_units = fallback.moved_units;
            nrs_v0_3_fallback_diagnostics = fallback.diagnostics;
        }
    }
    if matches!(
        refinement,
        DiscoveryRefinementArg::Fast | DiscoveryRefinementArg::Full
    ) {
        zero_population_cut_moves =
            improve_zero_population_boundary(&root, &adjacency, &mut assignment)?;
        same_population_swap_moves =
            improve_same_population_swaps(&root, &adjacency, &mut assignment)?;
        one_to_two_swap_moves =
            improve_one_to_two_population_swaps(&root, &adjacency, &mut assignment)?;
    }
    if refinement == DiscoveryRefinementArg::Full {
        two_to_two_swap_moves =
            improve_two_to_two_population_swaps(&root, &adjacency, &mut assignment)?;
    }
    let refinement_suffix = if refinement == DiscoveryRefinementArg::Full {
        String::new()
    } else {
        format!("; refinement={:?}", refinement).to_lowercase()
    };
    let partition_type = if nrs_profile { "recursive" } else { "kway" };
    let discovery = bisect_ilp::certified_split_discovery(
        &root,
        "METIS",
        Some(crate::bisection_runner::detect_gpmetis_version()),
        format!(
            "standard-bisect-discovery; seed={}; niter={}; ufactor=1.005; partition-type={}; zero-population-vertex-floor=1; metis-edge-scaling=heuristic; candidate-initialization=minimum-geoid-rooted-sorted-dfs-tree-edge-cut; initial-dfs-minimum-deviation-candidates={}; initial-dfs-minimum-deviation-cut-candidates={}; initial-dfs-minimum-deviation-cut-partitions={}; nrs-v0-2-fallback-activated={}; nrs-v0-2-fallback-evaluated-candidates={}; nrs-v0-2-fallback-minimum-deviation-candidates={}; nrs-v0-2-fallback-minimum-deviation-cut-candidates={}; nrs-v0-2-fallback-minimum-deviation-cut-partitions={}; nrs-v0-3-fallback-activated={}; nrs-v0-3-fallback-evaluated-candidates={}; nrs-v0-3-fallback-minimum-deviation-candidates={}; nrs-v0-3-fallback-minimum-deviation-cut-candidates={}; nrs-v0-3-fallback-minimum-deviation-cut-partitions={}; candidate-initialization-moves={}; connected-subtree-population-operations={}; connected-subtree-population-units={}; zero-population-cut-moves={}; same-population-swap-moves={}; one-to-two-swap-moves={}; two-to-two-swap-moves={}; certified-objective=raw-u64{}",
            seed,
            niter,
            partition_type,
            initial_dfs_minimum_deviation_candidates,
            initial_dfs_minimum_deviation_cut_candidates,
            initial_dfs_minimum_deviation_cut_partitions,
            nrs_v0_2_fallback_activated,
            nrs_v0_2_fallback_diagnostics.evaluated_candidate_count,
            nrs_v0_2_fallback_diagnostics.minimum_deviation_candidate_count,
            nrs_v0_2_fallback_diagnostics.minimum_deviation_cut_candidate_count,
            nrs_v0_2_fallback_diagnostics.minimum_deviation_cut_partition_count,
            nrs_v0_3_fallback_activated,
            nrs_v0_3_fallback_diagnostics.evaluated_candidate_count,
            nrs_v0_3_fallback_diagnostics.minimum_deviation_candidate_count,
            nrs_v0_3_fallback_diagnostics.minimum_deviation_cut_candidate_count,
            nrs_v0_3_fallback_diagnostics.minimum_deviation_cut_partition_count,
            contiguity_repair_moves,
            population_improvement_operations,
            population_improvement_units,
            zero_population_cut_moves,
            same_population_swap_moves,
            one_to_two_swap_moves,
            two_to_two_swap_moves,
            refinement_suffix,
        ),
        assignment,
    )?;
    Ok(discovery)
}

include!("certified_discovery_helpers.rs");
