---
pulse: 47
wave: nrs-2010-national-baseline
date: 2026-08-13
status: complete
---

# Block-Ensemble v3 GA Kruskal Replay

## Outcome

The sixth and final governed v3 replay, GA Kruskal, completed all four frozen
2,000-step chains with base seed `20260812`, 500-step burn-in, and snapshot
stride 10. Its fresh actual-volume admission passed immediately before process
creation. The runner returned zero after 562.3833 seconds with peak RSS of
1,399,767,040 bytes and no resource failure.

The wrapper validated the 453,636,076-byte raw replay trace, normalized it and
the retained primary, and found an exact match. It then deleted the replay
scratch trace. The ledger is complete at 6/6 primaries and 6/6 governed
replays, 46,043.8662 governed runner seconds, 20,664,538 retained bytes, and
zero failures. The general package verifier passes.

The resource record's platform-exact text hashes match the retained governed
execution environment and are bound separately from portable LF-canonical
source identity. No runtime artifact was rewritten.

## Claim Boundary

This pulse establishes completion, custody, and exact normalized replay
agreement for every frozen governed chain group. It does not itself report a
convergence or sampler-comparison result. The next phase is the registered
State/kernel-specific analysis of the retained primary traces.
