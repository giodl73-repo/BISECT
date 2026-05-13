---
wave: paper-rubric-uplift
pulse: 04
status: complete
depends_on: [03]
governing_roles:
  - SCALE
  - DATUM
  - COVENANT
  - BENCHMARK
  - LEDGER
---

# Pulse 04 - Ensemble Data Cleanup

## Mission

Refresh the weaker G-track ensemble papers so their empirical, diagnostic,
percentile, convergence, and short-burst claims match the current
`bisect-ensemble`/RPLAN/RCTX evidence boundary.

## Scope

| Surface | Target | Non-goal |
|---|---|---|
| G.1-G.3 | GerryChain comparison, partisan outcome, and compactness-position claims. | New external GerryChain experiments. |
| G.5 | Convergence and mixing diagnostics. | Proving asymptotic convergence from short runs. |
| G.12 | Short-burst paper/spec alignment. | Rewriting G.6/G.10/G.11/G.13. |
| Ledgers | Update scorecard, review ledger, wave docs, active goal, and paper index. | Claiming external validation not present in artifacts. |
| Build artifacts | Rebuild changed PDFs and copy to `docs/papers/`. | Repository-wide paper rebuild. |

## Pre-implementation Scout

Run and record:

```powershell
rg -n "GerryChain|percentile|R-hat|ESS|autocorrelation|mixing|convergence|representative|stationary|short-burst|burst|diagnostic|validation|external|RPLAN|RCTX" research/tracks/G-ensemble crates/bisect-ensemble docs/papers docs/PAPERS.md docs/specs
Get-ChildItem -Recurse research\tracks\G-ensemble | Select-Object FullName
git --no-pager status --short
```

## Deliverables

- [x] Materialize this pulse and fork context.
- [x] Run the scout commands and decide whether this is paper/docs-only.
- [x] Revise G.1-G.3/G.5/G.12 source where stale data-validation,
  convergence, percentile, diagnostic, external-validation, or package claims appear.
- [x] Update `PAPER-QUALITY-REVIEW.md`, `ALGORITHM-PAPER-SCORECARD.md`,
  `WAVE.md`, `docs/PAPERS.md`, and the active goal.
- [x] Rebuild changed PDFs and copy them to `docs/papers/`.
- [x] Run focused validation and commit.

## Completion Notes

- Decision: paper/docs-only. Scout found existing `bisect-ensemble` diagnostics
  and stronger G reference papers; the weak slice needed claim-boundary cleanup.
- Added data/validation/diagnostic boundary sections to G.1, G.2, G.3, G.5, and
  G.12.
- Scoped ensemble percentiles, partisan-distribution conclusions, compactness
  rankings, convergence/mixing statements, and short-burst implementation claims
  to archived traces, diagnostics, package manifests, and verifier paths.
- Validation: rebuilt G.1-G.3, G.5, and G.12 PDFs; `cargo fmt`;
  `cargo test -p bisect-ensemble` (113 passed, 5 L2 ignored); `git diff --check`.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```

Build every changed G-track paper with:

```powershell
pdflatex -interaction=nonstopmode main.tex
bibtex main
pdflatex -interaction=nonstopmode main.tex
pdflatex -interaction=nonstopmode main.tex
```
