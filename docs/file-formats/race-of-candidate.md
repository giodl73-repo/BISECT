# Race-of-Candidate CSV — Schema and Curator Attestation Protocol

**Schema version:** `race-of-candidate v1`
**Status:** Implemented (Callais Evidence Layer Task 4)
**Owner:** `redist/crates/redist-analysis/src/race_of_candidate.rs`
**Spec:** `docs/superpowers/specs/2026-04-30-callais-evidence-layer.md`
**Plan:** `docs/superpowers/plans/2026-04-30-callais-evidence-layer.md` Task 4

This document is the source of truth for the curator-attested race-of-candidate annotation file consumed by `redist analyze --types bloc-voting`. Without this protocol, *Louisiana v. Callais* (608 U.S. ___, 2026-04-29) p.36 evidence is not Daubert-defensible — the analysis cannot rest on race classifications a court cannot trace to a named, credentialed curator with a signed attestation.

## CSV columns (all required, case-sensitive headers)

| Column | Type | Description |
|---|---|---|
| `candidate_name` | string | Candidate's full name as it appears on the ballot. Quote with `"..."` if it contains a comma. |
| `party` | string | Party identifier (e.g., `DEM`, `REP`). Combined with `candidate_name` for uniqueness. |
| `race` | enum | One of `Black`, `white`, `Hispanic`, `Asian`, `Native`, `other`. Case-sensitive. The list is closed by design — see "Why the race vocabulary is closed" below. |
| `curator` | string | Name of the human curator who classified this candidate. Must match across rows for the same person. |
| `curator_credentials` | string | Curator's relevant credential (e.g., `Ph.D., Demography`, `J.D., Voting Rights Litigator`). |
| `curator_attestation_date` | string | ISO-8601 date the curator attested to the classification (e.g., `2026-04-15`). |
| `source` | string | Free-text source reference (campaign website, news article URL, court filing, candidate self-identification, etc.). |
| `independently_verified` | bool | `true` or `false`. When **any** row is `false`, the orchestrator prepends `[CAVEAT — annotations not independently verified]` to the analysis's `draft_interpretation` field (B-02 anchor 4). |
| `attestation_doc_path` | string | Path to the curator's signed attestation document, **relative to the directory containing this CSV**. Reproducibility-portable. |
| `attestation_doc_format` | enum | One of `pdf`, `docx`, `md`, `txt`, `png`, `jpg`, `jpeg`, `tif`, `tiff`. The BD-R2 reconciled union — see "Accepted attestation-doc formats" below. |

## Accepted attestation-doc formats (BD-R2 union)

The v2.1.1 patches reconciled a prior mismatch between Callais and Civic Bidirectional plans (one wanted `pdf|docx|md|txt`, the other wanted `pdf|png|jpg|jpeg|tif|tiff`). The union accepts both lanes:

**Curator-narrative formats** (the curator writes a justification document):
- `pdf` — preferred for court submissions; tamper-evident; Bates-stampable
- `docx` — Microsoft Word; commonly used by expert witnesses
- `md` — Markdown for GitHub-tracked attestations
- `txt` — plain text fallback

**Civic-submitter scanned-letterhead formats** (a civic group submits a scanned signed letter):
- `png`, `jpg`/`jpeg`, `tif`/`tiff` — image scans of letterhead with curator signature

