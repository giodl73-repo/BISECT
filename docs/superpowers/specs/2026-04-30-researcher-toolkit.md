# Researcher Toolkit
**Date:** 2026-04-30
**Updated:** 2026-04-30 (v2 — incorporates 6-role consensus on AEA replication standards + ensemble diagnostics)
**Status:** Revised; pending re-review
**Closes gap for:** academic researcher (★★★★→★★★★★)
**Depends on:** existing redist_py PyO3 bindings + redist-cli + redist-analysis
**Estimated effort:** 3-5 days (v2: notebook attestation + GerryChain handshake)

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

5. **Reproducibility recipe** for academic publication — **AEA Data and Code Availability Policy compliant (v2 — DATUM/COVENANT)**
   - `redist analyze --paper-mode` flag that produces an AEA-style replication package:
     - **README.md** with: dataset list (each with citation + access date + DOI/URL), software requirements (`redist` version, `rustc` version, Python version, every PyPI dep with pinned version), step-by-step run instructions, expected output checksums per step
     - **All seeds** recorded (master seed + per-step derivations)
     - **All input file SHA-256s** as a JSON sidecar
     - **Software environment** captured: `pip freeze`, `cargo metadata --format-version=1`, OS / arch / kernel
     - **Citation block** in BibTeX + APA + Chicago for paste into LaTeX
     - **Acceptance smoke test** the reviewer can run: `bash REPRODUCE.sh` produces the headline tables/figures from a clean checkout in ≤ 30 minutes on a 4-core laptop
   - Conformance: passes the `social-science-data-editors/template-readme` lint (where applicable)

6. **GerryChain version handshake (v2 — TRENCH/SCALE)**
   - Every notebook + the wrapper checks `gerrychain.__version__` against a pinned range (`>=0.3.2,<0.4`) on import
   - Mismatch raises a clear error: "Notebook tested against GerryChain 0.3.2; you have 0.3.5. Either downgrade or run `redist research check-compat` to confirm round-trip still works."
   - `redist research check-compat` runs the round-trip property test against the user's installed GerryChain and reports pass/fail

7. **Ensemble-bias diagnostics (v2 — SCALE/BENCHMARK)** — for any ensemble we generate or import:
   - **Trace plots** for key statistics (efficiency gap, MM count, PP mean) over MCMC steps
   - **Effective sample size** (ESS) per metric using the Gelman-Rubin convention; flag if ESS < 100
   - **Burn-in cut** auto-detected via the Geweke diagnostic; documented in the ensemble manifest
   - **Compactness/population proposal acceptance rates** logged per step
   - These artifacts go in `outputs/{version}/ensembles/{label}/diagnostics/` and are referenced from the notebook + the paper-mode README. Without them, an ensemble percentile is unfalsifiable.

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

### Notebook execution as part of CI (v2 — runtime budgets + kernel-state attestation)

Run notebooks via `nbconvert --execute` in CI to catch breakage. Each notebook must:

- **Declare a runtime budget** in cell 1 metadata: `{"runtime_budget_secs": 60}` for quickstart, `300` for parameter sweeps, `1800` for ensembles
- **Fail fast** if execution exceeds 1.5× budget (catches regressions)
- **Attest kernel state** at the end: a final cell asserts `redist_py.__version__`, `gerrychain.__version__`, and a hash of the notebook's input cells. Mismatches between the notebook's recorded state and what CI ran flag drift between the published `.ipynb` (with output cells) and the source.
- **No hidden state**: notebooks must run top-to-bottom; the CI step uses `--ExecutePreprocessor.allow_errors=False` and a clean kernel per notebook.

```yaml
- name: Execute notebooks
  run: |
    pip install nbconvert
    for nb in notebooks/*.ipynb; do
      budget=$(jq -r '.metadata.runtime_budget_secs // 120' "$nb")
      timeout $((budget * 3 / 2)) jupyter nbconvert \
        --to notebook --execute --output /tmp/out "$nb" \
        --ExecutePreprocessor.allow_errors=False
    done
```

Long-running ensemble notebooks are guarded behind `pytest -m slow`-equivalent CI selection so PR builds stay fast.

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
- L0: `redist research check-compat` against the installed GerryChain version (skipped if GerryChain not installed); asserts round-trip exact match
- L0 **kernel-state attestation**: notebook 01_quickstart.ipynb's final cell asserts `redist_py.__version__` matches a pinned value; CI fails on mismatch
- L0 **paper-mode acceptance** (BENCHMARK): `redist analyze --paper-mode` on a synthetic VT plan produces a `REPRODUCE.sh` whose execution from a clean Python venv reproduces all headline numbers byte-identically
- L0 **ensemble diagnostics**: synthetic 200-step ensemble has correct ESS, burn-in, and acceptance-rate JSON entries; assert numerical values against hand-computed truth
- L1: small-state ensemble (N=100, VT or DE) generates + analyzes within 60s; diagnostics directory populated
- L1 **runtime-budget enforcement**: a deliberately-slowed notebook (sleep 90s) with budget=60 fails CI with the budget-exceeded message
- L2: skipped-by-default — full notebook execution in CI on every PR (long ones gated by `pytest -m slow`-equivalent label)

## Risks

| Risk | Mitigation |
|---|---|
| GerryChain API changes break our wrapper | Pin GerryChain version; `redist research check-compat` runs round-trip on the user's installed version |
| MCMC ensemble generation is slow (N=10k for AL takes hours) | Document realistic times; offer `--max-time-secs` cap; runtime budget per notebook |
| Researchers want native Rust MCMC for speed | Ship Python wrapper first; revisit native in a later spec if demand |
| Notebook examples drift from CLI surface | Auto-execute notebooks in CI; kernel-state attestation cell catches output-only-vs-source drift |
| Ensemble percentiles published without convergence diagnostics | Mandatory `diagnostics/` artifact; report tooling refuses to cite a percentile from an ensemble missing ESS/burn-in records |
| AEA replication package goes stale between paper acceptance and final publication | `REPRODUCE.sh` is re-run in CI nightly against the pinned input set; staleness alerts before review |

## Definition of done

- All 5 notebooks render + execute end-to-end without manual intervention, within their declared runtime budgets
- `redist import --format gerrychain` round-trips a GerryChain plan; `redist research check-compat` passes against the pinned version
- `redist validate-against-ensemble` produces percentile output that matches a hand-computed value within 0.5 percentile points
- `redist analyze --paper-mode` produces an AEA-compliant replication package; `bash REPRODUCE.sh` reproduces headline numbers from clean checkout in ≤ 30 minutes on a 4-core laptop
- Every ensemble emits `diagnostics/{trace.png, ess.json, burn_in.json, acceptance_rates.csv}`; report tools refuse to cite undiagnosed ensembles
- One academic-style writeup template (paper_mode appendix) committed under `examples/`
- Notebooks tested in CI on every PR; long-running notebooks tested in nightly CI
