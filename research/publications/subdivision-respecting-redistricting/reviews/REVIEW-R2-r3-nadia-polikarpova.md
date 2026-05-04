# Round 2 Review — Nadia Polikarpova (R-37)
**Score: 3.0 / 4** (prev: 2.0)

The restriction of the Monotonicity Proposition to exact min-cut solvers is formally correct and the Remark is precisely scoped. Table 1 covering all 44 states removes the credibility gap. The Pareto plateau behavior at α≥5 is an important invariant for practitioners.

**Fixed**: P1-A (formal correction), P1-C (full 44-state coverage), P2-A/B (Pareto frontier + knee identification).

**Remaining**:
1. P2-D (hierarchical composition stability): The algorithm spec uses a sum over 4 levels, but experiments are county-only. A tract at a county/MCD/place triple boundary accumulates weight (1 + α_county + α_mcd + α_place) — for three levels at α=5 each, this is 16× the base weight, potentially overwhelming any geographic signal. Either scope claims to county-only or provide empirical bounds for multi-level composition.
2. Pareto figure absent (table-only).

**Score**: 3.0 — core algorithmic correctness resolved; multi-level gap is the remaining formal concern.
