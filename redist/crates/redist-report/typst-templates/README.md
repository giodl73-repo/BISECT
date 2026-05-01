# Typst templates for `redist report --format pdf`

**Status:** SCAFFOLD ONLY (2026-04-30). The PDF render path is not yet executable.
**Owner:** Court Submission Reports plan (`docs/superpowers/plans/2026-04-30-court-submission-reports.md`)

This directory will hold the Typst document templates that produce the court-ready PDF/A-2b expert-witness report. The plan splits the document into per-section partials so each can be unit-tested in isolation:

- `expert_report.typ` — top-level document, imports the partials
- `_cover.typ` — court caption, expert name, jurisdiction, generation date
- `_methodology.typ` — "Data sources and contributors" subsection (CM-02 above-the-fold) plus algorithmic methodology
- `_results.typ` — VRA / partisan / bloc / compactness sections
- `_qualifications.typ` — expert CV (PDF inclusion via Typst)
- `_conflicts.typ` — declarative conflicts of interest from `expert.toml`
- `_prior_testimony.typ` — FRCP 26(a)(2)(B) table from `expert.toml.prior_testimony[]`
- `_daubert.typ` — Daubert self-assessment checklist
- `_reproducibility.typ` — manifest table + bootstrap link + common-failures checklist (C-04)
- `_signature.typ` — wet-signature line
- `_citations.typ` — consumes structured Citation { source_class, fields } objects per `docs/file-formats/citation-strings.md`

## What's deferred

Per the v2.1.1 plan partial-implementation policy, this directory ships ONLY the version pins and this README in the current commit. The actual Typst source files + the Rust render path live behind the following gates:

- **Typst install** — pinned at `.typst-version` (currently 0.12.0). The Rust runner shells out to the `typst` CLI; its absence is detected at PDF-render time and produces a `[CONFIG]` actionable error per `docs/error-conventions.md`.
- **`verapdf` install** — pinned at `.verapdf-version` (currently 1.26.2). Java-based PDF/A validator. Required for the `--draft=false` court-ready gate (PP-32). Absence is also a `[CONFIG]` error.
- **Template authoring** — see Court Reports plan Task 5 for the per-section content requirements. CM-02 enforces "Data sources and contributors" appears before any algorithmic methodology subheading.

## When to bring this online

The Court Submission Reports plan has 10 tasks; this scaffolding covers Tasks 3 (version pins) and 4 (CLI flags + module shape). Tasks 5-7 (templates, determinism, verapdf) require Typst installed in the dev environment + CI. The implementer should:

1. Install Typst 0.12.0 locally (`curl -fsSL https://typst.community/typst.sh | sh -s -- -y` or platform package manager)
2. Install verapdf 1.26.2 (`brew install verapdf` or download from https://verapdf.org/software/)
3. Author each `_*.typ` partial against the per-section L0 tests in `redist-report/tests/typst_render/`
4. Wire CI install steps in `.github/workflows/{pr,nightly,release}.yml` per the roadmap CI strategy
5. Land the determinism contract (D-04: `SOURCE_DATE_EPOCH`, sorted-name tar+gzip with fixed mtime, zeroed PDF metadata)
6. Land the verapdf gate (PP-32: refuse to write `.pdf` when verapdf rejects the bytes)

Until then, `redist report --format pdf` falls back to the legacy `try_generate_pdf` path (wkhtmltopdf/pandoc with a `[CONFIG]` error if neither is installed). The new CLI flags (`--expert-name`, `--jurisdiction`, etc.) are wired but currently advisory — the legacy path doesn't consume them.

## Cross-references

- `docs/superpowers/plans/2026-04-30-court-submission-reports.md` — full task list
- `docs/file-formats/manifests.md` — manifest field conventions consumed by `_reproducibility.typ`
- `docs/file-formats/citation-strings.md` — citation-style templates consumed by `_citations.typ`
- `docs/superpowers/plans/2026-04-30-onboarding-and-tutorials.md` Task 6.6 — `bootstrap.sh` time-estimates referenced in the rebuild instructions
