# Next Session Handoff: Algorithm-Family Paper Portfolio

**Status:** Benchmark-tier RPLAN package frontier complete through construction, exact, and search slices
**Owner:** maintainer (you)
**Last update:** 2026-05-11

## Current State

The T/U algorithm-family implementation, paper-writing, and tiny golden package
goals are complete:

- T.14-T.17 have reviewed draft PDFs and paper-level review/revision artifacts.
- U.0 and U.12-U.20 have reviewed draft PDFs and paper-level review/revision artifacts.
- U.16-U.20 now have published draft PDFs in `docs/papers/` and links in `docs/PAPERS.md`.
- Track T and Track U module-level simulated review syntheses exist and PP1 items are addressed.
- RPLAN golden packages exist for T.14-T.17 and U.16-U.19, with the U.20
  reference package in the audit-certificate examples.
- `rplan verify-certificate` verifies package certificates.
- `bisect verify --manifest path/to/package/manifest.json` now consumes the
  same package manifest shape as a bridge surface.
- The completed method-produced package goal added small generated
  packages with transcripts, not hand-sized verifier fixtures.
- The construction-side method-produced package is
  `docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/`.
- The search-side method-produced package is
  `docs/examples/rplan-method-packages/U.18+local-search-generated-descendant/`.
- The first benchmark-tier package is
  `docs/examples/rplan-benchmark-packages/T.14+spectral-grid10-benchmark/`.
- The exact-side benchmark-tier package is
  `docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/`.
- The second benchmark-tier package is
  `docs/examples/rplan-benchmark-packages/U.18+local-search-grid10-benchmark/`.

Completed goal doc:

`docs/specs/2026-05-11-method-produced-rplan-package-goal.md`

Active goal doc:

`docs/specs/2026-05-11-benchmark-rplan-package-frontier-goal.md`

## Latest Commits

- `5920074` Add lineage-bearing RPLAN golden package corpus
- `82aec4c` Bridge bisect verify to RPLAN packages
- `2e3b0ae` Upgrade RPLAN package evidence in U papers
- `917b88d` Add U18 method-produced RPLAN package
- `04f16fa` Queue method-produced RPLAN package goal
- `0a7242e` Expand U20 negative package fixtures
- `c1c1b9d` Define RPLAN golden package contract
- `9bd5c60` Add RPLAN golden package corpus goal
- `c643018` Add U20 public audit verification examples
- `646fceb` Complete algorithm paper portfolio integration

## Remaining Work

The residual work area is benchmark-tier public fixture depth and publication
packaging:

- Keep `docs/PAPERS.md` current as paper titles or notes change.
- Extend public fixture/package examples beyond the current tiny golden corpus,
  two method-produced packages, and construction/exact/search benchmark
  packages.
- Use `docs/concepts/t-u-portfolio-dependency-map.md` as the visual dependency
  map for T/U publication packaging.
- Promote larger real method-produced packages when non-toy fixture outputs are
  ready.

## Verification Notes

All U.16-U.20 paper builds completed successfully on 2026-05-11. The builds
reported only draft-grade LaTeX warnings: underfull boxes, first-pass natbib
warnings that resolved by final output, and MiKTeX update reminders.

## Good Next Move

Plan the next frontier beyond synthetic benchmark-tier packages:

1. Choose whether the next package should use pinned real data or stay
   synthetic but cover exact/flow/clustering families.
2. Keep larger outputs in a release artifact bundle if they are no longer
   small enough for in-repo review.
3. Preserve the same verifier path: RPLAN/RCTX, certificate, manifest, command
   transcript, method transcript, and benchmark notes.
