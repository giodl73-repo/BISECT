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

2. **Get the state's proposed plan into a format `BISECT` can read.** This is the friction point — the state may publish only PDFs or shapefiles. Choose the path that matches what they published:
   - **RPLAN** (preferred interchange format):
     ```bash
     bisect label-import state_proposal --from state_plan.rplan --year 2020 --format rplan
     ```
   - **Shapefile** (most states publish .zip with .shp/.shx/.dbf):
     ```bash
     bisect label-import state_proposal --from state_plan.shp --year 2020 --format shapefile
     ```
     The shapefile must have DBF attributes with a GEOID field and a district field such as `DISTRICT`, `DISTRICTID`, `DIST_ID`, or `CD`.
   - **GeoJSON** (DRA exports, Districtr alternates):
     ```bash
     bisect label-import state_proposal --from state_plan.geojson --year 2020 --format geojson
     ```
   - **CSV** (GEOID,district assignments):
     ```bash
     bisect label-import state_proposal --from state_plan.csv --year 2020 --format csv
     ```
   - **State publishes only a PDF and won't release machine-readable form:** open a public-records request. Template language: *"Pursuant to [state public records law cite], I request the GeoJSON, shapefile, or CSV form of the proposed redistricting plan referenced as [plan name]. PDF maps alone are insufficient for analysis."* This is a real friction point we cannot solve in software.

3. **Draw your alternative** in Districtr (web, free) or Dave's Redistricting App. Save as JSON. Import as a *civic counter-proposal* (the tag is loud in every downstream artifact):
   ```bash
   bisect label-import lwvvt_alt --from lwv_alt_plan.rplan --year 2020 --format rplan
   ```

4. **(Optional) Ingest community-of-interest comments** gathered during the comment period (Civic Bidirectional plan, when shipped):
   ```bash
   bisect civic ingest community_comments.csv \
       --label lwvvt_comments --year 2020 --state VT \
       --submitter "Lake Champlain Neighborhood Council"
   ```
   Sheets template + plain-English how-to: `docs/civic/HOWTO.md`.

5. **Compare the two plans** with civic-friendly narrative + summary card:
   ```bash
   bisect label-compare state_proposal lwvvt_alt --year 2020
   bisect label-report lwvvt_alt --year 2020 --format html json
   ```
   Review any narrative before publication. The generated evidence package is audit material, not an official certification.

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

- Plain-English walkthrough of the comparison output: `docs/BISECT_CLI.md` `bisect compare` section
- Sheets template for COI comments: `docs/civic/templates/community-of-interest.xlsx` (when shipped)
- For the press release angle: the first paragraph of `narrative.md` is designed to read aloud verbatim
- If your state's official plan analysis includes civic data submitted under non-strict validation, court-mode reports will refuse to embed it without an explicit `--allow-non-strict-civic` flag — that's a feature, not a bug

## Optics

- The narrative leads with community-of-interest preservation ("District 3 keeps the Eastside neighborhoods together"), partisan effects come second. This framing is intentional and codified in the spec.
- Threshold values (what counts as "Democratic-leaning") are disclosed in every narrative ("using a 55% Dem-share threshold"). Do not strip these — opposing counsel will quote them otherwise.
- Close-call districts (within ±2 percentage points of the threshold) are auto-flagged. Do not strip.
- When the metric difference is within margin of error, the narrative substitutes "within margin of error; see numerical table" rather than asserting a directional claim. Trust the auto-suppression.