**Rejected formats**: `html`, `htm`, `rtf`. HTML and RTF are too easy to fake and offer no provenance. If you have an HTML page (e.g., a candidate's official biography), print to PDF first and submit that; record the original URL in the `source` column.

## Court-mode strictness (BD-R1)

Court Submission Reports (when the plan ships) gate non-PDF attestation docs behind an explicit `--allow-non-strict-civic` flag. Image scans (`png|jpg|tiff`) are accepted by `redist analyze` (we don't lose the data) but the court-PDF generator refuses to embed them in a court submission unless the operator opts in. Civic groups can still use the analyses internally; expert witnesses producing court evidence should re-attest in PDF form before signing.

## SHA-256 chain of custody (BD-R2)

At parse time, `redist analyze --types bloc-voting`:

1. Computes SHA-256 of the CSV file → recorded as `race_of_candidate_provenance.source_sha256` in `bloc_voting.json`.
2. Computes SHA-256 of every referenced attestation document → recorded as `attestation_documents[].sha256` in the same JSON.
3. Cross-checks each row's `attestation_doc_format` against the actual file extension; mismatch is a hard `[INPUT]` error.
4. Fails hard if any referenced attestation doc is missing.

The reproducibility-zip pipeline (Task 5) copies the CSV plus every unique attestation doc into `analysis/bloc_voting/{race_of_candidate.csv, attestations/<sha256-prefix>.<ext>}` so a special master can verify the chain end-to-end without re-fetching anything.

## Multi-curator dispute support

When two or more rows have the same `(candidate_name, party)` but different curators, the parser does NOT collapse them. It records the dispute in `AnnotationSet.disputes` and the orchestrator runs one analysis per curator's annotation set, surfacing both as `parallel_curator_runs` in the JSON output. The expert chooses which curator's classification to use in the headline number; both appear in the appendix.

This is more conservative than "majority vote among curators" and matches Daubert's preference for transparent, falsifiable methods over hidden aggregation.

## Why the race vocabulary is closed

The set `{Black, white, Hispanic, Asian, Native, other}` is closed because:

1. **Federal court convention.** §2 case law uses these categories; widening the vocabulary in a tool would introduce new ambiguity.
2. **Census alignment.** These map cleanly onto Census 2020 race + Hispanic-origin categories, which is the demographic data the bloc-voting regression uses on the precinct side.
3. **Daubert defensibility.** Custom labels invite challenges on classification methodology; a closed vocabulary is testable.

If you need a category outside this set (e.g., subdividing `Asian` into specific origin groups), use `other` and document the actual classification in the `source` column. This forces explicit acknowledgment that you are extending the schema, rather than silently doing so.

## Worked example: 2020 LA Democratic presidential primary

```csv
candidate_name,party,race,curator,curator_credentials,curator_attestation_date,source,independently_verified,attestation_doc_path,attestation_doc_format
"Biden, Joseph R.",DEM,white,Dr. Jane Smith,"Ph.D., Political Science, Tulane",2026-04-15,Wikipedia + campaign biography,true,attestations/Biden_Smith.pdf,pdf
"Sanders, Bernard",DEM,white,Dr. Jane Smith,"Ph.D., Political Science, Tulane",2026-04-15,Wikipedia + campaign biography,true,attestations/Sanders_Smith.pdf,pdf
"Booker, Cory",DEM,Black,Dr. Jane Smith,"Ph.D., Political Science, Tulane",2026-04-15,Wikipedia + campaign biography + candidate self-identification,true,attestations/Booker_Smith.pdf,pdf
"Castro, Julian",DEM,Hispanic,Dr. Jane Smith,"Ph.D., Political Science, Tulane",2026-04-15,Wikipedia + campaign biography + Spanish-surname analysis,true,attestations/Castro_Smith.pdf,pdf
"Yang, Andrew",DEM,Asian,Dr. Jane Smith,"Ph.D., Political Science, Tulane",2026-04-15,Wikipedia + campaign biography + candidate self-identification,true,attestations/Yang_Smith.pdf,pdf
"Patrick, Deval",DEM,Black,NAACP-LA Local Chapter,Civic submitter,2026-04-20,Letter of attestation on letterhead,true,attestations/Patrick_NAACP.png,png
```

The last row demonstrates a civic-submitter image attestation; the rest are curator-narrative PDFs.

## Reproducibility checklist for an expert witness

Before signing your bloc-voting analysis as evidence:

- [ ] Every row's `independently_verified` is `true`, OR you can defend the `[CAVEAT]` text on the stand.
- [ ] Every `attestation_doc_path` resolves to a file on disk relative to the CSV.
- [ ] Every attestation document contains, at minimum: curator name, credential, attestation date, and a per-candidate sentence justifying the classification.
- [ ] The reproducibility zip (`redist report --format pdf` will produce it) includes both the CSV and every attestation document.
- [ ] You have run `redist doctor --verify-manifest` on the analysis output and confirmed the binary version matches your court submission.

## Cross-references

- Implementation: `redist/crates/redist-analysis/src/race_of_candidate.rs`
- JSON Schema: `redist/crates/redist-analysis/schemas/bloc_voting.schema.json` (`race_of_candidate_provenance` block)
- Callais legal grounding: `docs/legal/CALLAIS_REFERENCE.md`
- Callais Evidence Layer plan: `docs/superpowers/plans/2026-04-30-callais-evidence-layer.md`
- Civic Bidirectional input plan (matched edit BD-R2): `docs/superpowers/plans/2026-04-30-civic-bidirectional.md`
- BD-R1 court-mode gating (Court Submission Reports plan): `docs/superpowers/plans/2026-04-30-court-submission-reports.md`
