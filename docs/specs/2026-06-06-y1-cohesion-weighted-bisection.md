---
title: "Y.1 -- Cohesion-Weighted Bisection"
series: Y.1
status: Proposed design
date: 2026-06-06
track: Y-bio-symmetry-spatial-factorization
layer: Layer 2 -- edge weight modifier, with optional validation sidecars
target_audience: algorithms / research
---

## Decision

Add a research-only cohesion weight profile to the BISECT three-layer
compositor. The first implementation should be a Layer 2 edge-weight modifier,
not a new Layer 1 structure mode.

Proposed CLI value:

```text
--weights-override cohesion
```

Fallback explicit name if the ordinary name is too broad:

```text
--weights-override bio-cohesion
```

This mode should compose with existing structures and searches:

```text
--structure prime-factor --weights-override cohesion --search convergence
--structure standard-bisect --weights-override cohesion --search multi
```

## Model

Y.1 treats the tract adjacency graph as a spatial cohesion network:

```text
topology finds the mesh
geometry measures physical connection
population supplies flow/mass
district count supplies factorization pressure
```

The edge score should make strong local tissue expensive to cut and weak
connective tissue easier to cut.

## Edge Formula

Start with the ordinary geographic boundary signal:

```text
w_boundary(u, v) = shared_boundary_length(u, v)
```

Compute additional local terms:

```text
cycle_support(u, v)
bridge_likeness(u, v)
population_mass(u, v)
module_affinity(u, v)
geo_affinity(u, v)        # disabled in first implementation
barrier_penalty(u, v)     # disabled in first implementation
```

First implementation formula:

```text
w_cohesion(u, v)
  = w_boundary(u, v)
    * cycle_factor(u, v)
    * mass_factor(u, v)
    * module_factor(u, v)
    * geo_factor(u, v)
    * bridge_factor(u, v)
    * barrier_factor(u, v)
```

where:

```text
cycle_factor = 1 + alpha_cycle * normalized_cycle_support
mass_factor = clamp(log1p(local_mass) / median_log1p_mass, min_mass, max_mass)
module_factor = if same_module { 1 + alpha_module } else { 1 }
geo_factor = 1 + alpha_geo * normalized_geo_affinity
bridge_factor = 1 - alpha_bridge * normalized_bridge_likeness
barrier_factor = 1 - alpha_barrier * normalized_barrier_penalty
```

Initial defaults:

```text
alpha_cycle = 0.50
alpha_module = 0.25
alpha_bridge = 0.25
min_mass = 0.50
max_mass = 2.00
max_cycle_depth = 6
alpha_geo = 0.00
alpha_barrier = 0.00
```

All defaults are research defaults, not production legal defaults.

## Cycle Support

For each edge `(u, v)`:

```text
remove edge (u, v)
path = bounded_bfs(u, v, max_depth = max_cycle_depth)
restore edge (u, v)

if path exists:
    cycle_len = path_len + 1
    cycle_support(u, v) = 1 / cycle_len
else:
    cycle_support(u, v) = 0
```

Interpretation:

- a short alternate path means the edge is inside local mesh tissue;
- no alternate path means the edge is bridge-like or tree-like.

Weighted variants may use boundary-length or distance costs later, but the
first fixture should use bounded BFS for inspectability.

## Population Mass

Population is a nonpartisan flow/mass field in this mode. It should not encode
identity, party, or legal preference by itself.

Candidate first mass term:

```text
local_mass(u, v) = pop(u) + pop(v) + sum(pop(n) for n in common_neighbors(u, v))
```

Alternative later terms:

```text
pair_mass(u, v) = sqrt(pop(u) * pop(v))
density_mass(u, v) = sqrt(density(u) * density(v))
radius_mass(u, v, R) = population within graph radius R of the edge
```

The mass factor must be normalized or clamped so dense urban regions thicken
connective tissue without dominating every cut:

```text
mass_factor = clamp(log1p(local_mass) / median_log1p_mass, 0.50, 2.00)
```

