# G Ensemble Evidence Packages Closeout

**Status:** complete

## Outcome

G.1-G.3 now have an auditable package boundary for their ensemble evidence
claims. The wave did not invent external traces; it made the absence explicit
through a hash-bound missing-evidence package and validator coverage.

## Delivered

- Added `bisect-ensemble::evidence_manifest` with
  `g-ensemble-evidence-manifest v1`.
- Added synthetic positive and negative fixtures for package validation.
- Added referenced-file SHA-256 validation.
- Added
  `docs/examples/g-ensemble-evidence-packages/G.1-G.3+missing-evidence/`.
- Updated G.1-G.3 boundary sections, `docs/PAPERS.md`, and the paper scorecard
  to cite the package gap.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```

## Carry-Forwards

- Replace the missing-evidence package with active evidence packages when
  external GerryChain/ReCom traces are archived.
- Add election-input and metric-output packages before citing G.2 partisan or
  G.1/G.3 compactness percentiles as completed findings.
- Attach deterministic RPLAN/RCTX baseline packages for each compared state and
  scenario.
