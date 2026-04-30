# Researcher Toolkit
**Date:** 2026-04-30
**Status:** Proposed; pending review
**Closes gap for:** academic researcher (★★★★→★★★★★)
**Depends on:** existing redist_py PyO3 bindings + redist-cli + redist-analysis
**Estimated effort:** 3-5 days

## Why this exists

A redistricting researcher today can use the project — performance is great, the algorithm is correct, the PyO3 bridge works — but the experience is engineer-coded:

- No Jupyter notebook examples; first-time users have to figure out the import surface
- No MCMC ensemble support (the academic gold standard for measuring fairness via random comparison)
- No formal interop testing against GerryChain (the field's canonical Python library)
- No statistical-validation framework for "is this plan an outlier compared to a random ensemble?"

Other tools (GerryChain, MGGG-states, OpenPrecincts) target this audience first. We have a faster engine; this spec packages it for them.

## Scope

### In scope

1. **Jupyter notebook examples** under `notebooks/`:
   - `01_quickstart.ipynb` — load adjacency, run a single bisection, plot
   - `02_parameter_sweep.ipynb` — run N seeds, compare metrics
   - `03_callais_evidence.ipynb` — load + run the bloc-voting analysis (cross-references Callais Evidence Layer spec)
   - `04_gerrychain_interop.ipynb` — load a GerryChain partition, validate via redist
   - `05_mcmc_ensemble.ipynb` — generate an ensemble of plans, compare to a target plan

2. **MCMC ensemble support** — Either:
   - **(a)** A Rust-native MCMC implementation in `redist-core` (recombination-based, like ReCom) producing `N` random plans
   - **(b)** A Python wrapper that calls GerryChain for the MCMC step and uses redist for analysis
   
   Spec recommendation: **(b) first, then (a) later**. Faster delivery, validates against the field standard, lets us focus on what redist does best (analysis).

3. **GerryChain interop**
   - `redist export --format gerrychain` already exists — verify it produces valid GerryChain v2.3 JSON
   - New: `redist import --format gerrychain <PATH>` — load a GerryChain Partition into a redist plan label
   - Notebook example showing round-trip

4. **Statistical-validation framework**
   - `redist validate-against-ensemble --plan LABEL --ensemble-dir <DIR>` — compares a single plan to an ensemble of random plans on key metrics (efficiency gap, mean-median, MM count, PP)
   - Outputs: percentile of the plan within the ensemble + plain-English flag ("Plan ranks at the 99th percentile for efficiency gap; visible-outlier candidate")

5. **Reproducibility recipe** for academic publication
   - `redist analyze --paper-mode` flag that adds extra rigor to outputs:
     - All seeds recorded
     - All input file SHAs
     - Embedded version table (Rust + Python + dependencies)
     - Citation block ready to paste into LaTeX

### Out of scope

- A native MCMC implementation in Rust (deferred; GerryChain is fine for the academic mainstream)
- Custom MCMC variants (ReCom, balanced-tree, etc. — leave to the academic literature)
- High-level statistical inference (academics do that themselves with R / pandas / etc.)
- Survey-design tools for fairness measurement (separate research category)

## Implementation notes

### MCMC via GerryChain wrapper

Rather than re-implement MCMC in Rust, we ship a thin Python wrapper:

```python
# scripts/research/mcmc_ensemble.py
import gerrychain
import redist_py

def generate_ensemble(state, year, n_steps=10000, seed=42):
    """Generate a GerryChain ensemble, return list of plan labels."""
    # ... use GerryChain to generate plans
    # ... call redist analyze on each via PyO3

def validate_against_ensemble(plan_label, ensemble_dir):
    """Compare a single plan to the ensemble; return percentile ranks."""
    # ... load all ensemble plans + target plan
    # ... compute percentile per metric
```

This is honest about the architecture: GerryChain is the field's MCMC infrastructure, redist is the field's fast analysis infrastructure; they compose well.

### Notebook execution as part of CI

Run notebooks via `nbconvert --execute` in CI to catch breakage:

```yaml
- name: Execute notebooks
  run: |
    pip install nbconvert
    for nb in notebooks/*.ipynb; do
      jupyter nbconvert --to notebook --execute --output /tmp/out "$nb"
    done
```

### redist import --format gerrychain

GerryChain's Partition object serializes to JSON via `Partition.to_json()`. Our import:
1. Read the JSON
2. Validate the schema
3. Map GerryChain's node IDs to our GEOIDs
4. Write a redist plan label with the same assignment

Round-trip property: `gerrychain → redist → gerrychain` should produce identical assignments (modulo ID encoding).

## Outputs

```
notebooks/                       # NEW directory
├── 01_quickstart.ipynb
├── 02_parameter_sweep.ipynb
├── 03_callais_evidence.ipynb
├── 04_gerrychain_interop.ipynb
└── 05_mcmc_ensemble.ipynb

scripts/research/                # NEW directory
└── mcmc_ensemble.py             # GerryChain wrapper

outputs/{version}/ensembles/{label}/   # NEW per-ensemble outputs
├── ensemble_plans/              # N plan JSONs
├── ensemble_metrics.csv         # one row per plan, all metrics
└── target_plan_percentiles.json # validation output
```

## Tests

- L0: GerryChain ↔ redist roundtrip on synthetic 10-node graph
- L1: small-state ensemble (N=100, VT or DE) generates + analyzes within 60s
- L2: skipped-by-default — full notebook execution in CI

## Risks

| Risk | Mitigation |
|---|---|
| GerryChain API changes break our wrapper | Pin GerryChain version; document the upgrade procedure |
| MCMC ensemble generation is slow (N=10k for AL takes hours) | Document realistic times; offer `--max-time-secs` cap |
| Researchers want native Rust MCMC for speed | Ship Python wrapper first; revisit native in a later spec if demand |
| Notebook examples drift from CLI surface | Auto-execute notebooks in CI |

## Definition of done

- All 5 notebooks render + execute end-to-end without manual intervention
- `redist import --format gerrychain` round-trips a GerryChain plan
- `redist validate-against-ensemble` produces percentile output that matches a hand-computed value within 0.5 percentile points
- One academic-style writeup template (paper_mode appendix) committed under `examples/`
- Notebooks tested in CI on every PR
