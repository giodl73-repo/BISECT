---
wave: rmath-core-matrix-kernel
date_open: 2026-05-14
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RMATH Core Matrix Kernel

## Mission

Extract small deterministic dense-matrix primitives from BISECT analysis into
`rmath-core` without moving regression, inference, or domain interpretation into
the math crate.

## Claim Boundary

This wave may add numeric linear-algebra helpers. It must not own WLS/HC3
semantics, bloc-voting reports, optimization objectives, route scoring, or
redistricting legality.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Dense matrix kernel | DONE | `rmath-core`; bloc-voting WLS/HC3 consumer; L0/L1/L2 tests |

## Close Summary

`rmath-core` now owns a small row-major dense matrix type plus matrix
multiplication, matrix-vector multiplication, transpose, and Gauss-Jordan inverse
with partial pivoting. `bisect-analysis::bloc_voting` consumes the shared matrix
helpers for WLS and HC3 calculations while retaining all regression and evidence
semantics locally.

## Follow-up Candidates

- `bisect-data::fiedler` owns weighted Laplacian, dot/norm/projection, and
  deflated power iteration for lambda2 certificates.
- `bisect-apportion::spectral` owns a smoothing-based spectral vector and
  normalized vector helper for balanced sweep cuts.
- `bisect-cli::geosection_orientation` owns a closed-form 2x2 covariance
  eigenvector for minor-axis orientation.

Those are the right next extraction targets if `rmath-core` grows from dense
matrix basics into vector operations, symmetric 2x2 eigensystems, or Laplacian
power-iteration helpers.

