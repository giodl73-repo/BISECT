# Performance Targets

Baseline measurements from current Python pipeline (2026-04-24, Windows 11, 12-core).

> **Note**: baselines below are from operational experience and CLAUDE.md documentation.
> Authoritative measurements must be run with `hyperfine` before Phase 0 and recorded in `migration-log.md`.

## Current baselines (to be verified)

| Stage | Python time | Bottleneck |
|---|---|---|
| Adjacency build (50 states, 2020) | ~20 min | Per-edge Shapely `intersection()` calls — O(E), not O(n²) |
| Single state redistricting (CA) | ~45 s | METIS subprocess runtime |
| Single state redistricting (VT) | ~3 s | 1 district, trivial |
| Full 50-state run (2020, 4 workers) | ~55 min | Adjacency + METIS (bottleneck: slowest state) |
| National post-processing | ~8 min | 9 parallel tasks |
| Dashboard generation (V4) | ~4 min | 308 image embeds |

## Rust targets by phase

| Phase | Stage improved | Target | Mechanism |
|---|---|---|---|
| Phase 2 | Adjacency build | < 4 min | Rayon-parallel intersection calls |
| Phase 3 | Full 50-state run | < 25 min | Phase 2 savings + more parallel workers |
| Phase 5 | Full 50-state run | < 10 min | Native METIS eliminates subprocess overhead |

The 55 min → 10 min target requires Phase 5. Phases 1-4 alone realistically achieve ~20-25 min.

## Measurement protocol

Before Phase 0 (establish baseline):
```bash
hyperfine --warmup 1 --runs 3 \
  'python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --version V3 --position 0'
```

After each phase, re-run and record in `migration-log.md`. Compare:
```bash
hyperfine --warmup 1 \
  'python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --version V3 --position 0' \
  'redist run --state CA --year 2020 --version V3'
```

## Adjacency bottleneck detail

The Python adjacency build breaks down as:
1. Queen contiguity detection via `libpysal` — uses STR-tree internally, already O(n log n). **Not the bottleneck.**
2. Boundary length computation (`adjacency.py:76-113`) — calls `geom_i.intersection(geom_j)` per edge. CPU-bound, single-threaded. **The bottleneck.**

Rust speedup comes from parallelizing step 2 with Rayon, not from replacing the STR-tree.
