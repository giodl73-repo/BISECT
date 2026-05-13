# Civic Evidence Layer Access Patterns

**Status:** Architecture spec  
**Date:** 2026-05-13  
**Scope:** Read/write patterns, dependency direction, and package boundaries for
the civic evidence package family  
**Related specs:** [`2026-05-13-civic-evidence-package-family.md`](2026-05-13-civic-evidence-package-family.md),
[`2026-05-13-rctx-boundary.md`](2026-05-13-rctx-boundary.md),
[`2026-05-13-rhist-boundary.md`](2026-05-13-rhist-boundary.md),
[`2026-05-13-downstream-evidence-boundaries.md`](2026-05-13-downstream-evidence-boundaries.md)

## Decision

Model the family as layered evidence packages, not as one growing package.
Packages may read lower or sibling evidence by hash, but they must not mutate
or reinterpret upstream facts.

```text
source bytes
  -> RCTX/RMAP/RHIST base dimensions
  -> RPLAN/RCOUNT domain packages
  -> RAUDIT/RCERT/RSTAT evidence analyses
  -> RLOG/RCHAIN supporting records
  -> RCASE public composition bundle
```

## Access Pattern Table

| Package | Writes | Reads | Primary query | Prohibited access |
|---|---|---|---|---|
| RCTX | unit universes, graph identity, source hashes, crosswalks | source bytes | "What are the canonical units and relationships?" | reading plans, counts, statistics, or cases |
| RMAP | rendered layers, map products, styling provenance | RCTX, source geometry | "How should this context be displayed?" | inventing canonical unit identity |
| RHIST | cycles, unit lineage, cross-cycle crosswalks | RCTX, source unit files | "How did units change over time?" | storing election totals or plan assignments |
| RPLAN | district assignments, plan metadata, plan audit certs | RCTX, optional RHIST | "Which unit belongs to which district?" | storing vote ledgers |
| RCOUNT | election totals, CVRs, manifests, audit evidence | RCTX, optional RHIST/RPLAN | "Do declared counts reconcile?" | owning long-run precinct history |
| RAUDIT | reusable audit/recount transcripts | RCOUNT, RCTX, optional RHIST | "Does an audit method support the reported outcome?" | changing base count ledgers |
| RCERT | certification actions and legal signoffs | RCOUNT, RAUDIT, RLOG/RCHAIN | "What official action was taken on what evidence?" | performing independent arithmetic without references |
| RSTAT | anomaly models, uncertainty, forensic reports | RCOUNT, RHIST, RPLAN, RCTX | "What statistical patterns need explanation?" | asserting certification pass/fail |
| RLOG | event logs and machine/admin status logs | source logs, optional RCOUNT refs | "What events were recorded and normalized?" | asserting physical custody not in sources |
| RCHAIN | seals, custody transfers, observers, physical evidence chain | source custody artifacts, RLOG | "Who had custody of what, when?" | inferring custody from counts or statistics alone |
| RROLL | aggregate registration and ballot-style universes | registration/eligibility sources | "What eligible/ballot-style universe is declared?" | publishing private voter-level rolls |
| RCASE | public evidence bundles and claim matrix | all child package hashes | "What evidence supports this public claim?" | creating facts not present in children |

## Read Modes

Packages should distinguish four read modes:

- **Embed:** copy a small record into the package and hash it as part of the
  package.
- **Reference:** record a child package id/hash and require the verifier to
  load it.
- **Snapshot:** preserve source bytes under `sources/` with hashes.
- **External:** record a URI/citation only; verifier can check metadata but not
  byte identity unless the source is also snapshotted.

Default preference:

```text
source evidence: snapshot when legally and practically possible
package inputs: reference by package hash
small generated transcripts: embed
large rendered artifacts: reference or snapshot by product hash
```

## Write Rules

Every package writes three kinds of facts:

- **Declared facts:** normalized records asserted by the package producer.
- **Derived facts:** deterministic outputs computed from declared facts and
  referenced package inputs.
- **Claim facts:** human-readable statements about what verification proves.

Verification can accept or reject declared and derived consistency. It cannot
turn a source record into a legal fact unless the package explicitly limits the
claim to "this source says X."

## Query Patterns

Expected cross-package queries:

- RPLAN asks RCTX for unit order and graph adjacency.
- RCOUNT asks RCTX/RHIST for reporting-unit identity and crosswalks.
- RCOUNT district aggregation asks RPLAN for assignment and RCTX for the
  crosswalk binding count units to plan units.
- RCOUNT package directories may record RHIST package-hash inputs in
  `normalized/rhist-refs.ndjson`, with each reference declaring the consumed
  cycles and role.
- RCOUNT package directories may record RCTX context/crosswalk inputs in
  `normalized/rctx-refs.ndjson`; each reference binds context hashes and
  optional crosswalk hashes used by unit-context or plan-aggregation claims.
  The first shared positive fixture is
  `docs/fixtures/rctx/l0-shared-context`; RCOUNT consumes it by stable context
  and crosswalk hashes rather than copying RCTX graph/source ownership.
- RPLAN examples may carry package-reference notes in `extensions`, as in the
  district-aggregation example's `civic_evidence_base_references` object. That
  object points to RCTX/RHIST package hashes for composition only; RPLAN remains
  the owner of district assignments, not unit history or count ledgers.
- Transitional RCOUNT embedded lineage maps through `rcount-rhist` into
  RHIST-compatible events; this is a bridge, not a transfer of RHIST ownership
  into RCOUNT.
- RAUDIT asks RCOUNT for contest outcomes, ballot manifests, CVRs, and audit
  samples.
- RSTAT asks RCOUNT/RHIST for normalized time series and asks RPLAN/RCTX for
  spatial context.
- RCASE asks every child package for hash, claim boundary, and verifier result.

## Package Interface Shape

Each package should eventually expose:

```text
manifest.json
sources/source-index.json
records/*.ndjson
proofs/package-hashes.json
transcripts/verify-transcript.json
claims/claim-boundary.json
```

Domain packages may add domain-specific directories. The common interface is
for composition and verification, not a mandate that every package have the
same internal record model.

## Implementation Order

1. Finish RCTX boundary and keep existing `.rctx` stable.
2. Specify RHIST minimal package and fixture ladder.
3. Continue RCOUNT with RCTX/RHIST-compatible access patterns.
4. Add RAUDIT only if audit transcripts need to stand alone.
5. Add RSTAT only after there are enough normalized RCOUNT/RHIST inputs.
6. Add RLOG/RCHAIN when real log/custody source artifacts demand them.
7. Add RCASE after child package claim boundaries are stable.

## Red Flags

- a package writes a fact whose owner is another package;
- a rendered map changes canonical unit identity;
- a statistic is phrased as certification;
- a case bundle creates a new factual assertion instead of composing child
  claims;
- a verifier silently accepts an external source that was not snapshotted or
  hash-bound.
