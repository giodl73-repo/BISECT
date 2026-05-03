# Review — Ce Zhang (R-30)
**Score: 2.0 / 4**

## Summary
The paper proposes SubdivisionWeighter, a Rust implementation that encodes county boundary crossings as an additive edge-weight signal in METIS-based recursive bisection. The mechanism derives county FIPS from the existing 11-digit GEOID, penalizes cross-county edges by α, and composes with any existing EdgeWeighter. A 50-state sweep (2020 Census, seed=1) reports Table 1 comparing base vs. weighted plans on edge-cut and county splits.

## Strengths
- **Implementation elegance.** O(|E|) overhead is negligible relative to METIS solve time, and deriving county identity from the existing GEOID avoids any additional data download. The composable EdgeWeighter interface is a sound design choice for a production pipeline.
- **Reproducibility of inputs.** TIGER/Line adjacency files are public and versioned to the 2020 decennial. Parameters are fully specified (α=5, seed=1, year=2020), so the experiment is in principle re-runnable.
- **Practical scope.** CA (52 districts, ~8K tracts, ~40K edges) demonstrates the approach is not a toy; METIS handles the scale and the O(|E|) weighter adds no meaningful wall-clock cost.

## Weaknesses
1. **Unexplained missing data in Table 1 (critical).** Many EC cells show "--" with no explanation. The paper does not state whether "--" means the state was not run, the metric was not computed, computation failed, or the value is suppressed. This alone prevents reproducibility verification.
2. **Single seed, no convergence analysis.** Reporting only seed=1 means reported CS and EC values may be outliers. Without even a 5- or 10-seed sensitivity check on a representative subset (CA, TX, FL), the effect size attributable to α=5 cannot be distinguished from seed-to-seed variance.
3. **No statistical testing.** CS reductions are reported as raw counts with no confidence intervals, effect sizes, or paired tests across states. It is unclear whether the aggregate improvement is significant or driven by a few large states.
4. **α=5 unjustified.** The paper presents α=5 as the chosen parameter but provides no ablation across the parameter space. Without this, the claim "preserves political subdivisions" is empirically weak.

## Detailed Comments
The "--" issue in Table 1 must be resolved before publication. Even a footnote would suffice, but the current silence is a reproducibility red flag. Population deviation results look correct in direction (weighting should not worsen PD much if METIS respects balance constraints), but the paper should confirm PD stays within ±0.5% for all states. The CLI flag --alpha-county is a reasonable interface; the paper should note whether ordering of composition matters. CA is the right stress test for scalability; wall-clock timing comparisons (base vs. weighted) should be provided to quantify "negligible" concretely.

**Score: 2/4** — Solid engineering foundation, but the missing-data problem in Table 1 and the single-seed design make the empirical claims unreliable. Revise after resolving the EC gap and adding a seed sweep.
