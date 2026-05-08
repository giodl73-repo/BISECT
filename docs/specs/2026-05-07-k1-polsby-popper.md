# Polsby-Popper Score in Algorithmic Redistricting

**Series**: K.1
**Status**: Accepted 3.5/4
**Target**: Political Analysis

## Algorithm / Subject

Empirical study of the Polsby-Popper compactness score (PP = 4$\pi$ · Area / Perimeter²) across all four bisect structure algorithms. Covers the formula derivation and properties, geographic projection effects on perimeter measurement, sensitivity to jagged coastline boundaries, and a normalization procedure that corrects for boundary artifacts. Implements PP computation in the `bisect-analysis` crate and reports district-level and state-level PP distributions for NC, WI, and TX under 2020 census data.

## Key Claims

1. The bisect ratio-optimal structure algorithm achieves mean PP $\geq 0.22$ across all 14 NC congressional districts under 2020 census data, outperforming standard-bisect (mean PP $\approx 0.17$) and prime-factor (mean PP $\approx 0.19$).
2. PP correlates 0.71 with Reock across all districts produced by B-series algorithms on NC 2020, confirming that both metrics capture a related but distinct dimension of compactness.
3. PP is systematically underestimated for coastal and riverine districts due to jagged TIGER/Line boundary segments; a geographic projection normalization (Albers Equal Area Conic, EPSG:5070) reduces this artifact and shifts mean PP upward by 0.03–0.06 for affected states.

## Layer

Standalone

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), 2020 census
- Compare across: standard-bisect, prime-factor, ratio-optimal, moving-knife structure algorithms
- Metrics: district-level PP distribution (min, median, mean, max), state-level mean PP, PP–Reock correlation, projection correction magnitude by state

## Test Invariants

- L0: PP of a circle approximated by a regular 1000-gon is within 0.001 of 1.0; PP is always in $[0, 1]$ for any valid polygon; PP of a square is $\pi/4 \approx 0.785$
- L1: on a $4 \times 4$ grid of unit squares with two equal districts, PP is computed without panic and matches the formula value for the rectangular geometry within 0.001
- L2: NC 2020 ratio-optimal mean PP $\geq 0.22$ (regression test); projection normalization increases mean PP by $\geq 0.01$ for NC coastal districts

## Legal / Practitioner Value

Polsby-Popper is the most frequently cited compactness metric in redistricting litigation. It appeared in the North Carolina legislative redistricting litigation (2017–2020), was referenced in expert reports in Gill v. Whitford, and is required by statute in several states (e.g. Ohio's 2015 redistricting reform). Courts value PP because it is a single interpretable number on a $[0, 1]$ scale; expert witnesses can explain it in minutes. This paper provides certified PP values for bisect-generated plans alongside the boundary-normalization methodology needed to make PP legally defensible for coastal states.

## Section Structure

§1 Introduction, §2 Mathematical Definition and Properties, §3 Projection and Normalization, §4 Implementation in bisect-analysis, §5 Empirical Comparison Across Structure Algorithms, §6 PP–Reock Correlation Analysis, §7 Legal Usage and Statutory Survey, §8 Conclusion
