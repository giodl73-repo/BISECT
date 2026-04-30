# Plan Comparison & Narrative
**Date:** 2026-04-30
**Updated:** 2026-04-30 (v2 — incorporates 7-role consensus on narrative guardrails + COVENANT versioning)
**Status:** Revised; pending re-review
**Closes gap for:** civic advocacy (★★→★★★★★), state staff, special master
**Depends on:** existing redist compare + redist map; existing analyze JSON outputs
**Estimated effort:** 3-4 days

## Why this exists

Today the project has `redist compare` (computes Jaccard, population diff, compactness diff between two plans) but no:
- Side-by-side visual that a non-technical person can read
- Diff view showing which districts changed
- Narrative paragraphs explaining what the comparison means in plain English ("Plan A would elect 5 Republicans and 1 Democrat; Plan B would elect 4 Republicans and 2 Democrats; the change is concentrated in Districts 3 and 6 around Baton Rouge")
- Historical baseline overlay (current/enacted map vs. proposed)

A civic advocacy group, a journalist, or a member of the public can't pick up the existing `compare` output and use it. This spec adds the storytelling layer.

## Scope

### In scope

1. **Plan-pair comparison report** — Standalone HTML page showing two plans side by side:
   - Two maps (left/right) with a unified legend
   - Per-district table: D1 metrics on Plan A, D1 metrics on Plan B, Δ
   - Aggregate change summary: how many districts flipped party, MM count change, partisan-bias change

2. **Diff visualization** — A third map showing only the precincts/tracts that moved between plans, color-coded by destination district

3. **Narrative paragraph generation (v2 — 7-role consensus on guardrails)** — Templated plain-English summaries with mandatory guardrails:
   - **Threshold parameterization**: `--leaning-threshold <FLOAT>` (default 0.55) controls when a district is described as "Democratic-leaning" or "Republican-leaning". Below the threshold the language is "competitive". The threshold appears in every narrative ("using a 55% Dem-share threshold"). Hardcoded thresholds are forbidden (TRENCH RP-03).
   - **Close-call auto-flag**: any district within ±2 percentage points of the threshold is flagged in the narrative ("District 6 at 55.4% Dem is just above the leaning threshold; classification is sensitive to the chosen cutoff").
   - **Margin-of-error suppression**: when narrative text would change direction within the metric's CI, the auto-text is replaced with "within margin of error; see numerical table." (BOUNDARY recommendation)
   - **`[DRAFT]` prefix**: all auto-generated paragraphs are prefixed `[DRAFT — review before publication]` until a human passes `--approved-by "<name>"` to mark them sign-off-ready. The signed name is committed to the manifest.
   - **Civic-friendly framing first** (COMMONS): the default narrative leads with community-of-interest preservation ("District X keeps the Eastside neighborhoods together; Plan B splits them between two districts"), partisan effects come second.
   - Examples:
     - "In Plan B, District 6 gains [N] tracts from former Districts 3 and 5, increasing the Black voting-age population from [X%] to [Y%]."
     - "[DRAFT — review before publication] Statewide partisan composition (using 55% Dem-share threshold): Plan A elects [A_DEM]/[A_REP]; Plan B elects [B_DEM]/[B_REP]."
     - "Compactness (Polsby-Popper, mean across districts): Plan A=[X], Plan B=[Y]."

4. **Historical-baseline overlay** — `redist compare --baseline ENACTED` compares a proposed plan against the state's currently-enacted map (downloaded via existing fetcher infrastructure)

5. **CLI surface**:
   ```
   redist compare --plan-a LABEL_A --plan-b LABEL_B --format html|narrative|both
   redist compare --plan-a LABEL_A --baseline ENACTED --year 2020
   ```

6. **Civic-facing summary card** — A one-page PNG image suitable for tweeting / press release: title, two thumbnail maps, one paragraph of narrative, key numbers

7. **Narrative manifest versioning (v2 — COVENANT)** — Every generated narrative writes a sidecar `narrative_manifest.json` capturing:
   - Template path + SHA-256
   - Threshold values used (`leaning_threshold`, `close_call_band`, MoE inputs)
   - Source analysis JSON SHAs (partisan.json, vra_analysis.json, compactness.json)
   - `approved_by` (string or `null` when `[DRAFT]`) + `approved_at` ISO-8601
   - `redist` build commit + version
   - Re-running the same template against the same inputs MUST produce a byte-identical narrative; CI asserts this. The manifest is the audit trail a court or newsroom can check before publishing.

8. **Public-comment overlay (v2 — COMMONS)** — `redist compare --comments <CSV>` accepts a CSV of community-of-interest annotations (`geoid, comment_id, label, source`) gathered during a public-comment period. The comparison report:
   - Flags districts where Plan A preserves a labeled community but Plan B splits it (and vice versa)
   - Tallies the percentage of comment-flagged tracts kept whole vs. split per plan
   - Includes a "comments incorporated" appendix listing each `comment_id` and which plan honored it
   - This is the read-only side of the Civic Bidirectional Input spec; the write side (intake / validation) lives there.

