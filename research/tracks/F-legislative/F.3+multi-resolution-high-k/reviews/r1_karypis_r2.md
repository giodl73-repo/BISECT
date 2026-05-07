# Review 1 — George Karypis
**Paper**: F.3: Resolution Selection for State Legislative Redistricting: When Census Tracts Are Too Coarse
**Round**: R2
**Score**: 3/4

## Response to Revision

F.3 enters R2 from a 3.0/4 R1 average with a revision plan focused on four critical items. I assess how well each has been addressed.

**C1 (Threshold derivation vs. calibration)** — Addressed. Section 3 has been restructured to clearly separate motivation (CLT and configuration-count arguments give intuition for 20 units per district) from empirical calibration (the 0.05 threshold is calibrated against C.1's crossover point). The revised Section 3.2 ("Empirical Calibration") is honest about the distinction. The old conflation of derivation with calibration has been removed. The paper now correctly says the threshold "is motivated theoretically and calibrated empirically."

**C2 (Configuration count formula)** — Addressed. The incorrect O(n^m/k^m) formula has been replaced with: "The number of valid configurations (contiguous, balanced subsets of m adjacent tracts in the graph) grows super-exponentially in m when m is small and plateaus as m approaches n/k. For m ≥ 20, the configuration space is large enough that METIS edge-cut minimisation can meaningfully distinguish compact from non-compact boundaries." This qualitative statement is correct and avoids the overcounting error of the original formula.

**C3 (MAUP generalisation from single data point)** — Partially addressed. The revised Section 5.4 now qualifies the ±2--4 seat claim: "For the one high-k/n chamber in our analysis (WA, k/n=0.067), the MAUP effect is +2 seats. We conjecture a range of ±2--4 seats for high-k/n chambers generally, but additional data is needed to validate this range." This is appropriately cautious. The paper would be stronger with additional states added to the MAUP analysis (e.g., ME at k/n=0.42, MT at k/n=0.36).

**C4 (--resolution auto data dependency)** — Addressed. The paper now explicitly states that the auto-resolution check reads n_tracts from the state configuration file, not from the TIGER shapefile, so tract data downloads are not required for block-group-resolution runs.

## Remaining Issue

The recommendation table (Section 7) is still not included in the paper text. The abstract promises this table, it is referenced in the introduction, and it is the paper's most practically useful output. For R2 to clear this concern, the table must be included (even as Appendix A) or the abstract must be revised to remove the promise.

**Score**: 3/4
**Recommendation**: Accept with minor revisions
