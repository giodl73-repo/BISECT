# G Ensemble Evidence Packages Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Turn the G.1-G.3 ensemble comparison claims from boundary-scoped paper
statements into auditable evidence packages. The target is not to invent new
headline numbers; it is to find, validate, package, or explicitly mark missing
the traces, election inputs, metric outputs, diagnostics, and manifests that
support those papers.

## First Target

G.1-G.3 are first because the previous paper-rubric wave identified their shared
ceiling: headline compactness and partisan percentiles need archived external
GerryChain/ReCom traces, deterministic BISECT plan packages, election-data
models, compactness metric versions, diagnostics, and verifier status.

## Acceptance

- [x] Inventory existing trace, election, metric, and package artifacts relevant
  to G.1-G.3.
- [x] Define the package schema/manifest contract for external ensemble evidence.
- [x] Add positive and negative validation coverage for the package shape.
- [x] Create at least one minimal G evidence package or, if data is unavailable,
  a hash-bound missing-evidence manifest that makes the gap explicit.
- [x] Update G.1-G.3 papers/ledgers only after package evidence exists.
- [x] Close the wave with validation commands and commit evidence.

## Closeout

The wave created `g-ensemble-evidence-manifest v1`, positive and negative
fixtures, consumer hash validation, and
`docs/examples/g-ensemble-evidence-packages/G.1-G.3+missing-evidence/`.
G.1-G.3 ledgers now point to that package as a validated evidence gap rather
than treating the missing external traces, election inputs, metric outputs,
diagnostics, and RPLAN/RCTX baselines as implicit.

## Non-Goals

- Do not run large external GerryChain experiments unless the local data and
  dependencies are already available and the pulse explicitly scopes them.
- Do not claim legal sufficiency, ensemble representativeness, or final
  percentile results from incomplete packages.
