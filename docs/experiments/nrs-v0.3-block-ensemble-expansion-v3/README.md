# NRS v0.3 NH/NM/GA Block-Ensemble Expansion v3

Status: Stage 0 complete; no governed v3 process has run.

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
