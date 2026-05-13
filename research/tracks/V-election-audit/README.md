# Track V -- Vote Counting, Certification, And Public Verification

Track V studies how election totals become public facts.

The track is built around RCOUNT, the proposed reproducible election-count and
canvass-audit substrate. RCOUNT is to election counts what RPLAN is to district
plans: a neutral package format and verifier path that lets third parties
recompute public claims from lower-level evidence.

## Papers

| Code | Working title | Status |
|------|---------------|--------|
| V.0 | RCOUNT Overview: Reproducible Election Count Packages | ready |
| V.1 | Canvass Arithmetic: From Unofficial Returns To Certified Totals | ready |
| V.2 | Precinct Lineage Across Elections | ready |
| V.3 | Tamper-Evident Precinct And Batch Hashing | ready |
| V.4 | Privacy-Safe Voter Inclusion Proofs | ready |
| V.5 | Ballot Manifest Verification | ready |
| V.6 | CVR-To-Summary Reconciliation | ready |
| V.7 | Replayable Risk-Limiting Audits | planned |
| V.8 | District Vote Aggregation With RPLAN | substrate slice landed |
| V.9 | Count-System Interoperability: Vendor Exports To RCOUNT | planned |
| V.10 | Certification Evidence Matrix | planned |
| V.11 | Performance And Parallel Verification In Rust | planned |

## Implementation Status

- `rcount-core`: started 2026-05-12 with canonical hash prefixes, core contest
  and summary records, synthetic package generators, and L0 checks for
  `contest_selection_sum`, `jurisdiction_contest_total`, canvass-correction
  event correlation, batch-summary accounting, reporting-unit lineage
  conservation, receipt-safe proof privacy gates, and CVR-to-summary
  reconciliation.
- `rcount-io`: started 2026-05-12 with package-directory read/write helpers,
  `summary-basic`, `canvass-correction`, `mail-batch-added`,
  `precinct-split-lineage`, `privacy-inclusion-sketch`, `cvr-summary`,
  `bad-selection-sum`, `missing-batch`, `bad-lineage`, `choice-bearing-proof`,
  and `bad-cvr-summary` generators, and docs fixture verification, including
  source-index hash checks.
- `rcount-audit`: started 2026-05-12 with pass/fail verification transcripts
  for `summary-basic` and `canvass-correction`, tampered manifest failures,
  bad arithmetic failures, batch-accounting failures, bad lineage failures,
  proof-privacy failures, CVR-summary failures, and missing/tampered
  source-hash failures.
- `rcount-district`: started 2026-05-12 as the optional bridge from verified
  RCOUNT summaries to RPLAN district assignments, producing district totals and
  transcripts that bind the RCOUNT package hash, RPLAN plan hash, and optional
  RCTX context hash. It also contains the first L2 synthetic multi-election
  harness with split/merge precinct lineage across three cycles, plus negative
  L2 cases for broken lineage, stale RPLAN units, and tampered cycle sources.
- `rcount-cli`: started 2026-05-12 with `rcount verify <package-dir>` and
  optional `--write-transcript`, plus `rcount aggregate-districts` for the
  RPLAN-backed district aggregation slice.

## Track Contract

Each paper should separate:

- what election officials must certify under law;
- what current systems and public exports actually provide;
- what RCOUNT can verify mechanically;
- what remains a human, statutory, judicial, recount, or audit-board decision.

The voter-facing rule is strict: prove inclusion, not candidate choices.

## Role Review

Initial substrate spec: `docs/specs/2026-05-12-rcount-substrate.md`.

Initial role-panel review: `docs/specs/reviews/rcount-track-r1_roles.md`.

RCOUNT uses the standard project roles plus three election-count roles:

- CANVASS: official canvass workflow, certification states, recount and
  adjudication lineage;
- TALLY: CVRs, ballot manifests, batch accounting, contest semantics, and
  vendor export meaning;
- VAULT: ballot secrecy, receipt-safety, canonical hashes, inclusion proofs,
  and cryptographic threat models.

V.0 now defines the substrate package anatomy, canonical hash rules,
reporting-unit model, ballot/contest semantics, synthetic fixture ladder, and
receipt-safe public verification boundary.
