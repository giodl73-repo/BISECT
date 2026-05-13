---
wave: k-exact-reock-evidence-packages
pulse: 03
status: done
depends_on:
  - 01
  - 02
governing_roles:
  - LEDGER
  - BENCHMARK
---

# Pulse 03 - Closeout

## Mission

Close the K exact Reock evidence wave after fixture, verifier, paper, PDF, and
ledger updates are complete.

## Deliverables

- [x] Mark the wave complete and archive it in `PHASES.md`.
- [x] Add `CLOSE.md` with delivered work, validation, and carry-forwards.
- [x] Run `cargo fmt`.
- [x] Run `$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis`.
- [x] Run `git diff --check`.

## Carry-Forwards

- Build real district polygon exact-MBC packages before making district-scale
  exact-MBC replay claims.
- Scope any production `reock()` migration separately because this wave preserves
  the centroid proxy in `all_metrics()`.
