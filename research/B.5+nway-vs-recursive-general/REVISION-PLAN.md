# Revision Plan — B.5 N-Way vs. Recursive Bisection: A General Architectural Comparison
Round 1 avg: 2.8/4
Round 2 avg: 3.6/4 (Karypis 3, Rodden 4, Duchin 3, Stephanopoulos 4, Liang 4)
Round 3 avg: 4.0/4 (Karypis 4, Rodden 4, Duchin 4, Stephanopoulos 4, Liang 4)

## STATUS: ACCEPTED

## Round 2 Status

P1 items addressed this revision:
- [x] Chamber count corrected to "450 chamber-year combinations" (was "727") — abstract, intro, conclusion
- [x] Statistical independence fixed — inference restricted to congressional chambers (n=50 independent); senate/house treated as descriptive
- [x] Figure 1 replaced by Table 2 (PP difference by k-bin) — "not shown; see supplementary data" removed
- [x] Partisan comparison paragraph added (Section 5.4) — mean |Δseats| = 0.2, WI/NC at median seed
- [x] DIA scope narrowed — congressional (DIA-governed) distinguished from state legislative (state-law-governed)
- [x] Rucho v. Common Cause (2019) citation added to Section 5.3

P1 items still open (from R2 reviews):
- [ ] Fix the mechanism explanation in Section 5.1 — "stretching across geographic features" is not a METIS mechanism; state as per-bisection FM on smaller subgraphs (Karypis R2, Duchin R2)
- [ ] Fix the coarsening-levels conflation in Section 2.1 — n-way uses a single coarsening phase, not O(log k) levels (Karypis R2)
- [ ] Prime-k mitigation: add before/after PP numbers for PA k=17 case to demonstrate 60% recovery (Karypis R2, Duchin R2)
- [ ] Add Bethune-Hill v. Virginia State Board of Elections (2017) citation for race-neutrality anchor (Stephanopoulos R2)
- [ ] Add 95% confidence intervals to Tables 1 and 3 (Liang R2)

## P1 — Required (original list)

- [x] Fix mechanism explanation in Section 5.1: the RB advantage is not "districts stretching across geographic features" but rather per-bisection FM operating on smaller, tighter-balance subgraphs. Restate the mechanism correctly citing METIS's actual operation. — **STILL OPEN in R2**
- [x] Fix the coarsening-levels conflation in Section 2.1: n-way (kmetis) uses a single coarsening phase, not O(log k) levels. RB uses O(log k) bisection levels. — **STILL OPEN in R2**
- [x] Clarify the prime-k mitigation in Section 5.2: "rounding to nearest power of two" is undefined when k is fixed by apportionment. Restate as: after the asymmetric bisection tree is built, apply one additional FM inter-district sweep on boundaries adjacent to asymmetric splits. Show before/after PP numbers for the PA k=17 case. — **PARTIALLY DONE (language fixed, PP numbers still missing)**
- [x] Fix the 727-chamber count: changed to "450 chamber-year combinations" throughout. — **DONE**
- [x] Add partisan comparison between RB and n-way outputs. — **DONE** (Section 5.4, |Δseats| = 0.2)
- [x] Include or replace Figure 1. — **DONE** (replaced by Table 2, k-bin PP differences)
- [ ] Add confidence intervals to Tables 1 and 3 (95% CI on mean PP difference and runtime difference). — **STILL OPEN**
- [x] Address the statistical clustering problem: restrict inference to congressional chambers (n=50 independent). — **DONE**
- [x] Narrow the DIA scope claim: state house and senate redistricting governed by state law, not DIA. — **DONE**
- [x] Add legal citation anchor for partisan-neutrality argument: Rucho v. Common Cause (2019). — **DONE**

## P2 — Suggested

- [ ] Add box plots of per-district PP distribution by strategy and chamber type.
- [ ] Test SCOTCH or KaHyPar on a 5-state subset.
- [ ] Add a figure showing (PP_RB - PP_NW) vs. state geographic compactness (Reock score).
- [ ] Report PP values at block-group resolution for the four state-level case studies (CA, TX, NH, PA). — flagged again in R2 by Duchin and Liang
- [ ] Add cross-census replication: report Cohen's d for 2000 and 2010 census data.
- [ ] Provide a replication script or Makefile target.
- [ ] Discuss minority-representation implications at large k for state house chambers.
- [ ] Add Bethune-Hill (2017) citation for race-neutrality (Stephanopoulos R2 P2).
