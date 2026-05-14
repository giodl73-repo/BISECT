---
wave: rmath-core-vector-kernel
date_open: 2026-05-14
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RMATH Core Vector Kernel

## Mission

Extract deterministic vector primitives from existing spectral and Fiedler math
into `rmath-core` without moving graph Laplacian, sweep-cut, certificate, or
redistricting interpretation into the math crate.

## Claim Boundary

This wave may add low-level vector helpers: dot products, L2 norms, centering,
and normalization. It must not own spectral partitioning, Laplacian construction,
Fiedler certificate semantics, or balanced sweep-cut selection.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Vector normalization helpers | DONE | `rmath-core` vector helpers; spectral/Fiedler consumers; L0/L1/L2 tests |

## Acceptance Gates

- `rmath-core` exposes typed-error vector helpers with positive and negative
  coverage.
- `bisect-apportion::spectral` consumes centered normalization from `rmath-core`.
- `bisect-data::fiedler` consumes dot/norm/centering helpers while retaining
  Laplacian and certificate semantics locally.
- Focused tests pass and `git diff --check` is clean.

## Close Summary

`rmath-core` now owns deterministic vector primitives for dot products, L2 norms,
centering, in-place normalization, and centered normalization with typed input
errors. `bisect-apportion::spectral` uses the shared centered-normalization
helper for smoothed spectral vectors, and `bisect-data::fiedler` uses the shared
dot, centering, and normalization helpers while keeping weighted Laplacian and
certificate semantics local.

## Follow-up Candidates

- Extract the closed-form 2x2 symmetric eigensystem from
  `bisect-cli::geosection_orientation` when another consumer needs small-matrix
  eigenvectors.
- Consider a graph-Laplacian power-iteration helper only if another package needs
  the same operator/eigenvalue routine; the current Fiedler certificate remains
  domain-specific enough to stay in `bisect-data`.
