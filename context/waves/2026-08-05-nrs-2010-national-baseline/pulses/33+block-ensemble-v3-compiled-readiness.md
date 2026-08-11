---
pulse: 33
wave: nrs-2010-national-baseline
date: 2026-08-11
status: complete
---

# Block-Ensemble v3 Compiled Readiness

## Outcome

Expansion v3 is frozen and locally ready for review. It has a fresh protocol
ID, ledger schema, package, execution classes, and mechanically unused base
seed `20260812`. Closed v1/v2 artifacts and completions cannot count toward it.

The release runner now has a side-effect-free contract validation path. The
exact hash-bound Windows release executable accepted all 12 frozen positive
tuples: NH/NM/GA, Wilson/Kruskal, and preflight/governed shapes. It rejected a
v3 tuple using the v2 seed and rejected the unsupported v2 execution class.
All 14 outcomes were retained and exactly replayed by the readiness verifier.

The probe commands resolved no RCTX or assignment path, specified no output,
wrote no trace, and created no package artifact. Rust contract tests and seven
focused Python tests passed. The general verifier reports an active pristine
ledger with 0/6 preflights, 0/6 primaries, and 0/6 replays.

At readiness observation, 95,035,809,792 bytes were free against the frozen
8,589,934,592-byte empty-package requirement. This snapshot is not reusable
admission authority; every future process still requires a fresh capacity
record.

## Evidence

- `docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v3.md`
- `docs/experiments/nrs-v0.3-block-ensemble-expansion-v3/`
- `scripts/research/prepare_block_ensemble_v3_readiness.py`
- `scripts/research/verify_block_ensemble_v3_readiness.py`
- `signals/simulate/contract/block-ensemble-expansion-v3-contract-2026-08-11.md`

## Next Gate

After commit and review, the first permissible chain process is the excluded
NH Wilson 25-step preflight through the capacity-admitted v3 wrapper. A started
nonzero process closes v3 without retry. No chain is authorized by this pulse
alone.

## Claim Boundary

This is implementation-integration and local executable-custody evidence only.
It is not an ensemble draw, feasibility result, convergence diagnostic, replay
result, or Wilson-versus-Kruskal comparison.
