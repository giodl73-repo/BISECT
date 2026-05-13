# Revision Plan

Round 1 synthesis: `reviews/SYNTHESIS.md`.

## P1.1 Certification And Count-Status Lifecycle

- Add a table in the verifier or boundaries section for unofficial, canvassed,
  recounted, amended, and certified states.
- For each state, name the relevant RCOUNT record type and what RCOUNT can or
  cannot verify.
- Repeat that certification remains a human/legal act.

## P1.2 Tally Semantics

- Add definitions for ballots, ballot cards, CVR rows, contests, selections,
  summaries, batches, and residual counts.
- Add one worked `mail-batch-added` equation.
- Clarify that RCOUNT v0 is summary-level and adapters must preserve raw source
  distinctions.

## P1.3 Privacy And Threat Model

- Add a threat model table for tamper evidence, parser substitution, package
  truncation, equivocation, small-cell reidentification, coercion receipts, and
  malware resistance.
- Change inclusion-proof wording to "proof sketch" where appropriate.
- Add explicit small-cell disclosure warning.

## P1.4 Synthetic Evidence Ladder

- Add an L0/L1/L2 table.
- Add fixture-to-command traceability for positive and negative fixtures.
- Name the tested crates and CLI commands.

## P1.5 Schema, Versioning, And Hash Contract

- Add required vs optional package files.
- Name `rcount_version`, source-index package-relative path rule, canonical hash
  projections, and domain-separated hash prefixes.
- Add RPLAN/RCTX transcript hash details.
