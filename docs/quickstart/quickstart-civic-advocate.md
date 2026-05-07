# Quickstart: Civic Advocacy Group

**Who you are:** A civic advocacy group, neighborhood association, or nonpartisan watchdog producing materials for the public-comment record or for press release. Your goal: explain to non-experts how a proposed map affects communities of interest, partisan composition, and minority representation.

**What you'll have at the end:** A side-by-side comparison of two plans (state's proposal vs. yours, or current vs. proposed) with civic-friendly narrative paragraphs, a 1200×675 social-media-ready PNG, and a manifest-pinned audit trail you can cite.

**Time:** 15–30 minutes for first run; 5 minutes per iteration.

---

## Steps

1. **Bootstrap** (one time):
   ```bash
   bash bootstrap.sh
   ```

2. **Get the state's proposed plan into a format `redist` can read.** This is the friction point — the state may publish only PDFs or shapefiles. Choose the path that matches what they published:
   - **Districtr-published** (newer states, e.g., MA, MO):
     ```bash
     redist import --format districtr state_plan.json --plan-label state_proposal --state XX --year 2020
     ```
   - **Shapefile** (most states publish .zip with .shp/.shx/.dbf):
     ```bash
     redist import --format shapefile state_plan.shp --plan-label state_proposal --state XX --year 2020
     ```
     The shapefile MUST contain a `district` column. If it does not, you may need to re-attribute it in QGIS first.
   - **GeoJSON** (DRA exports, Districtr alternates):
     ```bash
     redist import --format geojson state_plan.geojson --plan-label state_proposal --state XX --year 2020
     ```
   - **CSV** (DRA's most common export):
     ```bash
     redist import --format dra state_plan.csv --plan-label state_proposal --state XX --year 2020
     ```
   - **State publishes only a PDF and won't release machine-readable form:** open a public-records request. Template language: *"Pursuant to [state public records law cite], I request the GeoJSON, shapefile, or CSV form of the proposed redistricting plan referenced as [plan name]. PDF maps alone are insufficient for analysis."* This is a real friction point we cannot solve in software.

3. **Draw your alternative** in Districtr (web, free) or Dave's Redistricting App. Save as JSON. Import as a *civic counter-proposal* (the tag is loud in every downstream artifact):
   ```bash
   redist import --format districtr lwv_alt_plan.json \
       --as-civic-counter-proposal \
       --submitted-by "League of Women Voters of Vermont" \
       --plan-label lwvvt_alt --state VT --year 2020
   ```

4. **(Optional) Ingest community-of-interest comments** gathered during the comment period (Civic Bidirectional plan, when shipped):
   ```bash
   redist civic ingest community_comments.csv \
       --label lwvvt_comments --year 2020 --state VT \
       --submitter "Lake Champlain Neighborhood Council"
   ```
   Sheets template + plain-English how-to: `docs/civic/HOWTO.md`.

5. **Compare the two plans** with civic-friendly narrative + summary card:
   ```bash
   redist compare --plan-a state_proposal --plan-b lwvvt_alt \
       --comments-label lwvvt_comments \
       --format both --approved-by "Your Name"
   ```
   Without `--approved-by`, every paragraph is prefixed `[DRAFT — review before publication]`. The signed name is committed to `narrative_manifest.json` for accountability.

6. **Publish.** The artifacts are under `outputs/v1/comparisons/state_proposal_vs_lwvvt_alt/`:
   - `comparison.html` — full side-by-side for your website
   - `comparison_card.png` — 1200×675 PNG for Twitter/Facebook share preview (with watermark when either plan is a civic counter-proposal)
   - `narrative.md` — paragraphs you can paste into a press release
   - `narrative_manifest.json` — audit trail (what you compared, what threshold you used, your signed name)

---

## Expected output at each step

- **Step 5:** all six artifacts in the comparison directory; HTML opens cleanly in any browser
- **Civic-counter-proposal plans** carry a visible diagonal watermark on the summary card so a screenshot still attributes the proposal correctly

## Where to go next

- Plain-English walkthrough of the comparison output: `docs/REDIST_CLI.md` `redist compare` section
- Sheets template for COI comments: `docs/civic/templates/community-of-interest.xlsx` (when shipped)
- For the press release angle: the first paragraph of `narrative.md` is designed to read aloud verbatim
- If your state's official plan analysis includes civic data submitted under non-strict validation, court-mode reports will refuse to embed it without an explicit `--allow-non-strict-civic` flag — that's a feature, not a bug

## Optics

- The narrative leads with community-of-interest preservation ("District 3 keeps the Eastside neighborhoods together"), partisan effects come second. This framing is intentional and codified in the spec.
- Threshold values (what counts as "Democratic-leaning") are disclosed in every narrative ("using a 55% Dem-share threshold"). Do not strip these — opposing counsel will quote them otherwise.
- Close-call districts (within ±2 percentage points of the threshold) are auto-flagged. Do not strip.
- When the metric difference is within margin of error, the narrative substitutes "within margin of error; see numerical table" rather than asserting a directional claim. Trust the auto-suppression.
