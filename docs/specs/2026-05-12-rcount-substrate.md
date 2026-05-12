# RCOUNT Substrate Spec

Status: draft
Date: 2026-05-12
Track: `research/tracks/V-election-audit`
Role review: `docs/specs/reviews/rcount-track-r1_roles.md`
First crate slices: `crates/rcount-core`, `crates/rcount-io`,
`crates/rcount-audit`
CLI slice: `crates/rcount-cli`

## Purpose

RCOUNT is a reproducible election-count package format and verifier path.
It lets third parties replay public count claims from lower-level evidence:
source exports, normalized records, reconciliation equations, status events,
and canonical hashes.

RCOUNT verifies declared evidence packages. It does not replace official
canvass, recount, judicial, or certification authority.

## Non-Goals

RCOUNT does not:

- certify an election;
- prove that voting-system software was malware-free;
- prove that a ballot-marking device captured voter intent;
- publish individual voter choices;
- create vote-choice receipts;
- decide voter eligibility or ballot acceptance;
- replace risk-limiting audits, recounts, canvassing boards, or courts.

## Package Layers

An RCOUNT package is a directory or archive with typed file roles.

```text
package.rcount/
  manifest.json
  sources/
    source-index.json
    <raw public exports>
  normalized/
    contests.ndjson
    reporting-units.ndjson
    batches.ndjson
    lineage.ndjson
    ballot-manifest.ndjson
    cvr.ndjson
    summaries.ndjson
  reconciliation/
    equations.ndjson
    deltas.ndjson
  status/
    events.ndjson
  proofs/
    package-hashes.json
    inclusion-policy.json
  transcripts/
    verify-transcript.json
```

Only `manifest.json`, `sources/source-index.json`,
`normalized/contests.ndjson`, `normalized/reporting-units.ndjson`,
`normalized/summaries.ndjson`, `reconciliation/equations.ndjson`,
`status/events.ndjson`, and `proofs/package-hashes.json` are required for the
smallest summary-only package. CVR, ballot-manifest, and voter-facing proof
files are optional and gated by privacy rules.

## Canonical Hashing

Every hash uses:

- SHA-256 unless the manifest declares a later supported algorithm;
- a domain-separated ASCII prefix;
- canonical UTF-8 bytes;
- sorted JSON object keys for JSON objects;
- NDJSON records sorted by declared canonical key;
- no timestamps or local paths in content hashes.

Domain prefixes:

| Prefix | Meaning |
|---|---|
| `RCOUNT_SOURCE_V1\0` | raw source file bytes |
| `RCOUNT_RECORD_V1\0` | normalized canonical record |
| `RCOUNT_FILE_V1\0` | canonical normalized file projection |
| `RCOUNT_PACKAGE_V1\0` | package content hash |
| `RCOUNT_EVENT_V1\0` | status/canvass event |
| `RCOUNT_PROOF_V1\0` | public proof object |

The package records both:

- `package_file_sha256`: hash of the archive or directory manifest as written;
- `package_content_hash`: stable hash excluding volatile metadata.

## Core Records

### Manifest

```json
{
  "rcount_version": "0.1-draft",
  "jurisdiction": {
    "country": "US",
    "state": "WA",
    "county": "King"
  },
  "election": {
    "date": "2026-11-03",
    "type": "general",
    "scope": "county"
  },
  "status": "canvassed",
  "hash_algorithm": "sha256",
  "content_hash": "sha256:...",
  "created_by": {
    "tool": "rcount-cli",
    "version": "0.1-draft"
  }
}
```

Allowed status values:

- `unofficial`;
- `canvassed`;
- `recounted`;
- `amended`;
- `certified`;
- `withdrawn`;
- `superseded`.

### Contest

```json
{
  "contest_id": "wa-king-2026-county-exec",
  "title": "County Executive",
  "vote_for": 1,
  "selections": [
    {"selection_id": "cand-a", "kind": "candidate", "label": "Candidate A"},
    {"selection_id": "cand-b", "kind": "candidate", "label": "Candidate B"},
    {"selection_id": "write-in", "kind": "write_in_bucket", "label": "Write-in"}
  ]
}
```