## Module Affinity

The first implementation may omit module affinity and set:

```text
module_factor = 1
```

If included, it should come from product-neutral graph modules such as connected
communities over boundary-weighted adjacency. County, city, land-use,
demographic, or other policy fields must not silently enter this term.

## Bridge Likeness

First implementation:

```text
bridge_likeness = 1 - normalized_cycle_support
```

Later implementations may add:

- exact bridge detection;
- edge betweenness;
- articulation-neighborhood diagnostics;
- separator-corridor evidence.

Exact bridges and zero-cycle-support edges should be exposed in the sidecar
report even when they are not selected as cuts.

## Physical Geography Affinity

The first implementation should keep physical-geography terms disabled. Track Y
should still reserve the model slots because tracts can form natural local
groups through terrain and hydrography:

```text
same river corridor
same watershed or catchment
same valley floor or basin
same elevation band or slope shelf
same coastal shelf, island chain, delta, or lake edge
```

These terms are different from adjacency cycle support:

```text
cycle_support: "is this edge part of a local graph loop?"
geo_affinity:  "does this edge follow the same physical corridor or basin?"
barrier_penalty: "does this edge cross a ridge, water body, pass, or other separator?"
```

Candidate data sources for later phases:

```text
USGS/NHD hydrography for rivers and water bodies
USGS 3DEP or other DEM products for elevation bands, slopes, ridges, and passes
HUC watershed/catchment boundaries for basin membership
coastline/lake/river barrier layers from TIGER or NHD-derived joins
```

Any physical-geography term must be:

- explicitly enabled in the selected weight profile;
- named in the manifest and cohesion sidecar;
- symmetric across `(u, v)` and `(v, u)`;
- auditable by edge-term summaries;
- kept separate from party, race, county, city, land-use, or other policy fields
  unless the selected mode explicitly authorizes them.

Possible later weight profiles:

```text
--weights-override cohesion-terrain
--weights-override cohesion-river
```

or an explicit parameterized `cohesion` mode once config and sidecar support
exist.

## CLI And YAML

CLI:

```text
bisect state --state CA --districts 52 \
  --structure prime-factor \
  --weights-override cohesion
```

YAML:

```yaml
algorithm:
  structure: prime-factor
  weights: cohesion
  search: convergence
  cohesion:
    alpha_cycle: 0.50
    alpha_module: 0.25
    alpha_bridge: 0.25
    min_mass: 0.50
    max_mass: 2.00
    max_cycle_depth: 6
```

If the codebase prefers flat keys for compositor parameters, use:

```yaml
algorithm:
  weights: cohesion
  cohesion_alpha_cycle: 0.50
  cohesion_alpha_module: 0.25
  cohesion_alpha_bridge: 0.25
  cohesion_min_mass: 0.50
  cohesion_max_mass: 2.00
  cohesion_max_cycle_depth: 6
```

## Implementation Location

Start BISECT-local.

```text
crates/bisect-core/src/cohesion.rs
  CohesionParams
  CohesionGeography
  CohesionEdgeTerms
  edge_cycle_support
  edge_bridge_likeness
  population_mass_factor
  cohesion_edge_weight
  cohesion_edge_terms_with_geography

crates/bisect-cli / bisect-apportion
  WeightsOverride::Cohesion
  config/YAML parsing
  dispatch through existing Layer 2 path

crates/bisect-report
  cohesion sidecar summary
  optional edge-term sample rows
```

Do not create a new crate for the first slice.

## Evidence Sidecar

Every cohesion run should record a sidecar summary:

