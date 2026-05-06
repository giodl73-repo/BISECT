# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 2
**Score**: 3/4

## Summary

All three critical issues from Round 1 have been addressed. Theorem 1 is downgraded to the provable $O(n^3 \log n)$. The scaling law has been corrected to a power-law fit ($\hat{t} \propto k^{1.3}$). The theoretical vs. empirical chain mismatch is now explicitly acknowledged in the theorem's scope. The concentration explanation is corrected. The paper is now technically honest and the claims are accurate.

## Critical Issues — Resolution

**Issue 1 (Theorem 1 proof gap): Resolved.** The theorem now claims $O(n^3 \log n)$, which follows directly from $\sgap = \Omega(1/n^2)$ and $\log|\mathcal{P}| = O(n\log n)$. The Remark citing LPW Theorem 13.1 correctly identifies where the $O(n^2 \log n)$ tightening would come from. The proof sketch is now complete for the claimed bound.

**Issue 2 (Scaling law $\approx 400k$ is wrong): Resolved.** The revised text reports $\hat{t} \propto k^{1.3}$ from log-log regression, with the observation that the coefficient is not constant across states. This is correct. The data fully support a super-linear (not linear) relationship between $k$ and $\hat{t}_{\rm mix}$. The original "400k" claim was a numerical error (accurate only for CA); the power-law fit is a proper statistical analysis.

**Issue 3 (Chain identity): Resolved.** The "Scope of the Theorem" subsection in Section 3 clearly states: (1) Theorem 1 is for the lazy reversible chain; (2) G.4 measures the non-reversible chain; (3) the factor-of-2 relationship between lazy and non-lazy chains does not materially change the $10^4$–$10^5\times$ gap analysis.

**Concentration explanation (secondary issue): Resolved.** The revised explanation (worst-case initial state + weaker criterion) is correct for a uniform stationary chain.

## Secondary Issues — Status

**CS complexity formula:** I noted in Round 1 that the $O(T \cdot k \cdot n\log n)$ formula overstates CS cost for composite $k$. For $k = 14 = 2 \times 7$, only 3 METIS calls per seed are needed, not 14. The revised text from M3 in the REVISION-PLAN ("CS requires $\sum_{\text{levels}} p_i$ METIS calls per seed") appears to have been added based on context — if so, this is correct. If not, it remains a minor overstatement.

**R² value for the power-law fit:** The revised text reports $\hat{t} \propto k^{1.3}$ but should include the R² from the log-log regression. With only 6 data points, the fit quality matters for whether the scaling law is meaningful.

## Recommendation

Accept with minor revisions. All three critical issues from Round 1 are resolved. The paper is now a technically sound treatment of mixing time bounds for redistricting chains. Add the R² for the power-law scaling fit.
