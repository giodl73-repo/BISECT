---
title: "Housing Character Edge Weights via ACS"
series: M.3
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm
ACS 5-year estimates, three tables:
- B25024 (Units in Structure) → pct_single_family = 1-unit detached / total units, pct_multifamily = (5+ unit) / total units
- B25003 (Tenure) → pct_owner = owner occupied / total occupied units
- B25035 (Median Year Structure Built) → housing_vintage = (2024 − median_year_built) / 100

Per-tract character vector: [pct_single_family, pct_multifamily, pct_owner, housing_vintage].

Edge weight = w_base × cosine_similarity(char_u, char_v).

Blend: w(u,v) = alpha × w_boundary + (1-alpha) × w_housing, alpha = 0.5 default.

CLI flag: `--weights-override housing-character`

## Claims
1. Single-family tracts cluster together in districts; multifamily/apartment corridor tracts cluster separately — measured as reduction in within-district std(pct_single_family) of ≥10% vs geographic baseline on NC-14.
2. Pre-1960 housing stock tracts (median year built < 1960, housing_vintage > 0.64) cluster with other historic neighborhoods rather than being split across newer suburban districts.
3. Housing character weights are computable from ACS data already in the pipeline — no new download infrastructure required beyond extending the demographics CSV with 4 new columns.

## Data Sources
Census ACS 5-year API: tables B25024, B25003, B25035 at census tract geography.
Alternatively: NHGIS bulk download (same tables).
These tables are an extension of the existing demographics CSV already fetched by `bisect fetch`. No new download infrastructure is required — extend the demographics CSV with 4 new columns.
License: Public domain, U.S. Census Bureau.

## Layer
Plug-in similarity function for the M-track framework (M.0). Uniquely requires no new data download — it extends existing ACS demographics infrastructure already in the pipeline. This makes M.3 the lowest-cost M-track paper to implement empirically.

## Test Invariants (L0)
- single_family_tracts_high_similarity: two tracts both with pct_single_family > 0.85 → similarity > 0.85
- mixed_tenure_vs_homogeneous_low_similarity: tract with pct_owner=0.9, pct_single_family=0.9 vs tract with pct_owner=0.1, pct_multifamily=0.8 → similarity < 0.3
- vintage_far_apart_low_similarity: tract with housing_vintage=0.10 (built ~2014) vs tract with housing_vintage=0.80 (built ~1944) → similarity < 0.5

## Empirical Targets
State: NC (14 congressional districts, 2020).
Metrics:
- Within-district std(pct_single_family): must decrease ≥10% vs `--weights-override geographic` baseline.
- Historic neighborhood clustering: tracts with median_year_built < 1960 must have co-assignment rate ≥60% with other pre-1960 tracts (vs ≤40% under geographic weights).
- Implementation cost: no new bisect fetch calls required — confirm 4 ACS columns are addable to existing demographics CSV without pipeline restructuring.

Comparison baseline: `--weights-override geographic` on same NC-14 plan config.
