---
pulse: 05
title: Second verifier implementation
status: done
depends_on: 03, 04
wave: exact-canonical-benchmark-foundations
validation_level: independent verifier
---

# Pulse 05 - Second Verifier Implementation

Implement an independent verifier in a separate language or crate and require
both verifiers to accept the certificate corpus.

## Deliverables

- [x] Implement exhaustive verification independently in Python.
- [x] Recompute canonical instance, certificate, and proof hashes.
- [x] Recompute connectivity, objective order, and canonical tie-breaking.
- [x] Reconstruct the ordered binary search transcript.
- [x] Accept both committed positive fixtures.
- [x] Reject all five committed hostile submissions by declared class.
- [x] Commit a deterministic verifier report and source hash.
- [x] Add focused Python unit tests.
- [x] Bind verifier source and report into fixture and challenge custody.
- [x] Advance Pulse 06 to ready.

## Independence Contract

The Python verifier does not import, execute, call, or link the Rust solver or
reference verifier. Corpus generation may use Rust to create valid base
artifacts; verification does not.

## Claim Boundary

Two implementations now agree for the bounded E0 `k=2`, `n <= 24` model and
committed corpus. This does not establish production solver independence,
national block-scale tractability, or legal validity.

## Validation

```powershell
python scripts/research/verify_exact_canonical_independent.py corpus
python -m pytest -q tests/unit/test_exact_canonical_independent_verifier.py
cargo test -p bisect-ilp canonical -- --test-threads=1
cargo test -p bisect-ilp --test exact_negative_corpus -- --test-threads=1
python scripts/research/verify_exact_canonical_fixtures.py
python scripts/research/verify_nrs_challenge_bundle.py
cargo fmt --all -- --check
git --no-pager diff --check
```
