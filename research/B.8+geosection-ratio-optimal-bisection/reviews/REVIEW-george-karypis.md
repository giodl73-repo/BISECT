# Review: George Karypis
**Paper**: GeoSection: Isoperimetrically-Normalised Ratio-Optimal Bisection for Congressional Redistricting
**Reviewer**: George Karypis (University of Minnesota — METIS, graph partitioning, multilevel methods)
**Date**: 2026-05-02
**Score**: 3.5 / 4

---

## Summary

The paper introduces GeoSection, an extension of standard METIS-based recursive bisection that scans all feasible split ratios at the first bisection level and selects the ratio minimising the isoperimetrically-normalised edge-cut EC/sqrt(min(i,k-i)). A secondary contribution (Phase 2) adds a PCA-based directional penalty to encourage cuts perpendicular to a subregion's minor axis. The algorithm is applied to 45 multi-district states using 2020 Census data, producing a 50-state table of "natural ratios" and head-to-head partisan comparison against the standard near-equal bisection baseline.

The core technical contribution is well-motivated and cleanly executed. As the designer of METIS, I can confirm the proposed usage of part_kway with explicit tpwgts for unequal splits is the correct implementation path — part_recursive does not support unequal vertex weight targets, and the paper correctly handles this. The per-node ufactor formula (1 + delta/k) is a thoughtful solution to the problem of accumulating balance errors across recursive levels.

The paper makes a genuine algorithmic contribution that is new to the redistricting literature. I recommend acceptance with minor revisions.

---

## Strengths

**S1. Correct diagnosis of the caterpillar pathology.**
The paper correctly identifies why standard bisection always favours 1:(k-1) splits on states with compact urban cores: the raw edge-cut is not a fair comparison across ratios because a 1-district boundary is intrinsically shorter than a k/2-district boundary. This is a genuine insight that has not been articulated clearly in prior redistricting work.

**S2. Principled normalisation with a clean closed form.**
The EC/sqrt(min(i,k-i)) normalisation has a clean derivation from the isoperimetric inequality for convex planar regions. The paper is appropriately honest (Remark 3.1) that the result is an approximation rather than an exact theorem for non-uniform density. This level of intellectual honesty about the approximation is the correct posture.

**S3. Correct METIS implementation details.**
The algorithm correctly uses part_kway with tpwgts=[i/k, 1-i/k] for unequal ratios. The per-node ufactor construction (1 + delta/k) correctly distributes the cumulative balance budget across recursive levels — a non-obvious implementation detail that most redistricting papers omit. The post-hoc boundary-swap rebalancer for the cases where METIS falls slightly out of balance is practical and correct.

**S4. Computational cost analysis.**
Table 3 provides an honest analysis of METIS call counts. For Wisconsin (k=8, N=50), the first-level ratio scan adds 200 calls compared to 350 for standard bisection — a 57% overhead that scales linearly with k. This is a useful practical characterisation.

**S5. Self-correcting behaviour demonstrated concretely.**
The Wisconsin bisection tree (Table 2) illustrates the algorithm's key property: the ratio selection at each level is independent and can shift from asymmetric (1:7 for Milwaukee peel) to equal (2:3 for western Wisconsin). This demonstration is more convincing than a purely analytical argument.

---

## Weaknesses

**W1. The Phase 2 directional penalty is described but not evaluated.**
Section 3.4 defines the directional penalty weight w'(u,v) = w(u,v) * (1 + lambda * |sin(theta(u,v))|), but Remark 3.2 states that all empirical results use lambda=0 (Phase 1 only). Phase 2 is the algorithm's second claimed contribution but has zero empirical grounding in this paper. The reader is left with a definition and a promise of "noticeably straighter boundaries" based on "preliminary single-state tests" that are not reported.

**W2. Seed convergence analysis is absent for the new metric.**
The paper inherits the B.7 seed-sweep convergence result and applies it by reference. However, the convergence property was established for the near-equal MEC objective. It is not obvious that 50 seeds per ratio is sufficient for the normalised EC/sqrt(min(i,k-i)) metric, especially at unequal ratios where the METIS constraint surface is harder (tight tpwgts may force the solver into local optima more often). For three representative states, the paper should report variance in the normalised score across seeds to validate that 50 seeds is sufficient for the ratio selection to be stable.

**W3. Three large states have missing data.**
CA (k=52), TX (k=38), and FL (k=28) are marked "---" in Table 1 with a note about uncertain natural ratios. These three states account for 118 congressional seats — 27% of the House. The paper explicitly acknowledges the issue (Limitation: "higher seed counts may be needed") but makes no effort to bound the uncertainty. At minimum, the paper should report the spread of normalised scores across 50 seeds for the first-level ratio scan in CA, TX, and FL to show whether the "---" reflects genuine ambiguity or a solvable computational issue.

**W4. part_kway vs. part_recursive quality comparison missing.**
The paper correctly notes that part_recursive is unsuitable for unequal ratios. For the equal-ratio case (i = k/2), the paper uses part_recursive. A brief analysis comparing the edge-cut quality of part_recursive vs. part_kway at the equal-ratio baseline would be informative: if they produce comparable quality for the planar redistricting graphs used here, this justifies the choice; if part_recursive is substantially better, the switch matters for the comparison.

---

## P1 Items (Must Fix Before Acceptance)

**P1-I. Report seed variance for the normalised ratio score.**
For at least 5 representative states (including one with an asymmetric result and one with a near-equal result), report the standard deviation of EC/sqrt(min(i,k-i)) across the 50 seeds at the winning ratio. If variance is low (< 2% of the winning normalised score), this confirms that 50 seeds is sufficient and the natural ratio selection is stable. If variance is high, the paper must revise its confidence claims about the 50-state results.

---

## P2 Items (Should Fix)

**P2-I.** Provide at least preliminary Phase 2 results for one state (e.g., Wisconsin or North Carolina) showing the effect of lambda=0.1 on boundary straightness and seat count. The current paper describes an innovation that is entirely uncharacterised empirically.

**P2-II.** For CA, TX, and FL, report the normalised score spread across 50 seeds even if a definitive natural ratio cannot be claimed. A confidence interval on the natural ratio would be more informative than "---".

**P2-III.** Report actual achieved population balance (not just the ufactor target) for the three large-state failures noted in the discussion — Philadelphia, Detroit, NYC — to characterise the boundary-swap rebalancer's effectiveness.

**P2-IV.** Clarify the edge-cut units. The paper reports EC in km throughout, but METIS uses edge weight values directly. State explicitly how edge weights (boundary lengths in metres) are converted to km for reporting purposes.

---

## Verdict

Accept with Minor Revisions. GeoSection is a clean and well-motivated algorithmic contribution that fixes a genuine pathology in standard minimum-edge-cut redistricting. The implementation is technically correct, the normalisation is principled, and the self-correcting behaviour demonstrated in the Wisconsin tree is the paper's strongest empirical result. The weaknesses are primarily ones of completeness: the seed variance analysis is missing for the new normalised metric, Phase 2 is described but not evaluated, and the three large states have gaps that cover 27% of congressional seats.

The P1 item (seed variance for the normalised score) is essential to validate the core convergence claim for the new objective. The P2 items are recommended but not blocking. A revised paper with P1-I addressed will be a strong contribution at the intersection of graph partitioning and redistricting.

**Score: 3.5 / 4**
