# Plan: Plan Comparison & Narrative

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-30-plan-comparison-and-narrative.md` v2 (with v2.1 patches) has been reviewed and approved. Steps use checkbox (`- [ ]`) syntax for tracking.

**Date:** 2026-04-30
**Spec:** `docs/superpowers/specs/2026-04-30-plan-comparison-and-narrative.md`
**v2.1 tracking ref:** `docs/superpowers/specs/2026-04-30-v21-tracking.md`
**Goal:** Extend `redist compare` from a numerical Jaccard/population/compactness diff into a publication-ready storytelling layer: side-by-side HTML, a tract-level diff map, civic-friendly narrative paragraphs guarded by a `[DRAFT]` gate and `--approved-by` sign-off, a watermarked summary-card PNG, a `--comments-label` overlay, and a fully provenanced `narrative_manifest.json` that supports byte-identical re-render.

**Depends on:** existing `redist compare` (analysis crate), `redist-map` SVG→PNG via resvg, `redist-report` Tera infrastructure, Civic Bidirectional schema (consumer side only — schema is owned by `2026-04-30-civic-bidirectional.md` §1).
**Blocks:** civic advocacy quickstart end-state (★★→★★★★★); special-master and state-staff narrative consumption.

**v2.1 items addressed by this plan:**
- M-04 — path-portability rule (relative-to-package-root) for `narrative_manifest.json`
- S-04 — formal definition of "direction change" for non-monotone metrics (MM count) in MoE suppression
- PP-31 — narrative manifest path with spaces / non-ASCII; relative paths + CI test on path-with-spaces fixture
- BD-N3 — civic-counter-proposal watermark on the summary-card PNG, not just narrative text
- M-01 — apply `--plan-a` / `--plan-b` / `--comments-label` rename consistently across the comparison surface

---

## Pre-Conditions

- `redist compare --plan-a A --plan-b B` already produces `PlanComparison` (Jaccard/population/compactness/splits) via `redist_analysis::compare_plans`
- `redist-report` already vendors `tera = "1"`; this plan reuses Tera (no new template-engine dependency)
- `redist-map` exposes `build_svg`, `svg_to_png`, `default_font_db`, `canvas_size_from_dpi`
- Civic Bidirectional spec defines the canonical CSV schema `geoid,comment_id,label,source,source_url,confidence,submitted_at` (this plan consumes only)
- `SOURCE_DATE_EPOCH` and `REDIST_BUILD_COMMIT_SHORT` envs are honored project-wide (BENCHMARK P1 patch in spec)
- `PlanContext::from_label` resolves manifest SHAs for both plans (used for COVENANT C-3 pinning)

---

## Task 1: Extend `CompareArgs` for HTML/narrative surface

**Files:** `redist/crates/redist-cli/src/args.rs`

Today's `CompareArgs` only supports table/json/csv. Extend without breaking the existing surface.

- [ ] **1.1** Extend `CompareFormat` enum with `Html`, `Narrative`, `Both`. Keep `Table` as default.
- [ ] **1.2** Add fields to `CompareArgs`:
  - `pub baseline: Option<String>` (`--baseline ENACTED|<label>`); mutually exclusive with `--plan-b` and `--enacted` at runtime.
  - `pub leaning_threshold: f64` (`--leaning-threshold`, default `0.55`). TRENCH RP-03: forbid hardcoded thresholds anywhere downstream — every read goes through this field.
  - `pub close_call_band: f64` (`--close-call-band`, default `0.02`).
  - `pub approved_by: Option<String>` (`--approved-by "<name>"`).
  - `pub comments_label: Option<String>` (`--comments-label LWV_COMMENTS_2026`). MERIDIAN M-01: name is `--comments-label`, NOT `--label` or `--comments`.
  - `pub narrative_template: Option<PathBuf>` (`--narrative-template <PATH>`).
  - `pub report_dir: Option<PathBuf>` (`--report-dir`); when omitted, default to `outputs/{version}/comparisons/{plan_a}_vs_{plan_b}/`.
- [ ] **1.3** Validation: in `run_compare`, reject `--plan-b` + `--baseline` simultaneously; reject `--approved-by ""` (empty string).
- [ ] **1.4** Unit tests in `args.rs` for every new flag: presence, defaults, mutual exclusion, value parsing.

**Exit:** `cargo test -p redist-cli args::tests` green; `redist compare --help` shows the new surface.

---

## Task 2: Comparison report assembler crate module

**Files:** `redist/crates/redist-report/src/comparison.rs` (new), `redist/crates/redist-report/src/lib.rs`

Mirror the existing `report.rs` / `html.rs` split. Keep narrative generation in `redist-report` (the one crate that already depends on Tera) so `redist-cli` stays thin.

- [ ] **2.1** Define `ComparisonReport` struct: `plan_a: PlanSide`, `plan_b: PlanSide`, `baseline: Option<PlanSide>`, `diff: DiffSummary`, `comments: Option<CommentsOverlay>`, `narrative: NarrativeBlock`, `manifest: NarrativeManifest`, `generated_at`, `redist_build_commit_short`. Each `PlanSide` has `label`, `manifest_sha256`, `partisan`, `vra`, `compactness`, `population`, plus `submission_type: Option<String>` (passthrough from plan manifest for civic-counter-proposal detection).
- [ ] **2.2** `assemble_comparison(ctx: &ComparisonContext) -> anyhow::Result<ComparisonReport>` — pulls analysis JSONs (`partisan.json`, `vra_analysis.json`, `compactness.json`, `summary.json`) for both plans using the same `load_analysis_json` helper that `compare.rs` already uses. Compute manifest SHA-256 by hashing the bytes of each plan's `manifest.json` (COVENANT C-3 pinning).
- [ ] **2.3** `DiffSummary` computes `tracts_changed`, `population_changed`, `districts_with_changes`, `tract_destinations: HashMap<String, (usize, usize)>` (GEOID → (district_in_a, district_in_b)). Population sourced from per-tract population in the existing adjacency/tract data path; reuse `redist_data` accessors.
- [ ] **2.4** Re-export from `redist-report/src/lib.rs`.

**Exit:** `cargo build -p redist-report` clean; module compiles; no narrative content yet.

---

## Task 3: Narrative template engine + civic-friendly framing

**Files:**
- `redist/crates/redist-report/templates/comparison-narrative.j2.md` (new)
- `redist/crates/redist-report/templates/comparison-side-by-side.j2.html` (new)
- `redist/crates/redist-report/src/narrative.rs` (new)

Use **Tera** (already a workspace dep — see `redist-report/Cargo.toml`). Do NOT pull in `minijinja` or any Python.

- [ ] **3.1** `comparison-narrative.j2.md` lays out paragraphs in this fixed order (COMMONS — civic-friendly framing first):
  1. **Civic-counter-proposal framing** (rendered only when `plan_a.submission_type == "civic_counter_proposal"` or same for B): one-sentence label "Plan B is a civic counter-proposal submitted by {{ plan_b.submitted_by }} on {{ plan_b.submitted_at }}; it is not the state's official map."
  2. **Community-of-interest paragraph** (COMMONS): community-preservation effects come BEFORE partisan effects.
  3. **Diff scope**: tracts moved, population moved, districts touched.
  4. **Partisan composition**: Dem/Rep seats per plan, threshold disclosure ("using {{ leaning_threshold * 100 }}% Dem-share threshold"), close-call flagging for any district within `close_call_band` of the threshold.
  5. **MM-count and demographic shifts**: per-district BVAP/HVAP movement.
  6. **Compactness**: Polsby-Popper means.
- [ ] **3.2** Every paragraph rendered with the `[DRAFT — review before publication]` prefix gated by a top-of-template `{% if not approved_by %}[DRAFT — review before publication] {% endif %}` macro applied per paragraph (Task 5 covers the gate test).
- [ ] **3.3** `narrative.rs::render_narrative(report: &ComparisonReport, template_path: Option<&Path>) -> anyhow::Result<String>`. When `template_path` is `Some`, load it from disk; otherwise use the bundled `include_str!` template.
- [ ] **3.4** Variable-binding contract documented in a doc-comment table mirroring spec §"Narrative template": `plan_a.label`, `plan_a.partisan.dem_seats`, `plan_a.vra.mm_count`, `plan_a.compactness.pp_mean`, `diff.tracts_changed`, `diff.population_changed`, `diff.districts_with_changes`, `leaning_threshold`, `close_call_band`, `approved_by`, plus `plan_a.submission_type`, `plan_a.submitted_by`, `plan_a.submitted_at` (all `Option<String>` rendered as Tera-safe).
- [ ] **3.5** L0 unit test: render template against synthetic input; grep for any leftover `{{` / `}}` literals → fail.

**Exit:** Template renders for a synthetic `ComparisonReport`; no leftover Jinja artifacts.

---

## Task 4: Margin-of-error suppression (S-04 — direction change for non-monotone metrics)

**Files:** `redist/crates/redist-report/src/narrative.rs`, `redist/crates/redist-report/src/moe.rs` (new)

Spec §"Margin-of-error suppression" requires that when a directional claim flips within the metric's CI, the auto-text is replaced with "within margin of error; see numerical table." S-04 demands a formal "direction change" definition for non-monotone metrics like MM count.

- [ ] **4.1** Define `enum MetricMonotonicity { Monotone, NonMonotone }` and `struct CiBand { point: f64, low: f64, high: f64 }`.
- [ ] **4.2** For **monotone** metrics (Dem seats, mean PP, population deviation): direction-change condition is `(plan_a.point - plan_b.point).signum() != (plan_a_low_vs_b_high).signum()` — i.e., the sign of the comparison flips inside the CI overlap. Use `(a.low - b.high)` and `(a.high - b.low)` as the bracketing pair.
- [ ] **4.3** For **non-monotone** metrics (MM count is a step function over a continuous BVAP/HVAP variable): direction-change is defined as **the CI of the BVAP/HVAP input crossing any 50%-threshold edge that would alter the integer MM count**. Concretely: for each district, if `bvap_share.low < 0.50 < bvap_share.high` (or any other configured threshold), the MM count is "indeterminate" for that district. Aggregate plan-level MM-count direction change = `(plan_a.mm_count_low, plan_a.mm_count_high)` overlaps `(plan_b.mm_count_low, plan_b.mm_count_high)`.
- [ ] **4.4** `suppress_or_emit(metric: &str, monotonicity, plan_a: CiBand, plan_b: CiBand) -> Option<String>` returning `None` (use the auto-text) or `Some("within margin of error; see numerical table.")`.
- [ ] **4.5** Wire suppression into `render_narrative` for partisan-seat, mm-count, mean-PP, and population-deviation paragraphs.
- [ ] **4.6** L0 test (BENCHMARK): feed Plan A=5.0 ± 0.6 dem seats, Plan B=5.4 ± 0.5 → assert "within margin of error" substituted. Non-monotone test: Plan A MM=1 [BVAP CI 0.48–0.53], Plan B MM=0 [0.46–0.49] → assert suppression.

**Exit:** MoE suppression exercised by 6+ unit tests covering monotone + non-monotone + edge-of-band cases.

---

## Task 5: `[DRAFT]` gate + `--approved-by` sign-off

**Files:** `redist/crates/redist-report/src/narrative.rs`, `redist/crates/redist-cli/src/compare.rs`

- [ ] **5.1** When `approved_by: None` is passed to `render_narrative`, every paragraph (each rendered via a Tera macro `{% macro para(text) %}...{% endmacro %}`) is prefixed with `[DRAFT — review before publication] `. When `approved_by: Some("Jane Q. Citizen")`, the prefix is omitted.
- [ ] **5.2** The signed name flows into `NarrativeManifest.approved_by`; manifest also records `approved_at` (ISO-8601 UTC of generation time, but pinned via `SOURCE_DATE_EPOCH` for reproducibility — see Task 9).
- [ ] **5.3** L0 BENCHMARK test (spec line 124): omit `--approved-by`, assert every paragraph in the rendered Markdown starts with `[DRAFT — review before publication]` and `narrative_manifest.json` has `approved_by: null`.
- [ ] **5.4** L0 negative test: pass `--approved-by "  "` (whitespace) — reject in arg validation (Task 1.3).

**Exit:** Two paired tests (drafted vs. signed) pass; manifest sign-off field round-trips correctly.

---

## Task 6: Diff visualization (third map)

**Files:** `redist/crates/redist-map/src/diff_map.rs` (new), `redist/crates/redist-map/src/lib.rs`

The third map shows only tracts that moved between the two plans, color-coded by destination district.

- [ ] **6.1** `build_diff_svg(diff: &DiffSummary, tract_geoms: &HashMap<String, MultiPolygon<f64>>, proj: &Projection, w, h, palette: &Palette) -> String` — reuses `polygon_svg_element` and `coords_to_svg_path` from `renderer.rs`. Background tracts (unchanged) render in light grey at 0.25 opacity; changed tracts render at 0.95 opacity in the destination-district color.
- [ ] **6.2** Add a small legend block: "Tracts that moved (colored by destination district in Plan B)".
- [ ] **6.3** PNG output via existing `svg_to_png` + `default_font_db`.
- [ ] **6.4** L0 test: synthetic 100-tract case where 12 tracts moved; assert the SVG contains exactly 12 paths at opacity > 0.9 and 88 at opacity < 0.3.

**Exit:** `diff.png` produced for the synthetic case; visually distinct from either plan map.

---

## Task 7: HTML side-by-side renderer

**Files:** `redist/crates/redist-report/templates/comparison-side-by-side.j2.html`, `redist/crates/redist-report/src/comparison_html.rs` (new)

- [ ] **7.1** Self-contained HTML (no CDN, no external HTTP) — inherit the constraint from existing `report.html.j2`. Maps embedded as base64 data URIs.
- [ ] **7.2** Layout: title bar; two-column map row (Plan A left, Plan B right) with unified legend; full-width diff map row; per-district table (D1 Plan A | D1 Plan B | Δ for each numerical metric); aggregate change summary; narrative section (Markdown→HTML rendered server-side via `pulldown-cmark`); audit trail footer linking the manifest SHA.
- [ ] **7.3** When `submission_type == "civic_counter_proposal"`, the title bar carries a CSS-styled "Civic Counter-Proposal" badge above the affected plan's map.
- [ ] **7.4** Mobile overflow rule (matches existing `test_html_report_pre_has_overflow_auto`): every `<pre>` and `<table>` wrapper has `overflow-x: auto`.
- [ ] **7.5** L0: render against synthetic input; assert presence of "Plan A", "Plan B", "Diff", "[DRAFT" (when unsigned), and absence of `http://`.

