# Review: G.3 — Compactness Distribution Position
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 2
**Score**: 3/4

## Summary

The correlation formula correction resolves the dimensional error and produces substantially different numerical results that are more consistent with the observed data. The 75th/68th percentile inconsistency is resolved via the G.1 NC PP correction. The paper is now statistically coherent. Remaining gaps are the correlation confidence intervals (not added despite being explicitly specified in the REVISION-PLAN) and the skewness source.

## Changes Evaluated

**Correlation formula (B2): Resolved.** The corrected formula $\Delta S_D = \rho \cdot (\sigma_{S_D}/\sigma_{PP}) \cdot \Delta\bar{PP}$ is dimensionally correct. The NC result (−0.12 seats, essentially zero) is consistent with the 54th percentile partisan outcome. The Georgia result (−1.03 seats) explains the full deviation. The corrected formula is internally consistent with the empirical data — which the original formula was not. Importantly, I had computed in Round 1 that the correct formula would give −0.60 seats for NC (using the then-current PP value of 0.412); the revised formula with the corrected PP of 0.337 gives −0.12 seats, which is much more consistent with the observation. The formula correction and the PP correction work together.

**75th/68th inconsistency (B1): Resolved** via NC PP correction. With NC PP = 0.337 and the 68th percentile, the 75th percentile value of 0.355 is correctly above the AR score, and "falling between 75th and maximum" is no longer the claim.

**Significance of correlations:** I note that the statistical significance calculation I performed in Round 1 ($t \approx -8.7$ for GA, $t \approx -4.8$ for NC) confirms all correlations are highly significant. If the paper has not added this significance note, it should.

## Unresolved Issues

**H1 (Correlation confidence intervals): Not added.** The REVISION-PLAN explicitly specified the 95% CIs:
- NC: $(-0.25, -0.11)$
- WI: $(-0.16, -0.02)$
- GA: $(-0.38, -0.24)$
- PA: $(-0.18, -0.04)$

These have not been added to the table. This is a direct miss. The intervals are narrow enough that all correlations remain significant — adding them strengthens the paper.

**H2 (Skewness source): Not addressed.** The values (+0.3 to +0.7) still have no source.

**$\rho$ direction claim ("any compact algorithm"):** Now labeled as "Conjecture (to be tested in B.0)." This is the appropriate hedging for an unverified claim.

## Recommendation

Accept with minor revisions. Add the correlation confidence intervals (a four-row table, values already computed in the REVISION-PLAN) and source the skewness values. The blocking issues are resolved and the paper is statistically sound.