### Reporting Unit

```json
{
  "reporting_unit_id": "king:precinct:SEA-1234",
  "kind": "precinct",
  "parent_jurisdiction": "king",
  "source_ids": ["SEA-1234"],
  "valid_from": "2026-11-03",
  "valid_to": null
}
```

Allowed reporting-unit kinds:

- `precinct`;
- `split_precinct`;
- `vote_center`;
- `central_count_batch`;
- `mail_batch`;
- `provisional_batch`;
- `jurisdiction_total`;
- `district_total`.

### Summary

```json
{
  "contest_id": "wa-king-2026-county-exec",
  "reporting_unit_id": "king:precinct:SEA-1234",
  "batch_id": null,
  "status": "canvassed",
  "totals": [
    {"selection_id": "cand-a", "votes": 420},
    {"selection_id": "cand-b", "votes": 397},
    {"selection_id": "write-in", "votes": 2}
  ],
  "undervotes": 11,
  "overvotes": 1,
  "blank_contests": 0
}
```

### Status Event

```json
{
  "event_id": "event-0007",
  "event_type": "provisional_adjudication",
  "status_before": "unofficial",
  "status_after": "canvassed",
  "effective_at": "2026-11-12T18:22:00Z",
  "authority": "King County Canvassing Board",
  "source_refs": ["source:canvass-minutes-2026-11-12"],
  "explanation": "Accepted 14 provisional ballots after eligibility review."
}
```

Event types include:

- `initial_unofficial_report`;
- `late_mail_batch_added`;
- `provisional_adjudication`;
- `ballot_cure_update`;
- `duplicate_ballot_resolution`;
- `write_in_adjudication`;
- `recount_update`;
- `court_order`;
- `certification`;
- `amended_certification`;
- `correction`.

## Reconciliation Equations

RCOUNT verifiers evaluate declared equations against normalized records.

Minimum equations:

```text
contest_selection_sum(reporting_unit, contest)
  = sum(selection votes) + undervotes + overvotes + blank_contests

jurisdiction_contest_total(contest)
  = sum(reporting_unit contest totals)

accepted_ballots(reporting_unit)
  = counted_ballots + rejected_after_acceptance_adjustments
```

The exact equation set is package-profiled. A summary-only public package may
not have enough information to verify ballot-manifest equations; it must report
those checks as `not_present`, not `pass`.

## Correlation Checks

RCOUNT gets stronger when several independently produced surfaces agree:
precinct summaries, batch manifests, CVR exports, canvass events, jurisdiction
totals, precinct lineage, and district aggregations. A verifier should treat
agreement across those surfaces as the gold path, and should report exactly
which surface breaks when they diverge.

Baseline correlation checks:

| Check | Compares | Why it matters |
|---|---|---|
| `contest_selection_sum` | candidate/write-in/residual counts to counted ballots | catches local arithmetic and residual mistakes |
| `jurisdiction_contest_total` | precinct or batch rows to jurisdiction totals | catches dropped, duplicated, or changed reporting units |
| `status_transition_delta` | unofficial/canvassed/recounted snapshots to status events | requires movement between counts to be explained by a public event |
| `batch_summary_total` | batch manifests to reported summaries | catches missing mail, provisional, central-count, or vote-center batches |
| `source_hash_match` | raw exports to source index hashes | catches tampered or substituted source files |
| `lineage_conservation` | prior-cycle precincts to current-cycle reporting units | catches unexplained precinct splits, merges, or boundary changes |
| `district_aggregation_total` | RCOUNT units plus RPLAN/RCTX to district totals | catches plan/crosswalk errors when reporting by district |
| `proof_privacy_gate` | public proof objects to privacy policy | prevents voter-facing proofs from becoming vote-choice receipts |

Multiple counts help only when they are not all the same derived artifact. The
stronger pattern is a ladder:

```text
raw public export hashes
  -> normalized records
  -> per-unit arithmetic
  -> jurisdiction totals
  -> status-event deltas
  -> optional batch/CVR/proof/lineage/district checks
```

The verifier should label correlation failures by equation id and by the
surface pair being compared, so an auditor can tell whether the problem is a
math error, a missing source, an unexplained canvass movement, a lineage break,
or a privacy violation.

## Privacy and Public Proofs

Baseline RCOUNT supports package and batch tamper-evidence. Voter-facing
inclusion proofs are optional and must be receipt-safe.

Baseline rule:

```text
A public proof may show that an eligible record or anonymized ballot-accounting
token is included in a counted population. It must not prove candidate choices.
```

Required privacy gates:

- no proof object may include candidate selections;
- no proof object may combine voter identity with ballot style and timestamp;
- small reporting units must suppress or aggregate proof detail when linkability
  risk is high;
- salts/nonces must have clear ownership and publication rules;
- threat model must name coercion, vote buying, insider substitution, parser
  substitution, truncation, and equivocation.

## RPLAN Boundary

Base RCOUNT packages do not require RPLAN. V.8 attaches RPLAN/RCTX only when a
user wants district vote aggregation:

```text
RCOUNT summaries + RPLAN assignment + RCTX unit graph/context
  -> district contest totals
  -> district aggregation transcript
```

The district aggregation transcript must hash:

- RCOUNT package content hash;
- RPLAN content hash;
- RCTX content hash;
- aggregation code version;
- unit crosswalk source hashes.

## Crate Staging

Recommended crates:

| Crate | Responsibility |
|---|---|
| `rcount-core` | data model, canonical records, hash projections, first summary arithmetic checks |
| `rcount-io` | package read/write, NDJSON/JSON parsing, source index, docs fixture generation |
| `rcount-audit` | reconciliation equations, package verification, transcripts |
| `rcount-cli` | user-facing commands and fixture verification |

No crate should decide election certification. Certification status is a record
declared by source evidence and status events, then checked for consistency.

## Initial Fixtures

V.0 acceptance requires these fixtures:

1. `summary-basic`: one contest, two precincts, jurisdiction total; verifies.
   First in-crate generator: `rcount_core::synthetic_summary_basic_package`.
   Public package generator:
   `cargo run -p rcount-io --example summary_basic_package`.
2. `canvass-correction`: unofficial total superseded by canvassed event; both
   states replay and the final status explains the delta.
   First in-crate generator:
   `rcount_core::synthetic_canvass_correction_package`.
   Public package generator:
   `cargo run -p rcount-io --example canvass_correction_package`.
3. `privacy-inclusion-sketch`: inclusion proof fixture that verifies a
   non-choice token and refuses a choice-bearing proof.
4. `bad-arithmetic`: summary total mismatch; verifier fails with a specific
   equation id.
5. `missing-source-hash`: source index omits a raw file hash; verifier fails.
6. `tampered-source`: raw source bytes change after the source index hash is
   written; verifier fails with `source_hash_match`.
7. `bad-selection-sum`: manifest and source hashes verify, but one summary's
   `counted_ballots` no longer equals votes plus residuals; verifier fails
   with `contest_selection_sum`.
8. `mail-batch-added`: canvassed totals include an election-day batch and a
   late mail batch; `batch_summary_total` ties each batch summary to declared
   accounting evidence, and `accepted_ballots` verifies counted plus rejected
   ballot accounting inside each batch.
9. `missing-batch`: manifest and source hashes verify, but one summary
   references a batch absent from `normalized/batches.ndjson`; verifier fails
   with `batch_summary_total`.
10. `precinct-split-lineage`: cross-cycle reporting-unit lineage records
    `P-004 -> P-004A + P-004B` and `P-007 + P-008 -> P-078`; verifier passes
    `lineage_conservation`.
11. `bad-lineage`: manifest and source hashes verify, but lineage references a
    current unit not present in `reporting-units.ndjson`; verifier fails with
    `lineage_conservation`.
