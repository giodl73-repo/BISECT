---
pulse: 04-synthetic-integration-checks
wave: track-y-bio-symmetry-spatial-factorization
status: done
date: 2026-06-06
validation_level: L1
---

# Pulse 04 -- Synthetic Integration Checks

## Goal

Run cohesion weights through the ordinary compositor on small synthetic graphs
and compare against `geographic` and `unweighted` baselines.

## Planned Deliverables

- Synthetic square mesh run.
- Synthetic two-mesh bridge run.
- Synthetic dense-core/sparse-ring run.
- Symmetric relabeling run where canonical plan comparison is available.
- Optional synthetic river/valley/elevation-band run for a future
  geography-aware cohesion variant.
- Baseline comparison report for:
  - population deviation,
  - edge cut,
  - selected cut cycle support,
  - selected cut bridge-likeness,
  - retained internal cycle support,
  - default-mode independence from physical-geography fixture fields,
  - canonical plan hash stability.

## Progress

- Added runner-level synthetic Layer 2 check:
  `build_edge_weights_cohesion_prefers_cycle_edge_over_bridge_edge`.
  This verifies the ordinary `build_edge_weights` path makes a
  cycle-supported edge costlier than a bridge-like edge on a synthetic graph.
- Added sidecar summary check:
  `write_cohesion_sidecar_emits_v1_summary`.
- Added dense-core/sparse-peer compositor check:
  `cohesion_weighter_boosts_dense_core_over_sparse_peer`. This verifies
  matched local meshes receive different cohesion factors when their
  common-neighborhood population mass differs.
- Added core relabeling check:
  `cohesion_terms_are_invariant_under_vertex_relabeling`. This verifies the
  local terms and final cohesion weights are unchanged after an isomorphic
  vertex renumbering is mapped back to the original edge labels.
- Added synthetic baseline comparison report:
  `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/synthetic-baseline-comparison.md`.
  This records expected and observed fixture-level differences between
  `unweighted`, `geographic`, and `cohesion` without making California,
  legality, or fairness claims.

## Required Findings

- Cohesion weights must preserve finite symmetric edge weights.
- Bridge-like cuts should be preferred only when population balance permits.
- Dense mesh edges should become harder to cut than sparse mesh edges.
- Default `cohesion` should not change when synthetic terrain/river fields are
  present but disabled.
- Geography-aware variants, when added later, should strengthen same-corridor
  edges and weaken declared barrier crossings.
- Relabeling should not change canonical output.

## Validation Target

```powershell
cargo fmt
cargo test -p bisect-core cohesion -- --test-threads=1
cargo test -p bisect-cli cohesion -- --test-threads=1
git diff --check
```

Current focused CLI test coverage includes:

- `edge_weights::tests::cohesion_weighter_boosts_cycle_edge_over_bridge_edge`
- `edge_weights::tests::cohesion_weighter_accepts_declared_geography_terms`
- `edge_weights::tests::cohesion_weighter_boosts_dense_core_over_sparse_peer`
- `cohesion::tests::cohesion_terms_are_invariant_under_vertex_relabeling`
- `runner::tests::weights_override_cohesion_enables_cohesion_signal`
- `runner::tests::build_edge_weights_cohesion_prefers_cycle_edge_over_bridge_edge`
- `runner::tests::write_cohesion_sidecar_emits_v1_summary`

## Outcome

Pulse 04 is complete for L1 synthetic evidence. The remaining Track Y work now
belongs to Pulse 05: a small California subset comparison against geographic
weights, with no statewide claims.
