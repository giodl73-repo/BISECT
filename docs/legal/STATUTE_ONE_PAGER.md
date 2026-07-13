# Districting Integrity And Disclosure Act - One-Page Summary

**Status:** v0.2 candidate, 2026-07-10
**Full text:** `MODEL_FEDERAL_STATUTE.md`

## Problem

Federal law requires single-member, population-balanced congressional
districts, but most assignment-affecting choices remain difficult to audit.
Courts and the public often see only the final map, not a reproducible
comparison plan, alternatives, or a complete explanation of changes.

## Proposal

Require every State to publish:

1. a precommitted, partisan-input-excluded geographic benchmark;
2. the final legally adopted plan;
3. a machine-readable baseline-to-final diff;
4. legal authority, alternatives, public comments, and reasons for every
   departure; and
5. hash-bound source, software, analysis, and challenge records.

The benchmark is mandatory evidence, not the mandatory final map.

## Benchmark

- Census blocks and block population.
- Published national adjacency profile.
- Standard recursive bisection.
- Shared-boundary-length weights.
- One manifest-derived seed.
- Population equality as nearly exact as practicable.
- A certification pathway that proves the unique best cut at every required
  recursive node before exact readiness is claimed.
- No party, election, race, candidate, incumbency, or COI input.

## Final Plan

The State, commission, or court remains responsible for the final map.
Departures are:

- mandatory when required by the Constitution, Voting Rights Act, or court;
- permitted under published State-law, community, or error-correction
  authority; and
- subject to public notice, independent review, and a complete modification
  record. State-law/COI/correction changes include a reproducible lower-
  departure alternative; federally required remedies are measured against full
  legal compliance, not proximity to the benchmark.

The benchmark is not the *Milligan* vote-dilution baseline and is never a VRA
safe harbor or evidence negating discriminatory intent or effect. Protected
expert appendices may accompany public legal conclusions; required disclosure
is not itself an admission that race predominated. Where race-conscious and
State-authorized partisan considerations touch the same districts, the record
must disentangle each authority, input, stage, and effect.

## Governance

| Body | Role |
|---|---|
| Congress | Enacts assignment-affecting benchmark rules |
| Census Bureau | Census/geographic releases and corrections |
| NIST | Schemas, canonicalization, conformance, build guidance |
| EAC | Lead agency, reference service, independent review, assistance, grants, and challenge register |
| State/commission/court | Final plan and legal justification |

## Constitutional Posture

The primary theory is an Elections Clause national disclosure and process
floor, not federal adoption of district lines. A federal service can publish
the benchmark if a State does not, but cannot adopt the final plan. Commission
grants and a conditional-funding alternative are severable fallback postures.

The model authorizes a new $250 million decennial program for the reference
service, State implementation, community participation, independent review,
language access, and NIST conformance.

## What It Does Not Claim

- One algorithm defines fairness.
- The benchmark must be enacted.
- Reproducibility proves VRA compliance or partisan neutrality.
- A metric percentile determines legality.
- BISECT is already block-level or externally release-ready.

## Current Technical Status

BISECT implements substantial tract-level benchmark, provenance, analysis, and
verification infrastructure. A two-run Rhode Island reference package and a
three-state Rust/GerryChain ensemble package exist. Block-level benchmark
execution, manifest-derived seed wiring, external replication, and legislative
adoption remain incomplete.
