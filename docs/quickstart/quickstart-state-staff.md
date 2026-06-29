# Quickstart: State Legislative Staff (Map-Drawing Workflow)

**Who you are:** Staff for a state legislature, redistricting commission, or governor's office. You draw maps interactively (Districtr / Dave's Redistricting App / QGIS) and need fast analysis + court-quality reports as you iterate.

**What you'll have at the end:** A Districtr -> `BISECT` -> PDF round-trip that you can run without re-learning anything between map iterations.

**Time:** 5 minutes per iteration after first setup.

---

## Steps

1. **Bootstrap** (one time):
   ```bash
   bash bootstrap.sh           # Linux/macOS
   bootstrap.bat               # Windows
   ```

2. **Draw a map** in Districtr against your state's 2020 tracts. Save as JSON via Districtr's "Save Plan" -> "Download as JSON".

3. **Import into `bisect`:**
   ```bash
   bisect label-import senate_proposal_v3 --from senate_proposal_v3.rplan --year 2020 --format rplan
   ```
   Expected: `runs/senate_proposal_v3/2020/{state}/assignments.json` plus label index and registry evidence. For CSV, GeoJSON, or shapefile exports, use `--format csv`, `--format geojson`, or `--format shapefile`.

4. **Run analysis:**
   ```bash
   bisect label-analyze senate_proposal_v3 --year 2020 --types all
   ```
   Wall-clock: under 30 s for VT, under 5 min for the largest states.

5. **Generate a report:**
   ```bash
   bisect label-report senate_proposal_v3 --year 2020 --format html json
   ```
   Treat generated reports as evidence packages; legal/court filing status depends on your review and any downstream PDF packaging.

6. **Iterate.** Edit in Districtr, bump the label suffix (`senate_proposal_v4`), repeat steps 3-5.

---

## State legislative redistricting (F series)

Congressional maps use census tracts as the unit of geography. For **state legislative redistricting** — house seats numbered in the dozens or hundreds — use block group resolution to get enough units for clean boundary control.

```bash
# Washington State House: 98 districts at block-group resolution
bisect state --state WA --year 2020 \
    --districts 98 \
    --chamber house \
    --resolution block_group \
    --label wa_house_v1
```

Resolution guidance:

| Condition | Recommended resolution |
|-----------|------------------------|
| k / n_tracts <= 0.05 | `tract` (default) |
| k / n_tracts > 0.05  | `block_group` |

For Washington: k=98, n_tracts=1458, ratio=0.067 -- use `--resolution block_group`. For California: k=80 assembly seats, n_tracts=8057, ratio=0.010 -- tracts are fine.

The `--chamber` flag tags the output manifest so reports and comparisons can distinguish house from senate runs in the same version.

**Senate districts** (fewer seats, looser resolution requirement):
```bash
bisect state --state WA --year 2020 \
    --districts 49 \
    --chamber senate \
    --label wa_senate_v1
```

---

## Bicameral nesting with NestSection

Washington's constitution requires each senate district to be composed of exactly two whole house districts. The `--structure nestsection` flag draws both chambers in a single pipeline run, guaranteeing nesting by construction.

```bash
bisect state --state WA --year 2020 \
    --structure nestsection \
    --label wa_bicameral_v1
```

What happens:
1. `bisect` draws 98 house districts using METIS on census tracts (block-group resolution auto-applied)
2. It builds a new adjacency graph where nodes are house districts
3. METIS runs on the house-district graph with k=49
4. Each senate district is defined as exactly 2 whole house districts -- nesting is guaranteed by construction
5. `bisect` validates the nesting before writing output

Expected output:
```
outputs/v1/2020/plans/wa_bicameral_v1/
  house/final_assignments.json
  senate/final_assignments.json
  suite.json                     <- senate-to-house nesting map
  manifest.json
```

Validate after drawing:
```bash
bisect suite validate --name wa_bicameral_v1 --year 2020 --version v1
```

Output confirms nesting or lists violations:
```json
{
  "nesting": {
    "mode": "senate-in-house",
    "valid": true,
    "violations": []
  }
}
```

For variable-ratio states (Illinois, New York, Minnesota), add `--nest-ratio 2:1`:
```bash
bisect state --state IL --year 2020 \
    --structure nestsection \
    --nest-ratio 2:1 \
    --label il_bicameral_v1
```

See `docs/guides/nesting-guide.md` for per-state constitutional nesting requirements.

---

## Expected output at each step

- **Step 3:** plan label visible via `bisect show senate_proposal_v3`
- **Step 4:** five JSON files under `analysis/`, exit 0
- **Step 5:** `report.html` viewable in any browser; no external network calls
- **State legislative run:** `final_assignments.json` with district IDs 1..k, plus `manifest.json` tagging `chamber` and `resolution`
- **NestSection:** `suite.json` with `senate_to_house_map` in addition to per-chamber assignments

## Where to go next

- CSV round-trip: `bisect label-import <LABEL> --from <CSV> --format csv`
- Shapefile from QGIS: `bisect label-import <LABEL> --from <SHP> --format shapefile` (must have GEOID and district DBF fields)
- Plan-vs-plan comparison: `bisect label-compare senate_v3 senate_v4 --year 2020`
- Compare against the currently enacted map by importing it as a label, then use `bisect label-compare`
- Nesting guide: `docs/guides/nesting-guide.md`

## Format notes

- RPLAN tract-level plans round-trip through `rplan-io`.
- CSV exports vary in column order; the importer auto-detects `GEOID,DISTRICT` and `DISTRICT,GEOID` and headerless variants.
- Shapefile imports read DBF attributes from `.shp` sidecars and require a GEOID field plus a district integer field.

## Civic groups submitting counter-proposals

If you are a civic advocacy group submitting a counter-proposal rather than state staff drawing the official map, add `--as-civic-counter-proposal --submitted-by "Your Org"` to step 3. The plan is then tagged `submission_type=civic_counter_proposal` in its manifest, and downstream comparison reports surface that framing instead of treating it as authoritative state output.
