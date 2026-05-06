# The Three-Layer Compositor

## Short version

Every `redist` run is fully described by three independent choices: how the
bisection tree is structured, what signal METIS optimises, and how the seed
space is searched. These three choices are orthogonal — any combination of
valid layer values produces a well-defined run. The combination is recorded
in a YAML config and is part of the SHA-256 audit chain.

---

## Overview

The three-layer compositor is the design pattern that separates the *what* from
the *how* in redistricting algorithm design. Changing a single layer leaves the
other two unchanged:

- Swap the structure layer to test a different bisection topology.
- Swap the weights layer to change what METIS's objective means geometrically.
- Swap the search layer to change how many seeds you evaluate.

This separation makes it straightforward to compare algorithms: a paper
studying AreaSection (B.9) vs. GeoSection (B.8) holds the weights and search
layers constant and varies only the structure layer.

---

## Layer 1 — Structure (`--structure`)

The structure layer controls how the bisection tree is organised: how many
parts each node splits into, and in what order.

| Value | Description |
|---|---|
| `standard-bisect` | Split into two halves of size floor(k/2) and ceil(k/2) at every level. Default. |
| `prime-factor` | Split according to the prime factorisation of k. k=14=7x2 produces a 7-way primary split followed by 7 bisections. k=17 (prime) falls back to a 9+8 binary split. Paper B.11 (ApportionRegions). |
| `ratio-optimal` | At the first level, try all split ratios from 1:(k-1) through floor(k/2):ceil(k/2). Select the ratio with minimum EC/sqrt(min(i,k-i)). Subsequent levels use standard bisection. Paper B.8 (GeoSection). |
| `ratio-optimal-area` | GeoSection with an added dual area-balance constraint (ncon=2). ubvec = [1.001, 1+area_swing]. Paper B.9 (AreaSection). |
| `ratio-optimal-vra` | GeoSection with a minority geographic alignment score added to the ratio selection objective. No partisan data. Paper B.14 (VRASection). |
| `nway` | Direct k-way METIS partition of the whole state. No recursive tree. |
| `compact-polsby` | Polsby-Popper-guided bisection: at each node, select the bisection that maximises the geometric-mean Polsby-Popper of the resulting districts. |

The `prime-factor` structure is the geographic completion of the Huntington-Hill
apportionment: just as Huntington-Hill determines how many seats each state
gets by prime factorisation, `prime-factor` determines the shape of the
bisection tree by the same factorisation (see paper B.11 for the formal
derivation).

**Note**: `--partition-mode` (legacy flag) and `structure:` YAML key use
different value namespaces. Use `--structure` for the Layer 1 compositor.

---

## Layer 2 — Weights (`--weights-override`)

The weights layer controls what signal METIS optimises. METIS minimises the
total weight of cut edges. Changing edge weights changes what "minimising cuts"
means geographically.

| Value | Description |
|---|---|
| `geographic` | Edge weight = shared boundary length in metres (TIGER). Minimises perimeter. Default. |
| `county` | Multiplies intra-county edges by 3.0. Discourages cuts that cross county lines. 34% fewer county splits at 3% compactness cost. Paper B.10. |
| `unweighted` | All edge weights = 1. Pure population balance, no geometric signal. Used as a research baseline. |
| `vra-aligned` | Minority-to-minority tract edges are boosted: edges where both tracts are at least 40% minority receive 3× to 10× weight (floor: 3.0, computed as max(3.0, 10.0 × (1 − 0.7 × minority_fraction))), tapered by the statewide minority fraction. No partisan data. |
| `proportional` | ncon=2 vertex weights [population, D_votes]. Bisections are constrained to target both population and partisan vote balance simultaneously. Paper B.12. |

The `geographic` weight is the default for all production runs. It is the only
weight mode that has been validated across all 50 states and all three census
years.

The `vra-aligned` weight is used together with `ratio-optimal-vra` (VRASection).
It can also be composed with other structure layers, though cross-layer
combinations involving VRA weights require careful legal review under the
post-Callais disentanglement requirement.

---

## Layer 3 — Search (`--search`)

The search layer controls how many METIS calls are made and which result is
returned. METIS is a local optimiser: different seeds produce different local
minima. The search layer determines how that seed space is explored.

| Value | Description |
|---|---|
| `single` | One call with the SHA-256 content-derived seed. Fully deterministic. Used for statutory certification. |
| `multi` | T seeds, each producing a candidate plan. Returns the plan with minimum normalised edge cut. |
| `convergence` | Walks seeds starting from the content-derived seed. Stops after T consecutive seeds produce no improvement. T=600 is the proposed Districting Integrity Act stopping criterion (T=600, see B.02, B.16). |
| `percentile` | T seeds. Returns the plan at rank floor(p * T) in the sorted edge-cut distribution, not the minimum. Used by H.0 (PercentileSweep) to sample from the distribution of near-optimal plans. |
| `bisection-ensemble` | At each bisection node, runs a local ReCom chain of T steps and selects the bisection at percentile p. Combines METIS topology with ReCom sampling. Paper H.1. |

