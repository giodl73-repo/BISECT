---
wave: m-community-source-completion
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-m-community-source-completion-goal.md
---

# M Community Source Completion

## Mission

Promote complete M-track LaTeX sources from source-only ledger entries to
committed PDF-backed papers.

## Claim Boundary

This wave is a publication/ledger completion slice. It compiles and indexes
existing paper sources; it does not add new empirical community-character runs or
new CLI weight modes.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - M.1/M.3 PDF publication | DONE | Compiled PDFs, copied to docs, and updated ledgers |
| 02 - Closeout | DONE | Archived wave after validation and commit |

## Validation Gate

```powershell
pdflatex/bibtex passes for M.1 and M.3
Test-Path docs/papers/M.1+economic-character-lodes.pdf
Test-Path docs/papers/M.3+housing-character-acs.pdf
git diff --check
```

## Closeout

Completed. M.1 and M.3 are now PDF-backed paper entries with ledger and
scorecard updates. M.1 needed one LaTeX table fix before publication.
