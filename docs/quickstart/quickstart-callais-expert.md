# Quickstart: §2 Plaintiff's Expert (post-Callais)

**Who you are:** A plaintiffs' expert in a §2 Voting Rights Act case after *Louisiana v. Callais* (608 U.S. ___, 2026-04-29). You need to demonstrate within-party racial bloc voting that survives partisan controls — what *Callais* p.36 calls the disentanglement requirement.

**What you'll have at the end:** A defensible bloc-voting analysis (WLS + HC3 robust SE + Holm-Bonferroni multiple-testing correction + cluster-bootstrap by county + race-of-candidate provenance chain) plus a court-formatted PDF.

**Time:** 30–60 minutes for an unfamiliar state; 15 minutes once you know the data.

---

> **Reading order:** Run the Vermont walkthrough first (`bash examples/vermont-2020-walkthrough/run.sh`) to validate your setup. The Louisiana walkthrough below is the *advanced* path and assumes Vermont works end-to-end.

## Steps

1. **Bootstrap + Vermont smoke test** (one time):
   ```bash
   bash bootstrap.sh
   bash examples/vermont-2020-walkthrough/run.sh   # validates pipeline + provenance
   ```

2. **Fetch Louisiana election data** via the registry:
   ```bash
   redist fetch --type elections --states LA --year 2020 --source openelections
   ```
   See `scripts/data/elections/sources.json` for which state-year combinations have automated fetchers vs. require manual download.

3. **Produce a per-tract Democratic-share file** for the partisan baseline:
   ```bash
   python scripts/data/political/build_dem_shares.py \
       --state LA --year 2020 --mode vap-disaggregate \
       --output data/elections/dem_shares/LA_2020.tsv
   ```

4. **Run a Louisiana plan** with the partisan-weighted bisection mode:
   ```bash
   redist state --state LA --year 2020 --label la_2020_callais \
       --partition-mode partisan-weighted \
       --partisan-shares data/elections/dem_shares/LA_2020.tsv
   ```
   The Callais p.36 mutex enforces that VRA-aware bisection and partisan-weighted bisection are mutually exclusive in the same run — `redist state` will refuse a configuration that turns both on.

5. **Run the bloc-voting analysis** (Callais Evidence Layer — when implemented):
   ```bash
   redist analyze --label la_2020_callais --types bloc-voting \
       --candidate-race-csv data/elections/race_of_candidate/2020-presidential-primary.csv \
       --party DEM --election presidential-primary
   ```
   Produces `analysis/bloc_voting.json` with WLS+HC3 coefficients, cluster-bootstrap CIs, Holm-corrected p-values, robustness across 3 baselines, VIF diagnostic, and the `ecology_caveat`.

6. **Generate the court PDF** (Court Submission Reports plan — when implemented):
   ```bash
   redist report --label la_2020_callais --format pdf \
       --jurisdiction "EDLA" \
       --expert-config expert.toml
   ```
   Outputs PDF/A-2b validated against `verapdf`; reproducibility zip with attestation chain.

7. **Provenance receipt:** verify the manifest matches your binary, and embed the result in your Daubert disclosures:
   ```bash
   redist doctor --verify-manifest outputs/v1/2020/plans/la_2020_callais/manifest.json
   ```

---

## Expected output at each step

- **Step 4:** `final_assignments.json` for LA (~1100 tracts), exit 0
- **Step 5:** `bloc_voting.json` schema validates; `regression.specification` reads `"WLS weighted by precinct vote count; HC3 robust SE; cluster-bootstrap by county B=10000"`
- **Step 6:** PDF/A-2b certified by `verapdf`; refuses to mark non-compliant output as court-ready

## Where to go next

- Callais legal grounding: `docs/legal/CALLAIS_REFERENCE.md`
- Race-of-candidate provenance protocol: `docs/file-formats/race-of-candidate.md` (when shipped)
- Deposition prep: `redist depo` (Deposition Prep plan — fast iterative re-analysis daemon)
- AEA-compliant replication package for paper publication: `redist analyze --paper-mode`

## Daubert warnings

- Race-of-candidate annotations require curator attestation. Submitting an analysis where `independently_verified = false` injects a `[CAVEAT — annotations not independently verified]` prefix into the draft interpretation. You can rebut this on the stand, but the caveat is part of the JSON record.
- The `ecology_caveat` is verbatim in every bloc-voting output: precinct-level association ≠ individual-voter behavior. Do not strip it.
- Robustness across the three alternative baselines (statewide, district, prior-primary Dem share) MUST hold for the headline coefficient before claiming "significant." The `robustness_check.race_coefficient_significant_under_all` flag is the gate.
