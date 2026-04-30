# Court-Submission Reports: PDF Expert Witness Output
**Date:** 2026-04-30
**Updated:** 2026-04-30 (v2 — incorporates SURVEY/COVENANT/DATUM/BOUNDARY/TRENCH revisions)
**Status:** Revised; pending re-review
**Closes gap for:** special master, §2 expert, civic advocacy
**Depends on:** existing redist-report HTML output + Callais Evidence Layer (for §2 cases)
**Estimated effort:** 4-5 days (v2: PDF/A + expert conventions)

## Why this exists

Courts and special masters consume PDFs, not JSON or HTML. Today the project produces `analysis/*.json` (machine-readable) and `redist report --format html` (browser-readable) but not court-formatted PDFs with the conventions expert witnesses use: cover page, methodology section, reproducibility appendix, signature block, citations.

A practitioner who has finished their analysis still has to take our HTML output, copy-paste into Word, format manually, and hope nothing changed in the data underneath. This spec eliminates that step.

## Scope

### In scope

1. **PDF rendering pipeline** — `redist report --format pdf` produces a court-ready document from the same data the HTML report uses.

2. **Standard sections (v2 expanded per SURVEY)** — cover page, executive summary, methodology, results, **expert qualifications (CV attachment)**, **declaration of conflicts of interest**, **prior-testimony list (last 5 years per FRCP 26(a)(2)(B))**, **Daubert-readiness self-assessment**, reproducibility appendix, signature block. Each section is templated and pulls from the analysis JSON or expert-config.

3. **Reproducibility appendix (v2 detailed per COVENANT)** — automatically embeds:
   - `manifest.json` with explicit per-input-file SHA-256 + fetch date schema (every input file gets a row: path, sha256, fetched_at, source_url)
   - `provenance.json` (binary version, build commit, build date, rustc version)
   - **Race-of-candidate CSV** when bloc-voting analysis is included (BOUNDARY blocker — without this, evidence is not Daubert-defensible)
   - Verification command: `redist doctor --verify-manifest <path-to-manifest>`
   - **Step-by-step rebuild instructions** for a non-engineer special master. **Recommended path** (5–15 min on first build): use the bootstrap script from the Onboarding spec — `bash bootstrap.sh` (Linux/macOS) or `bootstrap.bat` (Windows), which handles `rustup`, METIS, the locked toolchain, and PATH setup. **Manual fallback** (30–60 min, for environments where running scripts is restricted):
     ```
     # Prereqs: rustup (https://rustup.rs), C compiler, METIS dev headers
     git clone <repo-url>
     git checkout <build_commit>
     cd redist && cargo build --release --locked
     ./target/release/redist doctor --verify-manifest <path>
     ```
     Common failures (`linker not found` → install build-essential / Xcode CLT / VS Build Tools; METIS missing → `apt install libmetis-dev` / `brew install metis` / Windows uses vendored copy) are documented in `docs/error-conventions.md`.
   - **Page limit**: appendix is bounded to 20 pages of the main PDF; full audit data is in the sidecar `reproducibility_package.zip`. The PDF references the zip's SHA-256.
   - **Path portability** (TRENCH): every embedded path is relative to the package root. Absolute paths (`C:\Users\...`, `/home/...`) are rewritten at report-generation time.

4. **Citations** — automatic citation generation for the data sources used (e.g., "Election data from Fekrazad 2025, DOI 10.7910/DVN/Z8TSH3, accessed 2026-04-29"). Pulled from the election-source registry.

5. **Per-analysis-type templates**:
   - VRA compliance (existing `vra_analysis.json` → MM district table + plain-English assessment)
   - Partisan metrics (existing `partisan.json` → efficiency gap table + interpretation)
   - **Bloc voting** (new from Callais Evidence Layer → regression coefficients + Callais-significance interpretation)
   - Compactness (existing `compactness.json` → Polsby-Popper / Reock distribution + state baseline comparison)

6. **Signature block** — placeholder for expert witness signature, date, qualifications. Configurable via `--expert-name`, `--expert-credentials`, `--expert-affiliation` CLI flags or an `expert.toml` config file.

### Out of scope

- WYSIWYG editing (use the `redist report --format html` output as a draft, then fix in Word if needed)
- Court-jurisdiction-specific formatting (Eastern District of Louisiana has different conventions than Northern District of Georgia; we ship a generic template, expert customizes)
- Automated trial-exhibit numbering (case-management software territory)
- Case caption auto-fill (requires per-case metadata; out of scope)

## PDF/A compliance (v2 — SURVEY blocker)

