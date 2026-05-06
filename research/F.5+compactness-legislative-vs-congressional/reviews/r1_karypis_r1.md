# Review 1 — George Karypis
**Paper**: F.5: Compactness at State Scale — Algorithmic State Legislative Districts Outperform Congressional
**Round**: R1
**Score**: 3/4

## Summary

F.5 provides the mathematical explanation for the compactness advantage of state legislative maps over congressional maps. The central result — that mean PP scales as PP_∞ - c/√k + O(k^{-1}) — is stated as a Proposition and validated empirically (c ≈ 0.120, predicted improvement 0.030, observed 0.027). This is a substantive mathematical contribution with direct algorithmic implications.

## Strengths

The hexagonal packing derivation (Section 2.2) is correct and provides the right geometric intuition. The argument that boundary districts scale as O(√k) for a compact state, giving a boundary fraction of O(1/√k), is the correct leading-order analysis. The boundary effect (state boundary irregularity inflates perimeters of boundary districts; as k increases, the fraction of boundary districts decreases) is the operative mechanism, and it is correctly identified.

The empirical validation (c ≈ 0.120 fitted, 0.030 predicted vs. 0.027 observed for 8-congressional/100-house-district states) is a convincing check. The cross-census stability (mean advantage: 0.026 in 2000, 0.027 in 2010, 0.028 in 2020) provides additional validation that the result is not census-cycle-specific.

The 50-state table (Table 1) is the most useful empirical contribution: congressional PP, house PP at tract, house PP at block group, and house seat count for all 50 states. This is a comprehensive reference dataset.

## Concerns

**C1 — Proposition regularity conditions exclude Alaska and Florida.** The Proposition states that the scaling law holds "under regularity conditions on the boundary of the geographic area," excluding "states with highly fractal boundaries (Alaska, Florida's coastline)." Yet Table 1 includes Florida (congressional PP 0.344, house PP 0.377, BG 0.393) and Alaska (congressional PP 0.521, house PP 0.489, BG 0.512). The Proposition's regularity conditions would exclude Florida from the theoretical analysis, yet the empirical table includes Florida without comment. The paper should either extend the Proposition to cover fractal boundaries or clearly mark the excluded states in Table 1.

**C2 — Alaska, Wyoming, Montana exceptions unexplained.** Table 1 shows that Alaska has congressional PP 0.521 versus house PP 0.489 — state house PP is lower than congressional PP. The paper notes this in Section 3.2: "Alaska, Wyoming, and Montana are exceptions (comparison is between 1 congressional seat and 40--100 state house seats)." The explanation is that for states with 1 congressional district (the whole state), the congressional "district" is the state itself, and state PP is high because the state boundary is relatively compact. But for 40--60 state house districts, the algorithm must carve compact districts from a geographically complex state (Alaska's coastline, islands). The theoretical prediction Δ PP ≈ c(1/√k₁ - 1/√k₂) would give a positive value (house more compact than congressional) even for Alaska. The Proposition must be inconsistent with these exceptions, suggesting the regularity conditions are more restrictive than stated.

**C3 — O(1/√k) scaling constant c estimated from subsample.** The paper fits c ≈ 0.120 from "the subset of states with approximately 8 congressional and 100 state house districts." How many states satisfy this criterion? The typical state has 8 congressional seats and 100 state house seats only approximately — the actual range is wider. A more rigorous estimation would fit c from the full 50-state dataset, not a subset.

**C4 — Resolution effect decomposition arithmetic.** Section 4.3 decomposes the total 0.040 PP improvement (state house BG vs. congressional tract) into 0.027 (scale effect) + 0.013 (resolution effect) = 0.040. But Table 1 shows mean house PP at tract = 0.388 and at block group = 0.401, a difference of 0.013 — consistent. Congressional tract PP = 0.361. So scale effect = 0.388 - 0.361 = 0.027 and resolution effect = 0.401 - 0.388 = 0.013. These add correctly. However, F.5's abstract says the resolution effect "contributes approximately 0.020 PP units" while Section 4.3 says "+0.013 PP units." This is an internal inconsistency.

## Recommendation

Accept with revisions. C4 (abstract-body inconsistency in resolution effect magnitude) is an internal error that must be corrected. C1 (Alaska/Florida Proposition applicability) and C2 (exceptions to the scaling law) require clarification.