**Exit:** Comparison HTML renders cleanly; no external fetches; mobile-safe.

---

## Task 8: Civic-facing summary card PNG + watermark (BD-N3)

**Files:** `redist/crates/redist-map/src/summary_card.rs` (new)

A 1200×675 PNG suited to social-media share previews.

- [ ] **8.1** `build_summary_card_svg(report: &ComparisonReport, plan_a_thumb_png: &[u8], plan_b_thumb_png: &[u8]) -> String` lays out: title bar, two thumbnail maps, one-sentence headline (the FIRST paragraph of the narrative, with `[DRAFT]` prefix preserved when unsigned), key-stats row ("Dem seats: 5→6", "MM districts: 1→1", "PP: 0.42→0.40"), footer ("Generated by redist · {SOURCE_DATE_EPOCH date} · {REDIST_BUILD_COMMIT_SHORT}").
- [ ] **8.2** **BD-N3 watermark**: when `submission_type == "civic_counter_proposal"` for either plan, overlay a diagonal semi-transparent text watermark "CIVIC COUNTER-PROPOSAL · NOT OFFICIAL STATE MAP" across the affected plan's thumbnail (NOT a tiny corner badge — must survive a screen-cap crop). Implement as an SVG `<text>` element at 30° rotation, opacity 0.35, font-size scaled to thumbnail width / 18.
- [ ] **8.3** Render via `svg_to_png` + `default_font_db`. Output `comparison_card.png`.
- [ ] **8.4** L0 test: build a counter-proposal-flagged report, render the card, decode the resulting PNG, assert the watermark text appears in the SVG source AND that the rendered pixel data has non-trivial variance in the diagonal stripe (PNG histogram-based smoke).

