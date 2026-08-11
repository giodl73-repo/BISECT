# Resource Audit Commands

```powershell
cargo build --release -p bisect-ensemble --example block_trace --example validate_block_input
```

The NH, NM, and GA input audits used `validate_block_input` with their certified
2020 block RCTX, governed NRS v0.3 assignment, apportioned district count, and
`--tolerance 0.005`.

Wilson ran first, followed by Kruskal:

```powershell
python scripts/research/measure_block_ensemble_resources.py measure --sampler wilson --output docs/experiments/nrs-v0.3-block-ensemble-resource-audit/resource-wilson.json --scratch-trace target/block-ensemble-resource-wilson.json --poll-ms 50
python scripts/research/measure_block_ensemble_resources.py measure --sampler kruskal --output docs/experiments/nrs-v0.3-block-ensemble-resource-audit/resource-kruskal.json --scratch-trace target/block-ensemble-resource-kruskal.json --poll-ms 50
```

```powershell
python scripts/research/measure_block_ensemble_resources.py summarize --wilson docs/experiments/nrs-v0.3-block-ensemble-resource-audit/resource-wilson.json --kruskal docs/experiments/nrs-v0.3-block-ensemble-resource-audit/resource-kruskal.json --input-audit docs/experiments/nrs-v0.3-block-ensemble-gate/input-audit.json --input-audit docs/experiments/nrs-v0.3-block-ensemble-resource-audit/input-audit-nh.json --input-audit docs/experiments/nrs-v0.3-block-ensemble-resource-audit/input-audit-nm.json --input-audit docs/experiments/nrs-v0.3-block-ensemble-resource-audit/input-audit-ga.json --output docs/experiments/nrs-v0.3-block-ensemble-resource-audit/summary.json
python scripts/research/verify_block_ensemble_resources.py docs/experiments/nrs-v0.3-block-ensemble-resource-audit
```
