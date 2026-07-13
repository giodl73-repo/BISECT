# Certified Recursive Bisection Fixtures

## Positive Package

`path8-k4` certifies an eight-unit path into four districts:

```text
4 -> 2/2
2 -> 1/1
2 -> 1/1
```

The package contains the complete nested tree, final RPLAN/RCTX, audit
certificate, and hash manifest.

## Hostile Corpus

The negative corpus commits:

- tree-ID tampering;
- a false root optimum with refreshed certificate/tree IDs;
- a missing district leaf;
- noncanonical split-node order; and
- a leaf-unit substitution with refreshed leaf and tree IDs.

Each case has a machine-readable expected rejection class and is loaded through
the Rust tree verifier integration test.

## Replay

```powershell
bisect exact `
  --method certified-recursive `
  --context docs\examples\certified-recursive\path8-k4\input.rctx `
  --out-dir <temp>\certified-recursive `
  --districts 4 `
  --tolerance 1.0 `
  --exact-fixture-limit 8 `
  --generated-at 2026-07-10T12:00:00Z
```

## Verification

```powershell
python scripts\research\verify_certified_recursive_fixtures.py
cargo test -p bisect-ilp --test certified_recursive_negative_corpus -- --test-threads=1
cargo run -p bisect-ilp --example certified_recursive -- verify-package `
  docs\examples\certified-recursive\path8-k4\output
```

## Claim Boundary

These fixtures replay bounded synthetic sequential bisection. They do not prove
global optimality among unrestricted maps, production proof scalability, or
real block-level readiness.
