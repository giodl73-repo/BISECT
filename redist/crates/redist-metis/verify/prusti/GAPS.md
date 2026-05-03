# Prusti Verification Gaps

Functions that cannot be verified by Prusti due to unsupported Rust features.
**CI gate**: This file must have ZERO entries before a release tag is created.

## Current status: ZERO GAPS

The three postconditions on `Partitioner::split` are annotated in `src/api.rs`.
`split()` uses only safe integer arithmetic, no closures, no async, no dyn Trait
in the verified code path — compatible with Prusti 0.2.x's Rust subset.

| # | Function | Reason | Fallback |
|---|----------|--------|----------|
| (none) | — | — | — |

*Release gate: `grep -c "^|" GAPS.md` must equal 2 (header + separator only).*
