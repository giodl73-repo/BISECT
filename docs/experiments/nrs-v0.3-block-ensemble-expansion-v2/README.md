# NRS v0.3 NH/NM/GA Block-Ensemble Expansion v2

Status: local Stage 0 readiness passed. No preflight or governed v2 process has
run.

This package is wholly separate from the failed and closed v1 package. Its
initial ledger has zero completions, zero retained bytes, zero runner wall
time, and no failure. The dedicated verifier must pass this empty active state
before Stage 0 can begin.

The frozen protocol is
`docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v2.md`. Every future
process must be created through the capacity-admitted v2 runner. Pulse 31
reverified all three inputs, built and hash-bound the two release executables,
confirmed the retained resource-audit package, and recorded a point-in-time
capacity snapshot. The dedicated readiness verifier rejects drift in any of
those bindings or any process artifact in this still-empty package.

Run the two non-executing checks with:

```text
python scripts/research/verify_block_ensemble_expansion_v2.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v2
python scripts/research/verify_block_ensemble_v2_readiness.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v2
```

The capacity snapshot is not an admission record. It cannot be reused to start
a process: the admitted runner must measure the actual evidence volume and
write a fresh immutable admission attempt immediately before every launch.
Likewise, local executable hashes establish author-machine custody, not a
reproducible build on another host.
