---
reviewer: George Karypis
round: 2
score: 3
date: 2026-05-05
---

## Summary

The authors have addressed my three P1 items from Round 1 with mixed success. The statistical independence problem has been correctly resolved — restricting formal inference to the congressional sample ($n = 50$ independent states) and treating the senate/house results as descriptive is the right approach and is executed cleanly. The reframing of the chamber count (from the indefensible "727" to "450 chamber-year combinations") is a significant improvement in precision. However, the other two P1 items — the mechanism explanation and the coarsening-levels description — remain outstanding and are not addressed in this revision.

## P1 Items: Response Assessment

**P1.1 (Mechanism explanation) — Not addressed.** Section 5.1 still reads: "can lead to 'stretching' districts across geographic features to satisfy global balance constraints." I flagged this in Round 1 as technically incorrect: METIS n-way FM refinement does not stretch districts; the mechanism is that RB's per-bisection FM operates on smaller subgraphs with tighter balance constraints, concentrating the objective on a structurally simpler sub-problem at each level. The current text has not been revised. This remains a required fix.

**P1.2 (Coarsening levels) — Not addressed.** Section 2.1 still conflates n-way and RB coarsening. The statement that n-way uses "O(log k) levels" is carried forward unchanged. As noted in Round 1: kmetis in METIS 5 uses a single coarsening phase to approximately 20k nodes, not O(log k) levels. RB uses O(log k) bisection levels. The runtime comparison in Section 4.3 rests on this distinction, and the current text is still incorrect.

**P1.3 (Prime-k mitigation) — Partially addressed.** Section 5.2 now reads: "apply one additional FM inter-district sweep on boundaries adjacent to asymmetric splits." This is cleaner than the original "rounding to nearest power of two" language, which was meaningless for fixed-apportionment k. However, the revised text still does not include before/after PP numbers for the PA k=17 case — the revision plan specifies this, and Section 4.2's prime-k table still shows only baseline values.

## New Contributions (from this revision)

**Statistical correction is well executed.** The revised methodology section clearly separates the primary inference (congressional, n=50) from the descriptive pattern (senate/house), and the new Table 2 (PP difference by k-bin) effectively replaces the missing Figure 1 with tabular data. The four k-bins (2–10, 11–30, 31–80, 81–400) cover the range cleanly and the results are consistent with the "broadly flat" claim. This is a satisfactory resolution of the Liang R1 P1.1 (missing figure) concern.

**Partisan comparison paragraph is added and correct.** Section 5.4 ("Partisan equivalence of RB and n-way") is a useful addition that directly addresses Rodden R1's concern. The mean |Δseats| = 0.2 claim and the WI/NC exact agreement at median seed are specific and appropriate. The Rodden reference is correctly cited. This closes the partisan neutrality gap.

**Chamber count is now internally consistent.** The abstract (450 chamber-year combinations), introduction, and conclusion all use 450. The methodology correctly derives this as 50 states × 3 chamber types × 3 census years. Duchin's count discrepancy concern from Round 1 is resolved.

## Remaining P1 Issues

1. **Mechanism explanation in Section 5.1** — still incorrect. Must fix before acceptance.
2. **Coarsening-levels description in Section 2.1** — still conflated. Must fix.
3. **Prime-k before/after PP numbers** — the FM post-processing claim (60% recovery) is not demonstrated with data.

## P2 Items

- The block-group PP values are still not reported for the case studies, as requested by Duchin R1. This remains a P2 gap.
- Confidence intervals on Table 1 means are still absent. Liang raised this; it is a P2 item.

## Score: 3 — Minor Revision (unchanged)

The statistical correction and the partisan paragraph are real improvements. But two of my three R1 P1 items remain unaddressed (mechanism and coarsening levels), and the prime-k mitigation evidence is still missing. A 3 is appropriate. I will move to 4 when the mechanism, coarsening, and PM prime-k evidence are fixed.
