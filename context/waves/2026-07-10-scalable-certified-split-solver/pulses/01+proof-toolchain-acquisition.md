---
pulse: 01
title: Proof toolchain acquisition
status: done
depends_on: none
wave: scalable-certified-split-solver
validation_level: external proof smoke
---

# Pulse 01 - Proof Toolchain Acquisition

Acquire, pin, hash, and smoke-test RoundingSat proof generation and independent
VeriPB proof checking on a committed OPB fixture.

## Deliverables

- [x] Pin source/version identities.
- [x] Build or install reproducibly.
- [x] Generate one UNSAT proof.
- [x] Verify the proof independently.
- [x] Commit toolchain provenance and smoke artifacts.
- [x] Document Windows/Linux replay commands.

## Result

RoundingSat commit `d4edbf7` generated a proof for the committed population
decision. VeriPB `SAT_competition_2023` commit `409a889` independently reported
`Verification succeeded.`

The smoke test also corrected the OPB compiler to the actual pinned parser:
consecutive `x1..xN` variables, `>=`/`=` relations, extended header fields, and
the `--proof-log` option.

## Validation

```powershell
python scripts/research/verify_proof_toolchain_smoke.py
python scripts/research/verify_certified_proof_backend.py
cargo test -p bisect-ilp proof_backend -- --test-threads=1
cargo fmt --all -- --check
git --no-pager diff --check
```
