---
pulse: 35
wave: nrs-2010-national-baseline
date: 2026-08-11
status: complete
---

# Block-Ensemble v3 Stage 0

## Outcome

All six excluded 25-step preflights completed in frozen NH/NM/GA,
Wilson/Kruskal order. Six fresh processes then regenerated every normalized
metric and canonical snapshot exactly in the same order. Replay scratch traces
were deleted only after comparison.

The 12 excluded processes used 130.8802 total runner seconds. Peak process RSS
was 180,961,280 bytes. The six retained preflight traces occupy 3,386,273 bytes.
Every fresh capacity admission passed, every runner returned zero, and the
active ledger contains no failure, governed wall time, primary, or governed
replay.

The dedicated Stage 0 verifier and general package verifier both pass. The
first governed primary, NH Wilson, remains separately gated behind retention,
review, and a new actual-volume capacity admission.

Portable verification preserves two text-custody layers without rewriting the
runtime records: LF-canonical hashes identify reviewable source content, while
the readiness map fixes the exact Windows source bytes recorded by every Stage
0 process. The same retained map fixes the original execution package and
ledger paths so cross-platform verification checks recorded custody rather than
pretending the processes ran at the verifier's checkout path.

## Claim Boundary

Stage 0 samples are excluded from statistical analysis and stopping decisions.
This pulse establishes implementation integration, bounded resource
observation, and exact deterministic preflight replay only. It is not governed
ensemble evidence, a convergence result, or a kernel comparison.
