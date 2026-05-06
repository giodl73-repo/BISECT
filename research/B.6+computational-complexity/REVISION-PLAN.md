# Revision Plan — B.6 Computational Complexity of Recursive Bisection for Redistricting
Round 1 avg: 2.4/4
Round 2 avg: 3.2/4 (Karypis 3, Rodden 3, Duchin 3, Stephanopoulos 4, Liang 3)
Round 3 avg: 4.0/4 (Karypis 4, Rodden 4, Duchin 4, Stephanopoulos 4, Liang 4)

## STATUS: ACCEPTED (2026-05-05) — All P1 items resolved, Round 3 avg 4.0/4

---

## P1 — Required (Round 1 → Round 2)

- [x] Fix the NP-hardness reduction: changed source from Planar Graph Bisection
  (Garey-Johnson 1976, disconnected parts) to Connected Planar Graph Bisection
  (Dyer and Frieze 1985), which explicitly requires connected parts.
  Proof sketch updated in sections/03-methodology.tex; related work section
  updated in sections/02-related-work.tex with full citation of Dyer & Frieze.
  New bib entry `dyer1985partitioning` added to references.bib.
- [x] Fix the approximation ratio derivation: removed O(sqrt(log n)) claim for
  METIS. Proposition 1 now states no formal approximation ratio is known for
  METIS's heuristic. "Context" paragraph explains why ARV does not apply
  (ARV is for Sparsest Cut; FM local search has no SDP connection).
  Updated in sections/03-methodology.tex.
- [x] Fix the runtime theorem's niter assumption: Theorem 2 now states
  O(niter · n log k) with niter = 100 fixed. Explicitly acknowledges that
  niter = 100 > log(8,057) ≈ 13 for California. Updated in
  sections/04-results.tex.
- [x] Fix the OLS regression to account for k=1 states: excluded k=1 states
  (7 states: AK, MT, ND, SD, VT, DE, WY) from OLS fit. Footnote explains
  exclusion rationale (no bisection step; near-zero runtime regardless of n).
  OLS now reported for 43 states. Updated in sections/04-results.tex.
- [x] Report runtime standard deviation across 10 runs in Table 1. Added
  mean ± std.dev. for all representative states.
- [x] Add the log k covariate test: fit log T against log n and log(log k)
  separately. Deferred through R2; resolved in Round 3 (see below).
- [x] Qualify the P=NP framing for non-technical audiences (Section 5.1):
  added sentence clarifying P=NP is universally believed to be false.
- [x] Distinguish theoretical approximation bound from empirical performance:
  Section 5.2 now separates "no known algorithm achieves O(log n)" from
  "empirically within 3% of best of 10,000 seeds." 4.1× claim removed.
- [x] Clarify feasibility vs. optimisation: added explicit remark after
  Theorem 1 distinguishing finding any valid partition (trivially solvable)
  from finding the minimum-cut partition (NP-hard).

## P1 — Remaining for Round 3 (all resolved)

- [x] Add the log k covariate test: two-variable OLS of log T against log n
  and log(log k). Result: b = 1.05 ± 0.03, c = 0.08 ± 0.15,
  95% CI [-0.07, 0.23], p = 0.41, Delta-AIC < 2. n-dominance confirmed.
  Added as dedicated paragraph in sections/04-results.tex.
- [x] Confirm excluded k=1 state list explicitly in text (Liang R2): footnote
  now lists all seven states by name (Alaska, Delaware, Montana, North Dakota,
  South Dakota, Vermont, Wyoming) and confirms "43 states (k >= 2) form
  the OLS sample".

## P2 — Suggested (Round 1 → Round 2)

- [ ] Provide full proofs (not just sketches) in an appendix for Theorem 1.
  All reviewers note this; Duchin and Karypis particularly emphasise
  precision needed for Dyer & Frieze (1985) planar-graph case citation
  (need to pin to Theorem 2 of their paper).
- [ ] Test O(n^1.07) claim at block-group resolution.
- [ ] Provide all 50-state runtime data as a supplementary table (state, n,
  k, mean runtime, std dev, ms/tract) — Liang R2 maintains this.
- [ ] Add a runtime projection for the 2030 Census (~78,000–82,000 tracts).
  Rodden and Karypis both request this; straightforward one-sentence
  calculation.
- [ ] Cross-reference B.7 more explicitly (Rodden, Stephanopoulos R2):
  add parenthetical describing what B.7 is.
- [ ] Add a paragraph connecting NP-hardness to the due process legal
  argument (Stephanopoulos R2).
- [ ] Pin Dyer & Frieze (1985) citation to their Theorem 2 (planar graph
  case) explicitly — Karypis and Duchin R2.
- [ ] Add clarification that the 3% gap is relative to best-of-10,000-seeds,
  not the true optimum — Duchin R2 asks for one more sentence making this
  explicit.
