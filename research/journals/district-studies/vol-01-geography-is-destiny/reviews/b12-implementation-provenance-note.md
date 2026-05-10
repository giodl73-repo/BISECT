---
journal: District Studies
volume: 1
title: "Geography Is Destiny"
status: audit-note
updated: 2026-05-09
source: T.5 implementation search
claim-class: reproducibility
---

# T.5 Implementation Provenance Note

## Result

The repository does not currently contain enough evidence to close the T.5
METIS parameter gate for the paper's reported Table 1 runs.

The current implementation provides useful defaults, but those defaults cannot
be assumed to be the exact T.5 paper-run vector without a run artifact or
author note tying T.5 to that implementation state.

## Current Rust Implementation Evidence

Current Rust wrapper:

- `crates/bisect-apportion/src/split.rs`
- `MetisPartitioner::default()`:
  - `balance_tolerance = 0.005`
  - `niter = 10`
  - default engine is C METIS FFI when the `c-ffi` feature is enabled.
- C METIS FFI path sets:
  - `UFactor(uf_int)`
  - `NIter(self.niter)`
  - `Contig(true)`
  - `MinConn(true)`
  - `Seed(...)` when a seed is supplied.
- The C FFI path does not visibly set `NCuts` or `Numbering`.

Pure-Rust METIS-like implementation:

- `crates/bisect-metis/src/api.rs`
- `MetisParams::default()`:
  - `ufactor = 5`
  - `niter = 10`
  - `coarsen_to = 20`
  - `ncuts = 1`
  - `contig_fm = true`
  - `use_recursive = false`
  - `objective = Cut`
  - `min_conn = true`
  - `lp_refine = true`
  - `lp_iter = 10`
  - `coarsen_method = Shem`

## Conflicting Or Insufficient Evidence

The wider repo contains multiple historical defaults and experiment settings:

- `crates/bisect-report/src/manifest.rs` documents a manifest `niter` default
  of `100`.
- Several older Python experiment scripts call partitioning with `niter=100`.
- T.5 source text says METIS 5.1.0, `ncon=2`, 30 seeds, and 1.5 percent
  balance tolerance, but does not report `niter`, `ncuts`, or `numbering`.

Because these values differ across eras and layers, the journal should not infer
the T.5 paper-run vector from the current Rust default alone.

## What Can Be Said Now

Safe:

> The current Rust partition wrapper uses `niter=10`, contiguity and minimum
> connectivity options, and explicit seed forwarding, but T.5's paper-run
> artifact does not yet identify whether those current defaults were used for
> the reported Table 1 values.

Unsafe:

> T.5 used `niter=10`, `ncuts=1`, and default zero-based numbering.

That may be true for the current implementation, but it is not yet proven for
the T.5 paper-run table.

## Closure Requirement

Close DS1-001 only when one of these exists:

1. A T.5 run artifact or manifest showing the exact METIS vector.
2. An author note stating the exact T.5 run vector.
3. A regenerated T.5 Table 1 under current implementation defaults.

The closure record should state:

- engine/backend;
- METIS version or implementation version;
- `niter`;
- `ncuts`;
- `numbering`;
- contiguity/minconn options;
- seed list or seed derivation rule;
- whether the table reports averages, maxima, worst seeds, or deterministic
  identical outputs.
