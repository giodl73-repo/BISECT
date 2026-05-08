# Population-Weighted Compactness (Moment of Inertia) in Algorithmic Redistricting

**Series**: K.6
**Status**: Accepted 3.5/4
**Target**: American Journal of Political Science

## Algorithm / Subject

Empirical study of Population-Weighted Compactness (PWC), the moment-of-inertia compactness metric defined as $\text{PWC} = \sum_i \text{pop}_i \cdot d(\text{centroid}, i)^2 / \text{total\_pop}$, where $d$ is the Euclidean distance from tract centroid $i$ to the district centroid, and weights are by tract population. Unlike area-based and perimeter-based metrics, PWC captures representation quality: a district where voters live far from the district centre is harder to represent than a geometrically compact but population-dispersed district. Compares PWC values across all four bisect structure algorithms for NC, WI, and TX under 2020 census data, and documents the weak correlation between PWC and PP.

## Key Claims

1. PWC captures a dimension of compactness distinct from geometry: the correlation between PWC and PP across all districts produced by bisect algorithms on NC 2020 is $r < 0.4$, confirming that geometric compactness (PP) and representational compactness (PWC) measure different properties.
2. The prime-factor structure algorithm minimises PWC better than standard-bisect due to hierarchical nesting: prime-factor's nested district structure aligns population concentrations with district centroids, producing mean PWC 18% lower than standard-bisect on NC 2020 (smaller PWC = residents closer to centre = better representational compactness).
3. PWC is sensitive to population clustering patterns: states with concentrated urban centres (TX) exhibit high PWC variance across districts, while states with more dispersed population (WI) exhibit lower variance, making PWC a state-specific rather than universal compactness benchmark.

## Layer

Standalone

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), 2020 census
- Compare across: standard-bisect, prime-factor, ratio-optimal, moving-knife structure algorithms
- Metrics: district-level PWC (raw and normalised), state-level mean and variance of PWC, PWC–PP correlation, PWC–Reock correlation, prime-factor vs standard-bisect PWC comparison

## Test Invariants

- L0: PWC = 0 for a single-tract district (all residents at centroid); PWC is always $\geq 0$; PWC increases monotonically as tracts are moved further from the district centroid (held population constant); PWC is invariant to district relabelling (renaming district 1 to district 2 does not change the PWC of either district)
- L1: on a $3 \times 3$ grid of unit tracts with equal population, the district centroid is at the geometric centre; PWC equals the sum of squared distances from each tract centroid to the grid centre, divided by 9 — verify this analytically and confirm the implementation matches within $10^{-6}$
- L2: NC 2020 prime-factor mean PWC $<$ standard-bisect mean PWC (regression test confirming hierarchical nesting claim); PWC–PP Pearson correlation on NC 2020 is $< 0.4$ across all structure algorithms

## Legal / Practitioner Value

Population-Weighted Compactness is the redistricting metric most directly tied to representation quality. While PP and Reock measure geometric shape, PWC measures how efficiently a district serves its residents: a district where most voters live far from the district centre imposes higher travel and coordination costs on constituents and their representative. PWC has been referenced in academic redistricting literature (Fryer and Holden 2011 used a moment-of-inertia formulation) and aligns with the "community of interest" criterion in state redistricting guidelines. Courts that emphasise representational criteria — rather than purely geometric criteria — may find PWC more relevant than PP or Reock. This paper provides the first systematic comparison of PWC across algorithmic redistricting approaches and documents prime-factor's advantage for population-centred districts.

## Section Structure

§1 Introduction (Representational Compactness vs Geometric Compactness), §2 Mathematical Definition (Moment of Inertia Formula), §3 Population Weighting and Centroid Computation, §4 Empirical Comparison Across Structure Algorithms, §5 PWC–PP and PWC–Reock Correlation Analysis, §6 Prime-Factor Hierarchy and PWC Minimisation, §7 Legal and Practitioner Usage, §8 Conclusion
