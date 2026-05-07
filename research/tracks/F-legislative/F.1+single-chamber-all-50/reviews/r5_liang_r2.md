# Review 5 — Christina Liang
**Paper**: F.1: Algorithmic State House Redistricting — A 50-State Empirical Study
**Round**: R2
**Score**: 3/4

## Response to Revision

The reproducibility note added to Section 3 is a substantial improvement over R1. The note explicitly lists the 33 additional states where block-group adjacency may be warranted and provides both a targeted command (for the 12 block-group-resolution states) and a full national command. The ufactor disclosure for high-k chambers is now present.

**C3 (Sub-unit splitting procedure)** — Addressed. Section 3 now includes a methodology paragraph describing when sub-unit splitting is triggered (block group population exceeds target remaining), what data is used (block-level P.L. 94-171 data), and the allocation rule (proportional-to-block-population assignment). The confirmation that this is implemented in the Rust binary is present. This was my primary concern and it is now adequately documented.

**C1 (Block-group adjacency data availability)** — Addressed to the extent possible at draft stage. The Section 3 reproducibility note and the conclusion now clearly describe what must be computed and the approximate compute/storage requirements. The distinction between pre-deposited and must-generate data is acknowledged.

**C4 (5-seed sensitivity table)** — Not addressed. The paper still asserts 5-seed consistency without exhibiting Table A.1. For a paper claiming this level of algorithmic stability across seeds, the supporting evidence should be in the paper. This remains a reproducibility gap.

**C2 (Enacted map comparison sources)** — Not addressed. The source of enacted map GIS data (Redistricting Data Hub, state portals, etc.) is not specified in the methodology. Download dates and sources are not listed. This is a reproducibility concern that is straightforward to address.

**C5 (Enacted map availability inconsistency)** — Partially addressed. The paper consistently uses 38 states as the comparison sample throughout. The 12 missing states are still not named.

## Assessment

The sub-unit splitting documentation is the most important addition and it is now present and adequate. The paper has moved from "undocumented algorithm" to "documented algorithm with stated implementation." I am upgrading from 2/4 to 3/4. The remaining gaps (seed sensitivity table, enacted map source table) are real but do not rise to the level of major revision.

**Score**: 3/4
**Recommendation**: Accept with minor revisions
