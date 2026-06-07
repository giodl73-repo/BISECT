# Track Y -- Bio-Symmetry Spatial Factorization

Track Y explores whether BISECT can move beyond raw recursive cuts toward
cohesion-weighted, symmetry-aware spatial factorization of tract adjacency
graphs.

The working intuition is that real spatial systems often organize by local
cohesion before they are partitioned. Spider webs, slime-mold transport
networks, roots, veins, rivers, and transit-like networks reinforce useful
connections, decay weak ones, preserve redundancy where it matters, and avoid
waste. Dense activity creates thicker connective tissue; sparse areas create
lighter connective tissue. BISECT can borrow this as a graph-weighting and
validation idea without claiming a biological model of voters or communities.

## Thesis

Redistricting can be framed as:

```text
state = D1 * D2 * ... * Dk
```

where each district is a connected spatial factor and the product is a
disjoint, population-balanced composition of the whole state. The algorithmic
task is to discover strong local cohesion first, then cut weak connective tissue
between spatial factors.

Group theory enters as the canonicalization and invariance layer:

- equivalent district labels should collapse to one canonical plan;
- left/right bisection child swaps should collapse to one canonical tree;
- local graph symmetries and near-symmetries should inform tie-breaking and
  fixture design;
- fairness claims should be tested as invariance or equivariance properties.

## Relationship To Existing Tracks

- Track B owns algorithm foundations and existing edge-weighted bisection.
- Track T owns plan-construction methods.
- Track U owns search and optimization over candidate plans.
- Track Y owns the new research question: whether biological network formation,
  local graph cycles, approximate symmetry, and spatial factorization can define
  better edge/vertex weighting and fairness-invariance harnesses.

## Compositor Placement

Track Y should layer into the existing three-layer compositor conservatively:

| Layer | Track Y role | Candidate surface |
|---|---|---|
| Layer 1 -- Structure | Only after evidence shows cohesion should change the bisection tree itself. | Future `--structure cohesion-factor` or `bio-factor` if needed. |
| Layer 2 -- Weights | Primary entry point: adjust edge weights before the existing partitioner runs. | `--weights-override cohesion`, `cycle-cohesion`, or `bio-cohesion`. |
| Layer 3 -- Search | Canonicalization, tie diagnostics, invariance checks, and optional seed/ensemble comparison. | `--search` sidecar or validation mode, not first implementation. |

The first buildable integration should be Layer 2:

```text
graph + geometry + population
-> cohesion edge weighter
-> existing structure mode
-> existing search mode
-> canonical/invariance evidence sidecar
```

This keeps Y.1 comparable to `geographic`, `county`, `vra-aligned`, and other
weight-layer methods.

## Candidate Papers

| Code | Working title | Status |
|------|---------------|--------|
| Y.0 | Bio-Symmetry Spatial Factorization Overview | planned; [plan](Y.0+bio-symmetry-spatial-factorization-overview/plan.md) |
| Y.1 | Cohesion-Weighted Bisection | planned; [plan](Y.1+cohesion-weighted-bisection/plan.md) |
| Y.2 | Local Symmetry And Tract Orbits | planned; [plan](Y.2+local-symmetry-and-tract-orbits/plan.md) |
| Y.3 | Cycle-Supported Cut Weighting | planned; [plan](Y.3+cycle-supported-cut-weighting/plan.md) |
| Y.4 | Biological Network Analogues For Spatial Districting | planned; [plan](Y.4+biological-network-analogues/plan.md) |
| Y.5 | Fairness Invariance Harness For Redistricting Modes | planned; [plan](Y.5+fairness-invariance-harness/plan.md) |

## Implementation Specs

- [`docs/specs/2026-06-06-y1-cohesion-weighted-bisection.md`](../../../docs/specs/2026-06-06-y1-cohesion-weighted-bisection.md)

## Non-Goals

- Do not claim census tracts literally form exact mathematical groups.
- Do not claim biological behavior proves legal or civic fairness.
- Do not replace population balance, contiguity, or mode-specific legal
  controls.
- Do not let approximate symmetry override VRA, statutory, or evidence
  boundaries.
- Do not move BISECT-specific fairness policy into RLINE.

## First Buildable Experiment

The first implementable slice should be a California or synthetic-grid
experiment:

1. build the tract adjacency graph;
2. compute edge cycle support, bridge-likeness, local module membership, and
   shared-boundary strength;
3. compute population mass or density around local edges and modules;
4. define a cohesion-weighted edge score where topology finds the mesh,
   geometry measures physical connection, population supplies flow/mass, and
   district count supplies factorization pressure;
5. run existing bisection over the adjusted weights;
6. compare compactness, split counts, population balance, canonical plan hash,
   and mode-specific fairness invariance checks.

This track remains research-only until a small fixture proves that the weighting
signal changes behavior in a useful and auditable way.
