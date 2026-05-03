# redist-metis Verification Sub-Spec
**Date:** 2026-05-02
**Phase:** 6 of 8
**Implements:** `verify/kani/` and `verify/prusti/`

## Overview

Two tools, two claims:
- **Kani** (bounded model checker) — safety floor: no UB, no overflow, no OOB for all valid inputs up to bounded size
- **Prusti** (deductive verifier) — three legal postconditions on the public partition output

## Kani — Safety Floor

**Version**: Kani 0.55+ pinned in `rust-toolchain.toml`.
**Dependency**: Kani harnesses use `#[cfg(kani)]` attribute gating; no explicit dev-dependency needed.
**CI**: runs on main/release only (35–70 min total), NOT on every PR.

### What Kani proves (per harness)
For all valid inputs up to the stated bound:
- No buffer overflows (`xadj`/`adjncy` indexing)
- No integer overflow (gain arithmetic, weight accumulation)
- No index out of bounds (partition assignment, coarse map)
- No reachable panics (`unwrap`, `expect`, array index)

### Harnesses and bounds

| Harness | File | Bound | Justification |
|---------|------|-------|---------------|
| `verify_is_valid_no_panic` | `csr_harness.rs` | n ≤ 8 | All `is_valid()` branches covered at n=4; 8 for edge cases |
| `verify_shem_no_oob` | `coarsen_harness.rs` | n ≤ 16 | SHEM bucket sort bugs in star/chain topologies need ≥ 10 |
| `verify_hem_no_oob` | `coarsen_harness.rs` | n ≤ 16 | Same as SHEM |
| `verify_gain_table_no_overflow` | `refine_harness.rs` | gains ∈ [-128, 128] | Full gain range; bucket indexing at extremes |
| `verify_fm_no_oob` | `refine_harness.rs` | n ≤ 16, k ≤ 4 | FM inner loop path-independent of n above ~8 |
| `verify_hierarchy_no_panic` | `multilevel_harness.rs` | levels ≤ 8 | Covers ≥ 5 coarsening rounds (CA-scale depth) |

Bounds documented in `verify/kani/BOUNDS.md` with LLVM bitcode coverage justification.

## Prusti — Three Legal Postconditions

**Version**: Prusti 0.2.x pinned in `rust-toolchain.toml`.
**Restricted subset**: verified functions avoid `dyn Trait`, closures, `async`, complex lifetimes.
**Fallback**: functions that cannot be verified are flagged `#[prusti_skip]`, documented in `verify/prusti/GAPS.md`.
**CI**: advisory on PR (`continue-on-error`), blocking on release tags.

### Three postconditions on `Partitioner::split` output:
1. `result.assignment.len() == graph.n()` — every vertex assigned
2. `forall |i: usize| i < result.assignment.len() ==> result.assignment[i] < result.k` — valid part IDs
3. `population_balance(&result, graph) <= params.epsilon` — balance ≤ 0.5%

### Proof artifacts
Prusti generates Viper `.vpr` files committed to `verify/prusti/artifacts/` — legally archivable proof artifacts (not just CI pass/fail logs).

## UNSAFE inventory

The redist-metis crate contains **zero unsafe blocks**. All array access and arithmetic is verified through safe Rust bounds checks and Kani formal verification harnesses. See `verify/kani/UNSAFE.md` for inventory (empty).

## CI gate

On every PR:
- Clippy + standard lints (fail-fast)
- Unit + integration tests (all platforms)
- Prusti harnesses (advisory, `continue-on-error`)

On main/release tags:
- All above, plus:
- Kani harnesses (all 6, bounded model checking)
- Count of `unsafe` blocks in `src/` asserted to match `verify/kani/UNSAFE.md` (currently 0)
- Prusti gates on release tags (no GAPS allowed)

## Related documents

- `verify/kani/BOUNDS.md` — Justification for bound choices per harness
- `verify/kani/UNSAFE.md` — Inventory of unsafe blocks (currently empty)
- `verify/prusti/GAPS.md` — Functions that cannot be verified (currently empty)
- `redist/crates/redist-metis/src/` — All code; zero unsafe