All generated PDFs target **PDF/A-2b** for federal court archival permanence. Implementation:
- Use Typst's PDF/A export mode (Typst ≥ 0.12 supports this)
- Validate every generated PDF with `verapdf` (open-source PDF/A validator) before reporting success
- If validation fails, report the violation and refuse to mark the file as court-ready

A non-PDF/A `--draft` mode is available for iteration but warns that the output is not court-admissible.

## Citation format (v2 — DATUM gap)

CLI flag `--citation-style {bluebook,apa,chicago}` with the following defaults:
- `redist report --format pdf --jurisdiction <COURT>` → defaults to `bluebook` for any U.S. court
- `redist report --format pdf --paper-mode` → defaults to `apa`

Citation generator pulls metadata from the election-source registry (`scripts/data/elections/sources.json`) and the per-input-file manifest entries. Every cited source includes: author/curator, title, year, DOI/URL, accessed date.

Known errata (when the registry has a `notes` field flagging dataset limitations) are auto-included as a footnote.

## Implementation choice

**Use Typst, not LaTeX.**

Typst has emerged as the modern alternative to LaTeX for programmatically-generated documents. Reasons:

- One-binary install (no TeX Live)
- Faster compile (~100ms vs LaTeX's seconds)
- Modern templating language (closer to Rust's or Python's f-strings than LaTeX macros)
- Active development; reasonable docs
- Already has redistricting/legal templates in the wild

Alternatives considered:
- **LaTeX**: industry-standard but slow, painful templating, large dep
- **Pandoc + Markdown → PDF**: easy but limited control over court-required formatting
- **Direct PDF generation in Rust**: too low-level for the date

Typst writes plain `.typ` files; we ship templates and a small Python or Rust wrapper that fills them with values from the analysis JSON.

## Outputs

```
outputs/{version}/states/{state_name}/reports/
├── expert_report.pdf            # Court-formatted document (NEW)
├── expert_report.typ            # Source for the PDF (debugging / customization)
├── reproducibility_package.zip  # Self-contained: PDF + JSONs + verify command
└── signed_summary.html          # Existing HTML report (unchanged)
```

The reproducibility package is a single zip containing everything a special master needs to verify: the PDF, all input JSONs, the manifest, the provenance, and a `README.md` with the exact `redist doctor --verify-manifest` command.

## CLI surface

```
redist report --label LABEL --format pdf [options]

Options:
  --expert-name "Dr. Jane Smith"
  --expert-credentials "Ph.D., Statistics, ..."
  --expert-affiliation "Princeton Gerrymandering Project"
  --case-caption-file path/to/caption.txt    # optional; freetext above title
  --jurisdiction "EDLA"                       # affects template tweaks (page size, margins)
  --output-dir <DIR>                          # default: outputs/.../reports/
```

## Tests

- L0: template rendering with synthetic data; assert all sections present, no NaN/null leaks
- L1: full PDF generation pipeline on Louisiana sample (mocked Callais results, verify the produced PDF parses and contains expected sections)
- L2: skipped-by-default test that runs the full pipeline + opens the PDF + checks page count is reasonable

## Risks

| Risk | Mitigation |
|---|---|
| Typst may break compatibility | Pin Typst version; document upgrade procedure |
| Typst rendering produces malformed PDFs without erroring (TRENCH PP-21) | Mandatory `verapdf` validation gate after every render; PDF checksums; CI fails on any Typst stderr |
| Different jurisdictions want different formatting | Ship one generic template; allow `--jurisdiction` to swap in alternates over time |
| Auto-generated text may sound robotic / not legally defensible | Mark draft text explicitly `[DRAFT]`; expert rewrites before signing |
| Citation accuracy depends on registry metadata | Reuse the same registry already powering the fetchers; metadata is in one place |
| Citation URLs go stale between report-generation and trial | Reproducibility package embeds a snapshot of the registry at generation time; `redist doctor --verify-manifest` checks URLs for 404 |
| Embedded paths non-portable across OS / users | Path-rewriting pass at report-generation time; only relative paths in the PDF |
| PDF appendix grows unbounded | 20-page hard cap inside the PDF; overflow goes to `reproducibility_package.zip` with SHA-256 reference |

## Definition of done

- `redist report --format pdf --label vt_test` produces a clean PDF in <10 seconds
- PDF contains all standard sections; no template variables leak through
- Reproducibility appendix correctly references the manifest + provenance
- One peer-reviewed sample (the Louisiana Callais walkthrough from the Onboarding spec) is committed under `examples/`