The `convergence` mode with T=600 is the recommended production setting. It
is both legally defensible (the seed is publicly derivable from the census
release identifier) and empirically stable: for all 50 states in the 2020
census, the last improving seed occurs before index 1,023.

---

## Orthogonality

The three layers are fully orthogonal. Any combination of layer values is a
valid configuration. Some combinations have been studied in the research papers;
others are available in the CLI but have not yet been evaluated systematically.

Studied combinations:

| Structure | Weights | Search | Paper |
|---|---|---|---|
| `standard-bisect` | `geographic` | `convergence` | B.1, B.7 |
| `prime-factor` | `geographic` | `convergence` | B.11 |
| `ratio-optimal` | `geographic` | `multi` | B.8 |
| `ratio-optimal-area` | `geographic` | `multi` | B.9 |
| `ratio-optimal-vra` | `vra-aligned` | `multi` | B.14 |
| `standard-bisect` | `county` | `convergence` | B.10 |
| `standard-bisect` | `proportional` | `multi` | B.12 |
| `ratio-optimal` | `geographic` | `bisection-ensemble` | H.1 |

---

## Config YAML format

Every run is driven by a `configs/{label}.yml` file. The config is the primary
input to the SHA-256 audit chain: the config hash is recorded in `index.json`
and carried forward into every downstream stage.

```yaml
name: official_proposal
description: "ApportionRegions with county-sticky weights, convergence T=600"

algorithm:
  # Layer 1: bisection tree structure
  structure: prime-factor        # standard-bisect | prime-factor | ratio-optimal |
                                 # ratio-optimal-area | ratio-optimal-vra |
                                 # nway | compact-polsby

  # Layer 2: edge weight signal
  weights: county                # geographic (default) | county | unweighted |
                                 # vra-aligned | proportional
  alpha_county: 3.0              # required when weights: county

  # Layer 3: seed search strategy
  search: convergence            # single | multi | convergence | percentile |
                                 # bisection-ensemble
  convergence_threshold: 600     # required when search: convergence

  # Optional: layer-specific parameters
  balance_tolerance: 0.5         # population tolerance in percent (default: 0.5)
  area_swing: 0.10               # area tolerance for ratio-optimal-area (default: 0.10)
  w_vra: 0.40                    # VRA alignment weight for ratio-optimal-vra (default: 0.40)

  # METIS backend
  engine: c-ffi                  # c-ffi (default) | redist-metis | gpmetis

# Pipeline metadata
workers: 6                       # parallel workers per census year (default: 4)
years: ["2020", "2010", "2000"]  # census years to build
analysis_types:
  - compactness
  - splits
  - summary
```

---

## CLI examples

```bash
# Run with explicit layer flags (overrides config values for one-off runs)
redist build official_proposal --year 2020

# Multi-seed search: 50 seeds, return minimum
redist state --state NC --year 2020 --label nc_multi \
  --partition-mode ratio-optimal --search multi --seeds 50

# Single deterministic run (for statutory certification)
redist state --state WI --year 2020 --label wi_statutory \
  --partition-mode prime-factor --search single

# Convergence sweep with T=600
redist state --state TX --year 2020 --label tx_convergence \
  --partition-mode standard-bisect --search convergence \
  --convergence-threshold 600
```

---

## The `--metis-engine` flag

The METIS backend is independent of the three algorithm layers. Three options
are available:

| Value | Description |
|---|---|
| `c-ffi` | Links `libmetis` via C FFI. Vendored inside the Rust workspace, statically linked. Battle-tested; supports all k values. Default for all production runs. |
| `redist-metis` | Pure Rust reimplementation of METIS. No C dependency; portable standalone binary. Suitable for environments where C toolchains are unavailable. |
| `gpmetis` | External `gpmetis` subprocess (reserved; not yet implemented). Allows using system-installed METIS for independent verification. |

The `c-ffi` engine is the statutory engine: its source hash is pinned in
`Cargo.lock` and is part of the reproducibility guarantee. A verifier using
a different METIS binary may obtain different local optima even with the same
seed.

---

## Further reading

- [section-algorithms.md](section-algorithms.md) — the B-series algorithm taxonomy
- [ensemble-methods.md](ensemble-methods.md) — how GerryChain evaluates the output
- [label-pipeline.md](label-pipeline.md) — how configs, builds, and audits connect
- Paper B.8: *GeoSection — Ratio-Optimal Bisection*
- Paper B.11: *ApportionRegions — Prime-Factor Redistricting*
- Paper B.16: *ConvergenceSweep — Statutory Stopping Criterion*
