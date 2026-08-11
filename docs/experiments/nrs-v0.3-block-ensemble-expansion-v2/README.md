# NRS v0.3 NH/NM/GA Block-Ensemble Expansion v2

Status: implementation gate only. No preflight or governed v2 process has run.

This package is wholly separate from the failed and closed v1 package. Its
initial ledger has zero completions, zero retained bytes, zero runner wall
time, and no failure. The dedicated verifier must pass this empty active state
before Stage 0 can begin.

The frozen protocol is
`docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v2.md`. Every future
process must be created through the capacity-admitted v2 runner. A later pulse
must reverify inputs, executable custody, runner tests, and this package before
authorizing the six excluded preflights.
