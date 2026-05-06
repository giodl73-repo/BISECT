# Review 5 — Christina Liang
**Paper**: F.3: Resolution Selection for State Legislative Redistricting: When Census Tracts Are Too Coarse
**Round**: R2
**Score**: 3/4

## Response to Revision

**C1 (Block-group adjacency data scope)** — Addressed. The methodology section now explicitly states that the empirical results require block-group adjacency data for Washington state only (the one state in the analysis with k/n > 0.05). The command to generate this file is provided, along with the estimated storage (~70 MB for WA). This is a significant clarification over R1.

**C4 (Recommendation table absent)** — Not addressed. The recommendation table for all 50 states and lower chambers (promised in the abstract and referenced in the introduction) is still not included in the paper. This is the primary remaining reproducibility gap and it was the most practically important output promised by the paper.

**C3 (Runtime table hardware specification)** — Addressed. A footnote now specifies the hardware used for the runtime measurements, including CPU model, RAM, and single-thread configuration. The measurement methodology (wall-clock time, single seed, single state) is now specified.

**C2 (MAUP Kuriwaki interpolation)** — Addressed. The paper now describes the interpolation methodology (block-level allocation from Kuriwaki's census-block-level dataset, aggregated to block groups by summing block-level proportional allocations). The potential direction of interpolation bias is noted.

## Assessment

Good improvements on hardware specification and interpolation description. The missing recommendation table (C4) remains the most important gap. The paper promises a comprehensive 50-state table and does not deliver it.

**Score**: 3/4
**Recommendation**: Accept with minor revisions (contingent on recommendation table inclusion)
