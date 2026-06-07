---
track: Y.1-cohesion-weighted-bisection
date: 2026-06-06
status: synthetic-l1
scope: edge-term fixtures only
---

# Synthetic Baseline Comparison

This note records the first L1 comparison for Track Y cohesion weighting. It is
not a California run, a districting validity claim, or a fairness finding. It
compares deterministic synthetic graph fixtures that isolate the edge weighting
signals used by:

- `unweighted`
- `geographic`
- `cohesion`

## Fixture A: Mesh Plus Bridge

Shape:

```text
0--1--2
|  |  |
3--4--5--6
```

The important contrast is a cycle-supported mesh edge versus a bridge-like
edge. In the implemented tests this is represented by a square/mesh component
with a dangling bridge.

Expected behavior:

| Weight mode | Cycle-supported mesh edge | Bridge-like edge | Finding |
|---|---:|---:|---|
| `unweighted` | equal | equal | no structural preference |
| `geographic` | depends on boundary length | depends on boundary length | geometry only |
| `cohesion` | higher | lower | prefers cutting the bridge when population balance permits |

Landed checks:

- `cohesion::tests::cohesion_bridge_edge_has_zero_cycle_support`
- `edge_weights::tests::cohesion_weighter_boosts_cycle_edge_over_bridge_edge`
- `runner::tests::build_edge_weights_cohesion_prefers_cycle_edge_over_bridge_edge`

Interpretation:

The cohesion signal is doing the intended Layer 2 work. It thickens locally
cycle-supported adjacencies and thins bridge-like adjacencies without changing
the graph topology.

## Fixture B: Dense Core Versus Sparse Peer

Shape:

```text
0--1      3--4
 \ |       \ |
  2        5
```

The two triangles are topologically matched. The only intended difference is
population mass:

- triangle `0,1,2`: dense
- triangle `3,4,5`: sparse

Expected behavior:

| Weight mode | Dense triangle edge | Sparse triangle edge | Finding |
|---|---:|---:|---|
| `unweighted` | equal | equal | no mass preference |
| `geographic` | fixture-dependent | fixture-dependent | no population term |
| `cohesion` | higher | lower | local population mass thickens the dense mesh |

Landed check:

- `edge_weights::tests::cohesion_weighter_boosts_dense_core_over_sparse_peer`

Interpretation:

This is the first executable version of the bio-inspired idea: denser local
flow/mass creates thicker local connections, while still using only population
and adjacency, not party, race, county, city, or land-use fields.

## Fixture C: Vertex Relabeling

The square mesh is permuted by an arbitrary vertex renumbering and mapped back
to the original labels.

Expected behavior:

| Term | Expected under relabeling |
|---|---|
| boundary weight | unchanged after mapping back |
| cycle support | unchanged |
| bridge-likeness | unchanged |
| local mass | unchanged |
| mass factor | unchanged |
| final cohesion weight | unchanged |

Landed check:

- `cohesion::tests::cohesion_terms_are_invariant_under_vertex_relabeling`

Interpretation:

The current term generator is label-invariant for isomorphic graph inputs. That
is the minimum symmetry requirement before using these terms in canonical plan
comparison or larger fairness invariance harnesses.

## Sidecar Diagnostics

The first `cohesion.json` sidecar summarizes the run-level signal:

- schema and selected weight mode
- cohesion parameters
- edge count
- cycle-supported edge count
- zero-cycle edge count
- mass factor min, median, and max
- selected cut edge count
- selected cut low-cycle share
- selected cut average bridge-likeness
- disabled geography layers
- forbidden fields used

Landed check:

- `runner::tests::write_cohesion_sidecar_emits_v1_summary`

Interpretation:

This gives later research runs a compact way to ask whether cohesion actually
changed the selected cut structure, instead of only inspecting final district
geometry.

## Current Findings

- Cohesion weights are finite on the synthetic fixtures.
- Cycle-supported mesh edges become harder to cut than bridge-like edges.
- Dense local population mass makes matched mesh edges harder to cut than
  sparse peers.
- Default cohesion uses no physical-geography fields; river, valley,
  elevation, watershed, ridge, and barrier terms are reserved for explicit
  geography-aware variants.
- Relabeling the graph does not change the mapped-back cohesion terms.

## Current Non-Findings

- No California statewide claim.
- No legal fairness claim.
- No claim that geography-aware river, valley, or elevation terms improve a
  real plan yet.
- No claim that cohesion alone chooses better districts. The current result is
  only that the Layer 2 signal is deterministic, symmetric, and observable.

## Promotion Gate

Pulse 04 is sufficient to move to a small subset research run when:

- the selected subset has stable adjacency and population inputs,
- `cohesion.json` is emitted beside the existing manifest,
- `cohesion` is compared against `geographic` with the same seed and balance
  settings,
- any geography-aware layer remains disabled unless explicitly selected.