**Exit:** Summary card produced; counter-proposal watermark visible and resilient to cropping.

---

## Task 9: `narrative_manifest.json` producer with SOURCE_DATE_EPOCH pinning + relative paths (M-04, PP-31, COVENANT C-3)

**Files:** `redist/crates/redist-report/src/narrative_manifest.rs` (new), `redist/crates/redist-report/src/lib.rs`

This is the audit trail spec §7 demands. Every field must be reproducible.

- [ ] **9.1** Define `NarrativeManifest` matching spec §7 schema:
  ```
  {
    "schema_version": "narrative-manifest v1",
    "template_path": "<relative-to-package-root>",
    "template_sha256": "...",
    "leaning_threshold": 0.55,
    "close_call_band": 0.02,
    "moe_inputs": {...},
    "plan_a_label": "...", "plan_a_manifest_sha256": "...",
    "plan_b_label": "...", "plan_b_manifest_sha256": "...",
    "baseline_label": null | "...", "baseline_manifest_sha256": null | "...",
    "analysis_sha256": {
      "plan_a": {"partisan.json": "...", "vra_analysis.json": "...", "compactness.json": "..."},
      "plan_b": {...}
    },
    "approved_by": null | "<signed name>",
    "approved_at": "<ISO-8601 UTC>",
    "redist_version": "0.1.0",
    "redist_build_commit_short": "<short SHA from REDIST_BUILD_COMMIT_SHORT>",
    "rustc_version": "...",
    "submission_type": null | "civic_counter_proposal",
    "civic_counter_proposal_attribution": null | {...},
    "comments_label": null | "<label>",
    "comments_overlay_sha256": null | "..."
  }
  ```
