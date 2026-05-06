# Quickstart: Algorithm Explorer

**Who you are:** A researcher or technical staff member who wants to run controlled experiments across algorithm variants and understand which design choices drive partisan outcomes.

**What you'll have at the end:** Six labeled plans for one state, covering different structure/weight/search configurations, a comparison table showing per-plan metrics, and a clear answer to which layer (structure, weights, or search) dominates the partisan outcome.

**Time:** 15–30 minutes for NC; under 10 minutes for VT or DE.

---

## The three-layer compositor

Every plan produced by `redist` is the output of three independently configurable layers:

| Layer | Flag | What it does |
|-------|------|--------------|
| **Structure** | `--partition-mode` | Defines the bisection tree: how k seats are recursively split. `apportion-regions` factors k by primes (Huntington-Hill derivation); `prime-factor` is similar; `ratio-optimal` searches for the best binary split ratio; `ratio-optimal-area` adds land-area balance to the ratio search. |
| **Weights** | `--weights` | Controls edge weights in the METIS adjacency graph. `geographic` uses TIGER boundary lengths (default, statutory choice); `county` adds a bonus to edges that cross county lines. |
| **Search** | `--search` | Determines which METIS solution is selected at each bisection level. `convergence` runs T seeds and takes the minimum edge-cut; `percentile` takes the solution at a given percentile of the edge-cut distribution. |

Changing any single layer while holding the others fixed gives a clean one-factor experiment.

---

## Running six variants

The commands below run six labeled plans for North Carolina (k=14 districts). Each varies exactly one layer from the default.

```bash
# 1. Default (statutory choice: apportion-regions + geographic + convergence)
redist state --state NC --year 2020 \
    --partition-mode apportion-regions \
    --weights geographic \
    --search convergence \
    --label nc_default

# 2. Structure: standard binary bisection (no prime factorisation)
redist state --state NC --year 2020 \
    --partition-mode standard-bisect \
    --weights geographic \
    --search convergence \
    --label nc_standard_bisect

# 3. Structure: ratio-optimal (searches binary ratios, no prime anchoring)
redist state --state NC --year 2020 \
    --partition-mode ratio-optimal \
    --weights geographic \
    --search convergence \
    --label nc_ratio_optimal

# 4. Structure: ratio-optimal-area (adds land-area balance constraint)
redist state --state NC --year 2020 \
    --partition-mode ratio-optimal-area \
    --weights geographic \
    --search convergence \
    --label nc_ratio_optimal_area

# 5. Weights: county-sticky (rewards keeping tracts inside their county)
redist state --state NC --year 2020 \
    --partition-mode apportion-regions \
    --weights county \
    --search convergence \
    --label nc_county_weights

# 6. Search: percentile 0.5 instead of convergence (explores off-optimal solutions)
redist state --state NC --year 2020 \
    --partition-mode apportion-regions \
    --weights geographic \
    --search percentile \
    --percentile 0.5 \
    --label nc_percentile_50
```

---

## Comparing variants

After all six runs complete, use `redist label-compare` to produce a side-by-side table:

```bash
redist label-compare \
    nc_default nc_standard_bisect nc_ratio_optimal \
    nc_ratio_optimal_area nc_county_weights nc_percentile_50 \
    --year 2020 \
    --metrics edge_cut dem_seats efficiency_gap polsby_popper \
    --output outputs/v1/2020/compare/nc_bakeoff.csv
```

The output CSV has one row per label. Open in pandas or Excel to compare. Typical findings for NC (k=14=7x2):

- `nc_default` (apportion-regions): 7D/7R, near-zero efficiency gap -- the prime factorisation of 14 forces a symmetric tree
- `nc_standard_bisect`: 5D/9R, larger efficiency gap -- binary bisection at k=14 creates an asymmetric tree that sorts voters by geography
- `nc_ratio_optimal` and `nc_ratio_optimal_area`: outcomes between the two extremes above
- `nc_county_weights`: similar to default on structure; small partisan shift from county preference
- `nc_percentile_50`: similar partisan outcome to default but with slightly lower compactness

**Key finding (the bakeoff result):** Structure dominates. For states where k has a prime factorisation that produces a symmetric tree (NC: 14=7x2, MN: 8=2x2x2), the `apportion-regions` mode is near-proportional regardless of weight or search choices. For states where k is prime or has an unfavourable factorisation (TN: 9=3x3), no structure mode reliably overcomes the geometric sorting inherent in the state's population distribution. Weight and search choices are second-order effects.

---

## Scanning all variants for one state

For a systematic bakeoff across all documented structure modes:

```bash
redist sweep --state NC --year 2020 \
    --modes apportion-regions standard-bisect ratio-optimal ratio-optimal-area \
    --weights geographic county \
    --search convergence percentile:0.5 \
    --n 1 \
    --label-prefix nc_bakeoff_full
```

This runs 4 structures x 2 weight modes x 2 search modes = 16 labeled plans and writes them to `outputs/v1/2020/plans/nc_bakeoff_full_*`. Aggregate with `redist aggregate` for a 16-row comparison table.

---

## Where to go next

- Full list of `--partition-mode` options: `redist state --help`
- Percentile sweep (compactness-partisan tradeoff): see `quickstart-researcher.md`
- Understanding the prime-factorisation structure: `docs/RECURSIVE_BISECTION.md`
- National 50-state bakeoff results: `docs/PAPERS.md` (B.11 paper)
