---
pulse: 46
wave: nrs-2010-national-baseline
date: 2026-08-13
status: complete
---

# Block-Ensemble v3 GA Wilson Replay

## Outcome

The fifth governed v3 replay, GA Wilson, completed all four frozen 2,000-step
chains with base seed `20260812`, 500-step burn-in, and snapshot stride 10.
Its fresh actual-volume admission passed immediately before process creation.
The runner returned zero after 3,203.1085 seconds with peak RSS of
1,399,513,088 bytes and no resource failure.

The wrapper validated the 454,914,612-byte raw replay trace, normalized it and
the retained primary, and found an exact match. It then deleted the replay
scratch trace. The active ledger contains 6/6 primaries, 5/6 governed replays,
45,481.4829 governed runner seconds, 20,664,538 retained bytes, and zero
failures. The general package verifier passes.

The resource record's platform-exact text hashes match the retained governed
execution environment and are bound separately from portable LF-canonical
source identity. No runtime artifact was rewritten.

## Claim Boundary

This pulse establishes exact normalized replay agreement for the fifth frozen
governed chain group. It is not a convergence or sampler-comparison result.
The frozen gate still requires the final GA Kruskal replay and the registered
State/kernel-specific analysis.
