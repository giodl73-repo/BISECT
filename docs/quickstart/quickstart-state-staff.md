# Quickstart: State Legislative Staff (Map-Drawing Workflow)

**Who you are:** Staff for a state legislature, redistricting commission, or governor's office. You draw maps interactively (Districtr / Dave's Redistricting App / QGIS) and need fast analysis + court-quality reports as you iterate.

**What you'll have at the end:** A Districtr → `redist` → PDF round-trip that you can run without re-learning anything between map iterations.

**Time:** 5 minutes per iteration after first setup.

---

## Steps

1. **Bootstrap** (one time):
   ```bash
   bash bootstrap.sh           # Linux/macOS
   bootstrap.bat               # Windows
   ```

2. **Draw a map** in Districtr against your state's 2020 tracts. Save as JSON via Districtr's "Save Plan" → "Download as JSON".

3. **Import into `redist`:**
   ```bash
   redist import --format districtr senate_proposal_v3.json \
       --plan-label senate_proposal_v3 \
       --state VT --year 2020
   ```
   Expected: `outputs/v1/2020/plans/senate_proposal_v3/{manifest.json, final_assignments.json, translation_log.txt}` written. The translation log records any block-to-tract aggregation decisions if your Districtr plan was at block resolution.

4. **Run analysis:**
   ```bash
   redist analyze --plan-label senate_proposal_v3 --year 2020 --types all
   ```
   Wall-clock: under 30 s for VT, under 5 min for the largest states.

5. **Generate a report:**
   ```bash
   redist report --plan-label senate_proposal_v3 --year 2020 --format html
   ```
   Or `--format pdf` (Court Submission Reports plan, when shipped) for court-ready PDF/A-2b.

6. **Iterate.** Edit in Districtr, bump the label suffix (`senate_proposal_v4`), repeat steps 3–5.

---

## Expected output at each step

- **Step 3:** plan label visible via `redist doctor --label senate_proposal_v3`
- **Step 4:** five JSON files under `analysis/`, exit 0
- **Step 5:** `report.html` viewable in any browser; no external network calls

## Where to go next

- DRA round-trip: `redist import --format dra <CSV>` and `redist export --format dra --plan-label <LABEL>`
- Shapefile from QGIS: `redist import --format shapefile <DIR>` (must have a `district` column)
- Plan-vs-plan comparison: `redist compare --plan-a senate_v3 --plan-b senate_v4 --format both`
- Compare against the currently enacted map: `redist compare --plan-a senate_v3 --baseline ENACTED`

## Format notes

- Districtr tract-level plans round-trip exactly. Block-level plans use a Census-published Block Assignment File (cached on first use); split blocks are recorded in `translation_log.txt`.
- DRA exports vary in column order; the importer auto-detects `GEOID,DISTRICT` and `DISTRICT,GEOID` and headerless variants.
- Shapefile imports require a `district` integer column. Export goes via GeoJSON + `ogr2ogr` (writing valid shapefiles natively requires geometry; we don't always have it).

## Civic groups submitting counter-proposals

If you are a civic advocacy group submitting a counter-proposal rather than state staff drawing the official map, add `--as-civic-counter-proposal --submitted-by "Your Org"` to step 3. The plan is then tagged `submission_type=civic_counter_proposal` in its manifest, and downstream comparison reports surface that framing instead of treating it as authoritative state output.
