# Y.1 Cohesion-Weighted Bisection

Goal: define a BISECT research path where the tract graph is first interpreted
as a local cohesion network, then partitioned by cutting weak connective tissue
rather than treating all adjacency evidence as raw bisection input.

## Motivation

Recursive bisection is transparent and reviewable, but raw graph cuts can miss
local structures that real spatial systems reveal:

- cycle-rich neighborhoods that behave as cohesive tissue;
- bridge-like connectors that naturally separate regions;
- repeated local street-grid or tract-grid patterns;
- approximate local symmetries where several tracts play equivalent roles;
- module boundaries formed by settlement, geography, infrastructure, or
  administrative edges.

Biological and physical network analogues are useful here:

- slime mold reinforces efficient paths and approximates transport networks;
- spider webs balance coverage, redundancy, and material cost;
- roots, veins, and rivers preserve flow while pruning waste;
- tilings and molecular structures preserve adjacency under local
  transformations.

These are analogies for graph design, not legal or biological claims about
voters.

The sharpened modeling rule is:

```text
topology finds the mesh
geometry measures physical connection
population supplies flow/mass
district count supplies factorization pressure
```

## Core Model

Start from the tract adjacency graph:

```text
G = (V, E)
```

where vertices are tracts or units and edges are adjacencies with spatial
attributes.

Define a cohesion score for each edge:

```text
cohesion(u, v)
  = boundary_strength(u, v)
  + cycle_support(u, v)
  + population_mass(u, v)
  + module_affinity(u, v)
  + local_role_similarity(u, v)
  - bridge_likeness(u, v)
  - separator_penalty(u, v)
```

Then use the cohesion score as an edge-weighting input:

```text
high cohesion -> expensive to cut
low cohesion  -> easier to cut
```

The intended behavior is:

```text
keep cycle-rich / module-rich areas together
thicken connective tissue where population mass is high
prefer cuts through weak bridges between local modules
preserve population balance and contiguity
```

Population is not only a final balance constraint in this track. It also acts
as a nonpartisan mass or flow field:

```text
more people across a local mesh -> stronger cohesion signal
sparse connector between modules -> weaker cohesion signal
```

Candidate mass terms:

```text
pair_mass(u, v) = sqrt(pop(u) * pop(v))
density_mass(u, v) = sqrt(density(u) * density(v))
local_mass(u, v) = population within radius R of edge (u, v)
```

The mass term should be normalized or clamped so large urban regions do not
overwhelm every other signal:

```text
mass_factor = clamp(log1p(local_mass) / median_log1p_mass, 0.5, 2.0)
```

This allows dense, cycle-rich urban fabric to resist arbitrary cuts while still
allowing apportionment pressure to split large regions into multiple districts.

## Geographic Cycles And Corridors

Track Y should also consider physical-geography cycles, not only adjacency
cycles. Some tract groups cohere because they sit inside the same geographic
system:

- same river corridor or watershed;
- same valley floor or basin;
- same elevation band, ridge shelf, or pass;
- same coastal shelf, delta, island chain, or lake edge;
- same mountain-side catchment where roads and settlement follow terrain.

These are not ordinary graph automorphisms. They are geography-conditioned
role classes and flow corridors:

```text
geo_affinity(u, v)
  = river_corridor_affinity
  + watershed_affinity
  + elevation_band_affinity
  + valley_or_basin_affinity
  - ridge_or_water_barrier_penalty
```

In the biological analogy, these features are the terrain field that shapes
where the web, root, vein, or transport network thickens. Dense population
inside a valley should strengthen the local mesh more than a sparse, high-pass
connector, but the term must remain clamped and reviewable.

Important boundary:

```text
topology says whether there is a loop
geography says whether the loop follows a real corridor or basin
population says how much mass flows through it
```

The first default `cohesion` implementation should keep physical-geography
terms disabled until data provenance and field authorization are explicit.
Later profiles can add them as declared variants:

```text
--weights-override cohesion-terrain
--weights-override cohesion-river
```

or as parameters under `cohesion` once sidecars can report which layers were
used.

## Compositor Layering

Y.1 should enter BISECT through Layer 2 of the existing three-layer compositor.
It is an edge-weight profile first, not a new structure mode.

Proposed first surface:

```text
--weights-override cohesion
```

or, if the name needs to stay explicitly experimental:

```text
--weights-override bio-cohesion
```

The data flow should be:

```text
Layer 1 structure:
  standard-bisect / prime-factor / ratio-optimal / nway / ...

Layer 2 weights:
  geographic base edge weights
  + cycle support
  + bridge-likeness
  + local module affinity
  + population mass factor
  -> cohesion edge weights

Layer 3 search:
  existing seed or ensemble strategy
  + optional canonical plan/tree hash sidecar
  + optional fairness-invariance checks
```

This preserves clean comparisons:

```text
same structure + same search + different weights = weight-layer experiment
```

For example:

```text
bisect state --state CA --districts 52 \
  --structure prime-factor \
  --weights-override geographic

bisect state --state CA --districts 52 \
  --structure prime-factor \
  --weights-override cohesion
```

Only after Layer 2 evidence is strong should Track Y consider a Layer 1
structure mode that pre-factors the state into cohesion regions before
bisection.

Possible later Layer 1 surface:

```text
--structure cohesion-factor
```

That mode would do:

```text
detect cohesion modules
apportion district counts to modules
run bisection inside modules
repair boundaries if needed
```

