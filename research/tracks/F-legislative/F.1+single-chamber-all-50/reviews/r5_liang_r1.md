# Review 5 — Christina Liang
**Paper**: F.1: Algorithmic State House Redistricting — A 50-State Empirical Study
**Round**: R1
**Score**: 2/4

## Summary

F.1 presents 50-state empirical results that depend on two categories of data infrastructure: (1) the standard tract-level adjacency data used in the B track, and (2) block-group adjacency data for 12 states that must be separately built. My review focuses on whether the block-group data requirements are realistic, whether the comparison to enacted maps is reproducible, and whether the computational claims are consistent.

## Strengths

The data prerequisites section (Section 3.2) is explicit: "Block-group adjacency data for the 12 block-group-resolution states must be built before running the pipeline" with the exact command provided. The estimated storage (800 MB for 12-state block-group adjacency) is specific and plausible (approximately 67 MB per state at block-group resolution, consistent with known TIGER file sizes).

The runtime table is internally consistent: NH at 420s, WA at 180s, TX at 95s (tract) and 380s (block-group), CA at 140s. The 3.2× speedup ratio for tract versus block-group at comparable k is stated and approximately verifiable from the individual figures.

## Concerns

**C1 — Block-group adjacency data not confirmed available.** Section 3.2 says these files "must be built" using build_unit_adjacency.py. There is no statement that these files have been pre-generated and are available for download. A replicator must: (1) download TIGER/Line block-group shapefiles for all 12 states (variable size, ~2-10 GB per state), (2) run the Python adjacency builder, (3) then run the Rust pipeline. The total setup time for a replicator is not stated. For a 50-state empirical paper, this is a substantial reproducibility barrier.

**C2 — Enacted map comparison data sources.** Section 6 compares to "enacted 2020 state house maps" for 38 states. The source of these enacted maps is not specified in the methodology section. Are these from the Redistricting Data Hub, individual state GIS portals, or another source? Enacted map GIS data is often not maintained in a stable, version-controlled repository. If the comparison maps were downloaded from state portals at a particular point in time, the paper should specify the download date and source URL (or archive URL) for each state.

**C3 — Sub-unit splitting procedure undocumented.** Section 5.3 states that NH results use "population-weighted sub-unit assignment when a block group is the only unit available to complete a district." This is a non-trivial algorithmic procedure: when k/n_bg > 0.5, many districts must consist of fractional block groups. The procedure for splitting block groups must access block-level population data. The paper does not document: (a) whether block-level data has been downloaded for NH, (b) what the exact splitting rule is, or (c) whether this procedure is implemented in the Rust binary or as a Python post-processing step. This is a reproducibility gap for the paper's hardest case.

**C4 — 5-seed sensitivity analysis not shown.** Section 7 states "we report 5-seed sensitivity checks for the four case study states; all produce identical partisan outcomes and PP scores within ±0.005." However, no table showing the 5-seed results appears in the paper. The claim "identical partisan outcomes" across seeds is significant — it means the partisan composition of each state's map does not depend on the random seed. This should be shown explicitly, not stated as an assertion.

**C5 — Enacted map availability inconsistency.** Section 7 says "12 states had not yet finalized their maps or did not release machine-readable shapefiles." Section 6.2 says comparison data covers 38 states. But Table 6 (implicit) in the county splits section shows only 7 states' data. The paper oscillates between 38 states with comparison data and a much smaller exhibited sample. A complete 38-state comparison table, even in an appendix, is needed for reproducibility.

## Recommendation

Major revision required. The block-group data infrastructure is described but not confirmed available. The sub-unit splitting procedure for NH is critical and undocumented. The 5-seed sensitivity analysis is asserted but not shown.
