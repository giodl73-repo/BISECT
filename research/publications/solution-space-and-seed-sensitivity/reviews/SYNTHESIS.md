# Synthesis: Round 1
**Paper**: The Solution Space of Minimum-Edge-Cut Redistricting: Seed Sensitivity and Partisan Variance
**Date**: 2026-05-01
**Reviewers**: R-50 (Ullman), R-1 (Liang), R-37 (Polikarpova), R-44 (Steinhardt), R-30 (Zhang)

---

## Scores

| Reviewer | Score |
|----------|-------|
| R-50 Ullman | 2.5 / 4 |
| R-1 Liang | 2.5 / 4 |
| R-37 Polikarpova | 3.0 / 4 |
| R-44 Steinhardt | 3.0 / 4 |
| R-30 Zhang | 3.0 / 4 |
| **Average** | **2.8 / 4** |

**Stage recommendation**: REVISION — addressable concerns, strong core.

---

## Panel Verdict

The paper has a genuinely original contribution (the Fiedler certificate + CompactBisect algorithm) and a real empirical result (WI +12.5pp improvement). The average score of 2.8 reflects a paper that is close to publishable but has specific gaps that need closing. All five reviewers found the core ideas sound; no reviewer recommended rejection.

---

## P1 Items (Must Fix)

### P1-A: Cheeger formula error in §3.3
**Reviewers**: Ullman (P1.1), Polikarpova (P1.1 resolution)
The paper states EC_min ≥ λ₂ × W_total / 4 but the correct unnormalized Laplacian bound is EC_min ≥ λ₂ × n / 4 (vertex count, not edge weight sum). The code already uses the correct formula. Fix the paper formula to match. Note: the actual PP upper bound derivation is probably fine — it's the citation of W_total vs. n that is wrong.

### P1-B: Proposition 3.1 needs a 4-line proof
**Reviewers**: Ullman (P1.2), Polikarpova (P1.1)
The "proof" of Certificate Immunity is circular. The actual proof is straightforward: for any bisection (L, R), EC(L,R) ≥ EC_min ≥ λ₂·n/4, so P(L) = EC(L,R) + P_ext(L) ≥ λ₂·n/4 + P_ext(L), therefore PP(L) ≤ PP_upper(L). Same for R. Product of bounds gives the GMPP bound. Write this out.

### P1-C: Election data source undocumented
**Reviewers**: Zhang (P1.1)
presidential_by_tract.csv needs a citation, a methodology note (how precinct votes were apportioned to tracts), and a validation metric (R² with precinct-level totals). This is fundamental for a paper making partisan claims. Likely source: Fekrazad DOI 10.7910/DVN/Z8TSH3 (referenced in the onboarding plan but not cited in the paper).

### P1-D: ε-tolerance is an unanalyzed attack surface
**Reviewers**: Steinhardt (P1.1)
The ε = 0.05 threshold in CompactBisect's near-minimum filter is not proven to be robust to adversarial manipulation. Either: (a) add ε to the statutory parameter table so it cannot be chosen post-hoc, or (b) show empirically that the selection outcome is stable across ε ∈ [0.01, 0.20] for the key contested states. The statutory fix (a) is cleaner.

### P1-E: Convergence claim needs cross-state support
**Reviewers**: Liang (P1.1)
The headline "seed is irrelevant" is demonstrated for PA (stable 819 seeds after last improvement) but GA's last improvement was seed 190 of 200 (only 10-seed tail — clearly not converged) and MI at seed 47 of 76. The claim needs either more seeds for GA/MI or a narrower headline. Quick fix: restate the claim as "seed is irrelevant for states where convergence is demonstrated" and provide a formal definition of convergence (e.g., 95th percentile of GEV model within 1% of observed minimum).

---

## P2 Items (Should Fix)