That is more invasive and should stay deferred.

## Spatial Factorization View

The final plan can be treated as a constrained spatial factorization:

```text
State = D1 * D2 * ... * Dk
```

where:

```text
D1 union D2 union ... union Dk = State
Di intersection Dj = empty for i != j
population(Di) ~= ideal
connected(Di) = true
boundary_cost(D1..Dk) is low
mode constraints hold
```

Recursive bisection then becomes a factorization tree:

```text
CA_52 -> A_26 * B_26
A_26 -> A1_13 * A2_13
B_26 -> B1_13 * B2_13
```

Prime district counts such as `13` are not failures of the model. They force an
unequal spatial factorization such as `6 + 7`, ratio-optimal branching, n-way
construction, or another declared structure policy.

## Group And Symmetry Layer

The group-theoretic layer should improve comparison, caching, fairness tests,
and tie-breaking rather than replace the graph cut.

Canonicalization targets:

- district-label permutations: `Plan / S_k`;
- bisection child swaps: `Tree / (C2 x C2 x ...)`;
- unit-id relabeling in fixtures;
- approximate local symmetry orbits in synthetic and tract-neighborhood
  fixtures.

Invariance and equivariance claims:

```text
invariant:   score(g * plan) = score(plan)
equivariant: build(g * input) = g * build(input)
```

Fairness harness examples:

- geographic mode should be invariant under party-field scramble;
- geographic mode should be invariant under demographic-field scramble;
- VRA mode may be sensitive to authorized demographic fields;
- all modes should be equivariant under unit-id permutation;
- all metrics should be invariant under district renumbering.

## Edge And Vertex Weighting Questions

Edge weighting questions:

- Does cycle support identify local cohesion that should resist cuts?
- Do bridge-like edges predict better cut boundaries?
- Does local module membership reduce county, city, or community splits without
  explicit policy overreach?
- Does population mass thicken dense local meshes without causing urban regions
  to dominate the objective?
- Do river, valley, elevation, watershed, or ridge features explain natural
  tract corridors better than adjacency cycles alone?
- Which physical-geography datasets are allowed in each mode, and how are their
  effects reported?
- Can edge-weight symmetry `w(u,v) = w(v,u)` be preserved under the new score?

Vertex weighting questions:

- Should geographic mode use population both as a balance constraint and as a
  nonpartisan mass field?
- Can local role similarity use non-policy structural features such as degree,
  boundary profile, and cycle participation?
- Which fields are explicitly forbidden in each mode?
- How do authorized VRA or county-sticky features interact with cohesion
  weights?

Population should be used in two separate ways:

1. Local reinforcement: population thickens the internal cohesion web where
   people are actually concentrated.
2. Factor feasibility: total population determines how many district factors a
   cohesive region must become.

This distinction keeps the method explainable:

```text
cohesion says what wants to stay together
apportionment says how many pieces it must become
bisection chooses weak cuts inside or between those pieces
```

## Experiment Design

Initial fixture tiers:

1. synthetic square or hex grids with known symmetries;
2. synthetic grids with bridges, holes, and modules;
3. one California tract subset;
4. full California congressional run only after small fixtures pass.

Metrics:

- population deviation;
- contiguity;
- Polsby-Popper and Reock compactness;
- county/city split counts where available;
- cut-edge cohesion loss;
- cycle-support retained inside districts;
- bridge edges selected as cuts;
- canonical plan hash stability;
- fairness-invariance pass/fail rows by mode.

## Acceptance Criteria

- The method is deterministic or records all seeds.
- The new edge score is inspectable per edge.
- The method does not use forbidden fields in the selected mode.
- Canonical plan and tree hashes are stable under label and child-swap
  symmetries.
- Synthetic fixtures demonstrate expected behavior before California-scale
  claims.
- Any empirical improvement is stated as research evidence, not legal
  certification.

## Build Sequence

1. Land the documentation and fixture expectations.
2. Implement `CohesionParams`, `CohesionEdgeTerms`, bounded cycle support,
   bridge-likeness, population mass, and clamped mass factor in BISECT-local
   code.
3. Wire `--weights-override cohesion` only after L0 term tests pass.
4. Emit a `bisect.cohesion.v1` sidecar for every cohesion run.
5. Run synthetic mesh, bridge, dense-core, and relabeling fixtures.
6. Run a California subset before any full CA-52 experiment.
7. Keep `--structure cohesion-factor` deferred until Layer 2 evidence justifies
   pre-factoring modules.

## Implementation Boundary

Start BISECT-local:

```text
canonical_plan_hash(assignments)
canonical_split_hash(left_units, right_units)
canonical_bisection_tree_hash(tree)
edge_cycle_support(graph)
edge_bridge_likeness(graph)
cohesion_weight(edge)
assert_mode_invariance(mode, fixture, transformation)
```

Suggested modules:

```text
bisect-core::cohesion
  edge_cycle_support
  edge_bridge_likeness
  population_mass_factor
  cohesion_edge_weight

bisect-apportion
  wire `WeightsOverride::Cohesion`
  preserve existing structure/search dispatch

bisect-report
  record cohesion sidecar rows
  record canonical plan/tree hashes when enabled
```

Only extract to RLINE after repeated consumers need generic permutation,
group-action, canonical representative, or semiring/path-scoring utilities.

Implementation spec:

- `docs/specs/2026-06-06-y1-cohesion-weighted-bisection.md`
