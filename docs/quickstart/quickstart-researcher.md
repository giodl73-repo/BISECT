# Quickstart: Academic Researcher (Parameter Sweeps)

**Who you are:** An academic running parameter sweeps over redistricting algorithms — testing how seed, balance tolerance, partition mode, or structural choices affect plan-level metrics.

**What you'll have at the end:** N plans for one state across a parameter grid, plus a per-plan metrics CSV ready for pandas / R, plus ensemble diagnostics and BisectionEnsemble outputs to characterise the feasible space.

**Time:** 10–15 minutes for VT / DE; longer for AL / TX.

---

## Steps

1. **Bootstrap** (same as every persona):
   ```bash
   bash bootstrap.sh
   ```

2. **Run a single plan** to confirm the pipeline works on your state of interest:
   ```bash
   redist state --state VT --year 2020 --label vt_baseline --seed 42
   ```
   Expected: `outputs/v1/2020/plans/vt_baseline/{manifest.json, final_assignments.json, ...}` and exit 0 in under a minute for VT.

3. **Sweep N seeds** with the same parameters, keep the top-K by a metric:
   ```bash
   redist sweep --state VT --year 2020 --n 50 --keep 5 --metric polsby_popper --label-prefix vt_sweep
   ```
   Expected: 50 plans run; 5 best by mean Polsby-Popper retained under `outputs/v1/2020/plans/vt_sweep_*`.

4. **Aggregate the results** for off-line analysis:
   ```bash
   redist aggregate --year 2020 --version v1 --states VT
   ```
   Produces `outputs/v1/2020/aggregated/{state_summary.csv, district_summary.csv, ...}` — one row per plan or district, ready to load in pandas.

---

## Section: PercentileSweep — studying the compactness-partisan tradeoff

The `--percentile` flag controls the METIS solution selected at each bisection step. Lower values favour compactness (minimum edge-cut); higher values allow the solver to explore off-optimal solutions. Sweeping this parameter reveals the compactness-partisan tradeoff directly.

```bash
# Sweep percentile from 0.0 (most compact) to 0.9 (most exploratory) for NC
for P in 0.0 0.1 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9; do
    redist state --state NC --year 2020 \
        --partition-mode apportion-regions \
        --percentile $P \
        --seed 42 \
        --label nc_pct_${P/./}
done
```

Then aggregate:
```bash
redist aggregate --year 2020 --version v1 --states NC \
    --labels nc_pct_00 nc_pct_01 nc_pct_02 nc_pct_03 nc_pct_04 \
             nc_pct_05 nc_pct_06 nc_pct_07 nc_pct_08 nc_pct_09
```

The resulting `district_summary.csv` gives you a 10-row table: edge-cut (proxy for compactness) and Democratic-seat count per percentile. Typical finding: the compactness-to-partisan tradeoff is largely flat between `0.0` and `0.4`, then the partisan composition shifts as the solver departs the compact extremum.

**Best-K aggregate shortcut:** Run the percentile sweep with `--keep 3` and let `redist aggregate` find the best plan per percentile level automatically:
```bash
redist sweep --state NC --year 2020 \
    --n 200 --keep 5 \
    --metric polsby_popper \
    --percentile 0.5 \
    --label-prefix nc_pct50_sweep
```

---

## Section: Ensemble diagnostics — characterising the feasible space

Before reporting a plan's partisan composition as a finding, use the ensemble command to characterise where that plan sits in the feasible space. The `redist ensemble` command generates multiple independent MCMC chains, computes Gelman-Rubin R-hat and ESS, and produces the `diagnostics/` artifact directory required by `redist research validate-ensemble`.

```bash
# Generate a 4-chain ensemble for NC (5000 steps each)
redist ensemble --state NC --year 2020 \
    --steps 5000 --chains 4 \
    --label nc_ensemble_2020
```

Expected output:
```
outputs/v1/ensembles/nc_ensemble_2020/
  ensemble_plans/chain_{0,1,2,3}/   # per-step assignments
  diagnostics/
    rhat.json          # R-hat per summary metric (target <1.05)
    ess.json           # ESS on summary statistics (target >=100)
    burn_in.json       # per-chain Geweke z-score + auto-cut
    acceptance_rates.csv
    hamming_autocorr.json   # partition-space mixing (tau_int per chain)
    trace.png
```

