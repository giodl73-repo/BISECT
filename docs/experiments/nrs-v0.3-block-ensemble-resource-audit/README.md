# NRS v0.3 Block-Ensemble Resource Audit

Status: candidate inputs, instrumented Rhode Island resource replays, and the
mechanical expansion-budget decision passed; package verification pending.

This package measures author-machine resource requirements for the completed
Rhode Island Stage 1 shape. Resource replays are excluded from statistical
analysis and are deleted after exact normalized comparison with the committed
governed traces.

The candidate expansion inputs contain 31,948 NH blocks, 107,215 NM blocks,
and 232,717 GA blocks. Each verified starting assignment is contiguous and
inside the frozen `0.005` population tolerance. These audits do not authorize
ensemble execution in those States.

See `docs/specs/2026-08-10-nrs-v0.3-block-ensemble-resource-audit.md`.
The exact runner source used for measurement is retained as
`block_trace-stage1.rs`, allowing later Stage 2 runner development without
changing this package's software identity.

## Result

Both excluded resource replays regenerated the committed normalized metrics
and canonical snapshots exactly. Wilson took `2154.316` seconds and peaked at
`164,225,024` bytes RSS; Kruskal took `381.056` seconds and peaked at
`175,448,064` bytes. The peak values are Windows OS-reported process peak
working sets and include final trace serialization.

The frozen unit-ratio formulas project `36,759.885` seconds for sequential
NH/NM/GA execution. They authorize a 21-hour compute budget, 2.25 GiB
per-process memory, 3 GiB retained storage, and 3 GiB scratch storage. These
values are below the 48-hour, 4 GiB, and 8 GiB hard ceilings, so a separate
expansion protocol is eligible to be drafted. No expansion execution is
authorized by this package.

## Verify

```powershell
python scripts/research/verify_block_ensemble_resources.py docs/experiments/nrs-v0.3-block-ensemble-resource-audit
```
