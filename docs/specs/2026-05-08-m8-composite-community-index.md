---
title: "Composite Community Character Index and Court Usage Guide"
series: M.8
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm

Combine all available M.1–M.7 signals into a weighted composite edge weight:

```
w_composite(u,v) = w_base × Σ_i (weight_i × sim_i(u,v)) / Σ_i weight_i
```

The sum runs over AVAILABLE signals only. Missing data for a component causes that component to be dropped from both numerator and denominator — graceful degradation without rescaling to a fallback.

**Default signal weights** (tuned empirically on NC-14, WI-8, TX-38):

| Signal | Paper | Default weight |
|--------|-------|---------------|
| zone_membership | M.6 | 0.30 |
| economic_character | M.1 | 0.20 |
| land_use | M.2 | 0.20 |
| housing_character | M.3 | 0.15 |
| commuting_shed | M.4 | 0.10 |
| topographic | M.5 | 0.03 |
| transit | M.7 | 0.02 |

Total: 1.00 (weights sum to one by design; normalization in formula handles partial availability).

**CLI flags**:

```
--weights-override composite-community
--community-weights zone=0.30,economic=0.20,land=0.20,housing=0.15,commute=0.10,topo=0.03,transit=0.02
```

Individual weights can be overridden; the normalization step ensures they need not sum to one when partially specified.

**Court usage**: The composite index provides a court-admissible, quantitative definition of "communities of interest." Expert witness statement template:

> "The submitted plan maximizes tract-pair co-zone membership and minimizes community character dissimilarity, as measured by the composite index. Districts were not drawn to split communities defined by shared administrative services, economic character, or physical geography."

## Claims

1. Composite weights reduce mean within-district economic variance by ≥25% vs standard-bisect geographic weights on NC-14 (2020).
2. School district boundaries are never crossed (the zone_score contribution alone, even at weight 0.30, is sufficient to prevent splits of same-school-district adjacent tracts).
3. The composite index is fully non-partisan: no electoral, racial, or partisan data enters any component signal (M.1–M.7 use only census demographic, geographic, and service-territory data).
4. Composite weights produce more compact districts (higher mean Polsby-Popper) than geographic weights alone, because cutting at natural economic/administrative seams tends to produce rounder geographic areas.

## Data Sources

No new data beyond M.1–M.7. The composite only activates signals where the underlying data is available for the state and year being processed. See individual M.1–M.7 specs for source URLs and coverage notes.

## Layer

Layer 2 — edge weight modifier. The composite is computed as a weighted average of per-signal similarity scores, each of which is independently a Layer 2 modifier. The result is a single scalar per adjacency edge passed to METIS. Does not alter bisection tree structure (Layer 1) or seed selection (Layer 3).

## Test Invariants (L0)

- `composite_nonneg` — w_composite(u,v) ≥ 0.0 for all tract pairs
- `composite_symmetric` — w_composite(u,v) = w_composite(v,u) for all pairs
- `all_sims_one_gives_max_weight` — when all sim_i(u,v) = 1.0, w_composite = w_base × 1.0 (no amplification beyond sum-of-weights normalization)
- `missing_signal_degrades_gracefully` — removing one signal recomputes normalized composite over remaining signals; result ∈ (0, w_base × 1.0]
- `weights_sum_to_one_normalized` — after normalization over available signals, effective weights sum to 1.0

## Empirical Targets

- NC-14 (2020): mean within-district economic variance (M.1 signal) ≤ 75% of the value produced by standard-bisect geographic weights.
- NC-14 (2020): zero school district boundary crossings.
- NC-14, WI-8, TX-38: mean Polsby-Popper under composite-community ≥ mean Polsby-Popper under geographic weights (compactness improvement or no regression).
- Non-partisan check: correlation between composite edge weights and tract-level Democratic vote share < 0.05 (Pearson r) — confirms no embedded partisan signal.
- Partial-data states (no GTFS, no fire district): composite produces valid, non-uniform edge weights using at minimum zone_membership + economic_character + land_use (3 of 7 signals always available).
