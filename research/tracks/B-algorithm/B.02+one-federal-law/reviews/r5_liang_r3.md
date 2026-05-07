---
reviewer: Percy Liang
round: 3
score: 3
date: 2026-05-05
---

## Summary

Round 3 makes two changes — the structural isomorphism paragraph and the VRA mutex scope paragraph — neither of which addresses my carry-forward P1 items (runtime claim qualification, TIGER/Line vintage pinning). Score maintained at 3. The isomorphism paragraph is a conceptual improvement; the mutex scope paragraph is legally sound. The paper's reproducibility gaps remain structurally unresolved.

## R3 Changes: Assessment

**Structural isomorphism paragraph.**
The new "limits of the isomorphism" paragraph correctly identifies where the isomorphism holds (decision procedure: priority sequence → tree) and where it does not (feasible set: states vs. contiguous subgraphs). From a computational standpoint, the "HH for tree structure, METIS for contiguous realization" framing is accurate: the bisection-ratio selection is a combinatorial decision over O(k) possible ratios at each level, and the contiguous partition for a chosen ratio is a METIS graph-cut problem. These are two well-defined computational steps. The paragraph makes the computation reproducible in principle.

One gap remains: the paper does not specify how the bisection-ratio comparison is performed in the AreaSection case. In the standard GeoSection (B.8), EC_norm is computed for each ratio and the minimum is selected. In the AR case (B.11), the ratio is fixed by HH priority — but when VRASection is applied as an overlay, the alignment score modifies the selection. The new paragraph should clarify whether the alignment overlay applies to the AR tree's fixed ratios or whether it can modify the tree structure. If the tree is fully determined by HH (only one valid ratio at each level), the VRASection alignment score has nothing to modify. This is a computational specification gap that B.11 must address.

**VRA mutex scope paragraph.**
The in-run prohibition / sequential-comparison distinction is legally sound and addresses Stephanopoulos P1.2 directly. From a reproducibility standpoint, the scope clarification also matters: a user who runs VRASection and then ProportionalSection independently on the same state needs to know that both plan manifests are separately valid and their comparison is permitted. The new text makes this clear.

## Remaining P1 Items (unchanged from R2)

**P1.2 — 30-minute runtime claim: NOT ADDRESSED.**
The paper still states "runs a 50-state apportionment in approximately 30 minutes on a laptop" without qualification. The AR-specific runtime (with multi-way METIS calls for large prime factors) is not measured. This must be qualified to "GeoSection-based components" or labelled as estimated pending B.11 implementation. A statute that cites a 30-minute feasibility claim in its supporting advocacy paper is potentially vulnerable to challenge if the actual AR runtime for California (k=52, with 13-way METIS calls) is substantially longer.

**P1.3 — TIGER/Line vintage pinning: NOT ADDRESSED.**
The statute text still lacks a specific TIGER/Line vintage year. The EAC delegation approach I suggested in R2 — "the statute specifies the vintage at time of initial publication; the EAC may designate updated vintages by regulation" — would be an acceptable resolution. Without this, the statute's reference to "the federal TIGER/Line adjacency graph for the census year" is ambiguous: there are typically 3-4 vintage years of TIGER/Line shapefiles for each census year (2020, 2021, 2022, 2023 updates all include 2020-era boundary corrections). Two states that run the algorithm using different TIGER vintages will produce different maps; the statute as written does not prevent this.

## Score: 3 / 4 — Minor Revision

The isomorphism paragraph and mutex scope paragraph are genuine improvements. My two P1 items (runtime qualification, TIGER vintage) are reproducibility requirements that must be resolved before the DIA is submitted to a law review or introduced as legislation. They are tractable: the runtime fix is one sentence; the TIGER vintage fix is one statutory provision.
