---
reviewer: George Karypis
round: 3
score: 3
date: 2026-05-05
---

## Summary

Round 3 addresses two of the paper's carry-forward items: the Rodden-null denominator (Rodden P1.1) and the software availability statement (Liang P1.3). Neither item directly relates to my P1 concerns (EC_norm/AreaSection interaction, estimation model source). I maintain my score at 3. The paper continues to be useful as a synthesis reference; the remaining gaps are P2 items for the journal submission.

## R3 Changes: Assessment

**Rodden-null denominator explanation (new in A2 AreaSection subsection).**
The added text reads: "We exclude the 7 single-district states ($k=1$: Alaska, Delaware, Montana, North Dakota, South Dakota, Vermont, Wyoming) and 9 states where the algorithm produces identical partisan outcomes under all tested modes, leaving 34 states where algorithm selection affects partisan outcomes; the 76% figure is computed over this contested subset."

This is a clear, accurate explanation of the denominator. The two exclusion categories are well-motivated: $k=1$ states are trivially excluded (there is nothing to compare between algorithms when there is one district), and the 9 invariant states are correctly excluded from a test of whether algorithm selection *affects* partisan outcomes. The 76% claim is now defensible in a legal context. This resolves Rodden P1.1 cleanly.

One minor technical note: the text does not enumerate the 9 invariant states. For a litigation-support paper, the enumeration would be valuable (a practitioner in state X needs to know whether X is in the invariant set). I flag this as P2.

**Data Availability statement (revised in Conclusion).**
The new statement provides: a repository URL (`https://github.com/apportionment-research/redist`), statement that redistricting outputs and adjacency graphs are available at the repository, and a specific statement that "pre-computed adjacency files for all 50 states are available as GitHub Release assets, with SHA-256 hashes recorded in the plan manifests." This substantially improves on "available as open-source software."

The gap that remains is the absence of a commit hash or software version pin. For exact reproducibility, a reader should be able to check out a specific commit and reproduce the reported results. The repository URL without a version pin allows the repository to change after publication. This is P2 for the journal submission.

## Remaining P1 Items (unchanged from R2)

**P1.3 — AreaSection EC_norm interaction: STILL PARTIALLY ADDRESSED.**
The B.0 text still does not state how EC_norm is computed in the AreaSection dual-constraint context — specifically, whether the isoperimetric normalisation remains valid when both population and area tpwgts vary across ratios. My R2 position stands: B.0 should add a reference to B.16's two-case EC_norm definition in the AreaSection subsection. This is a P2 item I accept as a journal-submission condition.

**P1.2 estimation model source (Liang concern): STILL UNADDRESSED.**
The estimated (†) cells still do not specify whether each estimate derives from the B.8–B.9 theoretical relationship, interpolation, or a model-derived prediction. The ±1 seat / ±3pp uncertainty quantification is now stated but without the source, an independent team cannot verify it.

## Score: 3 / 4 — Minor Revision

Rodden P1.1 cleanly resolved. Liang P1.3 substantially improved. My P2 items (EC_norm/AreaSection specification, invariant-state enumeration, software version pin) carry forward to the journal submission. The paper is publication-ready for the B-series internal review track.
