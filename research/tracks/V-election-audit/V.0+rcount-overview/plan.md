# V.0 - RCOUNT Overview

**Paper Type:** Substrate overview / technical report  
**Status:** Drafting  
**Track:** V - Vote Counting, Certification, And Public Verification  
**Code Home:** `crates/rcount-*`  
**Primary Spec:** `docs/specs/2026-05-12-rcount-substrate.md`

## Research Question

What is the smallest reproducible package format and verifier contract that lets
third parties replay public election-count claims without pretending to replace
legal certification or reveal voter choices?

## Claims

- **H1:** RCOUNT can separate arithmetic verification, source tamper evidence,
  status/canvass lineage, precinct lineage, district aggregation, and privacy
  gates into independently replayable checks.
- **H2:** Synthetic positive and negative fixtures are sufficient to define the
  first public contract before vendor/state adapters are attempted.
- **H3:** The RPLAN boundary should be optional: district vote aggregation uses
  RPLAN/RCTX only when a count package is being projected onto districts.

Falsification: a fixture class cannot be expressed without changing the data
model, or a verifier failure cannot be attributed to a specific contract layer.

## Evidence

- Positive fixtures: `summary-basic`, `canvass-correction`,
  `mail-batch-added`, `precinct-split-lineage`, `privacy-inclusion-sketch`,
  `district-aggregation-rplan`, `multi-election-harness`.
- Negative fixtures: `bad-selection-sum`, `missing-batch`, `bad-lineage`,
  `choice-bearing-proof`, `tampered-source`, `missing-source-hash`,
  `multi-election-harness-negatives/*`.
- Verification: `cargo test -p rcount-district -p rcount-audit -p rcount-cli`.

## Role Constraints

- CANVASS: RCOUNT verifies arithmetic and evidence lineage; officials certify.
- TALLY: source distinctions such as batches, contests, residuals, and reporting
  units must survive normalization.
- VAULT: public proofs must prove inclusion or tamper evidence, never voter
  candidate choices.

## Figures and Tables

- Package layer diagram.
- Verifier layer table.
- Synthetic fixture matrix.
- RPLAN boundary diagram.

## Panel Readiness Checklist

- [x] Main LaTeX draft exists.
- [x] Synthetic fixture matrix included.
- [x] Claims separated from legal certification.
- [x] Privacy boundary named.
- [ ] Simulated review round completed.

