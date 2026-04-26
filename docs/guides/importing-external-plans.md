# Importing External Plans

How to import redistricting plans from other tools into `redist` for analysis, comparison, and reporting.

---

## Why import external plans?

- **Compare alternatives**: A commission staff draft vs. a citizen-submitted plan vs. the enacted map
- **Independent audit**: Analyze a plan submitted by another party using your own tools
- **Cross-tool verification**: Verify that a plan produced in Maptitude or DRA meets legal constraints
- **Historical analysis**: Analyze the 2010 enacted map using 2020 population data

---

## Supported import formats

| Format | Source | Command |
|--------|--------|---------|
| RPLAN (`.rplan`) | Any RPLAN-compliant tool | `redist import --file plan.rplan` |
| GeoJSON (`.geojson`) | DRA, PlanScore, QGIS, etc. | `redist import --file plan.geojson` |
| Shapefile (`.shp`) | Maptitude, ArcGIS, Census | `redist import --file plan.shp` |
| CSV (GEOID,district) | Any tool | `redist import --file assignments.csv` |

---

## Importing from Dave's Redistricting App (DRA)

1. In DRA, open your plan and click **Export** → **GeoJSON**
2. Save the file as `dra_plan.geojson`
3. Import and analyze:

```bash
redist import \
  --file dra_plan.geojson \
  --state WA --year 2020 \
  --label wa_house_dra_v1 \
  --version WA_Plans

# Now analyze it like any other plan
redist analyze --label wa_house_dra_v1 --year 2020 --version WA_Plans --types all
redist compare --plan-a wa_house_dra_v1 --plan-b wa_house_draft1 --year 2020 --version WA_Plans
```

---

## Importing from PlanScore

PlanScore exports GeoJSON via its web interface. Download the plan GeoJSON and import:

```bash
redist import \
  --file planscore_wa_house.geojson \
  --state WA --year 2020 \
  --label wa_house_planscore \
  --version WA_Plans
```

---

## Importing from Maptitude / ArcGIS

Export as a shapefile with a district ID attribute. The shapefile should contain district polygons (not tracts). `redist` will assign each census tract to the district polygon containing its centroid.

```bash
redist import \
  --file maptitude_export.shp \
  --state WA --year 2020 \
  --label wa_house_maptitude \
  --version WA_Plans \
  --district-id-field DISTRICT  # attribute name in the shapefile
```

---

## Importing from GerryChain

GerryChain's partition object exports JSON with an `assignment` dict:

```python
# In Python, after running GerryChain:
import json
assignment = partition.assignment
json.dump({"assignment": dict(assignment)}, open("gerrychain_plan.json", "w"))
```

Convert to RPLAN:

```python
# gerrychain_to_rplan.py
import json
gc = json.load(open("gerrychain_plan.json"))
rplan = {
    "rplan_version": "0.1",
    "metadata": {
        "label": "wa_house_gerrychain",
        "state_fips": "53",
        "state_code": "WA",
        "year": "2020",
        "chamber": "house",
        "num_districts": 98,
        "population_source": "total",
        "balance_tolerance_pct": 5.0,
        "created_at": "2026-04-26T00:00:00Z",
        "created_by": "gerrychain v2.3"
    },
    "assignments": gc["assignment"],  # GerryChain uses 'assignment' (singular)
    "geometry": None
}
json.dump(rplan, open("gerrychain_plan.rplan", "w"))
```

Then import normally:
```bash
redist import --file gerrychain_plan.rplan --label wa_house_gc --state WA --year 2020 --version WA_Plans
```

---

## Importing a CSV

If you have a simple CSV of tract GEOIDs to district numbers:

```csv
geoid,district
530330001001,7
530330001002,7
530330002001,12
```

```bash
redist import \
  --file assignments.csv \
  --state WA --year 2020 \
  --label wa_house_csv \
  --version WA_Plans \
  --format csv
```

---

## What happens during import

1. **GeoJSON/Shapefile**: `redist` finds each census tract's centroid and determines which district polygon contains it (point-in-polygon). Tracts on district boundaries use the nearest polygon as a fallback.
2. **RPLAN**: Assignments are read directly — no spatial computation needed.
3. **CSV**: Assignments are read directly.

After import, a `manifest.json` is written noting `"source": "imported"` and the source filename. The plan is then treated identically to a generated plan for all analysis purposes.

---

## Validating imported plans

```bash
# Validate an RPLAN file before importing
redist validate --file external_plan.rplan

# After import, run contiguity check
redist analyze --label wa_house_dra_v1 --types contiguity

# Check population balance
redist analyze --label wa_house_dra_v1 --types summary
```

---

## Limitations

- **Centroid assignment** for GeoJSON/shapefile import may misassign tracts whose centroid falls in a different district than their majority area. This is noted in the import metadata as `"pip_method_note"` and `"fallback_count"`. Check the fallback count — a high number suggests many tracts on district boundaries.
- **External plans break the reproducibility guarantee**. The `manifest.json` for imported plans marks `"source": "imported"` and does not have a `verification_command`. The plan is verifiable by re-importing the same source file, but not by running a `redist state` command.
- **RPLAN format only** guarantees independent verification. If you need a fully court-verifiable plan, generate it with `redist state` rather than importing.
