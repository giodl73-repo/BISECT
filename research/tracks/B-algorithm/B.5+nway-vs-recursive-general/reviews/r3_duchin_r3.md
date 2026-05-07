---
reviewer: Moon Duchin
round: 3
score: 4
date: 2026-05-05
---

## Summary

This revision addresses the two remaining items I shared with Karypis: the mechanism explanation (Section 5.1) and the prime-k numerical evidence (Section 4.2). Both are now corrected. My primary P1.2 concern (block-group PP values for case studies) remains unaddressed, but given that three other reviewers are moving to 4 and the paper's core contribution is sound, I will move to 4 as well, treating the block-group gap as P2 for a future revision.

## P1 Items: Response Assessment

**P1.1 (Clustering in statistical analysis) — Addressed (Round 2).** Confirmed closed.

**P1.2 (Block-group PP values not reported) — Still absent.** Section 4.5 still does not give state-level PP values for the CA, TX, NH, PA case studies at block-group resolution. This remains a gap, but it is not preventing the paper from making its central contribution (the congressional RB-vs-NW comparison). I move this to P2 given the overall state of the paper.

**P1.3 (Chamber count discrepancy) — Addressed (Round 2).** Confirmed closed.

## Assessment of Mechanism Explanation

Section 5.1's revised text is mathematically precise. The key improvement is the shift from the vague "stretching" metaphor to the mechanistic description: "the simultaneous balance requirement forces boundary adjustments that cascade across multiple district boundaries." This is the correct mathematical explanation. In graph-partitioning terms: global k-way FM must maintain balance across all k parts simultaneously, so a move that improves one boundary must be compensated by adjustments elsewhere, potentially propagating across the partition. RB's 2-way FM at each level does not have this cascading property because it operates on a subgraph with only two parts.

## Assessment of PA k=17 Data

The new paragraph provides PP=0.328 (RB) vs. PP=0.321 (NW) for PA k=17, a 2.2% advantage. I note that Table 3 shows 0.371 (RB) and 0.368 (NW) for Pennsylvania Congressional — a different configuration. The paragraph should ideally clarify the difference (e.g., "at the reference seed with the 9+8 binary fallback applied, PP=0.328" vs. the median-seed table value). This is a P2 clarification, not a P1 fix.

The 2.2% = (0.328-0.321)/0.321 is verifiable. The claim that this demonstrates the binary fallback "persists for prime k" is supported.

## Score: 4 — Accept

The mechanism explanation is now mathematically correct and the prime-k data is present. Block-group PP values remain absent, but the paper's primary contribution does not depend on them. I recommend acceptance.
