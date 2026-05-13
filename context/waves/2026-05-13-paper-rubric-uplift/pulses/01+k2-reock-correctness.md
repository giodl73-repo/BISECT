---
wave: paper-rubric-uplift
pulse: 01
status: done
depends_on: []
governing_roles:
  - MERIDIAN
  - DATUM
  - BENCHMARK
  - SURVEY
  - BOUNDARY
---

# Pulse 01 - K.2 Reock Correctness Pass

## Mission

Lift K.2 Reock out of the scorecard blocker by aligning the paper with the
actual BISECT implementation boundary: canonical Reock uses a minimum enclosing
circle, `bisect-analysis` currently reports the centroid-plus-max-boundary-radius
polygon approximation, and moving-knife uses a separate point-cloud proxy.

## Scope

| Surface | Target | Non-goal |
|---|---|---|
| K.2 paper | Correct canonical-vs-implemented Reock language, MKA mechanics, evidence path, and claim boundary. | Replacing the production compactness implementation. |
| K.0/K.7 | Propagate caveats so overview/court guidance do not overstate exact MBC behavior. | Full K-family rewrite. |
| Ledgers | Update scorecard/review ledger with completed pass and residual ceilings. | Pretending external peer review occurred. |
| Build artifacts | Rebuild changed PDFs and copy them to `docs/papers/`. | Repository-wide paper rebuild. |

## Pre-implementation Scout

Run and record:

```powershell
rg -n "Reock|Welzl|minimum bounding|centroid|Moving-Knife" research/tracks/K-compactness crates/bisect-analysis crates/bisect-cli
rg -n "K.2|Reock|Welzl|implementation mismatch" docs/papers docs/PAPERS.md
git --no-pager status --short
```

## Deliverables

- [x] Decide whether the pulse is paper-only or requires compactness code.
- [x] Revise K.2 source so canonical, `bisect-analysis`, and moving-knife proxy claims are separate.
- [x] Add or revise a small reader-facing implementation/evidence table.
- [x] Propagate the caveat into K.0 and K.7 where they depend on K.2.
- [x] Update `PAPER-QUALITY-REVIEW.md`, `ALGORITHM-PAPER-SCORECARD.md`, this pulse, `WAVE.md`, and the active goal.
- [x] Rebuild changed PDFs and copy them to `docs/papers/`.
- [x] Run focused validation and commit.

## Completion Notes

- Decision: paper/docs only. The implementation already documents the production
  centroid-radius Reock approximation and the separate moving-knife point-cloud
  helper; the blocker was stale paper language.
- K.2 now distinguishes canonical exact-MBC Reock, `bisect-analysis::reock`, and
  `bisection_runner::welzl_mec`.
- K.0 and K.7 now disclose reported Reock as the centroid-radius approximation
  when exact-MBC reproduction matters.
- PDFs rebuilt and copied: `K.0+compactness-overview.pdf`, `K.2+reock.pdf`,
  and `K.7+composite-court-guide.pdf`.
- Commit evidence: this pulse is committed by the repository commit that includes
  this file update.

## Validation

```powershell
cd research\tracks\K-compactness\K.2+reock; pdflatex -interaction=nonstopmode main.tex; bibtex main; pdflatex -interaction=nonstopmode main.tex; pdflatex -interaction=nonstopmode main.tex
cd ..\K.0+compactness-overview; pdflatex -interaction=nonstopmode main.tex; bibtex main; pdflatex -interaction=nonstopmode main.tex; pdflatex -interaction=nonstopmode main.tex
cd ..\K.7+composite-court-guide; pdflatex -interaction=nonstopmode main.tex; bibtex main; pdflatex -interaction=nonstopmode main.tex; pdflatex -interaction=nonstopmode main.tex
git diff --check
```

Validation completed with MiKTeX `pdflatex`/`bibtex` for K.2, K.0, and K.7;
`git diff --check` passed.
