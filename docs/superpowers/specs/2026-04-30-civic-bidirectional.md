# Civic Bidirectional Input: Treat the Public as a Data Source, Not Just an Audience
**Date:** 2026-04-30
**Status:** Proposed; pending review (NEW spec from COMMONS 9-role review)
**Closes gap for:** civic advocate (★★→★★★★★) — the publish-only model only gets to ★★★★
**Depends on:** Plan Comparison & Narrative (read-side), State Staff Interop (civic bypass path)
**Estimated effort:** 3-4 days

## Why this exists

Every other spec in this roadmap treats the civic advocate as a *consumer* of redistricting analysis: they read the comparison report, they share the summary card, they cite the PDF. That's a one-way street, and it leaves the most valuable thing a civic group has on the floor: **local knowledge**.

A neighborhood association in Lafayette knows which blocks belong to the same community of interest. A church in Birmingham can name the pastor who's been doing get-out-the-vote work in three precincts that the official precinct file lists as separate. A civil-rights nonprofit has the candidate-race information that the state board of elections refuses to publish.

Today, none of that flows into the analysis. The advocate can write a press release saying "Plan B splits our community" but they cannot put their *evidence* into the analytical record in a way that the report consumes. This spec opens the inbound pipe.

The Plan Comparison spec already added `redist compare --comments <CSV>` (read side). This spec defines the write side: how civic groups produce that CSV, validate it, and submit it into the workflow without going through state staff.

## Scope

### In scope

1. **Community-of-interest CSV format**
   ```csv
   geoid,comment_id,label,source,source_url,confidence,submitted_at
   22055951400,COI-LAF-001,Downtown Lafayette,Lafayette Neighborhood Coalition,https://lnc.example.org/comment/123,high,2026-04-15
   22055951500,COI-LAF-001,Downtown Lafayette,Lafayette Neighborhood Coalition,https://lnc.example.org/comment/123,high,2026-04-15
   22055951600,COI-LAF-002,Mills District,Lafayette Neighborhood Coalition,,medium,2026-04-15
   ```
   - `comment_id` groups tracts into one community
   - `label` is the human name of the community
   - `source` is the submitting org (the source of record)
   - `source_url` lets a reader audit the underlying public comment
   - `confidence` is `high|medium|low` — the submitter's self-assessment of how well-defined the boundary is
   - `submitted_at` is ISO-8601

2. **`redist civic ingest` command**
   ```
   redist civic ingest comments.csv \
     --year 2020 --state LA \
     --label public_comments_2026_round_1 \
     --validate strict
   ```
   Validates:
   - Every GEOID exists in the year/state's tract set (typo guard)
   - No GEOID is in two different `comment_id`s without an explanatory note
   - `comment_id` membership is contiguous (warn-only — sometimes a community legitimately straddles non-adjacent tracts)
   - URL fields parse as URLs; localhost/private IPs are rejected
   - Writes to `outputs/{version}/civic_inputs/{label}/` with manifest, validation log, and a normalized canonical CSV

3. **Public-comment-period report mode**
   `redist report --comment-period --plan LABEL --comments-label LABEL` produces a report variant tailored to a public-comment hearing:
   - Front section: "comments received and addressed" (each `comment_id` + which districts honor vs. split it)
   - Per-district section: "communities of interest within this district per public comment"
   - Appendix: full submitter list with org name, URL, submission date — provenance for the public record

4. **Race-of-candidate annotation contributions**
   Civic groups (NAACP-LDF chapters, ACLU, etc.) are the highest-quality source for candidate-race annotations in jurisdictions where the state board of elections refuses to publish them.
   ```
   redist civic add-candidate-race candidates.csv --year 2020 --state LA \
     --submitter "NAACP Louisiana State Conference" \
     --attestation-doc path/to/letter_of_attestation.pdf
   ```
   - Goes into the same `race_of_candidate.csv` schema the Callais Evidence Layer consumes
   - Stored with submitter + attestation document + ingestion timestamp
   - Conflict resolution: when two submitters disagree on a candidate's race, the analysis flags the conflict in the report (does NOT silently pick one)
   - Cross-references the Callais Evidence Layer's `race_of_candidate_provenance.json` schema

5. **Civic-counter-proposal flow** — combines this spec with State Staff Interop's `--as-civic-counter-proposal` and Plan Comparison's `--comments` overlay:
   ```
   # Civic group flow (no state staff in the loop):
   redist import --format districtr theirplan.json \
     --as-civic-counter-proposal \
     --submitted-by "League of Women Voters Louisiana" \
     --label lwvla_alternative_2026
   redist civic ingest community_comments.csv --label lwv_comments
   redist compare --plan-a state_proposal --plan-b lwvla_alternative_2026 \
     --comments lwv_comments \
     --format html,narrative,card
   ```
   Produces a publishable comparison the advocate can submit to the public-comment record without depending on state staff cooperation.

6. **Validation policy modes** — `--validate {strict|lenient|advisory}`:
   - `strict` (default): any error fails the ingest
   - `lenient`: warnings allowed; manifest records the count
   - `advisory`: only validates structure; semantic warnings are summary-only
   - The selected mode is recorded in the manifest so a downstream reader knows the bar

### Out of scope

