---
wave: rmath-core-eigen-kernel
date_open: 2026-05-14
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# RMATH Core Eigen Kernel

## Mission

Extract the deterministic closed-form symmetric 2x2 eigensystem used by
GeoSection orientation into `rmath-core`.

## Claim Boundary

This wave may add a numeric helper for symmetric 2x2 eigenvalues and a stable
minor eigenvector. It must not own centroid loading, PCA interpretation,
GeoSection cut orientation, or directional edge-penalty semantics.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Symmetric 2x2 eigensystem | DONE | `rmath-core` eigensystem helper; GeoSection orientation consumer; L0/L1/L2 tests |

## Acceptance Gates

- `rmath-core` exposes a typed-error symmetric 2x2 eigensystem helper.
- Tests cover diagonal, off-diagonal, degenerate, and non-finite inputs.
- `bisect-cli::geosection_orientation` consumes the helper while preserving
  existing minor-axis angle behavior.
- Focused tests pass and `git diff --check` is clean.

## Close Summary

`rmath-core` now owns the deterministic closed-form symmetric 2x2 eigensystem:
ordered eigenvalues plus a stable unit minor eigenvector. GeoSection orientation
uses that numeric helper to compute the PCA minor-axis direction while centroid
loading, covariance construction, angle convention, and directional edge-penalty
semantics remain in `bisect-cli`.

## Carry-forwards

- Keep graph-Laplacian construction and Fiedler certificate semantics in
  `bisect-data` unless another consumer needs the same power-iteration operator.
- The full workspace test run exposed a stale `rcount-rhist` package-hash
  placeholder; the test now derives the deterministic hash through
  `rhist_core::package_content_hash`.
