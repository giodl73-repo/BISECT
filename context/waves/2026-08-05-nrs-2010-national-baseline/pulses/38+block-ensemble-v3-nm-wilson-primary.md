---
pulse: 38
wave: nrs-2010-national-baseline
date: 2026-08-11
status: complete
---

# Block-Ensemble v3 NM Wilson Primary

## Outcome

The third governed v3 schedule entry, NM Wilson, completed all four frozen
2,000-step chains with base seed `20260812`, 500-step burn-in, and snapshot
stride 10. Its fresh actual-volume admission passed immediately before process
creation. The runner returned zero after 15,151.0922 seconds with peak RSS of
665,784,320 bytes and no resource failure.

The outer orchestration shell reached its one-hour attachment timeout, but it
did not terminate either the original Python resource monitor or its runner
child. Both original PIDs remained linked and were observed until natural
completion; no relaunch, retry, or second admission occurred.

The wrapper validated the 173,107,481-byte raw trace, deterministically
compressed it to 1,708,625 bytes, and deleted the raw scratch copy. The active
ledger contains 3/6 primaries, 0/6 governed replays, 18,517.8955 governed
runner seconds, 6,454,665 retained bytes, and zero failures. The general
package verifier passes.

## Claim Boundary

This pulse establishes completion and custody of the third predeclared
governed chain group. The long NM Wilson runtime is descriptive resource
evidence only, not a mixing or sampler-quality result. The frozen gate still
requires three primaries, six exact governed replays, and the registered
State/kernel-specific analysis.
