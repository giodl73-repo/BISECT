---
title: "Economic Character Edge Weights via LODES Workplace Area Characteristics"
series: M.1
status: Accepted 3.5/4
date: 2026-05-08
track: M-community-character
layer: Structure (Layer 2 — edge weight modifier)
---

## Algorithm
LODES WAC (Workplace Area Characteristics) → aggregate census blocks to tracts → compute three per-tract signals:
- commercial_intensity = (CNS07 + CNS09 + CNS10 + CNS11) / C000
- industrial_fraction = (CNS01 + CNS02 + CNS05 + CNS08) / C000
- jobs_per_resident = C000 / tract_population

Edge weight = w_base × cosine_similarity(char_u, char_v) where char = [commercial_intensity, industrial_fraction, jobs_per_resident].

Blend: w(u,v) = alpha × w_boundary + (1-alpha) × w_econ, alpha = 0.5 default.

CLI flag: `--weights-override economic-character`

## Claims
1. Reduces mean within-district std(jobs_per_resident) by ≥15% vs geographic weights on NC-14 (14-district congressional plan).
2. Research Triangle Park tracts (jobs_per_resident ≫ 1) cluster into a single district rather than being split across neighboring residential districts.
3. NC proportionality gap shifts from −6.5 pp toward ±4 pp as employment centers form coherent single-character districts.

## Data Sources
https://lehd.ces.census.gov/data/lodes/LODES8/{state}/wac/{state}_wac_S000_JT00_2020.csv.gz
Format: CSV (gzipped), one row per census block, LEHD LODES v8.
License: Public domain, U.S. Census Bureau / BLS annual release.

## Layer
Plug-in similarity function for the M-track framework (M.0). Replaces or blends with the default geographic edge weights in the three-layer compositor. alpha=0.5 gives equal weight to boundary contiguity and economic character similarity. alpha is tunable per plan config YAML.

## Test Invariants (L0)
- lodes_aggregation_preserves_totals: sum of block-level C000 after tract aggregation equals state total jobs
- pure_residential_similarity_is_one: two tracts with identical WAC profiles → cosine_similarity == 1.0
- residential_vs_industrial_similarity_lt_0_3: tract with commercial_intensity=0.9 vs tract with industrial_fraction=0.9 → similarity < 0.3
- zero_jobs_no_nan: tract with C000=0 (no jobs) → char vector treated as zero vector, similarity = 0.0 (not NaN)

## Empirical Targets
State: NC (14 congressional districts, 2020).
Metrics:
- Within-district std(jobs_per_resident): must decrease ≥15% vs `--weights-override geographic` baseline.
- Research Triangle Park cluster: tracts with jobs_per_resident > 5.0 must appear in ≥1 single district with ≥80% of those tracts co-assigned.
- Proportionality gap: |D_seats/D_seats_expected − 1| target ≤ 4 pp vs −6.5 pp baseline.

Comparison baseline: `--weights-override geographic` on same NC-14 plan config.
