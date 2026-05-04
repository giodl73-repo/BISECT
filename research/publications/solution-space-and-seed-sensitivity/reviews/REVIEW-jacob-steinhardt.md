> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: R-44 Jacob Steinhardt
**Paper**: The Solution Space of Minimum-Edge-Cut Redistricting: Seed Sensitivity and Partisan Variance
**Date**: 2026-05-01
**Score**: 3.0 / 4

---

## Summary

The paper's central threat model — a challenger who claims the seed was chosen for partisan reasons — is well-defined and the Fiedler certificate is a credible defense. But the paper omits two significant attack vectors: (1) gaming the ε-tolerance in the near-minimum candidate filter, and (2) the compactness approximation (circular approximation for external perimeters) as a manipulation surface. Both need analysis before the paper's robustness claims hold.

---

## Strong Points

1. **The "you could have done better" attack is the right thing to defend against**, and the certificate mechanism correctly characterizes the cost of mounting that attack. A challenger who runs more seeds to find a "more compact" plan is doing legitimate computation, and the certificate bounds exactly how much improvement is achievable. This is the correct framing.

2. **The TIGER data source closes one major manipulation vector.** TIGER shapefiles are federal records with SHA-256 manifests. An attacker cannot change the graph without changing the manifest and being detected. This is correctly identified and defended.

3. **The §104(e) constraint (no partisan data as input) is the statute's primary defense**, and the paper correctly treats all four criteria (edge-cut, compactness, proportionality, CompactBisect) relative to this constraint. The analysis of which criteria are "attackable" (compactness choices) vs. "indisputable" (edge-cut) is the right framework.

---

## Concerns and Weaknesses

### P1: Critical

**P1.1 — The ε-tolerance parameter in CompactBisect is an attack surface that is not analyzed.**

CompactBisect selects the plan with highest GMPP among candidates within ε = 5% of the minimum edge-cut (the "near-minimum set"). The paper notes that ε = 0.05 is a parameter, but does not analyze whether ε itself can be manipulated.

Attack vector: A partisan actor could argue that ε = 0.05 is too permissive, and that a different ε (say ε = 0.001) would produce a more compact plan with a different partisan outcome. Or conversely, ε = 0.20 is proposed to include more candidates — and among those candidates, a partisan-favorable plan happens to have high GMPP. The paper must show either: (a) the selection outcome is insensitive to ε over a reasonable range (e.g., ε ∈ [0.01, 0.20]), or (b) ε is a statutory parameter that must be published before any runs and cannot be changed post-hoc.

The second defense is the statutory fix: if ε is in the parameter table (like the seed and the reference binary SHA-256), it cannot be chosen after seeing the partisan outcomes. But the paper currently does not propose this, and the § 107(c) parameter table description in the statute companion does not list ε.

**P1.2 — The circular approximation for external perimeters is an attack surface.**

The implementation approximates each tract's external perimeter as 2√(πA) (the circular approximation). This choice:
- Is not in the TIGER files (it's a computed quantity)
- Is not uniquely determined by the graph (it depends on the approximation choice)
- Affects the GMPP computation and thus the CompactBisect selection

An adversary could propose a different approximation (e.g., elliptical, or exact polygon perimeter) and show that it selects a plan with a different partisan outcome. Or argue that the choice of circular approximation was made after seeing which partisan outcome it would produce.

The fix: use exact TIGER perimeters (sum of coordinate-derived segment lengths), which are uniquely determined by the shapefile. The paper should switch to exact perimeters. If exact perimeters are too expensive, they need to be characterized as a component of the TIGER-derived graph structure (precomputed at adj.bin build time, not at runtime).

### P2: Significant

**P2.1 — The Fiedler certificate does not bound the partisan variance within the certified region.**

Proposition 3.1 guarantees that no challenger can find a plan with GMPP more than 5% higher than the certified plan. But it says nothing about whether all plans within the certified region have the same partisan outcome. For WI, the near-minimum 5% region contains plans producing 2D, 3D, and 4D outcomes. A challenger could argue: "your 5% tolerance includes plans with different partisan outcomes; you chose the one inside the band that happened to favor one party."

The defense: show empirically (for the key contested states) that the partisan variance WITHIN the certified region (ratio ≥ 1−δ) is small. If all plans with ratio ≥ 0.95 produce the same outcome (e.g., 3D in WI), the attack collapses. If the variance is large, the paper needs a stronger defense — e.g., certify with δ = 0.01 (within 1%) so that the certified region is so tight that partisan variation within it is negligible.

**P2.2 — The convergence i.i.d. assumption enables a Sybil-style attack.**

The paper models convergence as i.i.d. draws from F. But a sophisticated adversary could observe that seed k always produces plans with lower edge-cut than seed k-1 for a specific graph structure, violating the i.i.d. assumption. METIS's internal seed usage is not uniformly random across the solution space; some seeds may produce correlated outcomes. If the adversary knows METIS's internal heuristics, they could identify seeds that systematically favor certain outcomes. The paper should discuss whether METIS's seed mechanism satisfies sufficient randomness for the i.i.d. assumption, or use a different seed-generation protocol (e.g., seed = SHA-256(census_id ∥ iteration_index)).

### P3: Minor

**P3.1 —** The paper defends the δ = 0.05 threshold but does not characterize how the certified region's partisan composition changes as δ varies. A sensitivity analysis (δ ∈ {0.01, 0.02, 0.05, 0.10}) showing that partisan outcomes within the certified region are stable would substantially strengthen the robustness claim.

**P3.2 —** The content-derived seed proposal (SHA-256(census_release_id ∥ "DIA_SEED_V1")) from the plan.md discussion section is not in the paper. This is the cleanest defense against seed-choice attacks and should be in the paper, not just the plan. It belongs in the discussion section as the primary statutory recommendation.

---

## Verdict

The threat model is correct and the certificate is a genuine defense. The two main weaknesses — ε as an unanalyzed attack surface and the circular approximation as a manipulation vector — are fixable. The Sybil-attack concern (METIS seed correlation) needs at minimum a discussion. I would accept this paper after addressing P1.1 and P1.2 and with a discussion of P2.2.
