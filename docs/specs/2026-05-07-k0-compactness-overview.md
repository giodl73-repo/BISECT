# Compactness Taxonomy and Overview

**Series**: K.0
**Status**: Accepted 3.5/4
**Target**: Harvard Journal on Legislation

## Algorithm / Subject

Overview and taxonomy paper cataloguing all compactness metrics used in redistricting. Covers why compactness matters legally (Rucho v. Common Cause, Shaw v. Reno), the conceptual distinction between area-based metrics (Reock, Convex Hull) and perimeter-based metrics (Polsby-Popper, Schwartzberg), pairwise correlation among metrics across the bisect algorithm suite, and a practitioner decision table mapping use cases to recommended metrics. Not an empirical paper — synthesises K.1–K.7 into a unified reference.

## Key Claims

1. Compactness metrics fall into two families (area-based and perimeter-based) that are weakly correlated with each other (r < 0.5) but strongly correlated within families (r > 0.75), implying that no single metric captures all legally relevant dimensions of shape.
2. Courts have cited at least four distinct metrics across redistricting cases — Polsby-Popper (Rucho), Reock (NC and WI litigation), convex hull (visual test), and population-weighted compactness (efficiency gap proxies) — demonstrating that practitioners need a multi-metric profile, not a single score.
3. A six-cell practitioner decision table (metric × use case) identifies the preferred metric for: (a) litigation evidence, (b) algorithm certification, (c) public communication, (d) VRA district analysis, (e) statutory compliance, and (f) algorithmic optimisation.

## Layer

Standalone / Legal

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), 2020 census
- Compare across: standard-bisect, prime-factor, ratio-optimal, moving-knife structure algorithms
- Metrics: Polsby-Popper, Reock, Convex Hull Ratio, Schwartzberg, Length-Width, Population-Weighted Compactness — pairwise Pearson correlation matrix across all districts in NC/WI/TX

## Test Invariants

- L0: correlation matrix is symmetric; diagonal entries equal 1.0; all entries in $[-1, 1]$
- L1: on a synthetic circular district, all six metrics return their theoretical maximum (PP=1, Reock=1, CH=1, S=1, LW=1, PWC=minimum)
- L2: NC 2020 correlation matrix matches the paper's Table 1 values within 0.02 tolerance (real-data regression)

## Legal / Practitioner Value

Foundational reference for K-track. Shaw v. Reno (1993) established that bizarrely shaped districts trigger strict scrutiny; Rucho v. Common Cause (2019) confirmed that compactness is a justiciable state-law criterion even if not a federal one. Courts and special masters frequently request multi-metric compactness profiles; this paper provides the vocabulary and conceptual framework. The decision table gives practitioners a principled basis for metric selection in expert testimony.

## Section Structure

§1 Introduction and Legal Background, §2 Metric Taxonomy (area-based vs perimeter-based), §3 Mathematical Definitions of K.1–K.6, §4 Correlation Analysis, §5 Decision Table for Practitioners, §6 Synthesis of K.1–K.7 Findings, §7 Conclusion
