---
wave: m-community-source-completion
pulse: 01
status: in-progress
governing_roles:
  - LEDGER
---

# Pulse 01 - M.1/M.3 PDF Publication

## Mission

Compile M.1 and M.3 from existing LaTeX sources, copy PDFs to `docs/papers/`,
and update public ledgers.

## Pre-implementation Scout

```powershell
Get-ChildItem research\tracks\M-community-character\M.1+economic-character-lodes,research\tracks\M-community-character\M.3+housing-character-acs -Filter main.tex
git --no-pager status --short
```

## Deliverables

- [x] Compile M.1.
- [x] Compile M.3.
- [x] Copy PDFs to `docs/papers/`.
- [x] Update `docs/PAPERS.md`.
- [x] Update `docs/papers/ALGORITHM-PAPER-SCORECARD.md`.

## Validation

```powershell
git diff --check
```
