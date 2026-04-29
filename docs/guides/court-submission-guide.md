# Court Submission Guide

How to prepare a `redist`-generated plan for submission to a court or redistricting commission, including the audit trail and expert witness package.

---

## What courts require

A redistricting plan submitted as evidence needs to be:

1. **Reproducible** — any party can generate the same map from the same inputs
2. **Independently verifiable** — the source data comes from a public source (Census.gov), not a private database
3. **Documented** — the methodology is explained and the parameters are recorded
4. **Legally compliant** — population balance, contiguity, and applicable state constitutional requirements are documented

`redist` produces all of this automatically.

---

## The audit trail

Every plan has a `manifest.json` in its plan directory. This is the chain of custody document.

```json
{
  "label": "wa_house_draft1",
  "state_code": "WA",
  "year": "2020",
  "chamber": "house",
  "num_districts": 98,
  "population_source": "total",
  "balance_tolerance_pct": 5.0,
  "seed": 42,
  "created_at": "2026-04-26T14:23:00Z",
  "created_by": "redist v0.1.0",
  "binary_download_url": "https://github.com/...",
  "binary_release_sha256": "abc123...",
  "tiger_source_url": "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_53_tract.zip",
  "tiger_sha256": "def456...",
  "adjacency_file": "wa_adjacency_2020.adj.bin",
  "adjacency_sha256": "ghi789...",
  "population_balance_valid": true,
  "verification_command": "redist state --state WA --year 2020 ..."
}
```

### What each field proves

| Field | What it establishes |
|-------|---------------------|
| `tiger_source_url` | The geographic data came from Census.gov, not a private source |
| `tiger_sha256` | The exact Census shapefile used — any auditor can download and verify |
| `seed` | The run is deterministic — same seed produces the same map |
| `verification_command` | Exact command to reproduce the plan |
| `binary_release_sha256` | The software used can be independently obtained and verified |
| `population_balance_valid` | Constitutional population equality requirement is met |

---

## Step-by-step: prepare a court submission package

### 1. Generate the plan with a fixed seed

```bash
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house \
  --label wa_house_submission \
  --balance-tolerance 5.0 \
  --seed 7823461          # Record this in your commission minutes
```

### 2. Run full analysis

```bash
redist analyze --label wa_house_submission --year 2020 --version WA_Plans \
  --types all
```

This produces all required analysis files: population balance, contiguity verification, county splits, compactness, partisan fairness.

### 3. Generate the formal report

```bash
redist report --label wa_house_submission --year 2020 --version WA_Plans \
  --format html json \
  --out submission/wa_house_submission_report/
```

The HTML file is a self-contained single-page document — no internet connection required to view it.

### 4. Export for the record

```bash
redist export --label wa_house_submission --year 2020 --version WA_Plans \
  --format geojson shapefile rplan \
  --out submission/exports/
```

Submit `wa_house_submission.rplan` as the official machine-readable plan file. This format is independently verifiable by any party with `redist validate`.

### 5. Verify the audit trail is complete

```bash
# Validate the RPLAN file
redist validate --file submission/exports/wa_house_submission.rplan

# Expected output:
# PASS: valid RPLAN v0.1 (2489 tracts, 98 districts, WA 2020 house)
# Population balance: valid (max deviation 2.1%, tolerance 5.0%)
# SHA-256: verified
```

---

## What to submit

| Document | Format | Purpose |
|----------|--------|---------|
| Commission report | HTML (self-contained) | Public-facing summary |
| Commission report | JSON | Machine-readable data for technical review |
| Plan file | `.rplan` | Official plan for independent verification |
| District map | Shapefile | For GIS tools and court exhibits |
| `manifest.json` | JSON | Chain of custody document |

---

## Expert witness statement template

The manifest's `verification_command` is designed to be cited in an expert witness declaration:

> "The plan was generated using `redist` version 0.1.0 (SHA-256: {binary_sha256}), available at {binary_download_url}. The source geographic data is the 2020 Census TIGER/Line tract shapefile for Washington state (SHA-256: {tiger_sha256}), downloaded from {tiger_source_url}. The plan can be reproduced exactly by any party using the command: `{verification_command}`. I have verified that running this command produces a plan with maximum population deviation of 2.1% from the ideal of 761,169 persons per district, within the ±5% constitutional tolerance for state legislative districts."

---

## For opposing counsel: independent verification

Any party can independently verify the plan:

```bash
# 1. Download the binary at the stated version
curl -L {binary_download_url} -o redist_verify.exe
sha256sum redist_verify.exe  # must match binary_release_sha256

# 2. Verify the source data
curl -L {tiger_source_url} -o tiger_wa_2020.zip
sha256sum tiger_wa_2020.zip  # must match tiger_sha256

# 3. Reproduce the plan
./redist_verify.exe state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house \
  --label wa_house_independent_verify \
  --balance-tolerance 5.0 \
  --seed 7823461

# 4. Compare to original
diff <(jq -S . outputs/WA_Plans/2020/plans/wa_house_submission/data/final_assignments.json) \
     <(jq -S . outputs/WA_Plans/2020/plans/wa_house_independent_verify/data/final_assignments.json)
# Expected: no differences
```

---

## Limitations to disclose

Every report includes a methodology section with these disclosures. Be aware of them:

1. **Compactness metrics use WGS84 coordinates** (display projection, not equal-area). Polsby-Popper scores for east-west-elongated states like Montana are systematically compressed. The `crs_note` field in `compactness.json` documents this.

2. **Partisan analysis uses presidential election data** as a proxy for state legislative partisanship. Presidential coattail effects may overstate Democratic margins in urban districts. A `methodology_warning` field documents this when presidential data is used for non-congressional chambers.

3. **Majority-minority analysis uses total population**, not voting-age population (VAP). Thornburg v. Gingles uses VAP. The `pop_basis: "total_population"` field documents this. Consult counsel on whether VAP-based analysis is required in your jurisdiction.
