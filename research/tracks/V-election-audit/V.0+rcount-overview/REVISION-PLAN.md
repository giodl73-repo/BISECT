# Revision Plan

Round 1 synthesis: `reviews/SYNTHESIS.md`.

Round 2 recheck: `reviews/ROUND2-SYNTHESIS.md`. V.0 is ready with P2 polish
remaining.

## P1.1 Certification And Count-Status Lifecycle

- [x] Add a table in the verifier or boundaries section for unofficial, canvassed,
  recounted, amended, and certified states.
- [x] For each state, name the relevant RCOUNT record type and what RCOUNT can or
  cannot verify.
- [x] Repeat that certification remains a human/legal act.

## P1.2 Tally Semantics

- [x] Add definitions for ballots, ballot cards, CVR rows, contests, selections,
  summaries, batches, and residual counts.
- [x] Add one worked `mail-batch-added` equation.
- [x] Clarify that RCOUNT v0 is summary-level and adapters must preserve raw source
  distinctions.

## P1.3 Privacy And Threat Model

- [x] Add a threat model table for tamper evidence, parser substitution, package
  truncation, equivocation, small-cell reidentification, coercion receipts, and
  malware resistance.
- [x] Change inclusion-proof wording to "proof sketch" where appropriate.
- [x] Add explicit small-cell disclosure warning.

## P1.4 Synthetic Evidence Ladder

- [x] Add an L0/L1/L2 table.
- [x] Add fixture-to-command traceability for positive and negative fixtures.
- [x] Name the tested crates and CLI commands.

## P1.5 Schema, Versioning, And Hash Contract

- [x] Add required vs optional package files.
- [x] Name `rcount_version`, source-index package-relative path rule, canonical hash
  projections, and domain-separated hash prefixes.
- [x] Add RPLAN/RCTX transcript hash details.
