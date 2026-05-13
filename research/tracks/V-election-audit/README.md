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
| V.7 | Replayable Risk-Limiting Audits | sampler, margin, stopping-risk, and jurisdiction adapters landed |
| V.8 | District Vote Aggregation With RPLAN | substrate slice landed |
| V.9 | Count-System Interoperability: Vendor Exports To RCOUNT | statement CSV adapter slice landed |
| V.10 | Certification Evidence Matrix | draft landed |
| V.11 | Performance And Parallel Verification In Rust | first parallel verifier slice landed |
| V.12 | BRAVO Ballot-Polling Audits | atlas scaffold landed |
| V.13 | Minerva And Athena Ballot-Polling Audits | atlas scaffold landed; RI boundary adapter landed |
| V.14 | Kaplan-Markov And MACRO Comparison Audits | atlas scaffold landed |
| V.15 | ALPHA And Betting-Martingale Audits | atlas scaffold landed |
| V.16 | SHANGRLA Assorters | first assertion/assorter transcript schema landed |
| V.17 | Stratified And Hybrid RLAs | atlas scaffold landed |
| V.18 | Batch Comparison Audits | atlas scaffold landed |
| V.19 | RAIRE And AWAIRE For RCV Audits | atlas scaffold landed |
| V.20 | Bayesian Tabulation Audits | atlas scaffold landed |
| V.21 | Observable Ballot-Level Audits | atlas scaffold landed |

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
  RCTX context hash. The transcript now also carries a declared
  `rctx_reference_id` and `rctx_crosswalk_hash` when the aggregation context
  matches a package `rctx_refs` binding. `rcount aggregate-districts` now
  accepts `--crosswalk` to validate an explicit RCTX crosswalk NDJSON file and
  reject declared crosswalk-hash drift. Explicit crosswalk rows now drive the
  aggregation arithmetic; weighted allocations must be integral in the current
  transcript model. It also contains the first L2 synthetic multi-election
  harness with split/merge precinct lineage across three cycles, plus negative
  L2 cases for broken lineage, stale RPLAN units, and tampered cycle sources.
- `rcount-cli`: started 2026-05-12 with `rcount verify <package-dir>` and
  optional `--write-transcript`, `rcount aggregate-districts` for the
  RPLAN-backed district aggregation slice, and `rcount import-statement-csv`
  for the first V.9 source-export adapter.
- Interoperability slice: `rcount-io` now imports compact statement-of-votes
  CSV and NIST ERR/CDF-style JSON fixtures into ordinary RCOUNT records,
  preserves the original source bytes as `source:statement-csv` or
  `source:nist-cdf-json`, and verifies imported packages through the same
  source-hash and arithmetic checks as generated fixtures.
- Audit fixture ladder: now includes Colorado-style RLA, California-style RLA,
  and ordinary manual-audit model fixtures, each with positive and negative
  transcripts.
- First external validation adapter: `rcount import-ri2024-rep28-rla` imports
  Rhode Island's 2024 State Representative District 28 ballot-polling audit
  report, ballot manifest, and ballot retrieval CSV into a verifiable RCOUNT
  package. This slice verifies source hashes, contest arithmetic, manifest
  batch accounting, and sampled-ballot/retrieval key consistency, and writes a
  source-summary transcript; Minerva risk replay and ballot-level observation
  checks remain future work.
- External validation roadmap: `docs/specs/2026-05-13-rcount-validation-data-landscape.md`
  ranks Rhode Island RLA artifacts, Colorado RLA artifacts, MEDSL/OpenElections
  returns, and public CVR corpora as the first real-world validation targets.
- RCOUNT audit algorithm atlas: `docs/algorithm-atlas/` now carries V.12-V.21
  pages for BRAVO, Minerva/Athena, Kaplan-Markov/MACRO, ALPHA, SHANGRLA,
  stratified/hybrid RLAs, batch comparison, RAIRE/AWAIRE, Bayesian audits, and
  SOBA-style observable ballot audits, plus W.01 for non-certifying forensic
  anomaly analytics.
- First audit-algorithm substrate slice: `rcount-core` now has registered
  audit method ids, SHANGRLA/ALPHA-friendly assertion and assorter-value
  transcript records, optional package-level `audit_algorithm_runs`, and a
  verifier check for transcript shape, links, risk limits, and value bounds.
- Stats layer: `rcount-stats` is the IO-free home for exact rational arithmetic,
  ppm probability/risk-limit helpers, and future shared sequential-test
  primitives used by BRAVO, ALPHA, Minerva/Athena, and comparison audits.
- Package-family guardrail: `docs/specs/2026-05-13-civic-evidence-package-family.md`
  defines RCTX/RMAP/RHIST/RPLAN/RCOUNT/RAUDIT/RSTAT boundaries and recommends a
  minimal RHIST lineage slice before deeper multi-cycle RCOUNT expansion.

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
