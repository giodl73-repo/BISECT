# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 2
**Score**: 3/4

## Summary

The revision resolves the most critical blocking issue: Theorem 1 has been downgraded from the unproven $O(n^2 \log n)$ to the provable $O(n^3 \log n)$. The Remark following the theorem correctly notes that the tighter bound would require path congestion analysis beyond the paper's scope, with a proper citation to Levin-Peres-Wilmer Theorem 13.1. The table caption has been updated to match. The chain identity issue and the concentration explanation have been revised. Remaining issues are at moderate priority.

## Blocking Issues — Resolution

**B1 (Theorem 1 proof gap): Resolved.** The revised theorem now claims $\tmix = O(n^3 \log n)$ — the bound that follows directly from the spectral gap argument. The Remark is exactly right: "The tighter $O(n^2 \log n)$ bound requires path congestion analysis beyond the scope of this paper; see [Levin, Peres & Wilmer, Theorem 13.1] for the relevant machinery." This is an honest and appropriate demotion. The empirical conclusions are unaffected — both $O(n^2 \log n)$ and $O(n^3 \log n)$ are many orders of magnitude above the observed mixing times.

The citation to LPW Theorem 13.1 is correct: this is the canonical path congestion result that could establish the tighter bound, and pointing readers there is the right technical move for a paper whose purpose is legal-empirical rather than purely theoretical.

**B2 (Theoretical vs. empirical chain): Resolved.** The added Section 3 subsection "Scope of the Theorem" correctly distinguishes: Theorem 1 applies to the lazy reversible (Metropolized) chain; the empirical measurements use standard non-reversible ReCom. The relationship between the two (lazy chain is at most 2× slower by construction, so the $10^4$–$10^5\times$ theory-practice gap is not materially reduced by this factor) is accurately stated.

**B3 (Concentration explanation): Resolved.** The revised Section 3.4 replaces the incorrect "concentration near compact plans" explanation with the correct two-component explanation: (1) worst-case initial state doesn't arise in practice; (2) empirical $\hat{R} < 1.1$ criterion is weaker than full total-variation mixing. This is the correct account.

## High-Priority Issues — Status

**H1 ($< 1\%$ outlier frequency): Addressed.** The derivation from the B.7 50-state sweep (0/150 observed outliers, bounded above by $1/150 \approx 0.7\%$ at 95% confidence) has been added. This is adequate.

**H2 (Adversarial Theorem 1 paragraph): Added.** The paragraph noting that Theorem 1's bounds are worst-case and that G.4 empirical certification is the appropriate practical standard is present. This directly addresses Stephanopoulos's Round 1 concern.

## Moderate-Priority Issues — Status

**M1 (Planarity assumption): Added** via the paragraph noting $O(\sqrt{n})$ non-planar edges for water-body crossings. Adequate.

**M2 (Start-from-compact concentration): Added** via the Section 6.4 note that ensemble audits should use random initial plans, not the CS plan.

**M3 (CS complexity formula): Not addressed.** The $O(T \cdot k \cdot n\log n)$ formula still overstates the CS cost for typical $k$. For $k = 14 = 2 \times 7$, only 3 METIS calls are needed per seed, not 14. This is a minor accuracy issue.

## Recommendation

Accept with minor revisions. The blocking proof issue is resolved, the chain identity issue is addressed, and the concentration explanation is corrected. The remaining items are minor.
