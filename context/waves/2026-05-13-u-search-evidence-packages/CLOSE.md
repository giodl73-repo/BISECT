# U Search Evidence Packages Closeout

**Status:** complete

## Outcome

U.2 and U.4 now have a hash-bound synthetic evidence package and verifier
coverage for the audit shapes those papers require, while preserving the
boundary that real 50-state sweep and production CLI packages remain future
work.

## Delivered

- Added `bisect-ensemble::search_evidence` with
  `u-search-evidence-manifest v1`.
- Added
  `docs/examples/u-search-evidence-packages/synthetic-sweep-and-pt/`.
- Added verifier tests for manifest hash binding, parameter-sweep baseline/grid
  checks, parallel-tempering ladder/swap/record checks, and tamper rejection.
- Updated `docs/file-formats/manifests.md`.
- Updated U.2 and U.4 paper text, rebuilt PDFs, `docs/PAPERS.md`, and the
  scorecard.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble search_evidence
git diff --check
```

## Carry-Forwards

- Add real 50-state parameter sweep packages before citing U.2 for national
  robustness results.
- Add a production `bisect` parallel-tempering CLI, invocation test, and real run
  package before citing U.4 as a production search mode.
- Consider a broader `u-search-evidence-manifest v2` if archived sweep packages
  need multiple fixture files, plan packages, or statistical summaries.
