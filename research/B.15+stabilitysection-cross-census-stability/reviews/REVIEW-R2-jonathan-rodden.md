# Review R2: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Jonathan Rodden (Stanford University, Hoover Institution)
**Expertise**: Political geography, geographic sorting of partisan voters, urban-rural divide, unintentional gerrymandering, comparative electoral systems
**Round**: 2
**Date**: 2026-05-02

## Overall Assessment

The revision is a material improvement. The Iowa case study has been added as Section 4.3.3, and the authors have done the right thing: rather than forcing a premature mechanistic answer, they have clearly articulated two competing hypotheses (Lorenz drift vs. tract redesign) and correctly identified what data would distinguish them. The Jaccard proxy table (Table 4) and the threshold sensitivity table (Table 5) address my statistical concerns directly. The legal section's framing of the 33% instability problem has been substantially clarified.

My remaining concerns are smaller than in Round 1, and the paper is approaching journal submission quality. The principal outstanding issue is that the Iowa $p^*$ computation is still marked "[computation needed]" — the paper knows exactly what number would answer its central empirical question and has not produced it. The legal section's framing of the 67% finding for the 33% unstable states is much better than Round 1 but still slightly under-developed.

## Score: 3.5/4

**My score**: 3.5/4 — Iowa case study and threshold sensitivity address Round 1 major issues; $p^*$ computation still outstanding; legal framing for unstable states improved but not complete.

## Changes Since Round 1: What Was Addressed

### Iowa Case Study (Issue 1 from R1 — Substantially Addressed)

The Section 4.3.3 case study is the paper's best new contribution. The distinction between Hypothesis A (suburban sprawl expanding metro perimeter, reducing the isoperimetric advantage of the asymmetric peel) and Hypothesis B (Census Bureau tract redesign changing the graph topology) is precisely the mechanistic decomposition I requested. The observation that Iowa's 2010 tract count increased 17% due to suburban subdivision is a specific, verifiable claim that directly supports Hypothesis B as a candidate explanation.

The authors correctly identify what would confirm each hypothesis: if the 2020 graph with 2000 populations produces $f \approx 0.49$, Hypothesis B is dominant; if it produces $f \approx 0.18$, Hypothesis A is dominant. This is the Type I/Type II decomposition properly framed as a decision procedure, not just as future work.

What is still missing: the $p^*_{2000}$ and $p^*_{2010}$ values. The paper says "computation needed" and projects $p^*_{2000} \approx 0.25$ and $p^*_{2010} \approx 0.50$. These are projected from the ratio geometry, not computed from the Lorenz curve. The analysis would be substantially stronger if the actual Lorenz proxy values were reported from the sweep output rather than derived by implication from the ratio arithmetic.

### Threshold Sensitivity (Minor Issue from R1 — Fully Addressed)

Table 5 answers the question directly: 15/20/23/26 stable states at $\tau = 0.02/0.05/0.10/0.15$. The explanation that the 67% finding at $\tau = 0.05$ is not a threshold artifact because the four highly unstable states (Iowa at $\Delta f = 0.31$) remain unstable at all thresholds above is exactly the right argument. The calibration of the 5% threshold to "approximately the sampling uncertainty in $f$ for a typical 1,000-tract state at 50 seeds" is a good methodological justification that was not present in Round 1.

### Legal Section (Issue 2 from R1 — Addressed but Incomplete)

The limitations section now acknowledges that CSS for seat-count-changing states should be interpreted with the apportionment-change caveat. The legal section's "strongest litigation targets" paragraph correctly focuses on high-CSS states. The s_seat clarification for seat-count-changing states (MT, TX, NY named as "structurally changed" rather than "unstable") resolves a definitional inconsistency from Round 1.

The 33% instability framing is still handled primarily through the limitations paragraph on Congressional apportionment changes rather than through a direct legal discussion. A court challenging a GeoSection map in Alabama would ask: "you admit 33% of states are not census-stable — why should we accept your stability claim for this state?" The paper's answer (Alabama at $\Delta f = 0.06$ would be classified stable at $\tau = 0.10$; it is a borderline case) is available in the data but not explicitly made in the discussion.

### Jaccard Proxy Table (Issue 3 from R1 — Substantially Addressed)

