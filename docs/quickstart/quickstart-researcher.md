# Quickstart: Academic Researcher (Parameter Sweeps)

**Who you are:** An academic running parameter sweeps over redistricting algorithms — testing how seed, balance tolerance, partition mode, or VRA constraints affect plan-level metrics.

**What you'll have at the end:** N plans for one state across a parameter grid, plus a per-plan metrics CSV ready for pandas / R / etc.

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

5. **(Optional)** When the Researcher Toolkit plan ships:
   - 5 Jupyter notebooks under `notebooks/`
   - `redist research validate-ensemble --plan-label X --ensemble-label Y` for outlier detection vs. an MCMC ensemble
   - `redist analyze --paper-mode` for AEA-compliant replication packages
   - GerryChain interop: `redist research check-compat`

---

## Expected output at each step

- **Step 2:** ≤ 60 s for VT, ≤ 5 min for AL
- **Step 3:** N × per-plan time. The `--keep` filter trims after, not during, so disk usage is N plans until completion.
- **Step 4:** CSV files with one row per state-plan and one row per district-plan combination

## Where to go next

- Sweep tuning: `redist sweep --help` (metric choices, parallelism)
- GerryChain interop (when shipped): `redist research check-compat`
- Reproducibility for publication: `redist analyze --paper-mode` (Researcher Toolkit plan, when shipped)
- For statistical rigor on bloc voting: `redist analyze --types bloc-voting` (Callais Evidence Layer plan)

## Performance notes

- Single-state runs use METIS recursive bisection internally. Wall-clock scales with tract count: VT (184) ≈ 30 s, AL (1437) ≈ 5 min, TX (5265) ≈ 30 min on a 4-core laptop.
- Parallel sweeps use `rayon` thread pools; set `RAYON_NUM_THREADS` to cap.
- For ensembles larger than ~1000 plans, plan disk space: each plan dir is ~2–10 MB depending on analyses generated.
