# NRS v0.3 NH/NM/GA Block-Ensemble Expansion v3

Status: Stage 0 and governed primaries complete; governed replays active at 3/6.

V3 is a fresh successor to the permanently closed v1 and v2 protocols. It
uses base seed `20260812`, a fresh ledger and package, and compiled execution
classes `excluded-expansion-v3-preflight` and `governed-stage2-v3`. No
predecessor completion or trace can count toward this package.

Before any chain can be authorized, the readiness preparation must build the
release runner and exercise its side-effect-free `--contract-only true` path
against all 12 positive State/sampler/shape combinations plus negative
wrong-seed and predecessor-class controls. These probes load no input, write no
trace, and are not ensemble draws.

The still-empty readiness package was prepared and reviewed with:

```text
cargo build --release --example block_trace --example validate_block_input
python scripts/research/prepare_block_ensemble_v3_readiness.py
python scripts/research/verify_block_ensemble_expansion_v3.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v3
python scripts/research/verify_block_ensemble_v3_readiness.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v3
```

Those preparation/readiness commands describe the retained prelaunch gate and
must not be rerun against this populated Stage 0 package. Use the current
general and Stage 0 verifiers below.

The readiness capacity snapshot is observational and cannot authorize a chain.
Every future process still requires a new actual-volume admission record. The
frozen protocol is
`docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v3.md`.

Pulse 34 refreshed readiness before any launch after an ordinary Windows branch
switch exposed mixed LF/CRLF hash custody. Reviewable text is now LF-canonical;
executables and scientific inputs remain byte-exact.

Pulse 35 then completed all six excluded 25-step preflights in frozen order and
all six fresh-process normalized replays exactly. The active ledger retains
3,386,273 bytes of preflight traces, zero governed wall time, zero failures, and
no primary or governed replay. Verify this gate with:

```text
python scripts/research/verify_block_ensemble_v3_stage0.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v3
```

Pulse 36 completed the first governed primary, NH Wilson: all four frozen
2,000-step chains returned zero in 2,927.3477 runner seconds with 181,809,152
bytes peak RSS. The 52,663,157-byte raw trace was validated, deterministically
compressed to 713,443 bytes, and deleted. The active ledger now has 1/6
primaries, 0/6 governed replays, 4,099,716 retained bytes, and no failures.

The Stage 0 verifier intentionally applies only at the pre-governed boundary.
After a primary begins, verify the current package with:

```text
python scripts/research/verify_block_ensemble_expansion_v3.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v3
```

Pulse 37 completed NH Kruskal in 439.4555 runner seconds with 195,338,240
bytes peak RSS. Its validated 52,659,037-byte raw trace was deterministically
compressed to 646,324 bytes and deleted. The active ledger now has 2/6
primaries, 0/6 governed replays, 3,366.8033 governed runner seconds, 4,746,040
retained bytes, and no failures.

Pulse 38 completed NM Wilson in 15,151.0922 runner seconds with 665,784,320
bytes peak RSS. Its validated 173,107,481-byte raw trace was deterministically
compressed to 1,708,625 bytes and deleted. The outer orchestration shell
reached its one-hour attachment timeout, but the original Python resource
monitor and child runner remained alive and completed without relaunch. The
active ledger now has 3/6 primaries, 0/6 governed replays, 18,517.8955 governed
runner seconds, 6,454,665 retained bytes, and no failures.

Pulse 39 completed NM Kruskal in 1,942.9729 runner seconds with 703,852,544
bytes peak RSS. Its validated 173,103,802-byte raw trace was deterministically
compressed to 1,580,983 bytes and deleted. The active ledger now has 4/6
primaries, 0/6 governed replays, 20,460.8683 governed runner seconds, 8,035,648
retained bytes, and no failures.

Pulse 40 completed GA Wilson in 3,384.9358 runner seconds with 1,391,804,416
bytes peak RSS. Its validated 454,914,408-byte raw trace was deterministically
compressed to 6,708,674 bytes and deleted. The active ledger now has 5/6
primaries, 0/6 governed replays, 23,845.8041 governed runner seconds,
14,744,322 retained bytes, and no failures.

Pulse 41 completed GA Kruskal in 566.8571 runner seconds with 1,397,723,136
bytes peak RSS. Its validated 453,636,562-byte raw trace was deterministically
compressed to 5,920,216 bytes and deleted. All six governed primaries are now
complete. The active ledger has 0/6 governed replays, 24,412.6612 governed
runner seconds, 20,664,538 retained bytes, and no failures.

Pulse 42 completed the first governed replay, NH Wilson, in 2,830.6271 runner
seconds with 186,994,688 bytes peak RSS. Its 52,662,759-byte raw trace exactly
matched the retained primary after normalization and was deleted. The active
ledger now has 1/6 governed replays, 27,243.2884 governed runner seconds,
20,664,538 retained bytes, and no failures.

Pulse 43 completed the second governed replay, NH Kruskal, in 407.0650 runner
seconds with 196,825,088 bytes peak RSS. Its 52,658,019-byte raw trace exactly
matched the retained primary after normalization and was deleted. The active
ledger now has 2/6 governed replays, 27,650.3534 governed runner seconds,
20,664,538 retained bytes, and no failures.

Pulse 44 completed the third governed replay, NM Wilson, in 12,753.1016 runner
seconds with 670,076,928 bytes peak RSS. Its 173,108,465-byte raw trace exactly
matched the retained primary after normalization and was deleted. The active
ledger now has 3/6 governed replays, 40,403.4550 governed runner seconds,
20,664,538 retained bytes, and no failures.
