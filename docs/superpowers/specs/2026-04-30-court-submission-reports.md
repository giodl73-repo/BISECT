# Court-Submission Reports: PDF Expert Witness Output
**Date:** 2026-04-30
**Status:** Proposed; pending review
**Closes gap for:** special master, §2 expert, civic advocacy
**Depends on:** existing redist-report HTML output + Callais Evidence Layer (for §2 cases)
**Estimated effort:** 3-4 days

## Why this exists

Courts and special masters consume PDFs, not JSON or HTML. Today the project produces `analysis/*.json` (machine-readable) and `redist report --format html` (browser-readable) but not court-formatted PDFs with the conventions expert witnesses use: cover page, methodology section, reproducibility appendix, signature block, citations.

A practitioner who has finished their analysis still has to take our HTML output, copy-paste into Word, format manually, and hope nothing changed in the data underneath. This spec eliminates that step.

## Scope

### In scope

1. **PDF rendering pipeline** — `redist report --format pdf` produces a court-ready document from the same data the HTML report uses.

2. **Standard sections** — cover page, executive summary, methodology, results, reproducibility appendix, signature block. Each section is templated and pulls from the analysis JSON.

3. **Reproducibility appendix** — automatically embeds:
   - `manifest.json` (input adjacency hash, run parameters, binary version)
   - `provenance.json` (binary version, build commit, build date, rustc version)
   - Verification command: `redist doctor --verify-manifest <path-to-manifest>`
   - SHA-256 of every input data file used

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
| Different jurisdictions want different formatting | Ship one generic template; allow `--jurisdiction` to swap in alternates over time |
| Auto-generated text may sound robotic / not legally defensible | Run early templates by a litigator; iterate |
| Citation accuracy depends on registry metadata | Reuse the same registry already powering the fetchers; metadata is in one place |

## Definition of done

- `redist report --format pdf --label vt_test` produces a clean PDF in <10 seconds
- PDF contains all standard sections; no template variables leak through
- Reproducibility appendix correctly references the manifest + provenance
- One peer-reviewed sample (the Louisiana Callais walkthrough from the Onboarding spec) is committed under `examples/`
