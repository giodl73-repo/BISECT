---
wave: k-exact-reock-evidence-packages
pulse: 02
status: done
depends_on:
  - 01
governing_roles:
  - LEDGER
  - DATUM
---

# Pulse 02 - K Ledger Update

## Mission

Update K.0, K.2, K.7, the public paper index, and the scorecard to cite the
exact-MBC smoke package.

## Pre-implementation Scout

```powershell
rg -n "Reock|minimum bounding|MBC|proxy|exact" research\tracks\K-compactness docs\PAPERS.md docs\papers\ALGORITHM-PAPER-SCORECARD.md
git --no-pager status --short
```

## Deliverables

- [x] Update K.0 definitions.
- [x] Update K.2 exact/proxy boundary.
- [x] Update K.7 court-output disclosure.
- [x] Update `docs/PAPERS.md` and scorecard.
- [x] Rebuild affected PDFs and run validation.

## Result

K.0, K.2, and K.7 now cite
`docs/examples/k-reock-evidence-packages/exact-mbc-smoke/` as exact-MBC smoke
evidence while preserving the production Reock proxy boundary.
