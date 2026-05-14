---
pulse: 01
slug: divisor-method-smoke-package
status: complete
---

# Pulse 01: Divisor Method Smoke Package

## Purpose

Add a minimal package-backed evidence slice for J.2--J.5 so the Webster, Adams,
Jefferson, and divisor paradox-immunity claims have a replayable fixture rather
than only source/test references.

## Checklist

- [x] Scout `bisect-apportion` divisor and paradox implementations.
- [x] Decide fixture-only scope is sufficient; no new public CLI is needed for
  this slice.
- [x] Add `j-divisor-evidence-manifest v1` and verifier coverage.
- [x] Add a positive fixture and negative tamper test.
- [x] Update J.2--J.5 papers, manifest docs, scorecard, public paper index, and
  wave docs.
- [x] Rebuild PDFs and run validation.

## Validation

```powershell
cargo fmt
cargo test -p bisect-apportion divisor_method_smoke
cargo test -p bisect-apportion tampered_divisor_fixture_rejected
git diff --check
```
