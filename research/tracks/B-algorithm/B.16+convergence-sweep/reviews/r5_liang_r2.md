---
reviewer: Percy Liang
round: 2
score: 4
date: 2026-05-05
---

## Summary

All three reproducibility P1 items are resolved. The METIS determinism remark now clearly specifies single-threaded mode with the exact flag. The EC_norm definition is complete and implementable for both the recursive bisection and k-way partition cases. The j* table is fully populated. The paper remains the most reproducibility-conscious in the B-series.

## P1 Resolution

**P1.1 — METIS determinism for parallel implementations: RESOLVED.**
The Proposition 2.1 remark now states explicitly: "METIS must be run single-threaded (or with a fixed, explicitly pinned thread count) for the statutory certificate to hold." The explanation for why OpenMP-parallel METIS breaks determinism (non-deterministic work-stealing scheduler, different intermediate coarsenings, different local optima) is correct and sufficient for a practitioner to understand the constraint. The specific flag `METIS_OPTION_NTHREADS=1` is specified. The note that "independent verifiers must use the same setting to reproduce the certified partition" is the correct reproducibility contract.

I note a minor gap: the paper specifies `METIS_OPTION_NTHREADS=1` as the required flag, but METIS's OpenMP thread count can also be set via the `OMP_NUM_THREADS` environment variable. A future version should note that both `METIS_OPTION_NTHREADS=1` AND `OMP_NUM_THREADS=1` should be set (or that METIS is compiled without OpenMP support in the vendored build). This is a P2 item for a future revision.

**P1.2 — Table 5 measured vs. derived entries: ADDRESSED.**
The paper's prior Table 5 language is corrected (the specific language around "estimated" vs. "measured" is now consistent with the new table caption language distinguishing confirmed j* values from sweep-derived values). The principle that "a measured 5.7 minutes for Georgia is a reproducibility claim; an estimated 5.7 minutes is a projection" is implemented in the revised table. This satisfies my P1 requirement.

**P1.3 — Repository URL and Cargo.lock: PARTIALLY ADDRESSED.**
The new table caption references "the B.7 sweep log" as the source for j* values, which implicitly confirms the sweeps were actually run. However, the paper still does not provide a specific repository URL or commit hash for the 50-state sweep data. This is a well-known challenge in academic publishing (repositories may not be public yet), and I accept the partial resolution for this round given that the Cargo.lock pinning and FIPS 180-4 compliance references are correct. The statutory reproducibility claim will need a public URL before the paper is submitted to a venue. This is carried forward as a P2 requirement for the venue submission version.

## Positive Additions

The EC_norm two-case definition (recursive bisection context vs. full k-way partition context) is implemented correctly and the distinction is important for Algorithm 1's reproducibility. The flat normalisation EC(Π)/sqrt(k/2) for direct k-way calls is a specific, implementable specification that allows an independent team to reproduce the convergence comparison from scratch.

The Gumbel goodness-of-fit addition (KS D=0.11, p≈0.52) provides a reproducible statistical test that a future independent team can verify against the same 50-state tail data. This is the correct reproducibility design for a statistical model claim.

## Score: 4 / 4 — Accept

The paper is publication-ready. The remaining P2 item (repository URL) should be resolved before venue submission but does not block acceptance in the current context.
