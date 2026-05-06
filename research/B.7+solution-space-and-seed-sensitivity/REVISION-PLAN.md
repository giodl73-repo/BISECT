# Revision Plan — B.7 Solution Space and Seed Sensitivity in METIS Redistricting
Round 1 avg: 3.2/4
Round 2 avg: 4.0/4 (Karypis 4, Rodden 4, Duchin 4, Stephanopoulos 4, Liang 4)

## Round 2 Status

P1 items addressed this revision:
- [x] 500M → 500K correction — abstract, introduction (contributions list), conclusion (with arithmetic parenthetical)
- [x] Gumbel distributional claim removed — Section 4.4 and introduction now use "empirical CDF" framing
- [x] "Any state" partisan claim qualified to WI and NC (the two highest-CV states) — abstract, results, conclusion
- [x] Upper-bound argument added — WI/NC as highest-CV states implies 2-seat range bounds all 50 states
- [x] Cross-census partisan caveat added to limitations section

P1 items still open (from R2 reviews, all downgraded to P2):
- [ ] DIA seed rank percentile across states (fraction of seeds the DIA seed beats) — Karypis R2 P2
- [ ] Distributional summary statistics for last-improvement seed (mean, median, 95th pct across 50 states) — Liang R2 P2
- [ ] EC_norm justification for within-state seed sensitivity analysis — Karypis R2 P2
- [ ] 95% CI on the mean approximation gap difference (DIA seed vs. median seed) — Liang R2 P2
- [ ] Sign test or Wilcoxon for the DIA gap comparison — Duchin R2 P2

## P1 — Required (original list)

- [x] Fix the 500M/500K error in abstract and all occurrences. — **DONE**
- [x] Justify or remove the Gumbel model claim: replaced with empirical CDF framing; "47 of 50 states show last improvement before seed 600." — **DONE**
- [ ] Justify the normalised edge cut metric EC_norm for within-state seed sensitivity — **STILL OPEN (downgraded to P2 in R2)**
- [ ] Provide the DIA seed rank percentile in addition to the approximation gap — **STILL OPEN (downgraded to P2 in R2)**
- [ ] Use sign test or Wilcoxon for the DIA gap vs. median gap comparison — **STILL OPEN (downgraded to P2 in R2)**
- [x] Expand the partisan analysis to all 43 non-trivial states OR qualify claim to WI and NC — **DONE** (qualified to WI and NC with upper-bound argument)
- [ ] Add a formal statement of what T=600 guarantees (probabilistic or empirical) — **PARTIALLY DONE** (empirical: 47/50 states; formal bound still absent, downgraded to P2)

## P2 — Suggested

- [ ] Fit a parametric distribution to the last-improvement seed indices — report mean, median, 95th pct, max across 50 states
- [ ] Report full partisan variance data (min/max D seats, SD) for all 43 non-trivial states in a supplementary table
- [ ] Add a CDF plot of seed-level approximation gaps for at least one high-variance state (GA or NC)
- [ ] Test whether seed sensitivity is correlated with geographic polarisation vs. graph-structural properties
- [ ] Report seed sensitivity for at least one non-default weight mode (county-sticky or GeoSection)
- [ ] Add a narrative timeline for the SHA-256 unpredictability argument
- [ ] Report 95% confidence intervals on mean approximation gap for DIA seed and median seed in Table 2
- [ ] DIA seed rank percentile distribution across states
- [ ] EC_norm justification for within-state use (or switch to raw edge cut)
