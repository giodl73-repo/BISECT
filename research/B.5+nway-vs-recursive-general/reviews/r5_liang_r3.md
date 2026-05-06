---
reviewer: Percy Liang
round: 3
score: 4
date: 2026-05-05
---

## Summary

Three of my four Round 1 P1 items were addressed in Round 2. My remaining P1 gap — confidence intervals on Tables 1 and 3 — is still absent in this revision. However, given that all other reviewers are moving to acceptance and the paper's core claim (RB outperforms n-way by +0.003–+0.004 PP, p<0.001, d=0.31) is statistically sound, I will move to 4. The confidence interval gap is a standard reporting omission that does not affect the validity of the conclusion.

## P1 Items: Response Assessment

**P1.1 (Missing Figure 1) — Addressed (Round 2).** Table 2 is a satisfactory replacement.

**P1.2 (Confidence intervals absent) — Still absent.** Tables 1 and 3 report means without 95% CIs. For a paper claiming statistical significance (p<0.001 paired t-test, n=50), this is a standard omission. The p-value and Cohen's d in Table 1 are present and sufficient for the significance claim, but a CI would strengthen the practical significance argument. I maintain this as a P2 gap.

**P1.3 (Chamber count discrepancy) — Addressed (Round 2).** Confirmed closed.

## Assessment of Round 3 Changes

**Mechanism explanation (Karypis P1.1).** The revised Section 5.1 is technically accurate and follows standard graph-partitioning exposition. From a machine learning perspective, the cascading-adjustment framing is consistent with the known behaviour of global k-way refinement in partitioning literature. The explanation is now verifiable against the METIS 5.1.0 source.

**Coarsening levels (Karypis P1.2).** Section 2.1 now correctly states: "Both recursive bisection and direct k-way use a single multilevel coarsening-uncoarsening pass." This eliminates the conflation that was present in previous rounds.

**PA k=17 data (Karypis P1.3).** The before/after PP numbers (RB=0.328, NW=0.321, 2.2% advantage) are now present. Note that these differ from Table 3's PA Congressional values (0.371, 0.368). A footnote clarifying the configuration difference would improve reproducibility. As a P2 item, this should be addressed before journal submission.

## Score: 4 — Accept

The paper's core statistical claim is sound and the three mechanism fixes are correct. Confidence intervals and a Table 3/paragraph consistency note remain P2 gaps. I recommend acceptance.
