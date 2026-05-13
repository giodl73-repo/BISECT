# Paper Track V: Vote Counting, Certification, And Public Verification

**Status:** Draft paper-track roadmap  
**Date:** 2026-05-12  
**Substrate:** RCOUNT, with optional RPLAN bridge for district aggregation  
**Related specs:** [`2026-05-12-rcount-incubation.md`](2026-05-12-rcount-incubation.md),
[`2026-05-12-rcount-certification-research.md`](2026-05-12-rcount-certification-research.md),
[`2026-05-13-rcount-validation-data-landscape.md`](2026-05-13-rcount-validation-data-landscape.md),
[`2026-05-13-rcount-audit-algorithm-roadmap.md`](2026-05-13-rcount-audit-algorithm-roadmap.md)

## Track Thesis

Track V asks how election totals become public facts.

The redistricting work has been converging toward a fixed point: algorithm
output becomes an RPLAN/RCTX/certificate package that a third party can verify.
Election counting needs the parallel fixed point. A reported vote total should
not be only a headline number, PDF, or vendor export. It should be a
recomputable RCOUNT package with:

- canonical count ledgers;
- precinct, batch, contest, canvass, and global hashes;
- explicit precinct-lineage records across elections;
- privacy-safe voter inclusion proofs where appropriate;
- replayable audit samples; and
- plan-linked district totals when RPLAN is present.

The goal of the V track is to define and test the gold standard for public
election-count evidence.

## Gold Standard

The gold standard is not "trust the software." It is:

```text
public inputs
  -> canonical ledgers
  -> typed hashes
  -> reconciliation checks
  -> audit/recount/RLA evidence
  -> certified totals
  -> public certificate
```

For a voter-facing proof:

```text
private voter
  -> privacy-safe receipt token
  -> public inclusion proof
  -> precinct/election hash root
```

The receipt proves inclusion, not candidate choices.

For district reporting:

```text
RPLAN/RCTX plan hash
  + RCOUNT count hash
  + precinct/unit crosswalk
  -> district vote totals
  -> district count certificate
```

## Proposed Papers

| Code | Working title | Core technique | Gold-standard contribution |
|------|---------------|----------------|----------------------------|
| V.0 | RCOUNT Overview: Reproducible Election Count Packages | schema, crate boundary, certificate model | Defines the fixed point for count evidence. |
| V.1 | Canvass Arithmetic: From Unofficial Returns To Certified Totals | ledger reconciliation | Proves certified totals from tabulator totals plus canvass deltas. |
| V.2 | Precinct Lineage Across Elections | split/merge/rename/boundary-change graph | Makes election-to-election comparisons honest when precincts move. |
| V.3 | Tamper-Evident Precinct And Batch Hashing | typed Merkle roots and canonical JSON/CSV rows | Makes every precinct total recomputable into the global count root. |
| V.4 | Privacy-Safe Voter Inclusion Proofs | receipt-token inclusion without vote-choice receipt | Lets a voter test inclusion while avoiding coercion receipts. |
| V.5 | Ballot Manifest Verification | manifest row hashing, batch/container accounting | Checks that ballot storage manifests reconcile to counted ballot totals. |
| V.6 | CVR-To-Summary Reconciliation | CVR row counts, contest sums, summary-result checks | Verifies that cast vote records and published summaries agree. |
| V.7 | Replayable Risk-Limiting Audits | public seed, sample replay, stopping-rule verification | Lets anyone replay ballot/batch selection and audit conclusions. |
| V.8 | District Vote Aggregation With RPLAN | RCOUNT + RPLAN + crosswalk | Binds district outcomes to both plan hashes and count hashes. |
| V.9 | Count-System Interoperability: Vendor Exports To RCOUNT | adapters, source hashing, and normalization | Converts current election-system outputs into neutral public packages. |
| V.10 | Certification Evidence Matrix | evidence-family matrix, data availability, audit requirements | Separates what law requires from what RCOUNT can additionally prove. |
| V.11 | Performance And Parallel Verification In Rust | parallel precinct/batch hashing and reconciliation | Shows public verification can be fast enough for full-election packages. |
| V.12 | BRAVO Ballot-Polling Audits | sequential likelihood-ratio ballot polling | Replays the classic ballot-polling RLA family. |
| V.13 | Minerva And Athena Ballot-Polling Audits | round-level ballot-polling risk measures | Replays Arlo/Rhode-Island-style ballot-polling reports. |
| V.14 | Kaplan-Markov And MACRO Comparison Audits | overstatement-error risk math | Replays ballot/batch comparison audit evidence. |
| V.15 | ALPHA And Betting-Martingale Audits | sequential martingale tests | Provides a shared modern RLA test core. |
| V.16 | SHANGRLA Assorters | assertion/assorter mean-test framework | Gives RCOUNT a common audit assertion language. |
| V.17 | Stratified And Hybrid RLAs | multi-stratum evidence combination | Combines polling and comparison strata without flattening assumptions. |
| V.18 | Batch Comparison Audits | hand-tally versus reported batch comparison | Adds batch-level audit risk replay above manifest accounting. |
| V.19 | RAIRE And AWAIRE For RCV Audits | ranked-choice assertion generation | Extends RCOUNT to IRV/RCV audit claims. |
| V.20 | Bayesian Tabulation Audits | posterior audit analytics | Records Bayesian audit claims with calibration boundaries. |
| V.21 | Observable Ballot-Level Audits | privacy-preserving ballot/CVR linkage | Connects ballot-level public observability to RCOUNT evidence. |

