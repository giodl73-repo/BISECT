---
reviewer: Nicholas Stephanopoulos
round: 2
score: 4
date: 2026-05-05
---

## Summary

The revision addresses all three of my P1 concerns from Round 1. The
P=NP framing is now qualified appropriately for non-technical audiences.
The approximation ratio discussion now correctly distinguishes the
theoretical landscape from empirical performance, and the "4.1 factor"
claim is replaced by a defensible empirical statement. The feasibility
vs. optimisation distinction is now explicit. From a legal standpoint,
this paper is now suitable for use as an expert report foundation.

## Strengths

- **The P=NP qualification is exactly right.** The new sentence in
  Section 5.1 — "P = NP is universally believed to be false among
  theoretical computer scientists; making the NP-hardness result
  effectively unconditional for all practical purposes" — is the correct
  framing for a legal audience. It closes the rhetorical opening that
  opposing counsel could exploit from the conditional framing.
- **The approximation section will survive cross-examination.** The
  revised Section 5.2 no longer asserts a specific numerical factor
  (4.1) that could be challenged in deposition. The distinction between
  "no known algorithm achieves better than O(log n)" (formal theory) and
  "METIS is empirically within 3% of the best of 10,000 seeds" (empirical
  evidence) is the right separation for legal use. The caveat that 10,000
  seeds is not a lower bound on the true optimum — added by the authors —
  is legally important.
- **The feasibility clarification eliminates a significant legal risk.**
  The prior version could have been misread as "redistricting on planar
  graphs is NP-hard" (full stop), which might imply to a non-technical
  reader that valid redistricting plans are computationally infeasible.
  The new remark explicitly states that any connected graph can be
  partitioned into k connected parts, and NP-hardness concerns only
  optimisation. This is essential for courts.
- **The due process framing in Section 1 is stronger.** The revised
  introduction's final paragraph — "the hardness result concerns the
  optimisation problem; a valid redistricting plan always exists and can
  be found efficiently — only finding the optimal one is intractable" —
  directly supports the good-faith approximation argument for DIA
  defendants.

## Remaining Concerns (P2 — Minor)

- **A paragraph on the due process connection would add value.** I
  suggested in Round 1 adding a paragraph connecting NP-hardness to the
  due process argument (any enacted plan is a good-faith approximation,
  not a strategic manipulation). This was not added in the revision.
  I maintain this as a P2 suggestion for the final version.
- **The cross-reference to B.7 could be more explicit.** The paper
  references B.7 for the 3% empirical gap but does not describe what
  B.7 is for readers unfamiliar with the series. A brief parenthetical —
  "(B.7, which tests 10,000 random METIS seeds per state)" — would make
  the reference self-contained.

## Score: 4 — Accept with Minor Revisions

All three P1 items are addressed and the paper is now legally defensible.
The P2 suggestions would improve the paper further but are not required
for acceptance. This is a significant improvement over Round 1.
