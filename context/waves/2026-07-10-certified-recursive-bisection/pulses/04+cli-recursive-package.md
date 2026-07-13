---
pulse: 04
title: CLI recursive package
status: done
depends_on: 02, 03
wave: certified-recursive-bisection
validation_level: L1 package
---

# Pulse 04 - CLI Recursive Package

Emit bounded recursive split certificates, tree manifests, plans, and hostile
fixtures through the CLI.

## Deliverables

- [x] Add `bisect exact --method certified-recursive`.
- [x] Convert RCTX graph/population data into a certified root split.
- [x] Emit and reverify the complete recursive tree.
- [x] Derive the final district assignment from canonical leaves.
- [x] Emit RPLAN/RCTX and an audit certificate.
- [x] Emit a hash-bound tree package manifest.
- [x] Commit a deterministic path-8/four-district positive package.
- [x] Commit five hostile recursive tree fixtures.
- [x] Bind verified leaves to RPLAN and verify RCTX/audit/manifest hashes.
- [x] Add standalone and integration verifier paths.
- [x] Document package and claim boundaries.

## Claim Boundary

The package certifies bounded synthetic sequential bisection. It does not prove
global unrestricted-map optimality, production proof scalability, or real
block-level readiness.

## Validation

```powershell
cargo test -p bisect-cli exact_cmd --lib -- --test-threads=1
cargo test -p bisect-ilp --test certified_recursive_negative_corpus -- --test-threads=1
cargo run -p bisect-ilp --example certified_recursive -- verify-package docs/examples/certified-recursive/path8-k4/output
python scripts/research/verify_certified_recursive_fixtures.py
cargo fmt --all -- --check
git --no-pager diff --check
```
