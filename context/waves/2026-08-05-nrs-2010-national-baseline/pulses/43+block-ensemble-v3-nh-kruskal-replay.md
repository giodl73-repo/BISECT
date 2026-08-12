---
pulse: 43
wave: nrs-2010-national-baseline
date: 2026-08-12
status: complete
---

# Block-Ensemble v3 NH Kruskal Replay

## Outcome

The second governed v3 replay, NH Kruskal, completed all four frozen 2,000-step
chains with base seed `20260812`, 500-step burn-in, and snapshot stride 10.
Its fresh actual-volume admission passed immediately before process creation.
The runner returned zero after 407.0650 seconds with peak RSS of 196,825,088
bytes and no resource failure.

The wrapper validated the 52,658,019-byte raw replay trace, normalized it and
the retained primary, and found an exact match. It then deleted the replay
scratch trace. The active ledger contains 6/6 primaries, 2/6 governed replays,
27,650.3534 governed runner seconds, 20,664,538 retained bytes, and zero
failures. The general package verifier passes.

The resource record's platform-exact text hashes match the retained governed
execution environment and are bound separately from portable LF-canonical
source identity. No runtime artifact was rewritten.

## Claim Boundary

This pulse establishes exact normalized replay agreement for the second frozen
governed chain group. It is not a convergence or sampler-comparison result.
The frozen gate still requires the remaining four exact governed replays and
the registered State/kernel-specific analysis.
