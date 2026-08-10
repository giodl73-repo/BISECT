# Governed Stage 1 Commands

The release executable was built before execution:

```powershell
cargo build --release -p bisect-ensemble --example block_trace
```

Wilson ran first, followed by Kruskal. For each sampler, substitute `SAMPLER`
and `OUTPUT` below with `wilson` / `governed-wilson.json` and then `kruskal` /
`governed-kruskal.json`:

```powershell
target/release/examples/block_trace.exe --rctx data/2020/certified/ri_blocks_2020.rctx --assignments runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/baseline_assignments.json --state RI --year 2020 --districts 2 --tolerance 0.005 --sampler SAMPLER --steps 2000 --chains 4 --base-seed 20260810 --snapshot-stride 10 --execution-class governed-stage1 --output docs/experiments/nrs-v0.3-block-ensemble-gate/OUTPUT
```

Analysis used the frozen burn-in:

```powershell
python scripts/research/analyze_block_ensemble.py --wilson docs/experiments/nrs-v0.3-block-ensemble-gate/governed-wilson.json --kruskal docs/experiments/nrs-v0.3-block-ensemble-gate/governed-kruskal.json --burn-in 500 --output docs/experiments/nrs-v0.3-block-ensemble-gate/analysis.json --summary-csv docs/experiments/nrs-v0.3-block-ensemble-gate/summary.csv
```

The final verifier performs the separately required sequential regeneration:

```powershell
python scripts/research/verify_block_ensemble_gate.py docs/experiments/nrs-v0.3-block-ensemble-gate --stage1
```