```json
{
  "schema": "bisect.cohesion.v1",
  "weights": "cohesion",
  "params": {
    "alpha_cycle": 0.5,
    "alpha_module": 0.25,
    "alpha_bridge": 0.25,
    "min_mass": 0.5,
    "max_mass": 2.0,
    "max_cycle_depth": 6
  },
  "edge_count": 0,
  "geo_layers_used": [],
  "cycle_supported_edges": 0,
  "zero_cycle_edges": 0,
  "exact_bridge_edges": null,
  "mass_factor_min": 0.0,
  "mass_factor_median": 0.0,
  "mass_factor_max": 0.0,
  "cut_edges": 0,
  "cut_edges_low_cycle": 0,
  "cut_edges_low_cycle_share": 0.0,
  "cut_edges_avg_bridge_likeness": 0.0,
  "forbidden_fields_used": []
}
```

The sidecar must make it possible to review what signal changed the run without
reading raw tract geometry.

## Run Manifest Fields

The normal build manifest should include the selected weight profile and every
non-default cohesion parameter:

```json
{
  "cohesion_sidecar_path": "data/cohesion.json",
  "algorithm": {
    "structure": "prime-factor",
    "weights": "cohesion",
    "search": "convergence",
    "cohesion": {
      "alpha_cycle": 0.5,
      "alpha_module": 0.25,
      "alpha_bridge": 0.25,
      "min_mass": 0.5,
      "max_mass": 2.0,
      "max_cycle_depth": 6,
      "module_affinity": "disabled",
      "population_mass": "common-neighborhood-log-clamped",
      "geo_affinity": "disabled",
      "barrier_penalty": "disabled"
    }
  }
}
```

The manifest should not require storing every edge term. Edge-level samples and
summaries belong in the cohesion sidecar.

## Implementation Phases

### Phase 0 -- Documentation And Fixtures

- Add this spec and Track Y paper plans.
- Define tiny graph fixtures in code comments or JSON test helpers.
- Add expected cycle support, bridge-likeness, and mass-factor values for each
  fixture.
- Do not add a CLI value yet.

### Phase 1 -- Core Terms

- Add `CohesionParams`.
- Add `CohesionEdgeTerms`.
- Implement bounded BFS alternate-path cycle support.
- Implement bridge-likeness as `1 - normalized_cycle_support`.
- Implement common-neighborhood population mass.
- Implement clamped log mass factor.
- Add L0 tests for finite, symmetric edge weights.

### Phase 2 -- Weight-Layer Wiring

- Add `WeightsOverride::Cohesion`.
- Parse YAML/CLI parameters.
- Wire through the existing Layer 2 edge-weight path.
- Emit sidecar summary for build runs.
- Keep module affinity disabled.

### Phase 3 -- Synthetic Integration

- Run `standard-bisect + cohesion` on synthetic fixtures.
- Compare against `geographic` and `unweighted`.
- Add optional synthetic river/valley/elevation-band fixtures with declared
  geography-only fields, but keep them out of the default cohesion mode.
- Add canonical plan hash checks where the current code has canonical assignment
  support.
- Verify forbidden fields are not loaded in geographic mode.

### Phase 4 -- California Subset

- Select a small California tract subset with:
  - one dense cycle-rich urban mesh,
  - one sparse connector,
  - one bridge-like boundary,
  - enough population variance to exercise mass clamping.
- Run `geographic` versus `cohesion` with the same structure/search.
- Record summary only; no statewide claims.

### Phase 5 -- Full California Research Run

- Run CA-52 only after Phase 4 passes.
- Use `prime-factor` and `geographic` as baseline.
- Report differences as research evidence only.

## Synthetic Fixtures

### Fixture A -- Square Mesh

Purpose: prove cycle support protects mesh tissue.

```text
A -- B -- C
|    |    |
D -- E -- F
|    |    |
G -- H -- I
```

Expected:

- interior mesh edges have alternate paths;
- no exact bridge edges;
- cycle support is nonzero for most edges;
- cohesion weights exceed geographic weights on dense mesh edges when mass is
  nonzero.

### Fixture B -- Two Meshes With Bridge

Purpose: prove bridge-like connectors remain visible.

```text
A -- B -- C      X -- Y -- Z
|    |    |      |    |    |
D -- E -- F -- P -- Q -- R
|    |    |      |    |    |
G -- H -- I      U -- V -- W
```

