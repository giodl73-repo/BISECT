---
wave: g-active-ensemble-evidence-packages
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

Close the G active synthetic evidence wave after package, verifier, paper, PDF,
and ledger updates are complete.

## Deliverables

- [x] Mark the wave complete and archive it in `PHASES.md`.
- [x] Add `CLOSE.md` with delivered work, validation, and carry-forwards.
- [x] Run `cargo fmt`.
- [x] Run `cargo test -p bisect-ensemble active_g1_g3_package_fixture_validates`.
- [x] Run `git diff --check`.

## Carry-Forwards

- Add real external GerryChain/ReCom traces with diagnostics before citing real
  G.1-G.3 percentile findings.
- Add real election-input and metric-output packages for G.2/G.3.
- Attach real BISECT RPLAN/RCTX baseline packages for every compared scenario.
