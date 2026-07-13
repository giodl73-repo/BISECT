# Quickstart: Federal Benchmark And Disclosure Proposal

**Audience:** congressional staff, policy advocates, commissions, and technical
reviewers
**Model text:** `docs/legal/MODEL_FEDERAL_STATUTE.md`
**Technical specification:** `docs/specs/2026-07-09-national-redistricting-standard-v0.1.md`

## The Proposal In One Paragraph

Every State publishes a reproducible geographic benchmark before adopting its
congressional map. The State may adopt a different final plan, but it must
publish every moved unit, legal authority, alternatives, effects, public
comments, and reasons. Federal voting-rights obligations remain controlling.

## What The Benchmark Is

The candidate statutory profile uses:

- standard recursive bisection;
- geographic shared-boundary weights;
- one precommitted seed derived from the complete input manifest;
- census blocks as the normative units; and
- no political, racial, candidate, incumbency, or community input.

ApportionRegions, county-sticky weights, and convergence search remain research
comparators. They are not the candidate statute's benchmark.

## What BISECT Can Run Today

The current committed smoke profile is tract-level:

```powershell
cargo build --release --locked -p bisect-cli --bin bisect

target\release\bisect.exe build nrs_reference_v0_1 `
  --year 2020 --states RI --workers 1 --force --no-interactive

target\release\bisect.exe label-verify nrs_reference_v0_1 --year 2020
```

This is a reproducibility fixture, not full statutory conformance. The
candidate statute requires block-level execution and a manifest-derived seed,
which remain implementation work.

## What Verification Establishes

A passing chain establishes that:

- the named config and output indexes are hash-linked;
- the named software and inputs produced the recorded assignment; and
- the published record can be checked.

It does not establish:

- fairness;
- VRA compliance;
- community preservation;
- partisan neutrality;
- legal validity; or
- that the benchmark should be the final plan.

## Evidence Available

- Reference replay:
  `docs/fixtures/nrs-reference-v0.1/`
- Real ensemble evidence:
  `docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020/`
- Evidence posture:
  `docs/vtrace/READINESS_DECISION.md`

## Review Questions For Staff

1. Should Congress require benchmark publication, final-plan adoption, or both?
2. Which public body should own schemas and technical custody?
3. What precommitment deadline prevents profile gaming?
4. What public record is required for VRA, State-law, COI, and error
   modifications?
5. Which duties should be direct, federally supplied, grant-supported, or
   funding-conditioned?
6. What remedy should follow a technical defect versus a voting-rights
   violation?
