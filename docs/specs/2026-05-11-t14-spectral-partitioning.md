# Spec: T.14 Spectral Partitioning

**Status:** Stage 2 active  
**Track:** T.14 - Plan Construction  
**Depends on:** U.20 RPLAN audit integration fixed point

## Purpose

Add a deterministic spectral construction baseline for graph redistricting. The
T.14 baseline provides a cheap, reproducible alternative to METIS-family
construction and gives later comparison reports a transparent graph-cut method.

## Stage-One Algorithm

The first slice implements a bisection kernel in `bisect-apportion::spectral`:

1. Build a deterministic centered anchor vector from unit order.
2. Smooth the vector over graph adjacency for `max_iters`.
3. Sort units by the smoothed vector.
4. Sweep cut positions that satisfy population tolerance.
5. Pick the minimum edge-cut sweep, breaking ties by sweep index.
6. Emit convergence and sweep metadata in `SpectralSummary`.

This is a deterministic spectral-style baseline, not yet a full Lanczos
eigensolver. The CLI surface must record that method identity explicitly.

## Crate Boundary

`bisect-apportion::spectral` owns:

- spectral bisection kernel
- deterministic vector/sweep metadata
- L0 graph fixtures

`bisect-cli` owns recursive composition, CLI/YAML parsing, manifests, and RPLAN
sidecar emission.

## CLI Staging

Target surface:

- `--structure spectral`
- `--spectral-iters N`, default 200
- `--spectral-sweep population-balanced|min-cut`, initially
  `population-balanced`

Any final plan emitted through this mode must include RPLAN sidecars and an
`algorithm_lineage` entry naming `bisect-apportion` and method `spectral`.

## Tests

- path graph splits near midpoint
- two-clique bridge graph cuts the bridge
- fixed input produces identical vector, assignment, and summary
- CLI/RPLAN sidecar test after runner wiring
