# Citation String Conventions for `redist`

**Status:** Specification (consumed by Court Submission Reports plan when shipped).
**Owner:** Court Submission Reports plan Task 2 (docs/superpowers/plans/2026-04-30-court-submission-reports.md)
**v2.1 tracking:** D-06

This is the source of truth for the exact citation strings emitted by `redist report --format pdf` and (when the Researcher Toolkit plan ships) `redist analyze --paper-mode`. Every source class has Bluebook, APA, and Chicago templates with worked examples; the `--citation-style` flag selects between them.

If you're authoring a new source class (e.g., a new election-data registry entry, a new attestation-document type), add a `## §X — <source class>` subsection with all three template variants and a worked example.

---

## 1. Default style precedence

Selection rules, in order:

1. Explicit `--citation-style {bluebook|apa|chicago}` overrides everything.
2. `redist report --format pdf --jurisdiction <COURT>` → defaults to `bluebook` (any U.S. federal or state court). Bluebook is the conservative legal-citation default.
3. `redist analyze --paper-mode` → defaults to `apa`. APA is the dominant format in social-science journals.
4. `redist civic ingest` and the comment-period report → defaults to `chicago`. Chicago author-date is the default for community-document citation; not court, not paper.
5. Without any of the above, Bluebook is the conservative fallback (mismatching style in court is harder to defend than mismatching in a press release).

---

## 2. Common variables

These appear across multiple source classes; document once here.

| Variable | Type | Source |
|---|---|---|
| `${author}` | string | Curator/author of the source. For datasets with corporate authorship (e.g., Census), use the institutional name verbatim. |
| `${title}` | string | Source title in title case. |
| `${year}` | int | Year of publication. For datasets, the data vintage year (NOT the access year). |
| `${doi}` | string | Digital Object Identifier (`10.7910/DVN/Z8TSH3`). Prefixed with `https://doi.org/` in renderings. |
| `${url}` | string | Canonical URL for sources without a DOI. |
| `${accessed_date}` | string (`YYYY-MM-DD`) | Calendar date of access; derived from `fetched_at` per `docs/file-formats/manifests.md` §4. |
| `${publisher}` | string | For dataset citations, the dataset host (e.g., `Harvard Dataverse`). |
| `${commit_short}` | string | 8-char git SHA of the binary version that produced the citation. From `redist_build_commit_short`. |

---

## 3. Source classes and templates

### 3.1 Dataset with DOI (e.g., Fekrazad election data)

**Bluebook (default for jurisdiction):**

```
${author}, ${title} (${year}), https://doi.org/${doi} (last visited ${accessed_date}).
```

**APA:**

```
${author}. (${year}). ${title} [Data set]. ${publisher}. https://doi.org/${doi}
```

**Chicago author-date:**

```
${author}. ${year}. "${title}." ${publisher}. https://doi.org/${doi}.
```

