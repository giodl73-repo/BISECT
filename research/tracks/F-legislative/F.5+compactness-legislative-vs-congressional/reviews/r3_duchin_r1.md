# Review 3 — Moon Duchin
**Paper**: F.5: Compactness at State Scale — Algorithmic State Legislative Districts Outperform Congressional
**Round**: R1
**Score**: 3/4

## Summary

F.5 is the most mathematically substantive paper in the F track. The O(1/√k) scaling argument for mean PP as a function of the number of districts is well-formulated and the empirical validation is convincing. My review focuses on the rigor of the Proposition, the internal consistency of the scaling argument, and whether the claims about PP's behavior are correctly stated.

## Strengths

The Proposition (Section 2.3) is carefully stated: "under regularity conditions on the boundary of the geographic area" and "for a minimum-edge-cut partition." The O(k^{-1}) correction term is included. The empirical implication (Section 2.4) correctly derives the specific numerical prediction and validates it against data: c ≈ 0.120, predicted ΔPP ≈ 0.030, observed 0.027 — a 10% discrepancy that is within the expected range of the approximation error. This is honest and appropriate empirical validation.

The scale-invariance argument (Section 2.1) is correct: PP is dimensionless and invariant to uniform scaling. This establishes that the compactness advantage cannot be due to the absolute size of districts (smaller districts are not "more compact" by construction) and must be due to the boundary effect (fewer boundary districts as a fraction of total districts).

## Concerns

**C1 — Abstract claims resolution effect = 0.020, body says 0.013.** The abstract states "the resolution effect itself contributes approximately 0.020 PP units." Section 4.3 says "state house PP at block group (mean 0.401) versus congressional PP at tract (mean 0.361), a difference of 0.040 PP units" and decomposes: scale effect +0.027, resolution effect +0.013, total +0.040. The resolution effect is 0.013 in the body but 0.020 in the abstract. These cannot both be correct. The abstract figure appears to be derived from a different calculation (perhaps comparing house PP at BG to house PP at tract: 0.401 - 0.381 = 0.020 from F.1's Table 1, versus 0.401 - 0.388 = 0.013 from F.5's Table 1). The discrepancy arises because the 50-state mean PP differs between F.1 (0.381 at tract, 0.401 at BG) and F.5 (0.388 at tract, 0.401 at BG). The papers are internally inconsistent on the 50-state mean PP at tract resolution, and this inconsistency propagates to the resolution effect claim.

**C2 — Regularity conditions not specified.** The Proposition requires "regularity conditions on the boundary of the geographic area." The paper says these "exclude states with highly fractal boundaries (Alaska, Florida's coastline) and states with very large single geographic features." But "highly fractal" is not a formal condition, and the threshold for exclusion is not stated. For the Proposition to be mathematically rigorous, the regularity conditions must be stated precisely (e.g., the boundary is a rectifiable Jordan curve with bounded curvature, or the state graph has bounded degree, etc.).

**C3 — The PP∞ limiting value.** The Proposition states that PP_k → PP_∞ as k → ∞. This limiting value "is determined by the algorithm's behavior in the interior of the state." What is PP_∞? As k → ∞, districts become infinitesimally small and the problem reduces to dividing the state into infinitesimal cells. The limit of minimum-edge-cut redistricting as k → ∞ is not well-defined without additional assumptions (the districts become smaller than the geographic units). The paper should either restrict the Proposition to finite k or clarify what PP_∞ represents geometrically.

**C4 — Block-group resolution effect mechanism.** Section 4.1 explains that block groups allow "more precise boundary placement along the shoreline" and mountain ridgelines. This is qualitatively correct. However, the paper should note that the resolution effect is partially an artifact of measurement: when district boundaries are defined at tract resolution, the PP ratio is computed using the tract-resolution boundary, which has coarser geometry. When the same geographic boundary is traced at block-group resolution, the PP ratio changes because the boundary geometry changes. Part of the PP improvement from block-group resolution may be a measurement artifact (finer boundary tracing of the same districts) rather than a genuine geometric improvement (genuinely more compact districts). The paper should distinguish these two effects.

## Recommendation

Accept with revisions. C1 (abstract-body inconsistency) is an internal error. C2 (regularity conditions) affects the paper's mathematical rigor. Both must be addressed before publication.
