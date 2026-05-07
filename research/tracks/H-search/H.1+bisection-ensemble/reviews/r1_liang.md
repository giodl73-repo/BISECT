# Review 5: Yiting Liang (Computational Social Science, Reproducibility)
**Paper**: H.1: BisectionEnsemble
**Round**: 1
**Recommendation**: Major Revision (Reproducibility)

---

## Summary

BisectionEnsemble is a technically interesting approach to improving redistricting ensembles. My review focuses on the empirical claims: whether the GerryChain bipartition failure comparison is reproducible, whether the compactness and partisan results in Tables 1-3 are grounded in actual pipeline data, and whether the experimental setup is specified to a standard that permits independent replication.

## GerryChain Comparison (Table 1): Not Reproducible as Stated

Table 1 is the paper's central empirical claim: GerryChain fails on Texas ($k=38$) with acceptance rate below 1%, while BisectionEnsemble succeeds. This comparison is the primary motivation for the paper. Unfortunately, the experiment is specified insufficiently for independent reproduction.

Missing information:
1. **GerryChain version**: GerryChain has undergone substantial API changes across versions. The acceptance rate and failure behavior depend on the version.
2. **Configuration**: What tolerance $\varepsilon$ was used for the GerryChain population balance constraint? Acceptance rates are highly sensitive to this parameter.
3. **ReCom variant**: GerryChain supports multiple ReCom variants (reversible, non-reversible, two-step). Which was used?
4. **Initial plan**: What was the initial plan for the GerryChain run? The acceptance rate for a stalled chain depends on whether the initial plan is feasible and compact.
5. **Hardware and random seed**: None specified.

The paper claims "$<$1% of ReCom steps are accepted at 100 steps, yielding 0--1 accepted plans." Is this from a single run or multiple runs? For a stochastic algorithm, a single run of 100 steps is insufficient to characterize the acceptance distribution. If this is one run, the "<1%" figure has very high variance.

For GerryChain's known behavior on TX, the community has published informal benchmarks showing that it does stall for $k=38$ without custom tuning. But "known to stall" and "measured at <1% acceptance in this experiment" are different claims, and only the latter supports Table 1.

**Recommendation**: Provide a reproducibility appendix or code repository with GerryChain configuration, version, seed, initial plan, and per-step acceptance log for all three states.

## Compactness Data (Table 2): Are These Actual Pipeline Numbers?

Table 2 reports mean Polsby-Popper compactness for each method and state:
- NC standard bisection: 0.217
- NC BE($p=0.5$): 0.242 (+11.5%)
- NC BE($p=0.0$): 0.261 (+20.3%)
- WI standard bisection: 0.198
- TX standard bisection: 0.184

These numbers need verification. Polsby-Popper for census tract-level congressional districts in NC, WI, and TX are published in multiple prior studies. The NC baseline of 0.217 is consistent with the range reported for algorithmic redistricting in Herschlag et al. (2020), which provides some face validity.

However, the paper provides no information on:
- Which census year's TIGER geometry was used for PP computation (area and perimeter are geometry-dependent).
- Whether PP is computed on raw tract-level district polygons or dissolved to simplified boundaries.
- Whether the $+11.5\%$ relative improvements are consistent across multiple BisectionEnsemble runs (or from a single run each).

If these are from a single BisectionEnsemble run per method, the reported compactness values have variance that is not reported. The ensemble distribution (100 accepted plans per node) produces a distribution of compactness values, and the paper should report the mean and standard deviation of PP across the ensemble accepted plans, not just the final selected plan's PP.

The minimum-cut variant ($p=0.0$) reporting +20.3% improvement for NC and +25.5% for TX is a particularly strong claim. Selecting the minimum edge-cut plan from each node's ensemble should improve compactness by construction, but the magnitude depends on how diverse the local ensemble is. Without reporting the ensemble PP distribution (not just the selected plan's PP), it is impossible to assess whether 100 steps are sufficient to find substantial improvement over the METIS baseline.

## Partisan Outcomes (Table 3): Single-Run Results

Table 3 reports seat counts as deterministic values (7D/7R, 2D/6R, etc.). These are from single runs of each method. For a stochastic algorithm, seat counts across multiple runs would provide a distribution. A 7D/7R NC result that holds across 50 BisectionEnsemble runs with different seeds would be a much stronger claim than a single observation.

The WI one-seat shift (3D/5R at $p=0.5$ vs. 2D/6R at standard) is presented as meaningful, but a single-run comparison has no statistical power to attribute this shift to the method rather than to stochastic variation. If WI BE($p=0.5$) gives 2D/6R on 30 out of 50 runs and 3D/5R on the other 20, the one-seat shift is noise. If it consistently gives 3D/5R, the shift is a property of the method. The paper does not say.

**Recommendation**: Report seat-count distributions across at least 10 independent BisectionEnsemble runs per method per state, or report the fraction of runs that produce each seat count.

## Runtime (Table 4): Plausible but Hardware-Underspecified

Table 4 runtime numbers are plausible in order-of-magnitude:
- NC standard bisection: 0.8s; BE($T$=100): 4.3s (~5x overhead)
- TX standard bisection: 2.1s; BE($T$=100): 9.7s (~5x overhead)

The ~5x overhead for 100 ReCom steps per node is consistent with BisectionEnsemble's $O(T)$ scaling relative to a single METIS call. The parallelism efficiency claim (19 nodes / 16 cores = 1.2 imbalance for TX) is geometrically correct.

Missing: CPU model, clock speed, Rust compiler version, METIS version, whether METIS is called as a library or subprocess. Without these, the numbers cannot be replicated.

## Data Provenance: CCES Citation Is Incorrect

The paper states: "Partisan outcomes are measured against 2020 presidential two-party precinct returns interpolated to census tracts (Kuriwaki 2023)." The Kuriwaki (2023) citation is to the CCES (Cooperative Congressional Election Study), which is a survey. It is not a source of precinct-level presidential returns. Precinct-level 2020 returns are available from the Voting and Election Science Team (VEST) at the Harvard Dataverse. The citation needs correction, and the interpolation method (how precinct returns are weighted to census tracts) needs specification.

This is not a trivial error: partisan seat counts can be sensitive to the interpolation method, and readers need the correct data source to replicate Table 3.

## Missing Baseline: What Is Standard Bisection's Ensemble Distribution?

The paper compares BisectionEnsemble (with 100 ensemble steps) against "standard bisection" (a single METIS call). A more informative baseline would be MultiSeedMETIS($N=100$): run METIS 100 times with different seeds at each node and select the plan at rank $\lfloor p \times N \rfloor$. This baseline controls for the effect of sampling ($N=100$ samples) without the ReCom chain, and would test whether the ReCom chain provides additional value beyond what seed-level METIS diversity achieves.

Without this comparison, the paper cannot distinguish "BisectionEnsemble is better than single METIS" from "any 100-sample method is better than single METIS." The ReCom chain is the novel component, and it deserves a comparison that isolates its contribution.

## Overall Assessment

The experimental section has three recoverable problems: (1) GerryChain comparison is underspecified and not reproducible; (2) all results appear to be single-run observations, not distributions; (3) the partisan data source is incorrectly cited. The runtime results and compactness direction are plausible. If these experiments are drawn from actual pipeline runs (as the CLAUDE.md suggests), the underlying data exist and can be used to produce proper distributions and correct citations. The revision should include a reproducibility statement and either a public code repository or a supplementary appendix with full experimental configuration.
