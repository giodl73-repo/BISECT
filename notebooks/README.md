# `redist` Research Notebooks

**Status:** scaffolding shipped 2026-04-30; full GerryChain integration deferred (Researcher Toolkit plan Tasks 3–6).
**Owner:** `docs/superpowers/plans/2026-04-30-researcher-toolkit.md`

This directory holds the five canonical Jupyter notebooks the Researcher Toolkit ships for academic users running parameter sweeps, MCMC ensembles, and the AEA paper-mode replication-package workflow.

| Notebook | Purpose | runtime_budget_secs |
|---|---|---|
| `01_quickstart.ipynb` | Load adjacency, run a single bisection, plot. First-time researcher onboarding. | 60 |
| `02_parameter_sweep.ipynb` | Run N seeds across a parameter grid, compare metrics. | 120 |
| `03_callais_evidence.ipynb` | Load + run the within-party racial bloc voting analysis (cross-references the Callais Evidence Layer). | 300 |
| `04_gerrychain_interop.ipynb` | Round-trip a GerryChain `Partition` through `redist` and back; demonstrates the import/export contract. | 120 |
| `05_mcmc_ensemble.ipynb` | Generate an MCMC ensemble of plans, validate convergence diagnostics, compare a target plan to the ensemble. | 1800 (nightly only) |

## Conventions

### Cell-1 metadata: runtime budget

Every notebook declares its expected wall-clock budget in cell-1 metadata:

```json
{
  "metadata": {
    "runtime_budget_secs": 60
  }
}
```

The CI workflow that executes notebooks fails if the actual run exceeds `1.5 × budget`. This catches regressions early.

### Cell-2 header: kernel-state attestation

Every notebook starts with the same import + version-attestation block in cell 2:

```python
import sys
from packaging.specifiers import SpecifierSet

# Pinned compatible ranges per Researcher Toolkit plan Task 2 / B-06.
# These are RANGES (not single versions) so legitimate patch revs of redist_py
# or gerrychain don't cause false-positive failures every release.
REDIST_PY_RANGE = ">=0.4,<0.5"
GERRYCHAIN_RANGE = ">=0.3.2,<0.4"

import redist_py
assert redist_py.__version__ in SpecifierSet(REDIST_PY_RANGE), \
    f"redist_py {redist_py.__version__} not in compatible range {REDIST_PY_RANGE}"

try:
    import gerrychain
    assert gerrychain.__version__ in SpecifierSet(GERRYCHAIN_RANGE), \
        f"gerrychain {gerrychain.__version__} not in compatible range {GERRYCHAIN_RANGE}"
    GERRYCHAIN_PRESENT = True
except ImportError:
    GERRYCHAIN_PRESENT = False
```

Out-of-range versions raise; missing GerryChain is a soft fallback (notebooks 04 + 05 require it; the others run without).

### Final cell: notebook-completion sentinel

Every notebook ends with:

```python
print("notebook completed within budget")
```

CI greps for this exact string in stdout. If the notebook exited via exception instead of running through, the sentinel won't appear and CI fails.

## Launching

After installing the `redist_py` PyO3 wheel:

```bash
maturin develop --release  # from redist/python/redist_py/
jupyter lab notebooks/
```

For the GerryChain-dependent notebooks (04, 05), install GerryChain in the same Python env:

```bash
pip install gerrychain==0.3.5
```

The exact pinned version is recorded in `requirements.lock` (Researcher Toolkit plan Task 8 / D-05; deferred until paper-mode lands).

## What's deferred

Per `docs/superpowers/plans/2026-04-30-researcher-toolkit.md`:

- **Notebook BODY content** — the cells inside each `.ipynb` beyond cell-1 metadata + cell-2 kernel attestation are TODO. The shipped notebooks are scaffolding.
- **`redist research check-compat` CLI** (Task 3) — runs the round-trip property test against the user's installed GerryChain version.
- **`redist research validate-ensemble` CLI** (Task 5, M-02 rename) — percentile-rank computation against an ensemble.
- **`scripts/research/mcmc_ensemble.py`** (Task 6) — GerryChain wrapper that generates the N-chain ensemble.
- **Notebook execution in CI** (Task 1.5) — `.github/workflows/notebooks.yml` with the runtime-budget enforcement.
- **`redist analyze --paper-mode`** wiring (Task 8) — the AEA replication package emitter; the REPRODUCE.sh template is shipped at `scripts/research/paper_mode_template/REPRODUCE.sh`.

The diagnostics math the notebooks consume (`redist-analysis::ensemble_diagnostics`) IS shipped today — see the just-landed R-hat / ESS / Hamming-autocorrelation module.

## See also

- `docs/superpowers/plans/2026-04-30-researcher-toolkit.md` — the full plan
- `docs/legal/FAIRNESS_DOCTRINE.md` §1.5 + §3 — how the diagnostics serve the post-Rucho state-court strategy
- `redist-analysis::ensemble_diagnostics` — R-hat, ESS, Hamming autocorrelation
- `scripts/research/paper_mode_template/` — the AEA REPRODUCE.sh template (shipped)