Table 4 correctly positions the Jaccard proxy as a proxy, not the full computation, and the pseudocode in Section 4.4 now uses the Hungarian algorithm for injective matching — exactly the assignment-problem formulation I and Polikarpova requested. Moving the full Jaccard computation to future work with a clear statement of what files are needed to execute it is the right scoping decision.

## Remaining Issues

### Issue 1: $p^*_{2000}$ Must Be Computed, Not Projected
**Severity**: Medium-High
**Description**: The Iowa case study's Hypothesis A vs. Hypothesis B framing is excellent, but the paper then immediately says "computation needed: $p^*_{2000}$ requires running the full 2000 census sweep for Iowa." This is the paper's most important single number — the smoking gun that would distinguish a geographic explanation from a data-artifact explanation for the most dramatic shift in the dataset. The 2000 sweep is 47/50 complete; Iowa is presumably not one of the three missing large states (CA, TX, FL). If Iowa's 2000 sweep is complete, the $p^*_{2000}$ value is available from the pipeline output and should be reported here, not deferred.

**Recommendation**: Check whether Iowa's 2000 sweep output includes the normalised Lorenz proxy. If so, report $p^*_{2000}$ and $p^*_{2010}$ in Section 4.3.3. Even a value of "projected $\approx 0.25$ from ratio geometry, confirmed from pipeline output" would dramatically strengthen the case study.

### Issue 2: Alabama's Borderline Status Deserves Explicit Legal Discussion
**Severity**: Low-Medium
**Description**: Alabama at $\Delta f = 0.06$ is the clearest borderline case: classified unstable at $\tau = 0.05$ but stable at $\tau = 0.10$. For *Allen v. Milligan* and related VRA litigation, Alabama's geographic stability is legally significant. The paper mentions Alabama in the unstable states list but does not discuss what its near-borderline status means for the legal argument.

**Recommendation**: Add a sentence in Section 5.1 noting that for borderline states (Alabama at $\Delta f = 0.06$, South Carolina at $\Delta f = 0.07$), the legal argument appropriately distinguishes "marginal geographic reorganization" from "qualitative regime change" (Iowa at $\Delta f = 0.31$). This directly addresses the objection that the 33% instability rate is damaging to the geographic determinism claim.

## Minor Issues

- The 2D stability matrix (Table 6) still categorises Iowa as "High seed stability, High census stability" — but Iowa now has $\Delta f = 0.31$, making it a prominent census-unstable case. The matrix label should either move Iowa to the "High seed stability, Low census stability" cell or explain that the 2D matrix is a prediction (based on expected CSS $\approx 0.95$) that Iowa's instability is mechanistically explained by Hypothesis A/B and does not reflect genuine geographic indeterminacy.

- The Lorenz proxy table predicts Iowa as "High (ratio-symmetric)" based on the 2020 $p^*_{2020} = 0.50$. This is correct for the 2020 anchor year, but Iowa's 2000--2010 instability illustrates that the proxy can be misleading if it only looks at the anchor year without the $\Delta p^*$ across years. The table should note that Iowa's predicted high stability at $p^*_{2020} = 0.50$ is the 2020 equilibrium, not the trajectory.

## Questions for Authors

1. Is Iowa's 2000 sweep output complete? If so, what does the pipeline report for $p^*_{2000}$?

2. For the 4 unstable states at $\tau = 0.05$ (Alabama, Iowa, South Carolina/Dakota, Utah): do any also show low seed stability in B.7? Is the instability correlated across the two stability dimensions as the 2D matrix predicts?

3. The legal section says "Iowa (CSS $\approx 0.95$, predicted)" — but Iowa's 2000--2010 shift is $\Delta f = 0.31$, which would classify it as ratio-unstable. Why is the predicted CSS still $\approx 0.95$? Has the paper updated the Iowa CSS prediction in light of the observed instability?

## Recommendation

The revision has addressed the core issues from Round 1. The Iowa case study, threshold sensitivity table, and s_seat clarification all represent genuine improvements. The paper is ready for journal submission if the $p^*_{2000}$ value can be computed and the Iowa CSS prediction is reconciled with the observed instability. I withdraw my previous R\&R recommendation and would accept this paper as a minor revision. Accept pending resolution of the $p^*$ computation and the Iowa CSS prediction inconsistency.
