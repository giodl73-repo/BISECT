---
wave: paper-rubric-uplift
date_open: 2026-05-13
status: active
source_goal: docs/specs/2026-05-13-paper-rubric-uplift-goal.md
---

# Paper Rubric Uplift

## Mission

Move the highest-priority below-rubric papers toward publication-quality claims,
evidence paths, and reader experience. Start with the K.2 Reock compactness
paper because the scorecard identifies a concrete Welzl/implementation mismatch
that also propagates into K.0 and K.7.

## Claim Boundary

This wave may revise papers, ledgers, source claims, examples, and build outputs.
It must not rewrite implementation semantics merely to make a paper cleaner.
Where implementation and canonical mathematical definitions differ, the papers
must name the deployed behavior, the ideal definition, and the evidence ceiling.

## Inputs

| Input | Source |
|---|---|
| Active goal | `docs/specs/2026-05-13-paper-rubric-uplift-goal.md` |
| Paper scorecard | `docs/papers/ALGORITHM-PAPER-SCORECARD.md` |
| Paper review ledger | `docs/papers/PAPER-QUALITY-REVIEW.md` |
| Atlas rubric | `docs/algorithm-atlas/RUBRIC.md` |
| K paper sources | `research/tracks/K-compactness/` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - K.2 Reock correctness pass | DONE | `pulses/01+k2-reock-correctness.md`; K.2 source/PDF, K.0/K.7 caveat propagation, compactness scorecard |
| 02 - J apportionment implementation pass | DONE | `pulses/02+j6-apportionment-implementation.md`; J.6 current API/test/verification boundary, J.0-J.5 bridge caveats, rebuilt J PDFs |
| 03 - Older search paper refresh | DONE | `pulses/03+older-search-paper-refresh.md`; U.0-U.7, U.11, U.13-U.15 implementation/evidence-boundary pass |
| 04 - Ensemble data cleanup | TODO | G.1-G.3/G.5/G.12 validation and diagnostic evidence refresh |

## Validation Gate

Run the focused document build after every paper pulse:

```powershell
cd research\tracks\K-compactness\K.2+reock; pdflatex -interaction=nonstopmode main.tex; bibtex main; pdflatex -interaction=nonstopmode main.tex; pdflatex -interaction=nonstopmode main.tex
```

Also run any affected neighboring paper builds, then:

```powershell
git diff --check
```

## Next

Pulse 03 refreshed the older U search papers. Continue with pulse 04:
G.1-G.3/G.5/G.12 ensemble validation and diagnostic evidence cleanup.
