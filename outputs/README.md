# Outputs Directory Structure

This directory contains all redistricting pipeline outputs, experiments, and academic artifacts.

## Directory Organization

```
outputs/
  v1/                    # Production runs (version 1)
    2020/               # Census year 2020
      config.json       # Run configuration (metadata, algorithm, pipeline)
      states/           # Per-state outputs
      maps/             # National maps
      data/             # Aggregated data files
      index.html        # Dashboard
    2010/               # Census year 2010
    2000/               # Census year 2000

  experiments/          # Experimental runs (research variants)
    tract_vs_block_2020/
      tract_v1/         # Tract-level run
        config.json
        ...
      block_v1/         # Block-level run
        config.json
        ...
    edge_weight_comparison_2020/
      weighted_v1/
        config.json
        ...
      unweighted_v1/
        config.json
        ...

  dev/                  # Development/validation runs (hidden from dashboards)
    test_2020_vermont/  # Quick test runs
      config.json
      ...
    debug_alabama_edge/ # Debug runs
      config.json
      ...

  artifacts/            # Academic outputs (papers, presentations, figures)
    papers/             # LaTeX papers and PDFs
    presentations/      # LaTeX presentations and PDFs
    guides/             # Educational guides and PDFs
    figures/            # Generated figures (used by LaTeX)

  master_index.html     # Master dashboard (all runs)
```

## Run Types

### Production Runs (`v1/`)
- **Purpose**: Official redistricting results for publication
- **Directory**: `outputs/v1/{census_year}/`
- **Visibility**: Shown on master dashboard
- **Stability**: Versioned, reproducible

### Experimental Runs (`experiments/`)
- **Purpose**: Research variants (tract vs block, parameter sweeps)
- **Directory**: `outputs/experiments/{experiment_name}/`
- **Visibility**: Shown on master dashboard
- **Comparison**: Side-by-side config comparison in dashboard

### Development Runs (`dev/`)
- **Purpose**: Quick validation, debugging, development testing
- **Directory**: `outputs/dev/{state}_{year}_{timestamp}/`
- **Visibility**: Hidden from master dashboard
- **Cleanup**: Can be deleted anytime (excluded from git)

## Configuration Files

Every run includes a `config.json` file tracking:

```json
{
  "schema_version": "1.0",
  "metadata": {
    "created": "2026-01-17T14:30:00",
    "version": "v1",
    "census_year": 2020,
    "run_type": "production",
    "scope": "us",
    "states": ["all"]
  },
  "algorithm": {
    "partition_mode": "edge_weighted",
    "data_level": "tract"
  },
  "pipeline": {
    "skip_political": false,
    "dpi": 150
  },
  "system": {
    "python_version": "3.13.1",
    "execution_time_seconds": 7320
  }
}
```

## Running the Pipeline

### Production Run
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --run-type production
```

Creates: `outputs/v1/2020/`

### Experiment Run
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --run-type experiment --experiment-name tract_vs_block_2020
```

Creates: `outputs/experiments/tract_vs_block_2020/v1_2020/`

### Development Run (auto-detected)
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states VT,DE
```

Creates: `outputs/dev/test_2020_{timestamp}/`

## Viewing Results

### Individual Run Dashboard
```bash
# Automatically generated at end of run
outputs/v1/2020/index.html
outputs/experiments/tract_vs_block_2020/v1_2020/index.html
```

### Master Dashboard (All Runs)
```bash
python scripts/web/generate_master_dashboard.py
# Opens: outputs/master_index.html
```

## Academic Artifacts

### LaTeX Compilation
```bash
cd outputs/artifacts
compile.bat
```

Compiles:
- `papers/*/*.tex` → PDFs
- `presentations/*/*.tex` → PDFs
- `guides/*/*.tex` → PDFs

### Figures
Generated figures for papers/presentations:
- `outputs/artifacts/figures/real_tracts_examples/`
- `outputs/artifacts/figures/schematic/`
- `outputs/artifacts/figures/round_progression/`

## Notes

- All generated outputs excluded from git (see `.gitignore`)
- Config files enable full reproducibility
- Dashboard automatically discovers runs via config.json
- Dev runs excluded from master dashboard listings
- Academic artifacts moved from repo root to centralize outputs
