---
wave: paper-rubric-uplift
pulse: 03
status: complete
depends_on: [02]
governing_roles:
  - MERIDIAN
  - DATUM
  - SCALE
  - COVENANT
  - LEDGER
  - BENCHMARK
---

# Pulse 03 - Older Search Paper Refresh

## Mission

Lift the older U-track search/optimization papers by aligning their claims with
the current search, ensemble, certificate, and package architecture. The pulse
should focus on implementation/evidence boundaries rather than rewriting the
mathematical exposition from scratch.

## Scope

| Surface | Target | Non-goal |
|---|---|---|
| U.0-U.7 | Current search role, status, diagnostics, and implementation boundaries. | Full new empirical experiments. |
| U.11, U.13-U.15 | Bridge older framing to U.16-U.20 exact/search/certificate artifacts. | Reopening golden U.16-U.20 papers. |
| Ledgers | Update scorecard, paper review ledger, wave docs, and active goal. | External peer-review claims. |
| Build artifacts | Rebuild changed PDFs and copy them to `docs/papers/`. | Repository-wide paper rebuild. |

## Pre-implementation Scout

Run and record:

```powershell
rg -n "ConvergenceSweep|percentile|bisection-ensemble|branch-and-cut|branch-and-price|large-neighborhood|evolutionary|audit certificate|RPLAN|RCTX|status|optimal|converge|representative|legal" research/tracks/U-search-optimization crates docs/papers docs/algorithm-atlas docs/specs
Get-ChildItem -Recurse research\tracks\U-search-optimization | Select-Object FullName
git --no-pager status --short
```

## Deliverables

- [x] Materialize this pulse and fork context.
- [x] Run the scout commands and decide whether this is paper/docs-only.
- [x] Revise U.0-U.7, U.11, and U.13-U.15 source where stale implementation,
  convergence, optimality, representativeness, package, or legal claims appear.
- [x] Update `PAPER-QUALITY-REVIEW.md`, `ALGORITHM-PAPER-SCORECARD.md`,
  `WAVE.md`, `docs/PAPERS.md`, and the active goal.
- [x] Rebuild changed PDFs and copy them to `docs/papers/`.
- [x] Run focused validation and commit.

## Completion Notes

- Decision: paper/docs-only. Scout found current package substrates for several
  methods, but the slice required claim-boundary alignment rather than code.
- Added current implementation/evidence boundary sections to U.0-U.7, U.11,
  and U.13-U.15.
- Tightened stale claims around convergence, representativeness, global
  optimality, CLI availability, legal conclusions, and package evidence.
- Remaining ceilings: U.2 needs archived sweep packages, U.4 needs a production
  CLI/package path, and U.6 exact claims need replayable solver/model artifacts
  for real instances.
- Validation: rebuilt U.0-U.7, U.11, and U.13-U.15 PDFs; `cargo fmt`;
  `cargo test -p bisect-ensemble` (113 passed, 5 L2 ignored); `git diff --check`.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```

Build every changed U-track paper with:

```powershell
pdflatex -interaction=nonstopmode main.tex
bibtex main
pdflatex -interaction=nonstopmode main.tex
pdflatex -interaction=nonstopmode main.tex
```
