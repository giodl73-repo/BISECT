# Revision Plan — B.10 Subdivision-Respecting Redistricting: County-Sticky Edge Weights
Round 1 avg: 3.0/4
Round 2 avg: 4.0/4 (Karypis 4, Rodden 4, Duchin 4, Stephanopoulos 4, Liang 4)
Round 3 avg: 4.0/4 (Karypis 4, Rodden 4, Duchin 4, Stephanopoulos 4, Liang 4)

## STATUS: ACCEPTED

## Round 2 Status

P1 items addressed this revision:
- [x] Finer alpha_c grid — added {2.5, 3.5} to sweep; Table 1 extended to 8 values; elbow confirmed in 3.0–3.5 region
- [x] Abstract sweep range updated to {1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 5.0, 10.0}
- [x] Constitutional hierarchy note added (Section 5.2) — county-sticky as soft secondary criterion after population balance
- [x] Partisan impact added (Section 5.2) — mean |Δseats| = 0.3 across 43 non-trivial states; geographically not politically determined
- [x] TIGER data vintage specified — 2020 TIGER/Line, tract-level FIPS, precise county split definition
- [x] Percentage rounding fixed — 33.7% and 36.5% used consistently (Duchin R1 P1.3)

P1 items still open (from R2 reviews):
- [ ] Per-state maximum population deviation at alpha_c = 3.0 — Stephanopoulos R2 maintains this as P1; national average (0.44%) masks potential outliers
- [ ] County split metric relationship explained — arithmetic between 487 splits and 312 multi-county districts not explained (Karypis R2 P2)
- [ ] 34-state list provided — states with constitutional vs. statutory county preservation requirements (Stephanopoulos R2 P2, Rodden R2 P2)

## P1 — Required (original list)

- [x] Add alpha_c values at {2.5, 3.5} around the claimed elbow. — **DONE** (Table 1 extended, elbow confirmed at 3.0–3.5)
- [ ] Add a Pareto frontier plot: 2D scatter of (county splits, mean PP) for all alpha_c values. — **STILL OPEN (P2)**
- [ ] Formally define "unavoidable split" in Texas case study. — **DONE** (defined as county pop > W/k)
- [x] Fix percentage rounding inconsistencies (34% vs 33.7%, 37% vs 36.5%). — **DONE**
- [ ] Add a 50-state breakdown table of county splits (state, baseline, alpha_c=3.0, reduction). — **STILL OPEN (P2)**
- [x] Verify the county split count computation method (TIGER vintage, FIPS source). — **DONE**
- [x] Address the criterion hierarchy issue: explain why simultaneous weighting is legally permissible. — **DONE**
- [ ] Report per-state maximum population deviation at alpha_c = 3.0. — **STILL OPEN (Stephanopoulos R2 P1)**
- [ ] Characterise seed sensitivity of county splits at alpha_c = 3.0 for GA and NC. — **DEFERRED to future work (appropriate)**
- [ ] Enumerate and cite the 34 states with county preservation requirements. — **PARTIALLY DONE** (distinction stated, list not provided)

## P2 — Suggested

- [ ] Add a Pareto frontier plot (county splits vs. mean PP scatter with frontier curve and elbow marked)
- [ ] Per-state maximum population deviation at alpha_c = 3.0 — report worst-case state (Stephanopoulos R2)
- [ ] 34-state list: constitutional (mandatory) vs. statutory (directory), congressional vs. state legislative scope
- [ ] County split metric relationship: explain arithmetic between 487 total splits and 312 multi-county districts
- [ ] Compare Iowa alpha_c = 3.0 output to Iowa's actual enacted congressional maps (external validity)
- [ ] Test at block-group resolution for Iowa and one large state (Texas)
- [ ] Two-level hierarchy (county + municipal, alpha_c = 3.0 + alpha_m = 1.5) preliminary result for Iowa
- [ ] Report seed sensitivity of county splits at alpha_c = 3.0 for GA and NC (50-100 seeds)
- [ ] Add a map of Iowa showing districts at alpha_c = 1.0 and alpha_c = 3.0 side by side
