---
wave: g0-methodology-package-framing
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-g0-methodology-package-framing-goal.md
---

# G.0 Methodology Package Framing

## Mission

Modernize G.0 so the methodology paper reflects the current G evidence-package
contract: synthetic package shape exists, while real percentile claims require
real external traces, election inputs, metrics, diagnostics, RPLAN, and RCTX.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Paper framing update | DONE | Updated G.0 sections, rebuilt PDF, updated ledgers |
| 02 - Closeout | DONE | Archived wave after validation and commit |

## Validation Gate

```powershell
pdflatex/bibtex passes for G.0
Test-Path docs/papers/G.0+ensemble-methodology.pdf
git diff --check
```

## Closeout

Completed. G.0 now reflects the active synthetic and missing-evidence package
regime for G.1--G.3 and has a rebuilt PDF plus updated ledgers.