Then compare your target plan against the ensemble:
```bash
redist research validate-ensemble \
    --plan-label vt_baseline \
    --ensemble-label nc_ensemble_2020
```

Output: `target_plan_percentiles.json` — where the target plan ranks on efficiency gap, mean-median, and Polsby-Popper relative to the N ensemble plans. A plan at the 99th percentile for compactness is at the compact extremum of the feasible space: that is a publishable finding.

**Diagnostics requirement:** `validate-ensemble` will refuse to report a percentile if the `diagnostics/` directory is missing or incomplete. This is intentional — an ensemble without convergence diagnostics is not a defensible reference population.

---

## Section: BisectionEnsemble — using the bisection engine as a research mode

`BisectionEnsemble` uses the same METIS recursive-bisection engine as the standard pipeline, but samples the solution space by running many independent bisections with stochastic METIS seeds. This is faster than MCMC for small-to-medium states and is deterministically reproducible.

```bash
# BisectionEnsemble for NC: 200 independent bisection runs at percentile 0.5
redist state --state NC --year 2020 \
    --search bisection-ensemble \
    --percentile 0.5 \
    --ensemble-steps 200 \
    --label nc_bisect_ensemble
```

What this does:
- Runs 200 independent METIS bisections, each with a different derived seed
- Selects the solution at `--percentile` of the edge-cut distribution at each level
- Writes all 200 plans to `outputs/v1/2020/plans/nc_bisect_ensemble/ensemble/`
- Computes partisan metrics across the 200 plans and writes `ensemble_summary.csv`

**When to use BisectionEnsemble vs MCMC ensemble:**
- BisectionEnsemble is better when you want the compact tail of the distribution (plans near minimum edge-cut). It samples the _algorithmic_ solution space, not the _geometric_ feasible space.
- MCMC ensemble (`redist ensemble`) samples the full feasible space uniformly. Use it for percentile analysis and court-facing validate-ensemble reports.
- For a paper, run both and show that the BisectionEnsemble tail overlaps with the low-percentile MCMC plans. That corroborates the algorithmic claim.

```bash
# After running BisectionEnsemble, aggregate across all 200 plans
redist aggregate --year 2020 --version v1 --states NC \
    --ensemble-label nc_bisect_ensemble \
    --output outputs/v1/2020/aggregated/nc_bisect_ensemble_summary.csv
```

---

## Expected output at each step

- **Baseline plan:** under 60 s for VT, under 5 min for AL
- **Seed sweep:** N × per-plan time. The `--keep` filter trims after, not during, so disk usage is N plans until completion.
- **Aggregate:** CSV files with one row per state-plan and one row per district-plan combination
- **Ensemble (5000 steps, 4 chains, NC):** 20–40 min on a 4-core laptop; `diagnostics/` fully populated

## Where to go next

- Sweep tuning: `redist sweep --help` (metric choices, parallelism)
- Ensemble diagnostics reference: `docs/research/ensemble-diagnostics.md`
- Reproducibility for publication: `redist analyze --paper-mode` (AEA-compliant replication package)
- Statistical rigour on bloc voting: `redist analyze --types bloc-voting` (Callais Evidence Layer)
- GerryChain interop: `redist research check-compat` then notebook `04_gerrychain_interop.ipynb`

## Performance notes

- Single-state runs use METIS recursive bisection internally. Wall-clock scales with tract count: VT (184) approx 30 s, AL (1437) approx 5 min, TX (5265) approx 30 min on a 4-core laptop.
- Parallel sweeps use `rayon` thread pools; set `RAYON_NUM_THREADS` to cap.
- For ensembles larger than ~1000 plans, plan disk space: each plan dir is ~2–10 MB depending on analyses generated.
- BisectionEnsemble at N=200 for NC runs in under 5 min because the bisection engine is ~200x faster than Python MCMC.