**Worked example** (Fekrazad's "Replication Data for: Statewide Geographies"):

- Bluebook: `Hossein Fekrazad, Replication Data for: Statewide Geographies (2025), https://doi.org/10.7910/DVN/Z8TSH3 (last visited Apr. 30, 2026).`
- APA: `Fekrazad, H. (2025). Replication Data for: Statewide Geographies [Data set]. Harvard Dataverse. https://doi.org/10.7910/DVN/Z8TSH3`
- Chicago: `Fekrazad, Hossein. 2025. "Replication Data for: Statewide Geographies." Harvard Dataverse. https://doi.org/10.7910/DVN/Z8TSH3.`

### 3.2 Dataset without DOI (URL only — e.g., Census TIGER)

**Bluebook:**

```
${author}, ${title}, ${url} (last visited ${accessed_date}).
```

**APA:**

```
${author}. (${year}). ${title} [Data set]. ${url}
```

**Chicago author-date:**

```
${author}. ${year}. "${title}." Accessed ${accessed_date}. ${url}.
```

**Worked example** (TIGER 2020 Vermont tracts):

- Bluebook: `U.S. Census Bureau, TIGER/Line Shapefile, 2020, State, Vermont, Census Tract, https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_50_tract.zip (last visited Apr. 30, 2026).`
- APA: `U.S. Census Bureau. (2020). TIGER/Line Shapefile, 2020, State, Vermont, Census Tract [Data set]. https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_50_tract.zip`
- Chicago: `U.S. Census Bureau. 2020. "TIGER/Line Shapefile, 2020, State, Vermont, Census Tract." Accessed April 30, 2026. https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_50_tract.zip.`

### 3.3 Software (this binary)

For citing the `redist` binary itself in expert reports.

**Bluebook:**

```
redist v${redist_version} (commit ${commit_short}), https://github.com/<owner>/redist (last visited ${accessed_date}).
```

**APA:**

```
Della-Libera, G. (${year}). redist (Version ${redist_version}, commit ${commit_short}) [Computer software]. https://github.com/<owner>/redist
```

**Chicago author-date:**

```
Della-Libera, Gio. ${year}. "redist (Version ${redist_version}, commit ${commit_short})." https://github.com/<owner>/redist.
```

**Worked example** (Vermont 2020 walkthrough):

- Bluebook: `redist v0.1.0 (commit dee93fa5), https://github.com/<owner>/redist (last visited Apr. 30, 2026).`
- APA: `Della-Libera, G. (2026). redist (Version 0.1.0, commit dee93fa5) [Computer software]. https://github.com/<owner>/redist`
- Chicago: `Della-Libera, Gio. 2026. "redist (Version 0.1.0, commit dee93fa5)." https://github.com/<owner>/redist.`

### 3.4 Expert deposition transcript

For expert witnesses citing their own prior testimony per FRCP 26(a)(2)(B).

**Bluebook:**

```
${author} Dep., ${case_name}, No. ${case_number} (${court}, ${date}).
```

**APA + Chicago:** rare in academic / civic contexts; if needed, fall back to Bluebook even when the document is in APA mode (deposition citations are inherently legal and don't have established academic forms).

**Worked example:**

- `Smith Dep., Plaintiffs v. Louisiana, No. 22-cv-00211 (E.D. La., Mar. 14, 2025).`

### 3.5 Civic counter-proposal (Civic Bidirectional plan output)

For citing a civic group's counter-proposal in a comment-period report or press release.

**Chicago author-date (default):**

```
${submitted_by}. ${year}. "${plan_label}." Civic counter-proposal submitted ${submitted_at}. Available at: ${comparison_report_url}.
```

**APA (paper-mode):**

```
${submitted_by}. (${year}). ${plan_label} [Civic counter-proposal]. ${comparison_report_url}
```

**Bluebook (court-mode, requires `--allow-non-strict-civic`):**

```
${submitted_by}, ${plan_label}, ${comparison_report_url} (civic counter-proposal submitted ${submitted_at}).
```

**Worked example:**

- Chicago: `League of Women Voters of Louisiana. 2026. "lwvla_alternative_2026." Civic counter-proposal submitted 2026-04-15. Available at: https://lwvla.example.org/our-map.`

---

## 4. Multi-source ordering

Citation lists with > 1 source follow these rules:

- **Joiner:** semicolon (Bluebook), period (APA, Chicago).
- **Order:** chronological by `year` (oldest first); within the same year, alphabetical by `author`. Primary sources before secondary.
- **Source-class grouping:** when ≥ 5 citations, group by source class (Datasets, Software, Depositions, Civic) with a heading per class.

---

## 5. Errata footnotes

When `scripts/data/elections/sources.json` carries a `notes` field flagging dataset limitations (e.g., "tract boundaries may not match 2020 Census revisions"), the citation string emits an inline footnote anchor `[note ${n}]` and a footnote at the bottom of the citations section with the verbatim text.

The Typst template consumes this via the `_citations.typ` partial; the markdown narrative consumes it via `pulldown-cmark`'s footnote extension.

This is required: Daubert exposure if the dataset's documented limitations are not surfaced.

---

## 6. ASCII / Unicode policy

Citation strings written to files (PDF, Markdown, HTML) MAY use Unicode (em-dashes, smart quotes, etc.) — the consumer reads UTF-8.

Citation strings emitted to stdout/stderr (e.g., `redist analyze --paper-mode --stdout`) MUST be ASCII-only on Windows per PP-34. The renderer normalizes em-dashes to `--` and smart quotes to ASCII quotes when writing to a TTY.

This is enforced by the same convention as `docs/error-conventions.md` §"Windows console policy".

---

## 7. Implementation notes

- The Court Submission Reports plan's `_citations.typ` Typst partial consumes structured citation data, not pre-rendered strings. The renderer emits structured `Citation { source_class, fields }` objects; Typst formats them per the active `--citation-style`.
- The Plan Comparison plan's `narrative.md` consumes the same structured citation data via Tera; the rendered markdown matches the Typst output byte-for-byte (modulo footnote markup syntax).
- Validation: every citation string the binary emits MUST be byte-identical across two renders with the same inputs (per Plan Comparison plan Task 12 byte-identical re-render gate). Use `BTreeMap` for any field maps; never re-order arrays.

---

## 8. See also

- `docs/file-formats/manifests.md` §4 (`accessed_date` vs `fetched_at`)
- `docs/superpowers/plans/2026-04-30-court-submission-reports.md` Tasks 2, 5.11, 6.6, 9.4
- `docs/superpowers/plans/2026-04-30-researcher-toolkit.md` Task 8.2 (CITATION.bib.tmpl)
- `docs/superpowers/specs/2026-04-30-v21-tracking.md` D-06
