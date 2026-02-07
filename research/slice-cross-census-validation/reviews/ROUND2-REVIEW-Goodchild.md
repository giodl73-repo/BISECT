# Round 2 Review: Slice-Based Cross-Census Validation

**Reviewer**: Michael Goodchild (UC Santa Barbara)
**Expertise**: GIS theory, spatial analysis
**Round**: 2
**Date**: 2026-02-07

---

## Assessment of Revisions

The authors have done an excellent job addressing my major concerns. The new section 4.4 on spatial validation methodology is exactly what was needed. The Moran's I analysis (I = 0.42) quantifies spatial autocorrelation appropriately, and the K=3/5/7 sensitivity analysis demonstrates that the slicing approach is robust to MAUP effects (correlation > 0.85 across slice counts).

The boundary effects handling (majority-overlap assignment) is reasonable, though I still have minor concerns about edge cases. The addition of spatial validation literature citations (Roberts, Openshaw) properly grounds the work in GIScience methodology.

The census tract correspondence methodology (section 3.1) is now well-documented. The 18% split/merge rate is consistent with my knowledge of census geography evolution, and the population-weighted centroid definition is appropriate.

## Updated Score

**Score**: 3.5/4 — **Strong Accept**

(Increased from 2.5/4)

## Remaining Minor Issues

1. **Boundary districts**: The majority-overlap rule is simple but may introduce artifacts for districts that are nearly equally split between slices. Consider reporting how many such cases exist.

2. **Alaska/Hawaii**: Still not explicitly discussed. These non-contiguous states may behave differently—do slices work the same way?

3. **Figure quality**: The new Figure 4 (MAUP sensitivity) is clear, but would benefit from confidence intervals on the variance estimates.

## Verdict

**Accept** — All blocking issues have been addressed. The minor issues above are truly optional enhancements.

**Confidence**: High — The authors have demonstrated methodological rigor and appropriate engagement with spatial validation literature.
