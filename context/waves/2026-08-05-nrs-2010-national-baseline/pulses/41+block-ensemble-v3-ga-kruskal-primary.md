---
pulse: 41
wave: nrs-2010-national-baseline
date: 2026-08-12
status: complete
---

# Block-Ensemble v3 GA Kruskal Primary

## Outcome

The sixth and final governed v3 primary, GA Kruskal, completed all four frozen
2,000-step chains with base seed `20260812`, 500-step burn-in, and snapshot
stride 10. Its fresh actual-volume admission passed immediately before process
creation. The runner returned zero after 566.8571 seconds with peak RSS of
1,397,723,136 bytes and no resource failure.

The wrapper validated the 453,636,562-byte raw trace, deterministically
compressed it to 5,920,216 bytes, and deleted the raw scratch copy. The active
ledger contains 6/6 primaries, 0/6 governed replays, 24,412.6612 governed
runner seconds, 20,664,538 retained bytes, and zero failures. The general
package verifier passes.

The resource record's platform-exact text hashes match the retained governed
execution environment and are bound separately from portable LF-canonical
source identity. No runtime artifact was rewritten.

## Claim Boundary

This pulse establishes completion and custody of all six predeclared governed
primary chain groups. It is not a convergence or sampler-comparison result.
The frozen gate still requires six exact governed replays and the registered
State/kernel-specific analysis.
