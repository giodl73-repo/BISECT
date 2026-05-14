# RGRAPH Edge Cut Consumer Expansion Close

## Outcome

Expanded `rgraph-core::undirected_edge_cut` consumption to the remaining
slice-based edge-cut callers.

## Evidence

- `bisect-apportion::spectral` uses the shared helper for spectral summary cuts.
- `bisect-clustering::metrics` uses the shared helper for assignment summaries
  and repair scoring.
- `bisect-flow` uses the shared helper for constructive assignment summaries.
- `bisect-cli::runner` uses the shared helper for zero-based spectral recursive
  run summaries.

## Boundary

HashMap-shaped bisection-runner helpers and HashSet-shaped SA evidence helpers
remain local because their assignment forms encode domain-specific defaults.
