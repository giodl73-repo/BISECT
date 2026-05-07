# Review: G.0 — Ensemble Methodology
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 2
**Score**: 3/4

## Summary

The revision addresses all three blocking issues from Round 1. The ReCom stationary distribution claim has been corrected; the Rhat threshold is now consistent at 1.1 throughout; and the CS-to-ensemble bridge has been substantially rewritten. The paper is now technically sound as a framework document. Remaining issues are moderate-priority and do not affect the downstream G-series' validity.

## Blocking Issues — Resolution

**B1 (ReCom stationary distribution): Resolved.** Table 1 now correctly describes ReCom's stationary distribution as "Approximate (spanning-tree proportional)" and Section 2.2 contains the requisite paragraph distinguishing approximate-uniform from exact-uniform. The cross-reference to DeFord 2021 Section 3 is present. This is the correct characterisation.

**B2 (Rhat threshold inconsistency): Resolved.** Section 4.1 now states $\hat{R} < 1.1$ throughout, and the diagnostic standards table is consistent. The footnote explaining why 1.1 rather than 1.01 is appropriate for discrete redistricting statistics is precisely what was needed — it will deflect challenges from reviewers who apply the Vehtari 2021 continuous-parameter recommendation to this context.

**B3 (CS bridge false analogy): Resolved.** The revised Section 5.4 replaces "both T and ESS measure robustness to additional sampling" with the correct framing: "ConvergenceSweep T=600 and ensemble ESS are independent certificates of robustness: ESS measures MCMC mixing in the ensemble space; T=600 measures monotonic convergence in the seed-quality space." This is precisely what the paper needed. The orthogonality framing is accurate and will withstand scrutiny in expert witness settings.

## High-Priority Issues — Resolution

**H1 (Notation): Partially resolved.** I note that $q_f(P^*)$ notation appears in G.0's Section 3 but the body of the bridge section still refers to $\pi_\pp$ in several places (specifically Section 5.2 and 5.3). If G.1 has cascaded the notation change, G.0 should be consistent. Minor cleanup needed.

**H2 (Rhat citation): Resolved.** Vehtari 2021 is now cited as primary for the formula; Gelman 1992 retained as original reference. Correct.

**H3 (ESS threshold): The threshold in the text is still stated as 400.** Section 4.2 reads "an ESS of at least 400 per statistic is required." G.4 uses 500. The REVISION-PLAN called for changing this to 500 consistent with G.4. This has not been done. Fix required.

**H4 (Hamming reference plan): Resolved** via cross-reference to G.4.

## Moderate-Priority Issues — Status

**M1 (Table 1 compactness weight):** Not yet addressed. The footnote for Metropolized Forest ReCom compactness weighting should be added. Minor.

**M2 (Rodden effect):** A paragraph has been added in Section 6.1 noting ensemble-median is not normatively neutral in sorted states. This satisfies the revision request. The prose accurately distinguishes geographic-median from proportional-fairness.

**M3 (Legal accuracy post-Rucho):** The sentence distinguishing federal court limitations from state court viability has been added. Correct.

**M4 ("Exactly one AR plan" precision):** Now correctly states uniqueness is tied to SHA-256-derived seed sequence and census release. Good.

## Remaining Issues

The paper still defines ESS threshold as 400 (Section 4.2) while G.4 uses 500 and the REVISION-PLAN explicitly calls for 500. This should be fixed before final publication. It does not affect R2 scoring since it was a high-priority (not blocking) item and does not affect downstream G-series validity — G.4 is the authoritative diagnostic paper.

## Recommendation

Accept with minor revisions (fix ESS threshold 400→500, confirm $q_f$ notation consistency throughout). The blocking issues are all resolved. The paper is now adequate as a framework document for the G-series.
