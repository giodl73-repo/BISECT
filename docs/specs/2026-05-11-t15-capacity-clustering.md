# Spec: T.15 Capacity-Constrained Clustering

**Date:** 2026-05-11  
**Status:** Draft, implementation staging  
**Track:** T.15 - Plan Construction  
**Depends on:** U.20 RPLAN audit integration fixed point

## Goal

Add a deterministic, auditable construction family that builds districts by
placing geographically spread seeds, assigning units under hard population
capacity limits, repairing contiguity, and certifying the final plan through
`rplan-audit`.

This is a construction baseline, not a claim of legal or political neutrality.
Outputs must state the objective proxy used, the repair path, and the legal
profile applied by the audit certificate.

## Rust Surface

Create a new crate:

- `bisect-clustering`

Initial modules:

- `fixtures.rs`: canonical synthetic graph fixtures shared by T.15/T.16 tests.
- `seeds.rs`: deterministic farthest-point seed placement.
- `assign.rs`: capacity-aware graph-distance assignment.
- `repair.rs`: contiguity and population repair status types.
- `metrics.rs`: edge cut, population deviation, and cluster compactness proxy.
- `output.rs`: versioned run summary and audit-lineage payload.

The first implementation stage should keep the crate independent of
`bisect-cli`. CLI wiring comes only after L0/L1 crate behavior is stable.

## Algorithm Stage 1

Implement the smallest useful T.15 kernel:

1. Validate `k > 0`, `assignment.len() == n`, `weights.len() == n`, and total
   population can satisfy the requested tolerance.
2. Select `k` seeds with deterministic farthest-point traversal over unweighted
   graph distances.
3. Assign each unit to the nearest seed while respecting district capacity where
   possible.
4. Report structured status:
   - `valid`
   - `needs-repair`
   - `infeasible-capacity`
   - `invalid-input`
5. Emit a `ClusterSummary` with schema version, method, seed method, repair
   method, population deviation, edge cut, and deterministic parameter hash.

Stage 1 may return `needs-repair` for disconnected clusters. It must not label
those plans valid. Repair implementation is Stage 2.

## Audit Contract

T.15 final plans must eventually attach `algorithm_lineage` using
`rplan_audit::AlgorithmLineage::new`.

Required `algorithm_lineage.extra` fields:

- `lineage_schema_version`
- `method`
- `seed_method`
- `repair_method`
- `pre_repair_plan_hash`, when repair modifies the assignment
- `final_plan_hash`, once an RPLAN plan exists
- `capacity_status`
- `population_deviation`
- `edge_cut`
- `repair_status`

No clustering field may duplicate reserved audit-certificate fields such as
`plan_hash`, `context_hash`, or `legal_profile`.

## CLI Target

Later CLI surface:

- `--structure capacity-clustering`
- `--cluster-method kmedoids|balanced-kmeans`
- `--cluster-seeds farthest|kmeans-plus-plus|county`
- `--cluster-repair bfs|flow`
- YAML `structure: capacity-clustering`

The first CLI stage should expose only:

- `--structure capacity-clustering`
- `--cluster-seeds farthest`
- `--cluster-repair none|bfs`

## Fixtures

Canonical fixtures for L0/L1:

- `path_6_k2`: deterministic split near the midpoint.
- `grid_3x3_k3`: connected synthetic geography with equal weights.
- `two_clique_bridge_k2`: seed placement should choose opposite cliques.
- `impossible_capacity_k3`: total/weights/tolerance combination cannot satisfy
  all district capacities.
- `disconnected_assignment`: repair must detect disconnected clusters.

## Acceptance Checklist

- [x] `bisect-clustering` crate exists and compiles.
- [x] Canonical fixtures are available from the crate test helpers.
- [x] Farthest-point seeding is deterministic.
- [x] Capacity assignment returns structured infeasibility instead of panicking.
- [x] L0 tests cover path, grid, bridge, and impossible-capacity fixtures.
- [x] L1 test produces a complete plan summary on `grid_3x3_k3`.
- [x] Output summary is versioned and deterministic for fixed input.
- [x] Audit-lineage payload builder rejects reserved certificate fields.
- [x] Stage 2 repair path proves final plans pass `rplan-audit`.
- [ ] Stage 3 CLI wiring emits RPLAN sidecars and verifies through `bisect verify`.

## Stop Line For Stage 1

Stage 1 is complete when the crate can produce deterministic capacity-clustering
summaries for synthetic graphs and can honestly distinguish valid, infeasible,
and needs-repair outputs. Do not wire `bisect-cli` until that crate boundary is
covered by tests.
