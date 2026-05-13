---
wave: pure-rust-metis
date_open: 2026-05-02
date_close: 2026-05-04
status: archived
backfill: true
confidence: high
---

# Pure Rust METIS

## Mission

Design and implement a pure Rust METIS-style partitioner with coarsening,
initialization, refinement, multilevel pipeline, verification docs, Kani/Prusti
harnesses, benchmarks, and shadow validation.

## Evidence

Representative commits:

- `cada3652` redist-metis design spec
- `51c07dc2` redist-metis crate scaffold
- `1ae7c1e1` CsrGraph, Partition, CoarseMap, errors
- `946db198` HeavyEdgeMatch
- `258c9d22` GrowBisect and GrowKway
- `53386626` Fiduccia-Mattheyses FM and GreedyKWay
- `ab2b15fc` full e2e multilevel pipeline
- `f368e513` Kani harnesses
- `feddaeca` Prusti annotations and zero-gap confirmation
- `3f9c0bb1` shadow validation 50/50 states pass

## Tracks

- Pure Rust graph partitioning.
- Formal verification and correctness oracles.
- Performance benchmarking.
- C METIS compatibility/shadow mode.

## Established

- Portable partitioning path.
- Stronger graph/data invariants.
- Verification vocabulary later reused in package verifier work.

## Carry Forward

The crate was later removed or superseded in some forms, but the algorithmic and
verification lessons remain important historical context.

