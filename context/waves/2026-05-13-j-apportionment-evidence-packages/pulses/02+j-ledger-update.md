---
wave: j-apportionment-evidence-packages
pulse: 02
status: done
depends_on:
  - 01
governing_roles:
  - LEDGER
  - DATUM
---

# Pulse 02 - J Ledger Update

## Mission

Update J.0, J.1, J.6, the public paper index, and scorecard to cite the new
2020 Census Table 1 evidence package.

## Pre-implementation Scout

```powershell
rg -n "J.0|J.1|J.6|Census-reference|Census fixtures|SHA-256" docs\PAPERS.md docs\papers\ALGORITHM-PAPER-SCORECARD.md research\tracks\J-apportionment
git --no-pager status --short
```

## Deliverables

- [x] Update J.0/J.1/J.6 text from future fixture language to current 2020
  package language.
- [x] Update `docs/PAPERS.md`.
- [x] Update `docs/papers/ALGORITHM-PAPER-SCORECARD.md`.
- [x] Rebuild affected PDFs.
- [x] Run validation and commit.

## Result

J.0, J.1, and J.6 now point at
`docs/examples/j-apportionment-evidence-packages/2020-census-table01/` as the
first active Census/SHA apportionment evidence package. The remaining gap is
historical 2000/2010 packages and a standalone CLI verifier.
