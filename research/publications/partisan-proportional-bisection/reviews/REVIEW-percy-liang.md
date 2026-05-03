# Review R-5: Percy Liang
**Paper**: The Proportionality Compromise
**Date**: 2026-05-03
**Score**: 2.5 / 4

The analytical finding is clean. The 50-state classification is a well-structured empirical contribution. The partisan Lorenz curve is a creative import from inequality economics.

**Concerns:** Empirical methodology is underdeveloped. The tradeoff table is analytical but the paper draws empirical conclusions from it — conflating model predictions with observed outcomes. The proportionality gap metric is defined informally. No confidence intervals or sensitivity to vote data source.

**P1:**
1. Formally define the proportionality gap metric with an equation; appendix table for all 50 states.
2. Clearly distinguish analytical (formula-derived) vs. empirical (METIS output) results throughout.
3. Test sensitivity of σ classification to vote data source (presidential 2020 vs. Senate 2022).
4. Provide reproducibility statement: METIS version, seed, inputs for the 50-state table.

**Score: 2.5 / 4** — Weak accept / major revision.
