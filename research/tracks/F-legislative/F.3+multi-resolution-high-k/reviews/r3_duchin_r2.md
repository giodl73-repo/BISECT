# Review 3 — Moon Duchin
**Paper**: F.3: Resolution Selection for State Legislative Redistricting: When Census Tracts Are Too Coarse
**Round**: R2
**Score**: 3/4

## Response to Revision

The two mathematical concerns I raised in R1 have been addressed.

**C1 (CLT derivation clarity)** — Addressed. Section 3 is now structured as: §3.1 Motivation (CLT and configuration-count arguments give intuition), §3.2 Empirical Calibration (threshold calibrated against C.1 crossover point), §3.3 The Rule. This is the honest and correct structure. The paper no longer presents the empirical calibration as a theoretical derivation.

**C2 (Configuration count formula correction)** — Addressed. The incorrect O(n^m/k^m) formula has been replaced with a qualitative statement about connected subgraph count, correctly noting that the formula must account for contiguity and population balance constraints. The conclusion (richer optimisation space for m ≥ 20) is preserved without the incorrect formula.

**C4 (Near-boundary handling)** — Addressed. A note has been added that states with k/n between 0.045 and 0.055 are near the threshold boundary and should be handled by running both resolutions and selecting the better result.

**C3 (Tier classification from limited data points)** — Addressed by qualification. The revised text now says "we conjecture a range of ±2--4 seats for high-k/n chambers generally" rather than asserting it as a result. This is appropriate given the single high-k/n data point.

## New Observation

The MAUP partisan-direction paragraph (I3 from the revision plan) is a useful addition and is correctly qualified as "suggestive but not conclusive" for two observations. The structural hypothesis (block groups capture urban density more precisely) is the correct mechanism.

## Assessment

All four of my R1 concerns have been addressed — C1 and C2 with corrections, C3 and C4 with appropriate qualification. F.3 was already at 3/4 in R1 and the revisions represent genuine improvement. I maintain 3/4 because the recommendation table (C2 from revision plan) is still absent.

**Score**: 3/4
**Recommendation**: Accept with minor revisions
