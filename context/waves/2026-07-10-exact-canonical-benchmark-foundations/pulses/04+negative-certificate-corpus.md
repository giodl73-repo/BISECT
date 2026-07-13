---
pulse: 04
title: Negative certificate corpus
status: done
depends_on: 01, 03
wave: exact-canonical-benchmark-foundations
validation_level: L1 negative fixtures
---

# Pulse 04 - Negative Certificate Corpus

Add false-optimum, false-infeasibility, noncanonical-tie, disconnected, and
hash-tamper fixtures.

## Deliverables

- [x] Commit all five adversarial artifact triplets.
- [x] Add machine-readable expected rejection classes.
- [x] Add a reproducible hostile-corpus builder.
- [x] Add a command-line reference verifier surface.
- [x] Load every case through the public Rust verifier API.
- [x] Hash every negative artifact in the fixture root manifest.
- [x] Document replay and claim boundaries.
- [x] Advance Pulse 05 to ready.

## Claim Boundary

The corpus tests exact JSON artifact verification. It does not replay
RCTX-to-instance conversion, CLI plan emission, national-scale solving, or a
second independent verifier implementation.

## Validation

```powershell
python scripts/research/build_exact_negative_corpus.py
python scripts/research/verify_exact_canonical_fixtures.py
cargo test -p bisect-ilp canonical -- --test-threads=1
cargo test -p bisect-ilp --test exact_negative_corpus -- --test-threads=1
cargo run -p bisect-ilp --example exact_canonical -- verify <instance> <certificate> <proof>
cargo fmt --all -- --check
git --no-pager diff --check
```
