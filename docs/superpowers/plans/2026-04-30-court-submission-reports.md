# Plan: Court Submission Reports (PDF Expert Witness Output)

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-30-court-submission-reports.md` v2 (with v2.1 patches) has been reviewed and approved. Steps use checkbox (`- [ ]`) syntax for tracking.

**Date:** 2026-04-30
**Spec:** `docs/superpowers/specs/2026-04-30-court-submission-reports.md`
**v2.1 tracking ref:** `docs/superpowers/specs/2026-04-30-v21-tracking.md`
**Goal:** Ship `redist report --format pdf` producing a PDF/A-2b court-ready expert-witness document driven by Typst, validated by `verapdf`, accompanied by a deterministic reproducibility-package zip whose every embedded path, citation, and provenance field is auditable. Court-mode output refuses to embed civic inputs ingested under non-strict validation unless an explicit override flag is set.

**Depends on:**
- Existing `redist-report` HTML pipeline (`redist/crates/redist-report/`) and `report_cmd.rs` dispatch in `redist-cli`.
- `redist-cli/src/provenance.rs` for build commit + rustc version.
- `redist doctor --verify-manifest` (already shipped) for the rebuild-instructions block.
- Onboarding plan (`docs/superpowers/plans/2026-04-30-onboarding-and-tutorials.md`) — references `bootstrap.sh`/`bootstrap.bat` from the rebuild block; does not need to ship first.
- Callais Evidence Layer plan — provides bloc-voting JSON inputs and race-of-candidate CSV; the PDF template renders those when present, but tolerates absence.

**Blocks:** None directly. Plan Comparison and Deposition Prep both reference the manifest schema and citation conventions defined here.

**v2.1 items addressed by this plan:**
- M-03 (`docs/file-formats/manifests.md` — written first, referenced throughout)
- D-01 (`accessed_date` vs `fetched_at` rule, in manifests.md)
- D-03 (`sources.json` schema_version + content-hash; report-time snapshot)
- D-04 (deterministic Typst PDF: `SOURCE_DATE_EPOCH`, `tar --sort=name --mtime=...`, zeroed PDF metadata)
- D-06 (`docs/file-formats/citation-strings.md` — depo + civic strings, Chicago default scope)
- B-01 **P0** (PDF text extraction asserting hand-computed numbers; malformed-Typst-fails-verapdf negative test)
- B-10 (L2 stronger assertions — section-header presence + ordering)
- PP-32 (Typst stderr surfaced in manifest, fail on warnings)
- BD-R1 (`--allow-non-strict-civic` gate for civic-validated-as-lenient inputs)
- CM-02 (methodology section credits civic data contributors above the fold)
- C-04 (rebuild block links to bootstrap with time estimates + common failures — already patched in spec; this plan delivers the artifact)

---

## Pre-Conditions

- Existing HTML report path (`redist report --format html`) generates a clean `Report` struct via `redist_report::assemble_report` covering the analysis JSONs the PDF template will pull from.
- `redist-cli/src/report_cmd.rs::try_generate_pdf` exists today as a wkhtmltopdf/pandoc fallback. This plan replaces it with a Typst-driven path that runs by default; the legacy HTML-to-PDF code is removed.
- `redist doctor --verify-manifest` is callable from a script and exits 0 when manifest entries hash-match.
- Typst `>= 0.12` is available on developer machines (PDF/A export landed in 0.12); CI runners install a pinned version per task 3.
- `verapdf` (Java-based PDF/A validator) is installable; CI installs the pinned `verapdf-greenfield` release. Local dev fall-back: spec exit message names the install URL (consistent with the current pandoc/wkhtmltopdf hint pattern).

---

## Task 1: Followup-doc — `docs/file-formats/manifests.md` (M-03, D-01)

**Files:** `docs/file-formats/manifests.md` (new)

This doc must land before any of Task 5+ runs because the Typst template, the citation generator, and the reproducibility appendix all reference its field inventory and date semantics.

- [ ] **1.1** Inventory every manifest emitted today, **v2.1.1 P1 expanded**: `PlanManifest` (`redist-report::manifest`); deposition log sidecar (`deposition_log_{date}.manifest.json` from Deposition Prep plan); `narrative_manifest.json` (Plan Comparison plan, schema-version `narrative-manifest v1`); `what_if/.../manifest.json` (`whatif-manifest v1`, Deposition Prep plan); `reproducibility_package_manifest.json` (this plan); **`civic-coi v1` `CivicManifest`** (Civic Bidirectional plan); **`tutorial-checksums v1`** (Onboarding plan `examples/vermont-walkthrough/checksums.json`); **`import-compat v1`** (State Staff Interop plan, embedded SHA recorded in `PlanManifest.import_compat_sha256`); **`race_of_candidate_provenance v1`** (Callais plan). Each downstream plan that adds a new manifest type MUST land a one-task edit extending this inventory in the same release.
- [ ] **1.2** Document each field's type, required/optional, and exact derivation. **v2.1.1 P1**: also enumerate the **canonical required field set** that every manifest type MUST carry (`schema_version: "<kind> v<n>"`, `sha256`, `fetched_at` ISO-8601 UTC, `source_url`, `build_commit` long-form SHA, `redist_version`, `rustc_version`). Per-manifest-type extensions are additive.

  **Build-commit naming reconciliation (v2.1.1 P1)**: settle drift across plans. The canonical names are:
  - `redist_build_commit` — full 40-char git SHA, present in every manifest
  - `redist_build_commit_short` — 7-12 char prefix, present where space-constrained (e.g., narrative manifest header, summary card footer)
  - **NOT** `binary_build_commit`, `build_commit` (bare), or any other variant. Plans that currently use `build_commit` (Civic Task 1.3, Callais Task 6.1, Deposition Task 5.4) MUST adopt `redist_build_commit` at implementation time. This doc is the source of truth; plans inherit. The drift is recorded in v2.1 tracking item M-03.
- [ ] **1.3** **D-01**: Define the `accessed_date` vs `fetched_at` rule:
  - `fetched_at` — set by the fetcher when bytes were retrieved from a remote source; ISO-8601 UTC with seconds precision; lives inside per-input rows of any manifest that records bytes.
  - `accessed_date` — set by the citation generator at report-generation time, recorded as a calendar date (no time-of-day) per court-citation custom; appears only in `citations[].accessed_date`.
  - When both apply to the same row, `accessed_date` is derived from `fetched_at` by truncating to UTC date.
- [ ] **1.4** Document the path-portability rule (relative-to-package-root) as it applies to **every** manifest type, citing `docs/file-formats/manifests.md` from each spec.
- [ ] **1.5** Document the cross-manifest hash-link convention (e.g., `narrative_manifest.json.plan_a_manifest_sha256` references the upstream `PlanManifest.json` SHA-256).
- [ ] **1.6** Add a "Schema versioning" section: every manifest type carries a `schema_version` string (`"<kind> v<n>"`); migration is additive only without a major bump; readers must accept unknown fields.

**Exit:** `docs/file-formats/manifests.md` exists; every other v2.1 spec links to it; no field's semantics are defined twice in different specs.

---

## Task 2: Followup-doc — `docs/file-formats/citation-strings.md` (D-06)

**Files:** `docs/file-formats/citation-strings.md` (new)

- [ ] **2.1** Define the Bluebook, APA, and Chicago string templates for every source class the project currently emits citations for: dataset (DOI), dataset (URL only), software (this binary, with build commit), expert deposition transcript (per Deposition spec), civic counter-proposal (per Civic Bidirectional spec).
- [ ] **2.2** **D-06 default scope**: Chicago is the default for the `--paper-mode` *non-court* flow; Bluebook is the default for `--jurisdiction <COURT>`. Document precedence: explicit `--citation-style` overrides both; without either flag, Bluebook is the conservative default.
- [ ] **2.3** Worked examples per template per source class — exact strings produced from a known fixture row (build the example from `examples/vermont-walkthrough/` outputs once Onboarding ships).
- [ ] **2.4** Punctuation and ordering rules for the multi-source case (semicolon-joined; chronological; primary then secondary).
- [ ] **2.5** Errata-footnote convention: when a `sources.json` entry has a `notes` field flagging dataset limitations, the citation appends a footnote with the verbatim text; document the exact footnote anchor markup the Typst template will consume.

**Exit:** `docs/file-formats/citation-strings.md` exists with worked examples; the Court Reports Typst template and Deposition log citation block both reference it as the single source of truth.

---

## Task 3: Pin Typst and add it to the build/CI surface

**Files:** `redist/rust-toolchain.toml` (no change), `docs/REPRODUCIBLE_BUILD.md`, `.github/workflows/*.yml`, `redist/crates/redist-report/typst-templates/.typst-version` (new)

- [ ] **3.1** Pin Typst to a specific minor (e.g., `0.12.x`); record the pin in `redist/crates/redist-report/typst-templates/.typst-version` (single-line file, parsed by the wrapper at runtime to assert).
- [ ] **3.2** Pin `verapdf` to a specific release; record similarly under `redist/crates/redist-report/typst-templates/.verapdf-version`.
- [ ] **3.3** CI install steps: install Typst (curl release tarball; verify SHA-256), install Java + verapdf zip; cache both. Document the install commands in `docs/REPRODUCIBLE_BUILD.md` under a new "External tooling for PDF reports" section.
- [ ] **3.4** `redist report --format pdf` preflight: at startup, exec `typst --version` and `verapdf --version`, parse, and assert against the pinned strings. On mismatch, fail with `[CONFIG]` actionable error naming the expected version and install URL — do not silently produce a non-court-ready PDF.

**Exit:** Two pinned external tools, one place to update each, CI green with both installed; preflight catches version drift before render.

---

## Task 4: Carve out `redist-report` Typst module + remove legacy HTML-to-PDF fallback

**Files:** `redist/crates/redist-report/Cargo.toml`, `redist/crates/redist-report/src/lib.rs`, `redist/crates/redist-report/src/typst_render.rs` (new), `redist/crates/redist-report/typst-templates/` (new directory tree), `redist/crates/redist-cli/src/report_cmd.rs`

- [ ] **4.1** Add a `typst_render` module to `redist-report` with the surface: `render_pdf(report: &Report, ctx: &PdfRenderContext) -> Result<PdfRenderArtifacts>`. `PdfRenderContext` carries expert config (name, credentials, affiliation, jurisdiction, citation_style, draft flag, allow_non_strict_civic flag). `PdfRenderArtifacts` carries the produced `.pdf` bytes, the rendered `.typ` source, the captured Typst stderr, and the verapdf JSON output.
- [ ] **4.2** Delete `report_cmd::try_generate_pdf` and the wkhtmltopdf/pandoc fallback. `report_cmd.rs` for the PDF format now calls into `redist_report::typst_render::render_pdf` directly.
- [ ] **4.3** Wire `--draft` flag through `args::ReportArgs` to the render context; `--draft` skips the verapdf gate and stamps a "DRAFT — NOT COURT-ADMISSIBLE" watermark in the Typst template. Document that draft PDFs do not record `pdfa_validated: true` in the package manifest.
- [ ] **4.4** Add new CLI flags from the spec: `--expert-name`, `--expert-credentials`, `--expert-affiliation`, `--case-caption-file`, `--jurisdiction`, `--citation-style`, plus the new `--allow-non-strict-civic` (BD-R1, see Task 8). Allow loading from `expert.toml` if `--expert-config <path>` is set; CLI flags override file values. **v2.1.1 P1 (M-01 alias)**: `--plan-label` accepted as clap alias for `--label` (`#[arg(long, alias = "plan-label")]`) so workflows mixing `redist report --plan-label X` with `redist depo eval --plan-label X` are not flag-mismatched.
- [ ] **4.5** Author the directory `redist/crates/redist-report/typst-templates/` with: `expert_report.typ` (the document), `_cover.typ`, `_methodology.typ`, `_results.typ`, `_qualifications.typ`, `_conflicts.typ`, `_prior_testimony.typ`, `_daubert.typ`, `_reproducibility.typ`, `_signature.typ`, `_citations.typ`, `_assets/` (logos, fonts as needed). Each section is a separate file imported by `expert_report.typ` so per-section L0 unit tests can render them in isolation.
- [ ] **4.6** Pass values into Typst by writing a single `inputs.json` next to the `.typ` source, then invoking `typst compile --input data=inputs.json`. The `inputs.json` schema is defined in `redist-report/src/typst_render.rs` (a serde struct) and asserted against the template's expected keys via the Task 9 L0 tests.

**Exit:** `redist report --format pdf --label vt_test` produces `vt_test_report.pdf` plus a `vt_test_report.typ` sidecar via Typst; legacy wkhtmltopdf/pandoc paths are gone.

---

## Task 5: Per-section template authoring — required content

**Files:** `redist/crates/redist-report/typst-templates/_*.typ`

Section assembly follows the spec's expanded list (cover, exec summary, methodology, results, qualifications, conflicts, prior testimony, Daubert self-assessment, reproducibility appendix, signature, citations).

- [ ] **5.1** Cover page — court caption (from `--case-caption-file`), title, expert name/credentials, jurisdiction, generation date, `[DRAFT]` watermark when applicable.
- [ ] **5.2** Executive summary — one paragraph autogenerated from the analysis JSONs (assemble_report already builds the Report struct); explicitly tagged `[DRAFT]` so the expert rewrites before signing.
- [ ] **5.3** **Methodology — CM-02**: First subsection (above all algorithmic content) is a "Data sources and contributors" credit block. Renders the list of `sources.json` entries used, with curator/author per entry, and a parallel block crediting any civic data contributors named in `civic_inputs/*.json` provenance (per Civic Bidirectional spec). Layout: contributors above algorithm description.
- [ ] **5.4** Results — VRA / partisan / bloc / compactness sections; each renders only if the corresponding analysis JSON is present in the plan dir; each emits the "DRAFT plain-English" sentence pulled from the existing `Report` struct.
- [ ] **5.5** Expert qualifications — pulls from `expert.toml` `qualifications.cv_path` (PDF; embedded via Typst's image/PDF inclusion), or a free-text fallback block.
- [ ] **5.6** Conflicts of interest — declarative block from `expert.toml`; default text "Expert declares no conflicts" if unset, with a `[DRAFT]` mark.
- [ ] **5.7** Prior testimony (FRCP 26(a)(2)(B)) — table from `expert.toml` `prior_testimony[]` rows: case, court, year, role.
- [ ] **5.8** Daubert self-assessment — checklist (testable hypothesis, peer-review status, error rate, general acceptance) with the expert's assertion per row from `expert.toml`.
- [ ] **5.9** Reproducibility appendix — see Task 6.
- [ ] **5.10** Signature block — name, credentials, affiliation, dated line; intentionally blank ink-line for wet signature.
- [ ] **5.11** Citations — generated by the `_citations.typ` partial that consumes the snapshot from Task 7 and emits strings per `docs/file-formats/citation-strings.md`. Errata footnotes appear inline.

**Exit:** Every section renders with the reference Vermont and Louisiana fixtures; no `<<MISSING>>` template variables in the produced PDF.

---

## Task 6: Reproducibility appendix + sidecar zip (D-04 deterministic build)

**Files:** `redist/crates/redist-report/src/typst_render.rs`, `redist/crates/redist-report/src/repro_zip.rs` (new), `redist/crates/redist-report/typst-templates/_reproducibility.typ`

- [ ] **6.1** Build the reproducibility-package zip with deterministic byte output:
  - Set `SOURCE_DATE_EPOCH` from the manifest `created_at` (Unix seconds).
  - Use a Rust `tar`+`gzip` (or directly `zip` crate set to STORE-only) implementation that writes entries sorted by name, uses fixed mtime equal to `SOURCE_DATE_EPOCH`, fixed uid/gid 0, fixed file mode 0644 / 0755 dirs. Document the equivalent shell command (`tar --sort=name --mtime=@$SOURCE_DATE_EPOCH --owner=0 --group=0 --numeric-owner -cf ... | gzip -n`) in `docs/REPRODUCIBLE_BUILD.md` for third-party verification.
- [ ] **6.2** Zero PDF metadata in the Typst template: `set document(author: "", title: "", date: none)` plus pass Typst the `--root` and `--input` flags only; **do not** rely on Typst's defaults that would inject the build host date. Capture the PDF/A `xmpmeta` produced by Typst and assert in tests it is byte-stable across two builds.
- [ ] **6.3** Compute the zip's SHA-256 once it's finalized and embed it into the PDF body (so the PDF references the zip; the zip cannot reference the PDF, since the PDF would change). Embed the PDF separately into the zip but **after** PDF body is finalized — i.e., the zip is built last in the pipeline.
- [ ] **6.4** Page-cap the in-PDF appendix at 20 pages by truncating the embedded manifest table at row N (computed dynamically from page-fit) and pointing readers to the zip for full data.
- [ ] **6.5** **Path-portability pass** (TRENCH): before serializing manifest entries, rewrite any absolute path to be relative to the package root. Rejection rule: if the relative-conversion produces a path that escapes the root (`..` traversal), abort with `[INTERNAL]` error.
- [ ] **6.6** Reproducibility appendix content: reuse the spec's prereq + manual rebuild block verbatim (already provided in the spec's §3); link to `bootstrap.sh`/`bootstrap.bat` (Onboarding plan) with the time estimates "5-15 min first build" / "30-60 min manual fallback".
- [ ] **6.7** L1 determinism test: render the Vermont fixture twice in the same process with `SOURCE_DATE_EPOCH` set; assert PDF bytes equal AND zip bytes equal.

**Exit:** Two consecutive runs on the same inputs produce byte-identical `expert_report.pdf` and `reproducibility_package.zip`.

---

## Task 7: PDF/A-2b validation + Typst stderr surfacing (PP-32)

**Files:** `redist/crates/redist-report/src/typst_render.rs`, `redist/crates/redist-report/src/verapdf.rs` (new)

- [ ] **7.1** After Typst compile, capture its `stderr`. **PP-32**: any non-empty stderr — even a "warning" — fails the render unless `--draft` is set. The captured stderr is recorded verbatim in the package manifest's new `typst_stderr` field (always present; empty string on clean render).
- [ ] **7.2** Run `verapdf --format json --flavour 2b <pdf>` as a subprocess; parse the JSON; gate on `passedRules == totalRules`. On failure, emit `[VERAPDF]` error category with the exact rule IDs that failed (e.g., "PDF/A-2b rule 6.1.13: fonts must be embedded") and refuse to write the package zip — the caller gets a non-zero exit and the directory is left without a `.pdf` so partial outputs can't be submitted by mistake.
- [ ] **7.3** Record `pdfa_validated: true|false`, `pdfa_flavour: "2b"`, `verapdf_version`, and the verapdf raw JSON SHA-256 in the package manifest.
- [ ] **7.4** Document in `docs/error-conventions.md` that `[VERAPDF]` errors are "expected" in the sense that template authors will hit them when fonts aren't embedded; the actionable hint is the rule ID and a link to the PDF/A spec section.
- [ ] **7.5** **Sources.json snapshot (D-03)**: at report-generation time, copy `scripts/data/elections/sources.json` into the package zip at `sources/sources.json`; compute its content-hash and record both `sources_json_sha256` and `sources_json_schema_version` in the package manifest. Bump the live `sources.json` schema_version field if absent (it was 1; keep at 1, add a `content_hash_sha256` field to each source row recording the upstream-file hash where known).

**Exit:** A deliberately-malformed Typst template (Task 9.5) fails the gate and exits non-zero with `[VERAPDF]`; a clean render writes the package and reports `pdfa_validated: true`.

---

## Task 8: Civic-input gating — `--allow-non-strict-civic` (BD-R1)

**Files:** `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/report_cmd.rs`, `redist/crates/redist-report/src/typst_render.rs`

- [ ] **8.1** When assembling the Report, scan civic-input provenance fields (per Civic Bidirectional spec, every civic input has a `validation_level: "strict" | "lenient" | "advisory"`). If any input present in the analysis chain is `lenient` or `advisory`, refuse to render the PDF unless `--allow-non-strict-civic` is set, with `[BOUNDARY]` error message naming the offending file path(s) and validation level.
- [ ] **8.2** When `--allow-non-strict-civic` is set: render proceeds, BUT the package manifest records `non_strict_civic_inputs: [{"path": ..., "validation_level": "lenient"}]`, AND a Typst-rendered cover-page banner reads "This report includes civic inputs validated under non-strict mode. See Reproducibility Appendix §X." The Reproducibility Appendix gets a corresponding subsection enumerating the inputs and their validation levels.
- [ ] **8.3** L0 test: synthetic Report with one `lenient` civic input fails without the flag; passes with the flag and the manifest contains the banner annotation row.

**Exit:** Court-mode reports cannot embed lenient civic inputs silently; the override is auditable in the manifest and visible on the cover page.

---

## Task 9: Tests — value-correctness + verapdf gate (B-01 P0, B-10)

**Files:** `redist/crates/redist-report/tests/`, `tests/acceptance/test_court_report_*.py`

- [ ] **9.1** **L0 — template rendering with synthetic data.** Per-section unit test that renders each `_*.typ` partial in isolation against a deterministic `inputs.json` fixture; assert exit 0, no NaN/null leak (grep the `.typ` output for the literals `NaN`, `null`, `<<`, `MISSING`).
- [ ] **9.2** **L1 — full PDF generation pipeline (Vermont).** Renders the Vermont walkthrough Report end-to-end against pinned fixture JSONs; asserts the PDF exists, parses, and contains all 11 section headings via Typst's `--root` outline export OR a `pdftotext`-based extraction.
- [ ] **9.3** **B-01 P0a — value-correctness via PDF text extraction.** Render Vermont fixture; extract text via `pdftotext` (Poppler). Assert hand-computed numbers from the methodology fixture appear verbatim:
  - The MM district count: e.g., "1 majority-minority district" if VT fixture is calibrated to that.
  - The exact efficiency-gap value to two decimals from the partisan fixture (e.g., "Efficiency gap: -2.34%").
  - The exact Polsby-Popper mean from the compactness fixture.
  - The build-commit short SHA in the provenance block.
  - The package zip's SHA-256 (last 12 hex chars, since the full hash wraps).
  Hand-computed numbers are committed under `redist/crates/redist-report/tests/fixtures/court_report/expected_values.toml` so the assertions read the expected from disk, keeping the tests easy to re-pin when fixtures change.
- [ ] **9.4** **B-10 — section-header presence + ordering.** Replace the spec's draft "page count is reasonable" L2 assertion. Extract the document outline (Typst writes a PDF outline; verapdf-greenfield can also dump it). Assert the exact ordered list of headings: Cover, Executive Summary, Methodology (with "Data sources and contributors" first child), Results, Expert Qualifications, Conflicts of Interest, Prior Testimony, Daubert Self-Assessment, Reproducibility Appendix, Signature, Citations. Also assert that "Data sources and contributors" appears strictly before any algorithmic-methodology subheading (CM-02 above-the-fold).
- [ ] **9.5** **B-01 P0b — deliberately-malformed Typst output fails verapdf.** Negative test: a fixture template `tests/fixtures/court_report/_broken.typ` references an unembedded font OR removes `pdf-version: "PDF/A-2b"` from the document set; render via `redist report --format pdf`; assert exit non-zero, assert `[VERAPDF]` category, assert the failed rule ID is in stderr, assert no `.pdf` was written to the output dir (Task 7.2 invariant).
- [ ] **9.6** **L0 — Typst stderr fail-on-warning (PP-32).** Fixture template that emits a Typst warning (e.g., overflowing box); assert render fails without `--draft`, succeeds with `--draft` and records the stderr in the package manifest.
- [ ] **9.7** **L0 — civic gating (BD-R1).** Two synthetic Report fixtures (one strict-only civic, one with one lenient civic); assert render fails without `--allow-non-strict-civic` on the latter; succeeds with it, banner present, manifest annotation present.
- [ ] **9.8** **L1 — determinism (D-04).** Already specified at 6.7; lives in `redist-report/tests/`.
- [ ] **9.9** **L2 — slow acceptance, default-skipped.** Pytest under `tests/acceptance/test_court_report_louisiana.py` marked `slow + network` running the LA Callais fixture once Callais Evidence Layer plan ships; asserts section ordering + verapdf pass + value-correctness for at least one Callais bloc-voting coefficient.

**Exit:** `cargo test -p redist-report` and the new pytest acceptance file are green; B-01 negative test demonstrably catches the broken-Typst case; B-10 ordering assertion is firm.

---

## Task 10: CLI surface, examples, docs

**Files:** `docs/REDIST_CLI.md`, `examples/louisiana-callais-walkthrough/expert_report_example/` (new), `CLAUDE.md`, `docs/CHANGELOG.md`

- [ ] **10.1** Document every new flag from Task 4 in `docs/REDIST_CLI.md` under `redist report`. Cross-link to `docs/file-formats/manifests.md` and `docs/file-formats/citation-strings.md`.
- [ ] **10.2** Commit a peer-reviewed sample expert-report PDF + zip + `.typ` source under `examples/louisiana-callais-walkthrough/expert_report_example/` (per spec DoD). The example uses a fictional expert name to avoid implying endorsement; README.md in that subdirectory documents the exact command line that produced it.
- [ ] **10.3** Write a one-page `docs/court-reports-quickstart.md` for the special-master / §2-expert persona: invoke pattern, common flags, troubleshooting (verapdf failures, civic-gate failures, draft mode).
- [ ] **10.4** Update `CLAUDE.md` "Recent Changes" entry; update `docs/CHANGELOG.md`.
- [ ] **10.5** Cross-reference: every other v2.1 spec/plan that mentions citations, manifests, or PDF rendering points at this plan's followup-docs (manifests.md, citation-strings.md) — verify links resolve.

**Exit:** A reader following `docs/court-reports-quickstart.md` cold can produce a PDF/A-2b expert report from a finished plan in under 10 minutes; the LA example sits under `examples/` reproducible byte-for-byte.

---

## Definition of Done

- `redist report --format pdf --label vt_test` produces `expert_report.pdf` (PDF/A-2b validated by verapdf), `expert_report.typ` (debugging source), and `reproducibility_package.zip` in under 10 seconds wall-clock.
- Two consecutive renders on the same inputs produce byte-identical PDF and zip (D-04).
- All 11 standard sections present in section-ordering test (B-10); no template variables leak.
- B-01 P0 tests pass: hand-computed values appear in extracted PDF text; deliberately-broken Typst fails the verapdf gate and exits non-zero with no `.pdf` written.
- `docs/file-formats/manifests.md` and `docs/file-formats/citation-strings.md` exist and are referenced from every relevant v2.1 spec.
- `sources.json` carries `schema_version` and per-source `content_hash_sha256`; the snapshot is embedded in the report's reproducibility zip with its own SHA-256 in the package manifest.
- `--allow-non-strict-civic` gate works in both directions (refuses without it, banner-annotates with it); manifest records the override (BD-R1).
- Methodology section credits civic data contributors above the algorithm description (CM-02), verified by the section-ordering test.
- Typst stderr is surfaced verbatim in the package manifest; non-empty stderr fails the render unless `--draft` (PP-32).
- One peer-reviewed sample (Louisiana Callais) is committed under `examples/louisiana-callais-walkthrough/expert_report_example/`.
- `docs/REDIST_CLI.md` documents every new flag; `CLAUDE.md` and `docs/CHANGELOG.md` reflect the addition.

---

## Risks

| Risk | Mitigation |
|---|---|
| Typst version drift between dev and CI produces different bytes | Pin in `.typst-version`; preflight asserts on every invocation (Task 3.4) |
| `verapdf` Java dependency is heavy for civic-advocate users on Windows | Document under "External tooling for PDF reports"; `--draft` mode skips verapdf and is explicitly NOT court-admissible |
| Determinism breaks because Typst injects build-host date into `xmpmeta` | Task 6.2 zeros all PDF metadata in the template + asserts byte-identical xmpmeta in tests |
| Per-section partials drift out of sync with the `inputs.json` schema | Per-section L0 tests (Task 9.1) render each partial in isolation against the schema |
| Hand-computed values in B-01 break when fixtures legitimately update | `expected_values.toml` keeps assertions data-driven; re-pinning is one file edit |
| Civic-gate logic incorrectly flags strict inputs as lenient | L0 test 9.7 covers both directions; manifest annotation makes false positives discoverable in review |
| `verapdf` rule failures are cryptic to non-engineers | Error category `[VERAPDF]` includes the rule ID + link to the PDF/A spec section in the actionable-hint per `docs/error-conventions.md` |
| Reproducibility zip blows up size on large states | 20-page in-PDF cap forces overflow into zip; document the practical zip size ceiling per state in `docs/court-reports-quickstart.md` |
| `sources.json` content-hash field requires backfilling | Task 7.5 schedules the schema bump; missing-hash sources render with "(content hash unavailable)" rather than blocking |

---

## Out of Scope

- WYSIWYG editing of the PDF (use `--draft` then hand-edit in Word; we do not own a round-trip path).
- Court-jurisdiction-specific layout variants beyond `--jurisdiction <STR>` swapping in alternate Typst templates (one generic template ships; per-jurisdiction is future work).
- Trial-exhibit numbering and case-management integrations.
- Auto-fill of case captions from court-records APIs.
- Localized (non-English) reports.
- A GUI front-end on top of the CLI surface.
- Replacing the existing `redist report --format html` path — HTML stays as the iteration / draft surface.
