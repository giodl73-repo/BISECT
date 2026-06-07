---
pulse: 02-cohesion-term-l0-fixtures
wave: track-y-bio-symmetry-spatial-factorization
status: done
date: 2026-06-06
validation_level: L0
---

# Pulse 02 -- Cohesion Term L0 Fixtures

## Goal

Implement the BISECT-local core cohesion terms without exposing a CLI mode yet.

## Deliverables

- Added `bisect_core::cohesion`.
- Added `CohesionParams`.
- Added `CohesionEdgeTerms`.
- Added bounded BFS alternate-path cycle support.
- Added bridge-likeness from normalized cycle support.
- Added common-neighborhood population mass.
- Added clamped log mass factor.
- Reserved disabled physical-geography slots:
  - `alpha_geo`;
  - `alpha_barrier`;
  - `geo_affinity`;
  - `barrier_penalty`.
- Added explicit `CohesionGeography` input and
  `cohesion_edge_terms_with_geography` for synthetic river/terrain fixtures.
- Added finite, symmetric cohesion edge weights.

## Fixture Targets

- square mesh;
- two meshes with bridge;
- dense core and sparse ring;
- symmetric relabeling fixture where applicable.

## Validation

```powershell
cargo fmt
cargo test -p bisect-core cohesion -- --test-threads=1
git diff --check
```

Result:

- `cargo fmt`: pass.
- `cargo test -p bisect-core cohesion -- --test-threads=1`: pass, 11 tests.
- `git diff --check`: run at wave level after this pulse.

Focused tests:

- `cohesion_cycle_support_square_mesh_has_supported_edges`
- `cohesion_bridge_edge_has_zero_cycle_support`
- `cohesion_mass_factor_is_clamped`
- `cohesion_weight_is_symmetric_over_reversed_adjacency`
- `cohesion_default_physical_geography_terms_are_disabled`
- `cohesion_rejects_non_finite_geography_parameters`
- `cohesion_geography_affinity_boosts_declared_corridor_edge`
- `cohesion_barrier_penalty_lowers_declared_barrier_edge`
- `cohesion_geography_terms_accept_reversed_edge_keys`
- `cohesion_rejects_invalid_geography_terms`
- `cohesion_negative_population_is_rejected`

## Status

Done.