- [ ] **9.2** **M-04 / PP-31 path-portability**: every path field (`template_path`, any analysis-file refs) is stored RELATIVE to the package root, computed via a `path_relative_to_package_root(p: &Path) -> String` helper. Use forward-slash separators on all platforms; never embed Windows `C:\...` absolute paths. Path values containing spaces or non-ASCII are stored verbatim (UTF-8 in JSON), NOT percent-encoded; the consumer reads them as-is.
- [ ] **9.3** **COVENANT C-3**: `plan_a_manifest_sha256` is the SHA-256 of the canonicalized plan manifest bytes (use `serde_json::to_vec` on a `BTreeMap`-ordered representation so re-render is byte-stable). Same for plan B and baseline.
- [ ] **9.4** **Reproducibility (BENCHMARK P1 patched)**: when `SOURCE_DATE_EPOCH` env is set, `approved_at` is the corresponding ISO-8601 UTC; when unset for `[DRAFT]`, use `1970-01-01T00:00:00Z`. `redist_build_commit_short` is read from `REDIST_BUILD_COMMIT_SHORT` env (build-time) or falls back to compile-time `git rev-parse --short HEAD` baked in by `build.rs`. NEVER read `SystemTime::now()` for manifest fields.
- [ ] **9.5** Manifest written with `serde_json::to_string_pretty` and a stable BTreeMap key order so byte-identical re-render is achievable.
- [ ] **9.6** L0 test: write manifest twice with identical inputs and a pinned `SOURCE_DATE_EPOCH=1700000000`; assert byte-identical bytes.

