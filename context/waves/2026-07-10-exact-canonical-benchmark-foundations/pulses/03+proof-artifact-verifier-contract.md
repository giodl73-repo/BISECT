---
pulse: 03
title: Proof artifact and verifier contract
status: done
depends_on: 01
wave: exact-canonical-benchmark-foundations
validation_level: L1 verifier
---

# Pulse 03 - Proof Artifact And Verifier Contract

Define stable proof transcripts and a verifier surface that does not trust the
generating solver.

## Deliverables

- [x] Add the `exact-canonical-proof-v1` schema.
- [x] Commit to every candidate in deterministic ascending-mask order.
- [x] Bind the proof transcript ID into the exact certificate.
- [x] Recompute the proof through `verify_exact_canonical_artifacts`.
- [x] Emit and hash `exact-canonical-proof.json` in CLI packages.
- [x] Add proof-tampering and deterministic-transcript tests.
- [x] Regenerate positive and infeasible fixture packages.
- [x] Update public format, CLI, fixture, North-Star, and wave docs.

## Claim Boundary

The verifier rejects submitted artifact tampering but shares the Rust
enumeration implementation with the generator. It is not the second
independent implementation required by Pulse 05.

## Validation

```powershell
cargo test -p bisect-ilp canonical -- --test-threads=1
cargo test -p bisect-cli exact_cmd --lib -- --test-threads=1
python scripts/research/verify_exact_canonical_fixtures.py
python scripts/research/verify_nrs_challenge_bundle.py
cargo fmt --all -- --check
git --no-pager diff --check
```
