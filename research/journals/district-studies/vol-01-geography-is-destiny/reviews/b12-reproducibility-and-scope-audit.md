---
journal: District Studies
volume: 1
title: "Geography Is Destiny"
status: audit-note
updated: 2026-05-09
source: T.5 ProportionalSection
claim-class: formal/empirical
---

# T.5 Reproducibility and Scope Audit

## Result

T.5 remains the strongest lead source for the issue's formal geography claim,
but it should enter Vol. 1 as a reviewed summary with caveats, not as a locked
result-bearing article. Three reproducibility gates remain open:

- full METIS parameter vector;
- Table 1 seed determinism or seed variance;
- C(G) estimator provenance.

## What The Source Supports

T.5 supports this public-audition claim:

> Proportional partisan outcomes can be geographically hard to achieve under
> contiguous single-member districts. The difficulty is not just a matter of
> choosing the right algorithmic parameter; it depends on the spatial
> distribution of voters.

This claim is suitable for District Studies Vol. 1 because it is diagnostic. It
explains why a geography-only baseline may produce partisan asymmetry without
turning that asymmetry into a legal conclusion.

## Current Evidence

The current T.5 source reports:

- 2020 Census data and 2020 presidential election results.
- METIS 5.1.0.
- `ncon=2`.
- vertex weights `[population, D_votes]`.
- `tpwgts` from the paper's formula.
- `ubvec[0] = 1.001`.
- tested partisan tolerances `eta in {1.05, 1.10, 1.20, infinity}`.
- 30 seeds per run.
- 1.5 percent balance tolerance.

The current source does not visibly report:

- `niter`;
- `ncuts`;
- `numbering`;
- whether Table 1 values are identical across seeds, averages, maxima, or other
  summaries;
- whether C(G) is computed from the Lorenz curve analytically or inferred from
  METIS behavior empirically.

Implementation search note:

- `reviews/b12-implementation-provenance-note.md` records the current Rust
  wrapper defaults and the remaining mismatch with older manifest/Python
  defaults. Those implementation defaults are useful context, but they do not
  prove the T.5 paper-run vector.

## Review Evidence

Round 3 review records the same unresolved gates:

- Karypis asks for `niter`, `ncuts`, and `numbering`.
- Karypis asks whether C(G) is Lorenz-analytical or METIS-empirical.
- Duchin asks whether Table 1 is deterministic across 30 seeds or averaged.
- Liang asks for range, SD, or an explicit statement that all seeds produce
  identical outcomes.

## Public-Copy Rules

Use:

- "geography can constrain proportional outcomes";
- "the T.5 source argues";
- "under the tested data and parameter settings";
- "candidate explanation";
- "source-chain gate remains open."

Avoid:

- "T.5 proves geography determines partisan outcomes";
- "cannot gerrymander";
- "algorithmic neutrality is established";
- "courts can use this as a constitutional test";
- exact Table 1 claims without parameter and seed-variance disclosure.

## Legal-Scope Note

T.5's `Scope of Claims` section is appropriate for Vol. 1. It explicitly says
the paper does not show that courts can order the method, that any outcome is
constitutionally required, that the partisan-constrained method is neutral, or
that T.5 always improves proportionality.

Later legal language in the source is stronger than District Studies should
adopt. Vol. 1 should not say an algorithm "cannot gerrymander" or lacks the
necessary mental state. That is a legal argument for a later journal and should
receive BOUNDARY/WARD review before public use.

## Gate To Provisional

Before T.5 can anchor a locked article, add one short source-chain appendix or
review closure note with:

1. METIS `niter`, `ncuts`, and `numbering`.
2. Table 1 aggregation rule across 30 seeds.
3. Range, SD, or deterministic-seed statement for each reported cell.
4. C(G) estimator description.
5. Confirmation that Vol. 1 quotes only the `Scope of Claims` framing, not the
   stronger legal theory.
