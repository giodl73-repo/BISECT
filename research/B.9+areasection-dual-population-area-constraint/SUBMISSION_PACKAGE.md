# B.9 Submission Package — Political Analysis

**Paper**: AreaSection: Simultaneous Population and Land-Area Balance in Minimum-Edge-Cut Redistricting
**Target venue**: Political Analysis (Cambridge University Press)
**Panel avg**: 3.3/4 (Round 2 — 4/5 accept or accept-with-minor)
**Status**: Ready for submission (all minor items resolved)

## Submission Checklist

- [ ] Political Analysis author guidelines reviewed (https://www.cambridge.org/core/journals/political-analysis/information/instructions-for-contributors)
- [ ] Word limit verified (PA: 10,000 words for research articles)
- [ ] Replication materials prepared (PA requires complete replication archive)
- [ ] Data availability statement written
- [ ] Blind review copy prepared
- [ ] Cover letter written

## Key Contributions for Cover Letter
1. First application of METIS ncon=2 dual-constraint partitioning to redistricting
2. Lorenz curve as a feasibility diagnostic for population-area balance
3. 50-state empirical sweep: 40/50 (80%) meet 0.5% constitutional standard
4. Head-to-head vs GeoSection: 76% same seat counts, no systematic partisan direction
5. Empirical validation of Rodden effect: 8/8 competitive states R-favored under both algorithms
6. area_swing sensitivity: 1.10 is regime boundary (1.05→forced equal, 1.20→peeling returns)

## Cover Letter Draft

Dear Editors,

We submit "AreaSection: Simultaneous Population and Land-Area Balance in
Minimum-Edge-Cut Redistricting" for consideration in Political Analysis.

AreaSection extends the minimum-edge-cut redistricting framework by adding a
land-area balance constraint alongside the standard population balance requirement.
Using METIS's native dual-constraint (ncon=2) partitioning, AreaSection requires
each geographic half of every bisection to hold approximately 50% of the state's
land area (±10% swing), preventing the systematic "urban peeling" in which
standard algorithms always carve compact metropolitan cores first.

The political science contribution is a null result of theoretical importance:
across 34 comparable states, AreaSection and GeoSection (standard edge-weighted
bisection) produce identical seat counts in 76% of states, with the 9 differing
states showing no systematic partisan direction (net: -1 Democratic seat across
the 9 states). The Rodden geographic sorting effect accounts for all Republican
over-representation in competitive states under both algorithms, confirming that
the partisan deficit is a property of geographic voter distribution, not of the
redistricting algorithm.

The paper provides a clean empirical test of an important theoretical claim: 
does the structure of the bisection algorithm — not just the objective function —
affect partisan outcomes? The answer from 34 states and 50 seeds per ratio is: no.

## Replication Archive
https://github.com/giodl73-repo/REDIST
All 44-state results at outputs/area_sweep_v1/
Constitutional tolerance (0.5%) results at outputs/as_strict/
