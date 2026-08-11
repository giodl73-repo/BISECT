# NRS v0.3 NH/NM/GA Block-Ensemble Expansion v3

Status: readiness preparation only; no Stage 0 or governed v3 process has run.

V3 is a fresh successor to the permanently closed v1 and v2 protocols. It
uses base seed `20260812`, a fresh ledger and package, and compiled execution
classes `excluded-expansion-v3-preflight` and `governed-stage2-v3`. No
predecessor completion or trace can count toward this package.

Before any chain can be authorized, the readiness preparation must build the
release runner and exercise its side-effect-free `--contract-only true` path
against all 12 positive State/sampler/shape combinations plus negative
wrong-seed and predecessor-class controls. These probes load no input, write no
trace, and are not ensemble draws.

Prepare and verify the still-empty package with:

```text
cargo build --release --example block_trace --example validate_block_input
python scripts/research/prepare_block_ensemble_v3_readiness.py
python scripts/research/verify_block_ensemble_expansion_v3.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v3
python scripts/research/verify_block_ensemble_v3_readiness.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v3
```

The readiness capacity snapshot is observational and cannot authorize a chain.
Every future process still requires a new actual-volume admission record. The
frozen protocol is
`docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v3.md`.