External validation targets are tracked in
[`2026-05-13-rcount-validation-data-landscape.md`](2026-05-13-rcount-validation-data-landscape.md).
Algorithm implementation order is tracked in
[`2026-05-13-rcount-audit-algorithm-roadmap.md`](2026-05-13-rcount-audit-algorithm-roadmap.md).

## Paper Acceptance Contract

Each V paper should eventually include:

- a plain-language story of the real certification problem;
- a minimal formal model;
- an RCOUNT artifact schema or fixture;
- at least one passing and one failing synthetic example;
- a clear privacy boundary;
- a "what this proves / what this does not prove" section;
- a verifier command or pseudocode verifier path; and
- links to official legal or administrative sources when making current-practice
  claims.

No paper should claim that RCOUNT certifies an election by itself. RCOUNT
certifies arithmetic, hashes, inclusion, lineage, replay, and aggregation. Human
officials, statutes, courts, recount boards, and audit boards still own legal
certification.

## First Writing Batch

Start with papers that establish the substrate before the more ambitious claims:

1. **V.0 RCOUNT Overview**: fixed point, crate family, certificate model.
2. **V.1 Canvass Arithmetic**: the smallest useful count audit.
3. **V.3 Tamper-Evident Precinct And Batch Hashing**: root structure and public
   recomputation.
4. **V.4 Privacy-Safe Voter Inclusion Proofs**: inclusion without coercion.
5. **V.8 District Vote Aggregation With RPLAN**: the bridge back to the
   redistricting system.

Then move into election-administration techniques:

- V.2 precinct lineage;
- V.5 ballot manifests;
- V.6 CVR-summary reconciliation;
- V.7 risk-limiting audit replay;
- V.9 vendor/export adapters;
- V.10 legal matrix;
- V.11 performance.

## Implementation Tie-In

| Paper | Expected crate work |
|-------|---------------------|
| V.0 | `rcount-core`, `rcount-io`, certificate skeleton |
| V.1 | `rcount-audit` arithmetic checks |
| V.2 | unit-lineage model and movement reconciliation |
| V.3 | typed hash roots and proof paths |
| V.4 | receipt-inclusion verifier and negative coercion fixture |
| V.5 | ballot manifest parser and verifier |
| V.6 | CVR-summary reconciler |
| V.7 | RLA sampler replay and stopping verifier |
| V.8 | `rcount-rplan` bridge |
| V.9 | statement CSV and NIST CDF-style import adapters, source hash preservation, future jurisdiction adapters |
| V.10 | evidence matrix tied to RCOUNT fixture coverage |
| V.11 | parallel verifier benchmarks |
| V.12 | BRAVO fixture math and ballot-polling transcript |
| V.13 | Minerva/Athena replay, starting with RI/Arlo-style reports |
| V.14 | comparison-audit overstatement error verifier |
| V.15 | ALPHA/betting-martingale sequential test core |
| V.16 | SHANGRLA assertion and assorter records |
| V.17 | stratum coordinator and combined-risk transcript |
| V.18 | batch hand-tally comparison verifier |
| V.19 | ranked-choice CVR/assertion support |
| V.20 | Bayesian analytic transcript with calibration boundary |
| V.21 | commitment/opening records for observable ballot audits |

## Track Directory

The source track should live at:

```text
research/tracks/V-election-audit/
```

Paper slugs should follow the existing convention:

```text
V.0+rcount-overview
V.1+canvass-arithmetic
V.2+precinct-lineage
V.3+tamper-evident-hashing
V.4+privacy-safe-inclusion-proofs
V.5+ballot-manifest-verification
V.6+cvr-summary-reconciliation
V.7+rla-replay
V.8+district-vote-aggregation
V.9+count-system-interoperability
V.10+certification-evidence-matrix
V.11+rust-verification-performance
V.12+bravo-ballot-polling
V.13+minerva-athena-ballot-polling
V.14+kaplan-markov-macro-comparison
V.15+alpha-betting-martingales
V.16+shangrla-assorters
V.17+stratified-hybrid-rlas
V.18+batch-comparison-audits
V.19+raire-awaire-rcv
V.20+bayesian-tabulation-audits
V.21+observable-ballot-audits
```

## Goal Prompt

```text
/goal Build Track V for vote counting, certification, and public verification: create the V paper source track, draft V.0-V.4 as substrate papers with RCOUNT fixtures and verifier pseudocode, then extend through V.5-V.11 using official certification/audit sources, keeping privacy boundaries explicit, verifying docs, committing, and continuing stage by stage.
```
