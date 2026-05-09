# Review — C.1: Spatial Resolution and Algorithmic Redistricting (MAUP)
**Reviewer**: George Karypis (Graph partitioning, METIS, computational methods)
**Round**: 1
**Score**: 3/4
**Verdict**: Minor Revision

## Summary

C.1 tests whether the bisect algorithm's key findings are sensitive to the choice of spatial unit (county, tract, block group). The central finding — PP stability within 2–3% RSI across a 130× unit-count range, seat counts stable within ±1 — is credible and important for validating the pipeline's robustness. The five-state experiment is well-designed; the 50-state RSI summary adds breadth. The main gap is that all results are single-run (seed 42), and the paper does not establish whether the stability finding holds across seeds or is a seed-42 artifact.

## Strengths
- The 130× unit-count range (county to block group) is sufficiently broad to test MAUP robustness.
- The RSI metric (max−min/mean) is appropriate for resolution stability measurement.
- The monotonic PP increase with resolution is both predicted theoretically and observed empirically — the agreement with the log-linear prediction is a genuine contribution.
- The 50-state RSI distribution provides national-scale evidence beyond the five focus states.

## Concerns
- **Single-seed results**: Table 1 caption notes "All results single-run, seed 42." The stability finding — that PP varies by 2–3% RSI — is computed over one METIS realisation per state per resolution. If seed 42 produces anomalously stable or unstable results, the RSI figures could be misleading. The paper should report multi-seed RSI (mean ± SD across 5+ seeds) for at least the five focus states.
- **County resolution validity**: County-level redistricting requires population balance across counties. For states like NC and TX where congressional district populations (~750,000) are much smaller than many counties (e.g., Mecklenburg County, NC: ~1.1M), a county-level redistricting is not legally feasible. The paper should note this limitation: the county-level results test robustness to coarse aggregation, not a practical redistricting option.
- **RSI interpretation**: The paper reports RSI but does not compare it to the DIA statutory tolerance (0.5% population deviation). An RSI of 2–3% for PP is small relative to the PP range across states but may be large relative to the precision needed for legal proceedings.

## Required Changes (P1/P2)
- **P1**: Add multi-seed RSI estimates for the five focus states (5+ seeds). Report mean RSI ± SD. If seed-42 RSI is within 1 SD of the mean, the single-seed results are adequately representative.
- **P2**: Add a note clarifying that county-level redistricting is an analytical device for testing MAUP sensitivity, not a practical redistricting option.
- **P2**: Compare the RSI figures to practical thresholds (PP compactness litigation standards, if any) so readers understand whether 2–3% RSI is substantively significant.
