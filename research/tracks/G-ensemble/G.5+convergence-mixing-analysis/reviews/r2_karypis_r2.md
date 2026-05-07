# Review: G.5 — Convergence and Mixing Analysis
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 2
**Score**: 3/4

## Summary

The revision addresses the main theoretical and empirical concerns from Round 1. Theorem 1 is correctly downgraded to $O(n^3 \log n)$ — the bound supported by the spectral gap argument. The table caption is updated consistently. The scaling law has been corrected from the false "approximately 400k" to a proper power-law fit. The lazy-chain vs. non-reversible chain distinction is now stated in the theorem's scope section. The paper is now technically honest in a way that Round 1 was not.

## Changes Evaluated

**Theorem 1 (B1):** The downgrade to $O(n^3 \log n)$ is the correct change. The proof sketch now leads to the claimed bound without asserting an unproven tightening step. The Remark on the conjectured $O(n^2 \log n)$ bound with the LPW reference is the right approach for a paper that aims to be theoretically honest while pointing to future work.

**Table caption update:** The table heading now reads "$O(n^3 \log n)$ theoretical bounds." Consistent with the theorem. Good.

**Chain scope section (B2):** The distinction between the lazy reversible chain (Theorem 1's subject) and the standard non-reversible ReCom (G.4's empirical subject) is now explicit. The factor-of-2 relationship between lazy and non-lazy mixing times is correctly described as negligible relative to the $10^4$–$10^5\times$ gap.

**Scaling law (B3):** The revised Section 4.2 now reports $\hat{t}_{\rm mix} \propto k^{1.3}$ from a power-law fit, with the correct observation that "small states mix more efficiently per district than large states." The data supports approximately $\beta \approx 1.2$–$1.4$ as stated. This is more honest than the original "approximately 400k" claim.

**CS complexity (M3):** The correction noting that $k = 14 = 2 \times 7$ requires only 3 METIS calls per seed (not 14) has not been made — from what I can see. This is a minor overstatement of CS cost and should be corrected. For composite $k$ states, the factorization tree depth determines call count, not $k$ itself.

**Lazy-chain factor of 2 (L1):** Added appropriately.

## Recommendation

Accept with minor revisions. Correct the CS complexity formula to reflect the actual number of METIS calls per seed for composite $k$ (this is a clarification, not a reanalysis).