**Exit:** Manifest schema implemented; reproducibility test green.

---

## Task 10: `--comments-label` overlay (Civic Bidirectional consumer side)

**Files:** `redist/crates/redist-report/src/comments_overlay.rs` (new)

- [ ] **10.1** Load comments by label from `outputs/{version}/civic/{comments_label}/comments.normalized.csv` (canonical output of `redist civic ingest` per Civic Bidirectional spec §1). DO NOT redefine the schema here — import it. Required columns: `geoid, comment_id, label, source, source_url, confidence, submitted_at`.
- [ ] **10.2** For each `comment_id`, group its tracts; for each plan, classify the community as `kept_whole` or `split_across <list of districts>`.
- [ ] **10.3** `CommentsOverlay { per_comment: Vec<CommentResult>, plan_a_kept_pct: f64, plan_b_kept_pct: f64 }`. Surface in the HTML side-by-side as a dedicated "comments incorporated" appendix.
- [ ] **10.4** Surface in the narrative: a paragraph "Of [N] community-of-interest comments, Plan A keeps [X]% whole; Plan B keeps [Y]% whole. Plan B splits the [name] community across districts 3 and 5." Threshold for inclusion of a comment in the narrative paragraph: `confidence >= 0.5` (configurable via `--comments-min-confidence`, default 0.5).
- [ ] **10.5** L1 test (spec line 126): synthetic plans + a 4-comment CSV; assert appendix lists exactly the expected `comment_id`s.
- [ ] **10.6** Manifest: `comments_overlay_sha256` = SHA-256 of the input CSV bytes (post-canonicalization by `redist civic ingest`, so it's stable).

**Exit:** Overlay round-trips through the report; narrative paragraph correctly enumerates comments.

---

## Task 11: Wire `redist compare --format html|narrative|both` end-to-end

**Files:** `redist/crates/redist-cli/src/compare.rs`

- [ ] **11.1** When `format` is `Html`, `Narrative`, or `Both`, dispatch to `redist_report::assemble_comparison` + the renderers. Otherwise keep the existing `Table`/`Json`/`Csv` path untouched.
- [ ] **11.2** Output directory layout matches spec §"Outputs":
  ```
  outputs/{version}/comparisons/{plan_a}_vs_{plan_b}/
  ├── comparison.html
  ├── comparison_card.png
  ├── narrative.md
  ├── diff.png
  ├── comparison.json
  └── narrative_manifest.json
  ```
  `comparison.pdf` is NOT in this plan — it's delivered by Court Reports plan via Typst (cross-reference).
- [ ] **11.3** When `--baseline ENACTED` is supplied: load the baseline plan's assignments via the existing `load_plan_assignments` helper, hash its manifest, set `baseline_label` + `baseline_manifest_sha256` in the narrative manifest. The HTML and narrative present the baseline in plan-B's slot with a "(currently enacted)" badge.
- [ ] **11.4** Print on success: `[OK] comparison HTML -> {path}`, `[OK] narrative -> {path}`, `[OK] manifest -> {path}` (ASCII-only — PP-34).

**Exit:** End-to-end CLI produces all artifacts on the synthetic Vermont fixture used by Onboarding's L2 test.

---

## Task 12: Reproducibility CI gate — byte-identical re-render

**Files:** `redist/tests/reproducibility/test_narrative_byte_identical.rs` (new), `redist/crates/redist-report/tests/byte_identical.rs` (new)

- [ ] **12.1** Test fixture: small synthetic two-plan dataset under `redist/tests/fixtures/comparison_repro/` (~10 tracts, hand-computed expected values).
- [ ] **12.2** Invoke `redist compare --plan-a ... --plan-b ... --format both --approved-by "Test"` twice with `SOURCE_DATE_EPOCH=1700000000` and `REDIST_BUILD_COMMIT_SHORT=abcd1234`; assert `narrative.md` and `narrative_manifest.json` are byte-identical between the two runs.
- [ ] **12.3** PP-31 test: place the fixture at a path containing a literal space and a non-ASCII character (e.g., `tests/fixtures/comparison repro café/`). Assert `narrative_manifest.json["template_path"]` is a forward-slash relative path with the original characters intact, AND that running `redist compare` from a different CWD still produces the same manifest bytes.
- [ ] **12.4** Wire into the existing reproducibility CI workflow (referenced by Court Reports plan); marker `#[ignore]`-default if it depends on the Onboarding fixture, otherwise always-on.

**Exit:** Two consecutive runs are bit-identical; path-with-spaces fixture round-trips.

---

## Task 13: Value-correctness tests (BENCHMARK)

**Files:** `redist/crates/redist-report/tests/narrative_value_correctness.rs` (new)

Per spec line 121, value-correctness is the most load-bearing gate: catches off-by-one threshold errors and template-variable mis-binding.

- [ ] **13.1** Hand-computed synthetic input: Plan A=5D/1R, Plan B=4D/2R, threshold=0.55, two close-call districts at 0.553 and 0.547.
- [ ] **13.2** Render narrative; assert exact string presence:
  - `"5 Democratic-leaning"` and `"4 Democratic-leaning"`
  - `"55% Dem-share threshold"`
  - For the 0.553 district: a "just above the leaning threshold" close-call mention
  - For the 0.547 district: a "just below the leaning threshold" close-call mention
- [ ] **13.3** Negative test: change threshold to 0.50; assert seat counts re-bind correctly (no caching artifacts). Catches mis-bound template vars.

**Exit:** Hand-computed truth matches rendered text exactly.

---

## Task 14: Civic-counter-proposal end-to-end test (BD-N3 closure)

**Files:** `redist/crates/redist-report/tests/counter_proposal.rs` (new)

- [ ] **14.1** Construct a synthetic Plan B whose `manifest.json` has `"submission_type": "civic_counter_proposal"`, `"submitted_by": "Eastside Neighborhood Association"`, `"submitted_at": "2026-04-15T12:00:00Z"`.
- [ ] **14.2** Run full `redist compare --format both`. Assert:
  - Narrative leads with the framing sentence.
  - `narrative.md` first paragraph contains "civic counter-proposal" and "not the state's official map".
  - `comparison_card.png` SVG source contains the watermark `<text>` element with `transform="rotate(30...)"` and `opacity="0.35"`.
  - The rendered PNG passes the histogram-based crop-resilience smoke from Task 8.4.
  - `narrative_manifest.json` records `submission_type: "civic_counter_proposal"` and the attribution sub-object.
- [ ] **14.3** Negative test: a normal plan (no `submission_type`) — assert no watermark, no framing sentence.

**Exit:** BD-N3 closed: watermark present on PNG, not just narrative.

---

## Task 15: Documentation + tutorial cross-wiring

**Files:** `docs/REDIST_CLI.md`, `docs/quickstart/quickstart-civic-advocate.md`, `docs/CHANGELOG.md`, `CLAUDE.md`

- [ ] **15.1** `docs/REDIST_CLI.md`: `redist compare` section gets the new flags, the output-directory layout, the `[DRAFT]`/`--approved-by` workflow, and a worked example using the Onboarding Vermont walkthrough plan as input.
- [ ] **15.2** `docs/quickstart/quickstart-civic-advocate.md` (owned by Onboarding plan; this plan amends it): add a "produce a comparison narrative" section ending in `redist compare --plan-a vt_proposed --plan-b vt_enacted --format both --approved-by "Your Name"`.
- [ ] **15.3** `docs/CHANGELOG.md` entry naming spec + plan.
- [ ] **15.4** `CLAUDE.md` "Recent Changes" entry.
- [ ] **15.5** Document the `--narrative-template <PATH>` override path in REDIST_CLI.md, with one minimal worked example showing how a state advocacy group can fork the template.

**Exit:** A civic advocate following the quickstart can produce a signed-off comparison without reading this plan.

---

## Definition of Done

- `redist compare --plan-a X --plan-b Y --format both` produces every artifact in spec §"Outputs" (HTML, narrative, diff PNG, summary card PNG, comparison JSON, narrative manifest).
- HTML side-by-side renders cleanly in Chrome/Firefox/Safari (manually verified).
- Every auto-generated paragraph carries `[DRAFT — review before publication]` until `--approved-by` is supplied; signed name flows into the manifest.
- Re-running the same `compare` command on the same inputs with `SOURCE_DATE_EPOCH` + `REDIST_BUILD_COMMIT_SHORT` pinned produces byte-identical `narrative.md` and `narrative_manifest.json` (CI gate).
- `narrative_manifest.json` records `plan_a_manifest_sha256`, `plan_b_manifest_sha256`, and `baseline_manifest_sha256` (when `--baseline ENACTED`).
- `--comments-label LABEL` produces a "comments incorporated" appendix listing every consumed `comment_id`.
- Comparison summary card PNG carries a visible diagonal watermark when either plan is a civic counter-proposal (BD-N3); watermark survives screen-cap crops.
- Narrative-manifest paths are stored relative to package root and round-trip through a path-with-spaces fixture (PP-31).
- MoE suppression handles non-monotone metrics (MM count) per the threshold-crossing definition (S-04).
- Value-correctness tests assert exact-string presence of hand-computed seat counts and close-call flags.
- `docs/REDIST_CLI.md` documents the full surface; `quickstart-civic-advocate.md` references the comparison flow.

---

## Risks

| Risk | Mitigation |
|---|---|
| Tera template diverges silently from `ComparisonReport` schema | The template's variable contract is doc-commented and exercised by the value-correctness test (Task 13); a CI lint greps for new `{{...}}` references not covered by tests. |
| Byte-identical re-render gate flakes on git-SHA / timestamp leaks | Task 9.4 routes every timestamp through `SOURCE_DATE_EPOCH` and every commit short SHA through `REDIST_BUILD_COMMIT_SHORT`; Task 12 is the explicit CI gate. |
| Path with spaces / non-ASCII in absolute manifest paths breaks Windows runners (PP-31) | Always store relative paths; Task 12.3 fixture forces a path-with-space test on every CI run. |
| Civic counter-proposal watermark is too subtle and gets cropped out (BD-N3) | Diagonal placement + 35% opacity + ~5% text height ensures any reasonable crop still includes part of it; histogram-based smoke test asserts pixel variance in the diagonal stripe. |
| MoE suppression for MM count under-suppresses (false negative) on near-threshold districts | S-04 definition uses an explicit BVAP CI bracket on the 50% boundary; Task 4.6 includes a near-threshold negative case to catch this. |
| Narrative tone reads as robotic press-release | One default template, `--narrative-template <PATH>` override; Task 15 documents the override path so advocacy groups can iterate without forking the binary. |
| Civic comments overlay accidentally inflates "kept-whole" % by treating singleton communities as trivially whole | Threshold-confidence filter (Task 10.4) excludes low-confidence comments; the appendix lists every `comment_id` so reviewers can audit. |

---

## Out of Scope

- Comparison PDF output (delivered by Court Reports plan via Typst — this plan emits HTML + Markdown + PNG only).
- Multi-plan (>2) comparison narrative — the existing `--labels` table flow already handles N-plan summary tables; narrative is two-plan-only.
- Custom narrative templates *shipped per organization* — only one default; users override via `--narrative-template <PATH>`.
- The write side of `redist civic ingest` (intake, validation, GEOID-leading-zero protection, URL snapshot). Owned by Civic Bidirectional plan; this plan only consumes its output.
- Real-time / interactive comparison drawing (Districtr territory; State Staff Interop plan).
- Translation / localization of narrative paragraphs (English-only).
