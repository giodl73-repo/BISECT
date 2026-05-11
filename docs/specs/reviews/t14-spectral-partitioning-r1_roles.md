# Role Review: T.14 Spectral Partitioning

**Spec:** [`2026-05-11-t14-spectral-partitioning.md`](../2026-05-11-t14-spectral-partitioning.md)  
**Status:** Approved for stage-one implementation

## Summary

T.14 is approved as a deterministic baseline, with the important caveat that the
stage-one method must identify itself as a smoothed spectral-style bisection
rather than a full Fiedler-vector eigensolver. That distinction keeps
reproducibility and method claims aligned.

## Required Guardrails

- Do not silently call the method a full Laplacian eigensolver until Lanczos or
  another proper Fiedler approximation is implemented.
- Preserve deterministic tie-breaking.
- Record iteration count, convergence flag, sweep identity, edge cut, and
  population deviation.
- Delegate final validity to U.20 RPLAN audit sidecars in CLI integration.

## Stage-One Approval

Approved to add `bisect-apportion::spectral` with L0 path/two-clique and
determinism coverage before runner wiring.