### P2-A: Circular approximation for external perimeters should use exact TIGER values
**Reviewers**: Ullman (P2.2), Steinhardt (P1.2), Zhang (P1.2)
The 2√(πA) approximation can be 30-100% wrong for elongated or coastal tracts. Since exact polygon perimeters are available from TIGER shapefiles (they're already read for the adj.bin build), switch to exact values. This closes an attack surface (Steinhardt) and improves PP computation accuracy (Zhang).

### P2-B: CompactBisect termination specification needed
**Reviewers**: Ullman (P1.3), Polikarpova (P1.2)
Add: (a) a maximum seed budget B, (b) a fallback that publishes the best-achieved plan when ratio < 1−δ after B seeds with the certificate marked "unachieved: ratio = r", (c) a population balance REQUIRES clause. The algorithm must handle the case where no METIS seed satisfies both balance and high GMPP.

### P2-C: Re-run CompactBisect results with release binary
**Reviewers**: Liang (P1.2)
The WI/NC/PA CompactBisect comparison was run with the debug binary. Floating-point ordering differences between debug and release could affect METIS outcomes. Reproduce these 3 results with the release binary.

### P2-D: Fiedler certificate certification ratios not reported
**Reviewers**: Ullman (P2.3)
The paper defines the certificate but never reports what ratios are achieved on real graphs. For PA, what is achieved_GMPP / GMPP_upper? If ratios are near 1.0, the certificate is practically achievable. If ratios are near 0.3, the δ=0.05 threshold is unreachable and the certificate is theoretical only.

### P2-E: Add content-derived seed recommendation to the paper
**Reviewers**: Steinhardt (P3.2)
The plan.md proposes SHA-256(census_release_id ∥ "DIA_SEED_V1") as the seed selection protocol — this is the cleanest defense against seed-choice attacks. It belongs in the Discussion section as the primary statutory recommendation, not just in planning documents.

---

## P3 Items (Nice to Have)

- P3-A: Fix the O(n²) Lanczos claim to O(n × E × max_iter) (Ullman)
- P3-B: Report 48-state vs 50-state totals with best-guess for AK/HI (Zhang)
- P3-C: Add WGS84 caveat for cross-state PP comparisons (Zhang)
- P3-D: Sensitivity analysis of δ ∈ {0.01, 0.02, 0.05, 0.10} for certified region partisan composition (Steinhardt)
- P3-E: Update §3.2 convergence calibration from 100-seed to 1,100-seed PA data (Liang)
- P3-F: Spearman correlation between district PP and partisan lean (stated as future work in §3.4.3 but should be computed for WI/PA with available data)

---

## What's Strong (Do Not Change)

1. **Fiedler certificate concept** — correct, original, legally valuable. The mathematical chain λ₂ → PP upper bound → GMPP upper bound is the paper's primary contribution.
2. **CompactBisect algorithm definition** — Definition 3.4 is clean and implementable. The Huntington-Hill analogy is apt and legally potent.
3. **WI three-way comparison** — the strongest empirical result. Keep it prominent.
4. **SHA-256 partition uniqueness verification** — exactly the right methodology for this claim.
5. **The geometric bias theorem** (in Discussion) — the argument that any compact-respecting algorithm under-represents concentrated parties is important and should be foregrounded.

---

## Estimated Work to Address

- P1-A: 1 line fix (Cheeger formula)
- P1-B: 4 lines (write the proof)
- P1-C: 1 paragraph + citation
- P1-D: Either 1 line (add ε to parameter table) or 1-2h experiment
- P1-E: Run 200 more seeds on GA/MI (4h compute)
- P2-A: Code change + adj.bin rebuild (2h)
- P2-B: 1 paragraph + code addition (2h)
- P2-C: Re-run 3 states (3h compute)
- P2-D: Compute GMPP ratios for existing runs (1h)
- P2-E: Add content-derived seed section to Discussion (30min)

**Total estimated revision effort: ~15-20 hours (mostly compute time)**

---

## Stage Transition

Advancing to: **revision**
Next action: Address P1 items (A-E) as the minimum viable revision, then rebuild release binary and re-run CompactBisect (P2-A through P2-C).
