---
wave: u-search-evidence-packages
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

Close the synthetic U.2/U.4 search evidence wave after package, verifier, paper,
PDF, and ledger updates are complete.

## Deliverables

- [x] Mark the wave complete and archive it in `PHASES.md`.
- [x] Add `CLOSE.md` with delivered work, validation, and carry-forwards.
- [x] Run `cargo fmt`.
- [x] Run `cargo test -p bisect-ensemble search_evidence`.
- [x] Run `git diff --check`.

## Carry-Forwards

- Add real 50-state parameter sweep packages before making national
  robustness/headline claims for U.2.
- Add a production parallel-tempering CLI command, invocation test, and real run
  package before presenting U.4 as a production mode.
