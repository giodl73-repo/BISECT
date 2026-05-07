# Review: G.0 — Ensemble Methodology
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 2
**Score**: 3/4

## Summary

The revision adequately addresses the main concerns from Round 1. The bridge section in particular has been substantially improved: the false analogy between ESS and T=600 has been removed and replaced with a correct orthogonality framing. The framework is now technically consistent and appropriate as the foundation for G.1–G.5.

## Changes Evaluated

**CS bridge section (major concern, Round 1):** Substantially improved. The claim that T and ESS "measure the robustness of a result to additional sampling" has been correctly replaced with the orthogonal-certificates framing. The revised text accurately identifies that ESS operates in the MCMC draw space and T=600 operates in the seed-quality space. These are indeed incommensurable, and the paper now says so clearly. This is the right fix.

**Rhat threshold (minor concern, Round 1):** Resolved. The 1.05 vs. 1.1 inconsistency is gone. The threshold is 1.1 throughout, and the footnote explaining the choice relative to Vehtari 2021 is appropriate given the discrete nature of redistricting statistics. This is particularly important for G.4 consistency.

**ReCom stationary distribution:** The correction to Table 1 ("Approximate (spanning-tree proportional)") and the new paragraph in Section 2.2 are adequate. I note that for a computer science audience, the distinction between "unknown stationary distribution" and "approximately uniform" is important for interpreting the chain's coverage properties — the paper now makes this clear enough.

**Citation:** Vehtari 2021 added as primary citation for the formula. Correct.

## Remaining Concern

The 65th–75th compactness percentile estimate in Section 5.2 is presented as a forward reference to G.1 but still appears to be asserted without derivation. The text reads "empirically, we estimate P*_EC falls at approximately the 65th–75th percentile of the PP distribution." If G.1 has been revised to use a corrected NC PP score of 0.337 (from 0.412), this estimate should be rechecked — the G.0 range was apparently calibrated against the original 0.412 figure. The "65th–75th" range in G.0 Section 5.2 may need to be updated to match G.1's revised results.

This is a minor consistency issue rather than a blocking concern. The framework paper's conclusions do not depend on this specific range.

## Recommendation

Accept with minor revisions. Confirm the 65th–75th estimate in Section 5.2 is consistent with revised G.1 data (particularly the corrected NC PP score). Fix ESS threshold to 500 to match G.4.
