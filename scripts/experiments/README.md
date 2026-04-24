# Experimental Scripts

Research and validation scripts that are NOT part of the production pipeline.

These were developed during the research phase to test algorithm variants, compare approaches, and validate findings for the academic papers. They are one-off runs, not maintained pipeline steps.

## VRA / Minority District Experiments

| Script | What it tests | Related paper |
|--------|--------------|---------------|
| `test_vra_comprehensive.py` | All VRA methods (n-way, adaptive, recursive) vs 5 covered states | D.0, D.3 |
| `test_vra_states.py` | VRA compliance across AL, GA, LA, MS, SC | D.0, D.1 |
| `test_nway_partition.py` | Direct n-way METIS partitioning vs recursive bisection | D.2 |
| `test_nway_vs_recursive_comprehensive.py` | Full comparison n-way vs recursive | D.2 |
| `test_nway_vs_recursive_simple.py` | Simplified n-way vs recursive | D.2 |
| `test_nway_vs_recursive_v2.py` | v2 of n-way vs recursive comparison | D.2 |
| `test_adaptive_bisection.py` | Greedy adaptive tree selection | D.2 |
| `test_adaptive_recursive_full.py` | Full adaptive recursive on Alabama | D.2 |
| `test_minority_edge_weighting.py` | Edge weighting with minority VAP | D.0 |
| `test_majority_vs_total_minority.py` | Majority vs total minority as VRA constraint | D.1 |

## Algorithm Ablation Studies

| Script | What it tests |
|--------|--------------|
| `test_50_states_ablation.py` | Parameter sweep across all 50 states (~3-5h runtime) |
| `test_50_states_recursive_ablation.py` | Recursive-specific parameter sweep |
| `test_50_states_recursive_batched.py` | Batched version of recursive ablation |
| `test_edge_weighting_comprehensive.py` | Edge weight scaling factor sensitivity |
| `test_geographic_edge_weighting_p22.py` | Geographic edge weighting (paper 2.2) |
| `test_all_states_total_minority.py` | Minority % across all 50 states |

## State-Specific Tests

| Script | What it tests |
|--------|--------------|
| `test_california_texas.py` | Large state behavior (CA 52D, TX 38D) |

## Also in this directory (existing experiments)

The `mmd_*.py` scripts test multi-member districts. The `partisan_similarity_*.py` scripts are for the E.4 paper.

## Running

These scripts are standalone — run directly with Python from the project root:

```bash
py -3.13 scripts/experiments/test_vra_comprehensive.py
```

Most require data to already be downloaded and adjacency graphs built. Check the individual script's argparse help for options.

## Not for production

These scripts do not follow the STATUS protocol and are not called by the main pipeline. Do not add them to `process_single_state.py` or `run_complete_redistricting.py`.
