# Downstream Evidence Package Boundaries

**Status:** Phase 0 boundary spec  
**Date:** 2026-05-13  
**Scope:** RAUDIT, RCERT, RSTAT, RLOG, RCHAIN, RROLL, RCASE, and RMAP package
boundaries before implementation  
**Related specs:** [`2026-05-13-civic-evidence-package-family.md`](2026-05-13-civic-evidence-package-family.md),
[`2026-05-13-civic-evidence-layer-access-patterns.md`](2026-05-13-civic-evidence-layer-access-patterns.md)

## Decision

Do not implement these packages yet. Define their access patterns now so RCOUNT,
RPLAN, RCTX, and RHIST do not accidentally absorb their responsibilities.

## RMAP

Owns rendered and public map products.

Writes:

- map product manifest;
- rendered image/vector/tile hashes;
- styling and label provenance;
- display projection and layer ordering;
- source references for geometry payloads.

Reads:

- RCTX context hash and unit ids;
- optional RPLAN assignment for district maps;
- optional RCOUNT/RSTAT summaries for thematic maps.

Must not:

- invent canonical unit ids;
- change RCTX graph identity;
- become required for machine verification.

## RAUDIT

Owns reusable audit, recount, and RLA method transcripts when they need to stand
outside a full RCOUNT package.

Writes:

- audit method id and parameters;
- assertions, assorters, risk limits, sample schedule, stopping decisions;
- sampled ballot or batch references;
- audit transcript hashes.

Reads:

- RCOUNT contest outcomes, manifests, CVRs, and sample evidence;
- RCTX/RHIST only when the audit method depends on unit strata or geography.

Must not:

- mutate RCOUNT ledgers;
- certify an election by itself;
- hide method assumptions inside prose.

## RCERT

Owns certification actions and legal/administrative signoffs.

Writes:

- board or official action records;
- certification dates, minutes, motions, signatories;
- evidence matrix linking actions to child package hashes;
- legal caveats and scope.

Reads:

- RCOUNT verification result;
- RAUDIT transcript result;
- RLOG/RCHAIN records where certification depends on process evidence.

Must not:

- recompute count arithmetic without binding to RCOUNT;
- turn exploratory RSTAT findings into official findings by implication.

## RSTAT

Owns statistical and forensic analysis outputs.

Writes:

- model id and version;
- input package hashes;
- anomaly scores, uncertainty intervals, diagnostics, and caveats;
- reproducible model transcript.

Reads:

- RCOUNT normalized totals and CVR summaries;
- RHIST unit lineage for time series;
- RPLAN/RCTX for spatial or district features.

Must not:

- assert certification pass/fail;
- silently change verifier outcomes;
- publish privacy-sensitive small-cell outputs without a disclosure policy.

## RLOG

Owns normalized event and status logs.

Writes:

- event records;
- source log index;
- parser transcript;
- normalization caveats.

Reads:

- EMS logs, scanner logs, tabulator logs, upload logs, adjudication logs;
- optional RCOUNT ids for linking events to count artifacts.

Must not:

- infer physical custody beyond source logs;
- replace RCHAIN when custody evidence is required.

## RCHAIN

Owns physical chain-of-custody evidence.

Writes:

- seal records;
- transfer records;
- observer records;
- storage/location records;
- custody gap and caveat records.

Reads:

- custody forms, seal logs, ballot retrieval sheets, observer reports;
- RLOG when machine or administrative events help explain timing.

Must not:

- infer custody solely from successful count reconciliation;
- certify count correctness.

## RROLL

Owns aggregate registration, eligibility, and ballot-style universes.

Writes:

- aggregate registration snapshots;
- ballot-style universe records;
- source hashes and privacy caveats;
- derived eligibility summaries.

Reads:

- public aggregate registration and ballot-style sources;
- optional RCTX unit universes for geography alignment.

Must not:

- publish private voter-level records;
- become a substitute for voter-roll custody or access-control systems.

## RCASE

Owns public evidence bundles and claim matrices.

Writes:

- child package index;
- claim matrix;
- exhibit order;
- public report metadata;
- verifier summary over child hashes.

Reads:

- all child packages by hash;
- external citations when source bytes cannot be preserved.

Must not:

- create facts unavailable in child packages;
- loosen child-package caveats;
- merge conflicting claims without preserving conflict metadata.

## Common Deferred Work

Each package will need, before implementation:

- manifest schema;
- source-index schema;
- package hash domain prefix;
- claim-boundary schema;
- L0 positive and negative fixture;
- one real-source pressure test before broad modeling.
