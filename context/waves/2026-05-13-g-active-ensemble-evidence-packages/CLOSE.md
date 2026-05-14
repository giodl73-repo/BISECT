# G Active Ensemble Evidence Packages Closeout

**Status:** complete

## Outcome

G.1-G.3 now have an active synthetic evidence package that exercises every
artifact role required by external-ensemble percentile claims. The package
validates manifest and hash-binding mechanics, while the papers still reserve
real empirical percentile claims for real external trace, election, metric,
diagnostic, RPLAN, and RCTX packages.

## Delivered

- Added
  `docs/examples/g-ensemble-evidence-packages/G.1-G.3+active-synthetic/`.
- Added a `bisect-ensemble::evidence_manifest` test that validates the docs
  package hashes.
- Updated G.1, G.2, and G.3 current-boundary sections.
- Rebuilt G.1/G.2/G.3 PDFs.
- Updated `docs/PAPERS.md` and `docs/papers/ALGORITHM-PAPER-SCORECARD.md`.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble active_g1_g3_package_fixture_validates
git diff --check
```

## Carry-Forwards

- Add real external GerryChain/ReCom trace packages with sampler diagnostics.
- Add real election-input packages for G.2 partisan outcome claims.
- Add real metric-output packages for G.1/G.3 compactness percentile claims.
- Attach real BISECT RPLAN/RCTX baseline packages for each compared scenario.