- A web form for civic groups to submit comments (UI; out of scope; civic group's existing infra fills the CSV)
- Automated translation of free-text public comments into structured COI entries (NLP territory; civic group does the translation)
- Authentication / credential checks for submitters (we record `submitted_by` as free text; chain of custody is the submitter's responsibility, not ours)
- Litigation-grade evidentiary chain for civic submissions (those are not court-grade unless the submitter takes them through an expert; we record provenance, we do not assert it)

## Implementation notes

### Why not a database?

A flat CSV + a manifest is auditable end-to-end with `git diff` and SHA-256. A database makes provenance harder to reproduce. Civic groups can author the CSV in Excel / Sheets / a Python notebook — accessible.

### Confidence semantics

The `confidence` field is *the submitter's self-assessment*, not our inference. The narrative can say "the Lafayette Neighborhood Coalition rated this community boundary as high-confidence" without making a claim of our own. This is the same legal posture as the race-of-candidate provenance protocol.

### Conflict resolution for race-of-candidate

When two civic groups submit different races for the same candidate:
1. The analysis halts the candidate-race lookup for that candidate
2. The Callais report's race-of-candidate appendix records both attributions + the disagreement
3. The bloc-voting regression is re-run with that candidate excluded as a sensitivity check; both results (with + without) appear in the report
4. The expert decides which to use in the headline number (signature on the line)

This is more conservative than "majority vote among submitters" and matches Daubert's preference for transparent, falsifiable methods over hidden aggregation.

### Public-comment-period report templating

Reuses the Court Submission Reports infrastructure with a different Typst template:
- `redist-report/templates/comment-period.typ`
- Same data sources, different presentation
- No expert-witness signature block; the comment-period report is a community document, not an evidentiary one

## Outputs

```
outputs/{version}/civic_inputs/{label}/
├── manifest.json                  # Submitter, validation mode, source URL, SHAs
├── normalized.csv                 # Canonical form (GEOID-sorted, schema-checked)
├── original.csv                   # Verbatim copy of the submitted file
├── validation_log.txt             # Per-row validation outcomes
└── conflicts.json                 # Cross-label conflicts (if any)

outputs/{version}/states/{state}/reports/
└── comment_period_report.pdf      # NEW report variant
```

## CLI surface

```
redist civic ingest <CSV> --year YYYY --state ST --label LABEL \
  [--validate {strict|lenient|advisory}] \
  [--submitter "Org Name"] \
  [--source-url URL]

redist civic add-candidate-race <CSV> --year YYYY --state ST \
  --submitter "Org Name" --attestation-doc PATH

redist civic list                                # show ingested civic inputs
redist civic show --label LABEL                  # show one input + validation
redist civic conflicts --label LABEL_A --label LABEL_B   # show overlaps + disagreements

redist report --comment-period --plan LABEL --comments-label LABEL
```

## Tests

- L0 **schema validation**: malformed CSV rows produce specific error messages naming the offending row + column
- L0 **GEOID typo guard**: include a fake GEOID; assert ingest fails in strict mode and warns in lenient
- L0 **localhost URL rejection**: `source_url=http://127.0.0.1/...`; assert non-zero exit
- L0 **conflict detection**: ingest two civic inputs that disagree on a tract's COI label; assert `redist civic conflicts` finds the disagreement
- L0 **race-of-candidate conflict**: two submitters disagree on a candidate's race; assert the Callais Evidence Layer flags + runs the with/without sensitivity check
- L0 **report contains comments**: render a comment-period report on synthetic input; assert the "comments received and addressed" section names every `comment_id`
- L1: full civic-counter-proposal flow on a synthetic VT plan + 5 civic comments; assert the comparison report's narrative honors the comment-incorporation appendix
- L2: skipped-by-default — ingest a real public-comment dataset (Louisiana 2024 round, if available); produce a comment-period report

## Risks

| Risk | Mitigation |
|---|---|
| Civic groups submit garbage / spam to inflate their narrative | Validation modes; manifest records submitter; downstream reader sees the bar; bad submissions are visible in `redist civic list` |
| Two groups submit overlapping but different communities (legitimate disagreement) | `redist civic conflicts` surfaces it; report shows both with attribution; we do not adjudicate |
| Race-of-candidate contributions become a vector for biased annotations | Attestation document required; conflict-aware sensitivity check; expert signs off on headline number |
| Ingestion runs ahead of state-staff approval and confuses readers about authority | `as-civic-counter-proposal` tag is loud in every downstream report; comparison-report narrative leads with "civic counter-proposal" framing |
| Submitter URL goes 404 between submission and trial | We snapshot the URL contents into the manifest at ingestion time (with submitter consent); reproducibility package embeds the snapshot |
| Civic infrastructure (CSV authoring) is too technical for grassroots groups | Ship a Google Sheets template + a one-page how-to alongside the spec; the validator's error messages are written for non-engineers |

## Definition of done

- `redist civic ingest` validates + normalizes a CSV submission with full provenance
- `redist civic add-candidate-race` accepts a submission with an attestation document; conflicts trigger the sensitivity check
- `redist report --comment-period` produces a non-evidentiary report that references every `comment_id` it consumed
- A worked civic-counter-proposal example is committed under `examples/civic-counter-proposal/` with sample CSV + sample comparison output
- Google Sheets template + plain-English how-to committed under `docs/civic/`
- One real civic group (not us) ingests a sample CSV without our help; we record the friction
