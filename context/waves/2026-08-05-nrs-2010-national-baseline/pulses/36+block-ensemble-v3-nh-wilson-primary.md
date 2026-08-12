---
pulse: 36
wave: nrs-2010-national-baseline
date: 2026-08-11
status: complete
---

# Block-Ensemble v3 NH Wilson Primary

## Outcome

The first governed v3 schedule entry, NH Wilson, completed all four frozen
2,000-step chains with base seed `20260812`, 500-step burn-in, and snapshot
stride 10. Its fresh actual-volume admission passed immediately before process
creation. The runner returned zero after 2,927.3477 seconds with peak RSS of
181,809,152 bytes and no resource failure.

The wrapper validated the 52,663,157-byte raw trace, deterministically
compressed it to 713,443 bytes, and deleted the raw scratch copy. The active
ledger contains 1/6 primaries, 0/6 governed replays, 2,927.3477 governed runner
seconds, 4,099,716 retained bytes, and zero failures. The general package
verifier passes.

LF-canonical source identity and per-process platform-exact hashes are retained
separately. This preserves the runtime resource record without pretending that
its mixed pre-normalization Windows text bytes are the portable source
identity.

## Claim Boundary

This pulse establishes completion and custody of one predeclared governed
chain group. It is not a convergence finding, a Wilson-versus-Kruskal
comparison, or a passing expansion result. The frozen all-or-nothing gate still
requires the remaining five primaries, all six exact governed replays, and the
registered State/kernel-specific analysis.
