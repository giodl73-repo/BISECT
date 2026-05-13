# K Exact Reock Evidence Packages Closeout

**Status:** complete

## Outcome

K-track Reock claims now have a hash-bound exact polygon-MBC smoke package and
verifier coverage while preserving the production `reock()` output as the
centroid-radius proxy used by `all_metrics()`.

## Delivered

- Added exact polygon-MBC reference helpers in `bisect-analysis`.
- Added `k-reock-evidence-manifest v1` and verifier tests for hash binding,
  exact fixture values, and exact/proxy divergence.
- Added
  `docs/examples/k-reock-evidence-packages/exact-mbc-smoke/` with unit-square
  and 3-4-5 right-triangle fixtures.
- Updated K.0, K.2, K.7 paper text, rebuilt PDFs, `docs/PAPERS.md`, and the
  scorecard.
- Documented the manifest in `docs/file-formats/manifests.md`.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis
git diff --check
```

## Carry-Forwards

- Add real district polygon exact-MBC packages before making district-scale
  exact-MBC replay claims.
- Keep production `reock()` migration separate from this evidence/reference
  slice because `all_metrics()` still exposes the centroid proxy.
- Consider a CLI verifier if K papers need public command-line replay rather
  than crate-level tests.
