---
wave: u-search-evidence-packages
pulse: 02
status: done
depends_on:
  - 01
governing_roles:
  - LEDGER
  - DATUM
---

# Pulse 02 - U Ledger Update

## Mission

Update U.2, U.4, the public paper index, and the scorecard to cite the synthetic
U search evidence package while preserving the real-sweep and production-CLI
claim boundaries.

## Deliverables

- [x] Update U.2 current-boundary section.
- [x] Update U.4 current-boundary section.
- [x] Update `docs/PAPERS.md`.
- [x] Update `docs/papers/ALGORITHM-PAPER-SCORECARD.md`.
- [x] Rebuild U.2 and U.4 PDFs.

## Result

U.2 and U.4 now cite
`docs/examples/u-search-evidence-packages/synthetic-sweep-and-pt/` as synthetic
package evidence, while explicitly retaining the need for real 50-state sweep
and production CLI packages.
