---
pulse: 02
title: Exact CLI package integration
status: done
depends_on: 01
wave: exact-canonical-benchmark-foundations
validation_level: L1 package
---

# Pulse 02 - Exact CLI Package Integration

## Deliverables

- [x] `canonical-exhaustive` CLI method.
- [x] Optimal RPLAN/RCTX/audit package.
- [x] Infeasibility package without a plan.
- [x] Hash-bound exact package manifest.
- [x] Fixed generation timestamp support.
- [x] Positive and negative committed fixtures.
- [x] CLI and file-format documentation.

## Validation

```powershell
cargo test -p bisect-cli exact_cmd --lib -- --test-threads=1
cargo test -p bisect-ilp canonical -- --test-threads=1
python scripts/research/verify_exact_canonical_fixtures.py
```

All gates passed.
