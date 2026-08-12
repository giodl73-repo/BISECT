---
pulse: 40
wave: nrs-2010-national-baseline
date: 2026-08-12
status: complete
---

# Block-Ensemble v3 GA Wilson Primary

## Outcome

The fifth governed v3 schedule entry, GA Wilson, completed all four frozen
2,000-step chains with base seed `20260812`, 500-step burn-in, and snapshot
stride 10. Its fresh actual-volume admission passed immediately before process
creation. The runner returned zero after 3,384.9358 seconds with peak RSS of
1,391,804,416 bytes and no resource failure.

The wrapper validated the 454,914,408-byte raw trace, deterministically
compressed it to 6,708,674 bytes, and deleted the raw scratch copy. The active
ledger contains 5/6 primaries, 0/6 governed replays, 23,845.8041 governed
runner seconds, 14,744,322 retained bytes, and zero failures. The general
package verifier passes.

The resource record's platform-exact text hashes match the retained governed
execution environment and are bound separately from portable LF-canonical
source identity. No runtime artifact was rewritten.

## Claim Boundary

This pulse establishes completion and custody of the fifth predeclared
governed chain group. It is not a convergence or sampler-comparison result.
The frozen gate still requires GA Kruskal, six exact governed replays, and the
registered State/kernel-specific analysis.
