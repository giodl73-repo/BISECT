---
wave: g-ensemble-evidence-packages
pulse: 04
status: done
depends_on:
  - 01
  - 02
  - 03
governing_roles:
  - DATUM
  - LEDGER
  - COVENANT
---

# Pulse 04 - Paper Ledger Closeout

## Mission

Point G.1-G.3 public ledgers and paper boundary sections at the new
missing-evidence package, then close the wave.

## Pre-implementation Scout

```powershell
rg -n "G.1|G.2|G.3|missing-evidence|external-trace package needed|election/ensemble model boundary|metric/trace boundary" docs\PAPERS.md docs\papers\ALGORITHM-PAPER-SCORECARD.md research\tracks\G-ensemble context\waves\2026-05-13-g-ensemble-evidence-packages
git --no-pager status --short
```

## Deliverables

- [x] Update G.1-G.3 paper boundary sections with the package path.
- [x] Update `docs/PAPERS.md`.
- [x] Update `docs/papers/ALGORITHM-PAPER-SCORECARD.md`.
- [x] Update the goal doc and wave closeout.
- [x] Run validation and commit.

## Result

G.1-G.3 now point to
`docs/examples/g-ensemble-evidence-packages/G.1-G.3+missing-evidence/` as a
validated artifact gap. The package does not support headline percentile claims;
it prevents the gap from remaining implicit.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```
