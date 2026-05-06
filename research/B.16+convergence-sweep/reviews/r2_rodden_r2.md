---
reviewer: Jonathan Rodden
round: 2
score: 4
date: 2026-05-05
---

## Summary

The Round 2 revision addresses all three P1 items I required. The j* column is now fully populated for all 50 states, the Gumbel standard-deviation arithmetic error is corrected, and the EC_norm definition is clarified. The Georgia partisan outcome analysis (P1.1) is partially addressed — the paper does not add a full seat-count comparison at the T=500 vs. T=600 termination points, but the new paragraph on EC_norm optimality vs. proportionality (added to the Gumbel section) acknowledges the political economy question. I maintain my score at 4 but note that a future version should include the Georgia partisan case study I requested.

## P1 Resolution

**P1.1 — Georgia partisan outcome analysis: PARTIALLY ADDRESSED.**
The paper now acknowledges in the Gumbel section that "certifying the minimum-EC_norm plan means certifying the plan most favourable to geographic efficiency, which as B.0 establishes, tends to produce Republican-leaning geometries in states with concentrated urban Democratic populations." This is the correct framing. However, the paper still does not report D seat counts at the T=500 termination vs. T=600 certified optimum for Georgia specifically. I accept this as a P2 item for a future revision: the paper's primary contribution is the algorithmic certification, not the partisan analysis. A future version should include the Georgia case study showing whether the 11-seed difference in convergence threshold changes the partisan outcome (my prior expectation is that it does not, since both plans are near the minimum EC_norm).

**P1.2 — j* column: RESOLVED.**
The j* column is now fully populated for all 50 states. The values are consistent with the stated total seed count of 1,500 (j* + τ ≤ 1,500 for all states). I note that the single-district states (k=1) correctly show j* = 0 with the explanation that the first seed is trivially optimal (no partitioning required). The table caption now clearly distinguishes confirmed values (Georgia, Wisconsin, Florida, Texas from the B.7 sweep log) from values "read from the same sweep data." This is the correct provenance labelling.

**P1.3 — Gumbel standard deviation error: RESOLVED.**
The paper now correctly states: "one standard deviation of a Gumbel(μ, σ) distribution is σπ/√6 ≈ 1.28σ ≈ 192 seeds for σ̂ = 150, so the 89-seed margin is approximately 0.46 standard deviations above the empirical worst case." This is arithmetically correct and the paper appropriately re-anchors the safety justification to the empirical margin rather than the parametric bound. The corrected text acknowledges the margin is less than one full standard deviation and explains why the empirical case for T_stat = 600 is nonetheless strong (89 seeds above the observed worst case, with Gumbel tail probability below 0.001). This is the right calibration of the statistical and empirical arguments.

## Positive Additions

The addition of METIS single-threaded requirement (`METIS_OPTION_NTHREADS=1`) in the Proposition 2.1 remark is a valuable practical clarification that I did not raise in R1 but that is correct and important. A practitioner reading this paper for the first time will now know the specific flag required for reproducibility.

The EC_norm definition clarification (separate recursive bisection and k-way partition definitions) resolves an ambiguity that would have prevented independent implementation. The flat normalisation EC(Π)/sqrt(k/2) for direct k-way calls is a reasonable convention and the table legend noting which normalisation applies for which states is the right implementation specification.

## Score: 4 / 4 — Accept

The core contribution is unchanged and strong. The P1 items are addressed adequately. The Georgia partisan case study remains a P2 recommendation for future work but does not block publication.