Expected:

- edge `F-P` or the declared connector has zero or low cycle support;
- bridge-likeness is higher on the connector than inside either mesh;
- when population balance permits, selected cuts should prefer the connector.

### Fixture C -- Dense Core And Sparse Ring

Purpose: prove population mass thickens dense local tissue.

```text
low-pop ring around high-pop center
```

Expected:

- cycle support alone does not distinguish dense and sparse mesh edges;
- mass factor raises high-population core edge weights;
- mass clamping prevents the core from dominating all cut choices.

### Fixture D -- Symmetric Relabeling

Purpose: prove input-order and unit-id transformations do not change the
canonical output.

Expected:

- shuffled unit order returns the same canonical plan hash;
- permuted unit ids return an equivalent canonical plan;
- district-label permutations do not change metrics.

### Fixture E -- River Or Valley Corridor

Purpose: prove physical corridor affinity is separable from graph cycle
support.

```text
tracts arranged in two local meshes, with one mesh following a declared river
or valley attribute and another crossing a declared ridge/barrier
```

Expected:

- default `cohesion` ignores the synthetic geography layer;
- future terrain/river-enabled profiles strengthen same-corridor edges;
- future terrain/river-enabled profiles weaken declared barrier-crossing edges;
- sidecar output records which geography layer was used.

## Validation Tiers

### L0 Unit Fixtures

- `cycle_support_square_grid`: all interior grid edges have alternate paths.
- `cycle_support_bridge_graph`: bridge edge has zero cycle support.
- `mass_factor_clamped`: dense fixture values remain within `[min_mass,
  max_mass]`.
- `cohesion_weight_finite`: no NaN or infinite edge weights.
- `cohesion_weight_symmetric`: `w(u,v) = w(v,u)`.
- `cohesion_uses_no_forbidden_fields`: geographic mode does not read party or
  demographic fields.
- `cohesion_default_ignores_physical_geography`: default `cohesion` does not
  read terrain, river, or watershed layers.

### L1 Synthetic Runs

- symmetric grid plan is stable under unit-id relabeling;
- bridge fixture prefers bridge-like cuts when population feasible;
- dense mesh fixture preserves high-cycle/high-mass local tissue better than
  unweighted baseline;
- canonical plan hash is stable under district-label permutation.

### L2 Research Runs

- one California tract subset;
- full California `k=52` only after synthetic fixtures pass;
- compare against `geographic` weights under the same structure/search;
- report compactness, splits, population balance, retained cycle support,
  selected cut bridge-likeness, and invariance rows.

## Compositor Boundaries

Y.1 changes only Layer 2:

```text
same structure + same search + different weights = clean comparison
```

It must not:

- change the bisection tree;
- change METIS seed search;
- silently add VRA, party, county, city, or land-use fields;
- claim legal fairness from biological analogy;
- make production claims before fixture and research runs exist.

## Deferred Layer 1 Mode

A future structure mode may be justified if Layer 2 evidence shows that cohesion
modules should be apportioned before bisection:

```text
--structure cohesion-factor
```

Deferred behavior:

```text
detect cohesion modules
assign district counts to modules
run bisection inside modules
repair boundaries if needed
```

This is out of scope for Y.1.

## Open Questions

- Should the public CLI value be `cohesion` or `bio-cohesion`?
- Should module affinity remain disabled for the entire first paper?
- Should population density use tract area when area is available, or should
  first implementation use population mass only to avoid projection/area
  coupling?
- Should sidecar rows sample top-N strongest and weakest edges, or only summary
  histograms?
- Should invariance checks live under `bisect validate-invariance` or as a
  build option that emits a heavier validation packet?

## RLINE Extraction Boundary

Keep BISECT-specific policy local. Only consider RLINE extraction for generic
pieces after another consumer needs them:

- bounded cycle support over weighted graphs;
- canonical representative helpers;
- permutation/group-action test fixtures;
- semiring-style path scoring.
