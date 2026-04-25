# Performance Targets

Baseline measurements from current Python pipeline (2026-04-24, Windows 11, 12-core).

## Current baselines

| Stage | Python time | Notes |
|---|---|---|
| Adjacency build (50 states, 2020) | ~22 min | GeoPandas spatial join |
| Single state redistricting (CA) | ~45 s | Includes METIS subprocess |
| Single state redistricting (VT) | ~3 s | 1 district, trivial |
| Full 50-state run (2020) | ~55 min | 4 parallel workers |
| National post-processing | ~8 min | 9 parallel tasks |
| Dashboard generation (V4) | ~4 min | 308 image embeds |

## Rust targets

| Stage | Target | Rationale |
|---|---|---|
| Adjacency build (50 states) | < 3 min | R-tree replaces O(n²) Python loop |
| Single state redistricting (CA) | < 15 s | Eliminate subprocess + Python overhead |
| Full 50-state run (2020) | < 10 min | Rayon parallelism, no GIL |
| Dashboard generation | < 30 s | Pure I/O-bound, just faster |

## Measurement protocol

After each phase, run:
```bash
hyperfine --warmup 1 \
  'python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --version V3' \
  'redist run --state CA --year 2020 --version V3'
```

Record in `migration-log.md`.
