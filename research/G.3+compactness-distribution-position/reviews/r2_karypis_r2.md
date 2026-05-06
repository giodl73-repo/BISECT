# Review: G.3 — Compactness Distribution Position
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 2
**Score**: 3/4

## Summary

The correlation formula correction is the most technically important fix and it has been made correctly. The revised formula $\Delta S_D = \rho \cdot (\sigma_{S_D}/\sigma_{PP}) \cdot \Delta\bar{PP}$ is dimensionally correct. The Georgia result — the corrected formula gives exactly $-1.03$ seats, matching the observed deviation — is a significant improvement. The paper's internal consistency is substantially better than in Round 1.

## Changes Evaluated

**Correlation formula (B2): Resolved and strengthened.** The correct regression formula produces a result that is actually more persuasive: the GA deviation of −1 seat is now fully explained by the compactness-partisan correlation, not just partially accounted for. This is the paper's strongest empirical finding in revised form — the corrected formula turns a partial explanation into a complete one.

**NC PP table update (B1 propagation): Resolved.** The percentile table in Section 3 correctly shows NC PP = 0.337, consistent with G.1's revised value. The 68th percentile is now internally consistent with the score being modestly above the ensemble mean (z = 0.61), not far above it.

**"Between 75th and maximum" claim:** I note that with NC now at the 68th percentile (below the 75th), the Section 2.3 claim that AR falls "between the 75th percentile and the ensemble maximum for all states" must be gone or revised. If this phrase remains anywhere in the paper, it needs to be updated to reflect the actual range of 61st–72nd.

## Remaining Technical Concerns

From a graph-partitioning standpoint, the most interesting remaining question is about Metropolized Forest ReCom (Autry 2021). If the Metropolis correction shifts the compactness distribution toward higher-PP plans (which is plausible since the correction upweights plans with fewer spanning-tree choices), then the AR plan's position in a Metropolized ensemble could differ from its position in the standard ReCom ensemble. The paper still doesn't address this. It is a moderate concern but not blocking.

The skewness values (+0.3 to +0.7) should be sourced. If these come from the author's own chain runs, that should be stated.

## Recommendation

Accept with minor revisions. The blocking issues are resolved and the corrected formula actually strengthens the paper's argument. Verify the "between 75th and maximum" claim has been removed, source the skewness values, and address the Metropolized ReCom question as a minor caveat.
