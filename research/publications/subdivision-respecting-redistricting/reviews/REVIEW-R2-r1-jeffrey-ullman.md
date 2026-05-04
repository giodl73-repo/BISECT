# Round 2 Review — Jeffrey Ullman (R-50)
**Score: 2.5 / 4** (prev: 1.0)

The revision has addressed my core blocking concern. The Monotonicity Proposition is now formally correct — restricted to exact solvers with an honest Remark about METIS being a heuristic. The Pareto frontier data finally gives the paper a quantitative backbone: the knee at α≈2 and plateau by α=5 is a genuinely useful engineering result.

**Fixed**: P1-A (monotonicity remark), P1-C (all 44 states), P2-A/B (Pareto table + α=5 justification).

**Remaining**:
1. No Pareto frontier *figure* — a table at 7 α values obscures whether the knee is sharp or gradual and whether GA is representative. The authors promised a figure; a table is incomplete.
2. P2-D (multi-level composition stability) entirely unaddressed. The paper describes an algorithm over 4 levels (county/MCD/place/VTD) but experiments validate only county-level. That gap between described and tested must close.

**Score**: 2.5 — major R1 blocking items resolved; figure and multi-level gap prevent 3.0.
