# Review: section-algorithms.md
**Reviewer**: MERIDIAN (computational-geographer)
**Date**: 2026-05-05

---

## What's accurate

The GeoSection normalisation formula `EC_min(i:(k-i)) / sqrt(min(i, k-i))` is stated correctly and the motivation is sound. The explanation of why 1:(k-1) splits win without normalisation ("a small region can be enclosed cheaply") is an accurate description of the caterpillar problem. The AreaSection description correctly identifies that METIS ncon=2 is used with ubvec[0]=1.001 and ubvec[1]=1+area_swing, and correctly states that the population constraint is tighter and takes precedence when constraints conflict. The VRA weight adaptive boost formula is not repeated verbatim here but the VRASection description correctly notes that w_vra=0 reduces to standard GeoSection, which is an important algorithmic invariant. The BisectionEnsemble description is algorithmically accurate: it correctly explains that Wilson's loop-erased random walk is used for UST sampling, that binary nodes use ReCom, and that p-way splits for prime p>2 fall back to METIS.

The B.6 complexity claim that "METIS runs in O(m log n)" is correct, and the full-state complexity "O(k * m log n)" is the right per-level scaling assuming the graph is not significantly reduced between levels (which it is in practice, but the bound is a correct upper bound).

---

## P1 — Required fixes

**Section "B.7 (Seed Sensitivity)"**: The guide states "The last improving seed occurs before index 1,023 for all 50 states across all three census years." This is presented as a proved result from B.7, but the boundary 1,023 (= 2^10 - 1) is suspiciously round for an empirical result. If 1,023 is a worst-case upper bound from the 50-state sweep, the guide should say "for all 50 states and all three census years, across X runs." If it is a theoretical bound, the proof mechanism should be stated. A court reviewing this claim to validate T=600 as sufficient needs to know whether "before index 1,023" is empirically verified across all states or is an analytically derived bound.

**Section "VRASection (B.14)", alignment_penalty**: The guide states the ratio selection objective is `EC_min / sqrt(min(i, k-i)) + w_vra * alignment_penalty(i:(k-i))` but does not define `alignment_penalty`. The guide says it "measures how well minority VAP concentrates on one side of the proposed split (Gingles Prong 1: geographic compactness of minority population)" but provides no formula. For MERIDIAN, an algorithm description without a computable formula is incomplete. Researchers and court-appointed experts need to know whether alignment_penalty is a Gini coefficient on the minority VAP distribution, a signed difference of minority fractions across the split, or something else. The formula must be specified.

**Section "ApportionRegions (B.11)", split prescription**: The guide states "If k <= 3: split into k equal parts directly." For k=3, this means a 3-way METIS call at the first level, not a binary split. This is a non-trivial departure from the binary bisection framework and has significant consequences for contiguity and METIS configuration (nparts=3, not nparts=2). The guide should explicitly flag this as a 3-way partition and note any specific METIS parameters required. The current description is terse enough to be misread as "split into 3 binary halves."

**Section "County-Sticky (B.10)"**: The guide correctly classifies County-Sticky as a weights-layer algorithm, not a structure algorithm. But the section appears within the "Structure algorithms (B.8-B.15)" heading, which contradicts this classification. This creates potential confusion for readers who scan section headers.

---

## P2 — Suggested improvements

The NestSection (B.13) entry describes `gcd(H,S)` as determining the shared top-level partition. The example — "each senate district is the union of exactly H/S house districts" — implies H is divisible by S, which is not always true (e.g., California has 80 assembly seats and 40 senate seats: H/S=2, which is clean; but New York has 150 assembly, 63 senate: gcd=3, H/S is not integer). The guide should clarify what happens when H/S is not an integer and how gcd determines the nesting structure in that case.

StabilitySection (B.15) is described as "a cross-census analysis tool, not a bisection algorithm." This is correct and important. It might be even clearer to move this section out of the "Structure algorithms" block and into a separate "Analysis extensions" section to prevent it from being confused with partition modes.

---

## Score: 3/4

The algorithmic content is largely accurate and demonstrates understanding of how METIS, ReCom, and the prime-factor tree interact. The two P1 issues — the unspecified alignment_penalty formula and the 1,023 seed-boundary provenance — are both verifiability gaps that would be challenged in a court or peer review context. Both can be fixed with a paragraph addition.
