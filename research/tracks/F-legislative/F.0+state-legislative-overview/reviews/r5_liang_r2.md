# Review 5 — Christina Liang
**Paper**: F.0: Algorithmic State Legislative Redistricting — A Research Program
**Round**: R2
**Score**: 3/4

## Response to Revision

The authors have addressed the data availability statement and the block-group adjacency documentation, which were my primary concerns. The revised conclusion now specifies the command to generate full 50-state block-group adjacency data and clarifies the ~4 GB storage requirement.

**C1 (Block-group adjacency availability)** — Partially addressed. The data availability statement now specifies that block-group adjacency data for states studied in F.1--F.3 are available at the project repository, with a specific command to generate them. However, the statement still does not confirm that these files have been pre-generated and deposited — it describes how to generate them. For a replicator, this distinction matters: the estimated generation time for full 50-state block-group adjacency is multi-hour. A forthright statement of what is pre-deposited versus what must be computed would be more useful.

**C2 (build_unit_adjacency.py maintenance status)** — Not addressed. The script's maintenance status remains unclear. Given that the paper explicitly instructs replicators to use this Python script, its test coverage and handling of edge cases (including the 5 block-group-scale-inadequate states) should be documented.

**C3 (Vague data availability statement)** — Addressed. The "version tagged with the paper's release" language remains, but is now supplemented by specific version context in the methodology section. This is acceptable.

**C4 (Sub-unit splitting)** — Not addressed in F.0. This is addressed in F.1 Section 3, which is appropriate — the mechanism belongs in the methodology paper. F.0 need not duplicate it.

**C5 (Single-seed results)** — Addressed. The revised overview now previews that F.1 includes 5-seed sensitivity analysis for selected states, and the finding that partisan outcomes and PP scores are consistent across seeds. This is the disclosure I requested.

## Assessment

The paper has addressed its most important reproducibility gaps and is substantially improved from R1. The remaining issue (C2, maintenance status of build_unit_adjacency.py) is a documentation concern rather than a technical error. I am upgrading from 2/4 to 3/4. The overview paper does not need to resolve all reproducibility details — those belong in F.1 and F.3 — but the overview should not over-promise on what is readily replicable.

**Score**: 3/4
**Recommendation**: Accept with minor revisions