12. `privacy-inclusion-sketch`: one anonymized inclusion token verifies without
    voter identity or candidate choices; verifier passes `proof_privacy_gate`.
13. `choice-bearing-proof`: manifest and source hashes verify, but a proof
    exposes a candidate selection; verifier fails with `proof_privacy_gate`.

## Validation Harness

RCOUNT should be validated first on a synthetic election office, not on real
state data. The harness should generate a small jurisdiction where every
underlying fact is known, then emit RCOUNT packages and expected verifier
results.

### Synthetic State

The baseline fixture is a grid state:

```text
state: SYN
counties: 2
municipalities: 4
precincts: 12
split_precincts: 2
vote_centers: 2
district_plan: optional 3-district RPLAN over the same units
```

Each precinct has:

- stable unit id;
- county and municipality membership;
- adjacency and optional RPLAN/RCTX mapping;
- registered-voter count by synthetic party or demographic bucket;
- turnout propensity;
- ballot style;
- precinct lineage id.

### Election Cycles

The harness should generate at least three elections:

| Cycle | Purpose | Controlled changes |
|---|---|---|
| `SYN-2024-general` | clean baseline | stable precincts, one contest, no corrections |
| `SYN-2026-general` | canvass events | late mail batch, provisional acceptance, write-in adjudication |
| `SYN-2028-general` | precinct lineage | one precinct split, two precincts merged, one district boundary changed |

The important test is not only that each package verifies in isolation. The
lineage verifier should also explain how reporting units changed across cycles:

```text
P-004 (2024) -> P-004A + P-004B (2028)
P-007 + P-008 (2024) -> P-078 (2028)
```

### Synthetic Vote Generation

The generator should keep two layers:

1. hidden ground truth, used only by tests;
2. public exports, used by RCOUNT.

Hidden ground truth may include synthetic voter records and generated ballot
choices. Public RCOUNT fixtures must not expose individual choices. Public
exports should include only summaries, batch manifests, CVR-like rows when the
privacy profile allows them, and status events.

Recommended deterministic inputs:

- `master_seed`;
- precinct turnout rates;
- contest candidate preference rates;
- mail/provisional/write-in rates;
- undervote and overvote rates;
- recount perturbation script.

### Package Generation Matrix

For each synthetic election, generate positive and negative packages:

| Fixture | Expected result | What it tests |
|---|---|---|
| `summary-basic` | pass | precinct sums equal jurisdiction total |
| `mail-batch-added` | pass | status event explains delta from unofficial to canvassed |
| `provisional-adjudication` | pass | accepted provisional ballots reconcile |
| `recount-amended` | pass | recount supersedes canvassed totals with lineage |
| `precinct-split-lineage` | pass | cross-cycle unit lineage is replayable |
| `district-aggregation-rplan` | pass | RCOUNT + RPLAN/RCTX produces district totals |
| `multi-election-harness` | pass | three synthetic cycles replay count arithmetic, precinct lineage, and RPLAN district aggregation |
| `multi-election-harness-negatives/bad-lineage` | fail | cycle lineage references a current precinct that is absent |
| `multi-election-harness-negatives/stale-plan` | fail | RPLAN assignment references a precinct with no current summary |
| `multi-election-harness-negatives/tampered-2028-source` | fail | one cycle's raw source evidence no longer matches its source index |
| `bad-selection-sum` | fail | candidate votes + undervotes + overvotes mismatch |
| `missing-batch` | fail | summary references absent batch manifest |
| `missing-source-hash` | fail | source evidence is omitted from the source index |
| `tampered-source` | fail | raw source hash mismatch |
| `choice-bearing-proof` | fail | proof reveals candidate selection |
| `small-cell-linkage` | fail or suppress | privacy gate catches reidentification risk |

### Test Levels

RCOUNT should use the same rough L0/L1/L2 ladder as the rest of the repo:

| Level | Scope | Examples |
|---|---|---|
| L0 | pure functions | canonical hash projection, equation evaluator, status transition parser |
| L1 | package fixtures | synthetic package round trip and verifier pass/fail cases |
| L2 | multi-election harness | generate three cycles, track precinct lineage, aggregate with RPLAN |
| L3 | real-data adapter smoke | one public state/county export, marked non-golden unless source is archived |

The first milestone should stop at L2. Real data belongs after the synthetic
state catches arithmetic, lineage, privacy, and tamper-evidence failures.

Current L0 coverage:

- canonical record/package hash stability;
- `contest_selection_sum` pass/fail;
- `jurisdiction_contest_total` pass/fail, including per-status snapshots;
- `status_event_declared` and `canvass_correction_event`;
- synthetic `summary-basic` package generation.
- synthetic `canvass-correction` package generation.
- `district_aggregation_total` over a two-district synthetic RPLAN.
- synthetic multi-election harness replay over three cycles.
- negative synthetic multi-election harness replay for bad lineage and stale
  RPLAN units.

Current L1 coverage:

- `rcount-io` package directory round trip;
- public `bad-selection-sum` fixture generation with valid package/source
  hashes;
- public `mail-batch-added` and `missing-batch` fixture generation;
- public `precinct-split-lineage` and `bad-lineage` fixture generation;
- public `privacy-inclusion-sketch` and `choice-bearing-proof` fixture
  generation;
- public `district-aggregation-rplan` fixture generation with package, RPLAN,
  and aggregation transcript;
- public `multi-election-harness` fixture generation with three package/plan
  pairs and one replay transcript;
- public `multi-election-harness-negatives` fixture generation for bad lineage,
  stale RPLAN units, and tampered source evidence;
- manifest `content_hash` mismatch rejection;
- source index hash verification and empty-source-index rejection;
- docs fixture verification for
  `docs/examples/rcount-golden-packages/summary-basic`;
- docs fixture generation for
  `docs/examples/rcount-golden-packages/canvass-correction`.

Current audit coverage:

- `rcount-audit` pass transcript for `summary-basic`;
- `rcount-audit` pass transcript for `canvass-correction`, including
  event-correlation check;
- `rcount-audit` pass transcript for `mail-batch-added`, including
  batch-accounting checks;
- `rcount-audit` pass transcript for `precinct-split-lineage`, including
  split/merge lineage checks;
- `rcount-audit` pass transcript for `privacy-inclusion-sketch`, including
  receipt-safe proof privacy checks;
- fail transcript for tampered manifest/content hash;
- fail transcript for missing batch evidence;
- fail transcript for bad lineage evidence;
- fail transcript for choice-bearing proof evidence;
- fail transcript for missing or tampered source hash evidence;
- fail transcript for bad summary arithmetic;
- CLI district aggregation transcript for a synthetic RPLAN assignment;
- L2 synthetic multi-election replay transcript with split/merge lineage and
  per-cycle district aggregation;
- L2 negative CLI coverage for bad lineage, stale district plan units, and
  tampered cycle source evidence;
- real docs transcript generated by
  `cargo run -p rcount-audit --example write_summary_basic_transcript`.

## CLI Shape

Draft commands:

```text
rcount verify package.rcount
rcount hash package.rcount
rcount explain-delta package.rcount --event event-0007
rcount verify-proof package.rcount proofs/example.json
rcount aggregate-districts package.rcount --plan plan.rplan --context context.rctx
```

Initial implementation:

```text
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/summary-basic
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/summary-basic --write-transcript
cargo run -p rcount-cli -- aggregate-districts docs/examples/rcount-golden-packages/district-aggregation-rplan/package --plan docs/examples/rcount-golden-packages/district-aggregation-rplan/plan.rplan.json
```

## Open Questions

- Which NIST CDF fields should be first-class versus retained as source-specific
  metadata?
- How should RCOUNT represent jurisdictions that publish only PDFs?
- What minimum public data is required before a package can be called
  `canvassed` rather than `summary-only`?
- Can receipt-safe voter inclusion proofs be useful without cooperation from an
  election office?
- Should RCOUNT support detached signatures in v0.1 or defer them to v0.2?
