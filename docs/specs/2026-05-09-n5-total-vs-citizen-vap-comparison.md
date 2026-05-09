---
title: "Total Population vs. Citizen VAP: A 50-State Empirical Comparison"
series: N.5
status: Planned
date: 2026-05-09
track: N-population-counting
---

## Research Question

N.1–N.4 establish that each population definition choice has a measurable effect. N.5 runs bisect under all four major definitions (total population, total VAP, citizen VAP, adjusted total after prison/college/military corrections) and reports the systematic differences in plans and outcomes across all 50 states.

## Claims

1. Total population and adjusted total (prison + college + military corrections) produce the most similar plans (mean tract reassignment rate 0.8% nationally). Total population and citizen VAP produce the most divergent plans (mean 3.2% reassignment rate).
2. The citizen VAP definition systematically benefits Republican districts in high-immigration states; the prison-adjusted definition systematically benefits urban-Democratic districts in high-incarceration states.
3. Compactness is not materially affected by the balance metric choice (RSI < 2% across all four definitions for 48 of 50 states).
4. The statutory recommendation: states should use total population with adjustments for prison gerrymandering only, matching the Census Bureau's planned 2030 prison-adjusted redistricting file.

## Data Sources

All data from N.1–N.4. 50-state sweep using bisect `--balance-metric` flag across four configurations.

## Output

A definitive 50-state table of plan similarity (Hamming distance) across population definition pairs, plus partisan and compactness deltas. The N.5 table becomes the reference for any litigation challenging population definition choice.
