# Recursive Bisection

## Short version

Split the state in half by population. Split each half again. Keep going until you have the right number of districts. Each split is a graph partition problem solved by METIS. The shape of every district is determined entirely by geography — no political data enters at any stage.

---

## Why bisection

When you split something in half, neither side gets to choose which half is theirs. The cut is determined by the constraint (equal population) and the geometry, not by who benefits. This is the core fairness property: no one controls the outcome.

Repeating recursively means every district is the product of a sequence of neutral halvings. There is no single moment where a mapmaker decides which community ends up together. The algorithm makes that decision the same way every time, everywhere.

## The binary tree structure

A state with N districts requires ⌈log₂ N⌉ rounds of bisection. Each round doubles the number of regions:

```
Round 0:  1 region  (the whole state)
Round 1:  2 regions
Round 2:  4 regions
Round 3:  8 regions  ← Minnesota (8 districts) stops here
```

For odd-district states, some splits are uneven:

```
Alabama (7 districts):
Round 1:  1 → [4 | 3]
Round 2:  4 → [2|2],  3 → [2|1]
Round 3:  four 2s each → [1|1],  one 1 stays
```

The `bisection_tree.json` output records this tree for every state.

## How METIS does the splitting

Each state is a graph:
- **Nodes**: census tracts (~1,000–8,000 per state)
- **Node weights**: tract population (ensures balanced splits)
- **Edges**: two tracts are connected if they share a border
- **Edge weights**: length of the shared border in meters (edge-weighted mode)

METIS (`gpmetis`) partitions this graph into two halves with equal total population, minimizing the total weight of cut edges. Because edge weight = shared boundary length, minimizing cut weight = minimizing district perimeter = maximizing compactness.

## METIS engines

Two METIS backends are available:

| Engine | How to select | Use case |
|--------|---------------|----------|
| `c-ffi` (default) | no flag needed | Production runs; uses the system-installed METIS C library via FFI. Fastest. |
| `redist-metis` | `--engine redist-metis` | Portable builds where the C library is not available; pure-Rust reimplementation. Identical outputs, ~15% slower. |

The engine is recorded in `runs/<label>/index.json` alongside the config hash, so the audit chain captures which backend was used.

## Population balance

The Huntington-Hill method determines how many districts each state gets. Within a state, each district must be within ±0.5% of the ideal population (total state population ÷ number of districts).

METIS enforces this via its `ufactor` parameter (imbalance tolerance). We set `ufactor=5` which allows ±0.5% — tight enough for legal compliance, loose enough for the partitioner to find good cuts.

## Determinism

The same input always produces the same output. METIS uses a fixed random seed. This matters for:
- **Reproducibility**: other researchers can verify results
- **Cross-census comparison**: differences between 2010 and 2020 results are due to geography changes, not randomness
- **Debugging**: a run that fails can be re-run identically

## The three-layer compositor

Before METIS runs, the build config is resolved through three layers:

**Structure** controls how the bisection tree is seeded. The default mode runs unconstrained recursive bisection across all tracts. Alternative structure modes pre-group tracts before bisection begins:

- `geo-section` — groups tracts into geographic sections (e.g., mountain vs. piedmont vs. coastal) to improve regional coherence in multi-region states
- `area-section` — sections based on land area rather than population; useful for sparse western states where population-based grouping can produce geographically extreme regions
- `apportion-regions` — pre-assigns tracts to named apportionment regions and runs bisection independently within each region

Pass the structure mode via the config YAML or the `--structure` flag:

```bash
redist build my_plan --structure geo-section
```

**Weights** control what METIS minimizes. The default edge weight is the shared boundary length in meters, which drives compactness. The `weights_override` config key can scale or replace this:

```yaml
weights_override:
  boundary_scale: 1.0   # weight by shared boundary length (default)
  area_scale: 0.0        # additionally weight by mean tract area
```

**Search** controls solver parameters passed to METIS: `ufactor` (population tolerance), random seed, and restart count. Post-partitioning contiguity repair is also governed here — if a bisection produces a disconnected component, the search layer can attempt a repair pass before failing.

## What the algorithm does NOT do

- It does not look at political party registration or voting history
- It does not look at race or ethnicity (except in the VRA variant — see [vra-compliance.md](vra-compliance.md))
- It does not try to make districts competitive, safe, or fair by any political metric
- It does not consider incumbents, county lines, or communities of interest

These are features, not bugs. The algorithm's neutrality is the point.

## How to run

### Get the census adjacency data

Download pre-built adjacency files from GitHub Releases (fastest — no local geometry processing needed):

```bash
redist fetch --year 2020 --release
```

Or build adjacency from raw TIGER data you have locally (requires TIGER shapefiles already downloaded):

```bash
redist fetch --year 2020
```

### Run redistricting for a single state

```bash
redist state --state NC --year 2020 --version v1
```

### Run all 50 states

```bash
redist states --year 2020
```

### Run via a named plan config

```bash
redist build nc_2020 --workers 8
```

Named plans record the config hash in `runs/<label>/index.json`, enabling SHA-256 chain verification. Use named plans for any results intended for publication or court submission.

### Run with a non-default structure mode

```bash
redist build nc_2020 --structure geo-section
redist build nc_2020 --structure area-section
redist build nc_2020 --structure apportion-regions
```

### Check a state's bisection tree

```bash
redist analyze --state NC --year 2020 --types contiguity
```

## Further reading

- [RECURSIVE_BISECTION.md](../RECURSIVE_BISECTION.md) — detailed algorithm walkthrough with pseudocode
- [edge-weighted-bisection.md](edge-weighted-bisection.md) — how edge weights improve compactness
- [pipeline-stages.md](pipeline-stages.md) — full four-stage pipeline with SHA-256 audit chain
- Paper B.1: *From Apportionment to Boundary Design*