### Out of scope

- Interactive web app for plan editing (Districtr territory)
- Real-time polling integration ("how popular is this plan?")
- Public-comment intake (separate civic-tech category)
- Multi-plan (>2) comparison matrix (out of common use case)
- Custom narrative templates per organization (one default template; users edit downstream)

## Outputs

```
outputs/{version}/comparisons/{plan_a}_vs_{plan_b}/
├── comparison.html              # Interactive side-by-side
├── comparison.pdf               # Print-ready (uses Typst from court-reports spec)
├── comparison_card.png          # Social-media-ready 1200x675
├── narrative.md                 # Plain-English summary
├── diff.png                     # Tract-level diff visualization
└── comparison.json              # Machine-readable totals
```

## Narrative template

The narrative paragraphs use a small template engine (Jinja2 or similar) with these variables available:

| Variable | Type | Source |
|---|---|---|
| `plan_a.label`, `plan_b.label` | str | manifest |
| `plan_a.partisan.dem_seats`, etc. | int | analysis/partisan.json |
| `plan_a.vra.mm_count`, etc. | int | analysis/vra_analysis.json |
| `plan_a.compactness.pp_mean` | float | analysis/compactness.json |
| `diff.tracts_changed` | int | computed |
| `diff.population_changed` | int | computed |
| `diff.districts_with_changes` | list[int] | computed |

Template lives at `redist-report/templates/comparison-narrative.j2.md` (or Typst equivalent). Customizable but ships with a default that reads like a press release.

## Civic-facing summary card

A 1200×675 PNG that fits Twitter/Facebook share previews:
- Title bar: "Plan A vs Plan B"
- Two thumbnail maps side by side
- One-sentence headline ("Plan B yields 1 more Democratic-leaning district while reducing the average compactness by 4%")
- Key stats row: "Dem seats: 5→6", "MM districts: 1→1", "PP: 0.42→0.40"
- Footer: Generated by redist · {date} · {git commit short}

Built via redist-map (existing infrastructure) + a small layout-on-PNG helper.

## Tests

- L0: narrative template renders with synthetic input; assert no leftover `{{var}}` literals
- L0: diff computation (tracts that changed assignment between plans) on synthetic 100-tract case
- L0 **value-correctness (BENCHMARK)**: render the narrative against a synthetic input where the truth is hand-computed (Plan A=5D/1R, Plan B=4D/2R, threshold=0.55, two close-call districts at 0.553 and 0.547). Assert the rendered text contains the exact strings "5 Democratic-leaning", "4 Democratic-leaning", "District X is just above the leaning threshold". Catches off-by-one threshold errors and template-variable mis-binding.
- L0 **byte-identical re-render**: render twice with the same inputs + template SHA; assert outputs are bit-identical. Guards the manifest's reproducibility claim.
- L0 **MoE suppression**: feed a comparison where Plan A=5.0 ± 0.6 dem seats, Plan B=5.4 ± 0.5; assert the narrative substitutes the "within margin of error" text for any directional claim.
- L0 **`[DRAFT]` gate**: omit `--approved-by`; assert every paragraph starts with `[DRAFT — review before publication]` and `narrative_manifest.json` shows `approved_by: null`.
- L1: full comparison on two synthetic plans; verify all output artifacts created (including `narrative_manifest.json`)
- L1: `--comments <CSV>` overlay on synthetic plans; assert the "comments incorporated" appendix lists exactly the expected comment IDs
- L2: skipped-by-default acceptance against a real two-plan comparison

## Risks

| Risk | Mitigation |
|---|---|
| Narrative reads as "robotic press-release" and gets dismissed | Run early templates by a journalist for tone; iterate |
| Auto-generated text could mislead on close cases | Always include the underlying numbers; flag close calls (e.g., "within margin of error") |
| Civic-facing card is one image, can be cherry-picked | Add a citation footer; document that the full report is the audit reference |
| Different states / chambers have different "what matters" lists | One default template; allow `--narrative-template <PATH>` override |

## Definition of done

- `redist compare --plan-a X --plan-b Y --format html` produces all output artifacts (including `narrative_manifest.json`)
- HTML side-by-side renders cleanly in Chrome/Firefox/Safari
- Narrative reads as professional plain-English (vetted by at least one non-technical reader)
- Every auto-generated paragraph starts with `[DRAFT — review before publication]` until `--approved-by` is supplied
- Re-running the same `compare` command on the same inputs produces a byte-identical narrative (CI gate)
- `--comments <CSV>` overlay produces a "comments incorporated" appendix
- Comparison card looks good at Twitter/Facebook preview sizes
- Tutorial in docs/REDIST_CLI.md shows one full comparison
