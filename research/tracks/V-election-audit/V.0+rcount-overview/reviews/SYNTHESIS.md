# V.0 RCOUNT Overview: Review Synthesis

> AI-generated quality-improvement simulation, not real peer review.

## Scores

| Role | Score |
|---|---:|
| CANVASS | 3 / 4 |
| TALLY | 2 / 4 |
| VAULT | 2 / 4 |
| BENCHMARK | 3 / 4 |
| LEDGER | 2 / 4 |

Average: 2.4 / 4. Minimum: 2 / 4.

## P1 Blocking Items

### P1.1 Define the certification and count-status lifecycle

CANVASS needs a visible lifecycle table showing unofficial, canvassed,
recounted, amended, and certified states; the RCOUNT records/events associated
with them; and what RCOUNT can verify at each stage. This prevents V.0 from
seeming to replace certification with a package hash.

### P1.2 Add core tally semantics before the package model

TALLY needs explicit definitions for ballots, ballot cards, CVR rows, contests,
selections, summaries, batches, and residual counts. The draft says RCOUNT is
summary-level, but it should define what that excludes and what future adapters
must preserve.

### P1.3 Strengthen the privacy/threat model boundary

VAULT needs a threat model table separating tamper evidence, parser
substitution, package truncation, equivocation, small-cell reidentification,
coercion receipts, and malware resistance. The inclusion-proof language should
say "proof sketch" until a full cryptographic protocol exists.

### P1.4 Make the synthetic evidence ladder reproducible

BENCHMARK needs fixture-to-command traceability, including which CLI path is
expected to pass or fail for each negative case. The paper should name the L0,
L1, and L2 levels and explain why L2 synthetic coverage precedes real adapters.

### P1.5 State the schema/version/hash contract

LEDGER needs `rcount_version`, required vs optional files, source-index path
rules, canonical hash projections, and domain-separated hash prefixes to be
visible. Otherwise the paper is not yet a format contract.

## P2 Important Improvements

- Add one worked batch-accounting equation from `mail-batch-added`.
- Add a small "RCOUNT proves / does not prove" table.
- Mark RPLAN/RCTX as optional but show exactly which hashes enter the district
  aggregation transcript.
- Add a compatibility roadmap for NIST CDF, vendor exports, statements of
  votes, and RPLAN/RCTX.
- Fix overfull LaTeX tables with `tabularx`, smaller text, or split tables.

## P3 Nice-to-Have

- Add a package layer figure.
- Add a sample one-cycle transcript excerpt.
- Add future-work bullets for signatures and detached attestations.

## Recommended Next Action

Revise V.0 before starting V.1. The main work is not more prose volume; it is
adding five contract tables so that later papers can rely on V.0 as the stable
substrate definition.

