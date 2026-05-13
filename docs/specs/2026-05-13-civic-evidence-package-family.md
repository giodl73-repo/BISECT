# Civic Evidence Package Family

**Status:** Umbrella architecture draft  
**Date:** 2026-05-13  
**Scope:** Shared package model for geography, plans, history, counts, audits,
statistics, and public evidence bundles  
**Related specs:** [`2026-05-10-rplan-incubation.md`](2026-05-10-rplan-incubation.md),
[`2026-05-12-rcount-incubation.md`](2026-05-12-rcount-incubation.md),
[`2026-05-12-rcount-substrate.md`](2026-05-12-rcount-substrate.md),
[`2026-05-13-rctx-boundary.md`](2026-05-13-rctx-boundary.md),
[`2026-05-13-civic-evidence-layer-access-patterns.md`](2026-05-13-civic-evidence-layer-access-patterns.md),
[`2026-05-13-rhist-boundary.md`](2026-05-13-rhist-boundary.md),
[`2026-05-13-rhist-implementation.md`](2026-05-13-rhist-implementation.md),
[`2026-05-13-downstream-evidence-boundaries.md`](2026-05-13-downstream-evidence-boundaries.md),
[`2026-05-13-r-package-completion-goal.md`](2026-05-13-r-package-completion-goal.md)
**Review records:** [`civic-evidence-package-family-r1_roles.md`](reviews/civic-evidence-package-family-r1_roles.md),
[`civic-evidence-layer-access-r1_roles.md`](reviews/civic-evidence-layer-access-r1_roles.md),
[`rhist-implementation-r1_roles.md`](reviews/rhist-implementation-r1_roles.md),
[`civic-evidence-package-family-roles.md`](reviews/civic-evidence-package-family-roles.md)

## Decision

Treat RPLAN and RCOUNT as the first two members of a broader civic evidence
package family. The family exists to prevent every new verifier from inventing
its own geography, lineage, source-hash, and transcript layers.

The fixed point is:

```text
source bytes
  -> shared context/geography
  -> lineage/history
  -> domain package
  -> audit/statistical transcript
  -> certificate/report bundle
```

Layer access patterns are specified in
[`2026-05-13-civic-evidence-layer-access-patterns.md`](2026-05-13-civic-evidence-layer-access-patterns.md).

## Package Names

| Package | Expansion | Owns | Does not own |
|---|---|---|---|
| RCTX | Reproducible Context | shared unit ids, graph identity, source universe, crosswalk context | plan assignments, vote totals |
| RMAP | Reproducible Map | rendered/cartographic map products, layers, styling provenance | canonical unit identity unless paired with RCTX |
| RHIST | Reproducible History | unit lineage across time: splits, merges, renames, boundary edits | election totals or district assignments |
| RPLAN | Reproducible Plan | district assignments, plan metadata, plan audit certificates | election counts, precinct history beyond referenced context |
| RCOUNT | Reproducible Count | election totals, CVRs, canvass, ballot manifests, audit evidence | district plan validity or long-run unit history |
| RAUDIT | Reproducible Audit | reusable audit/recount/RLA transcript bundles | base count ledgers unless embedded or referenced |
| RCERT | Reproducible Certification | canvass board actions, certification signoffs, legal/evidence matrix | independent count arithmetic |
| RSTAT | Reproducible Statistics | statistical models, forensic reports, uncertainty/caveat records | certification pass/fail claims |
| RLOG | Reproducible Log | event logs, EMS logs, custody/status logs | normalized counts or plan validity |
| RCHAIN | Reproducible Chain | seals, custody transfers, observer records, physical evidence chain | statistical audit math |
| RROLL | Reproducible Roll | aggregate registration/eligibility ledgers and ballot-style universes | private voter roll publication |
| RCASE | Reproducible Case | court/commission/public evidence bundle tying packages together | new domain facts not present in child packages |

Names are provisional except RPLAN and RCOUNT. The important decision is the
ownership boundary, not the final acronym.

## RCTX Versus RMAP

RCTX and RMAP should not collapse into one thing.

The implementation boundary is detailed in
[`2026-05-13-rctx-boundary.md`](2026-05-13-rctx-boundary.md). The short rule is
that RCTX owns machine identity and crosswalks, while RMAP owns presentation.

**RCTX** is the canonical machine context:

- stable unit ids;
- canonical unit order;
- adjacency graph;
- population/source attributes needed for verification;
- source hashes and provenance;
- crosswalks between source unit systems.

**RMAP** is the public/cartographic presentation layer:

- GeoJSON/TopoJSON/vector/raster outputs;
- layer styling;
- label placement;
- map projection metadata;
- rendered map hashes.

RMAP may reference RCTX, but RCTX must be useful without a rendered map. RPLAN
and RCOUNT should depend on RCTX-like context for machine verification, not on
RMAP display artifacts.

## Why RHIST Comes Before Deeper RCOUNT

Precinct lineage is the main base dimension that can cause rewrites if it is
buried inside RCOUNT or RPLAN.

RHIST should own:

- reporting-unit identity across cycles;
- precinct/vote-center/batch lineage where public sources support it;
- split, merge, rename, closure, and boundary-change events;
- crosswalk weights and evidence quality;
- source hashes for historical unit files.

RCOUNT can then reference RHIST when comparing elections or aggregating counts
through changing units. RPLAN can reference RHIST when comparing plans across
cycles. Neither package has to become the permanent home for historical unit
truth.

## Recommended Build Order

This is the order that minimizes rewrites:

1. **RCTX minimal context boundary.** Confirm canonical unit ids, unit order,
   graph/source hashes, and crosswalk record shape. RPLAN already has an
   implemented `.rctx`; treat it as RCTX-compatible while the shared boundary
   stabilizes. The first shared crate slice is `rctx-core`, with the
   `docs/fixtures/rctx/l0-shared-context` fixture covering manifest,
   source-index, unit-index, graph, crosswalk, package-hash, transcript, and
   claim-boundary records. `rctx-io` remains deferred until an independent
   directory loader is needed beyond this fixture/helper shape.
2. **RHIST minimal lineage package.** Extract the precinct/reporting-unit
   lineage model now, before adding more multi-election RCOUNT features.
3. **RCOUNT current work.** Continue count arithmetic, source adapters, audit
   algorithms, and `rcount-stats`, but keep long-run unit history as references
   to RHIST-compatible records.
4. **RAUDIT extraction decision.** If RLA/manual/recount transcripts become
   useful outside full RCOUNT packages, split reusable audit bundles. Until
   then, keep them as RCOUNT sections.
5. **RCHAIN/RLOG after real source pressure.** Add custody/log packages only
   after we have public artifacts that need them. Do not model imaginary
   custody detail before source examples exist.
6. **RSTAT/W-series after RCOUNT adapters.** Forensics need clean public data;
   build them after enough normalized RCOUNT/RHIST inputs exist.
7. **RCASE last.** Evidence bundles should compose stable child packages, not
   compensate for unstable ones.

## Dependency Direction

Allowed:

```text
RPLAN -> RCTX
RPLAN -> RHIST only for historical comparison workflows
RCOUNT -> RCTX
RCOUNT -> RHIST for cross-cycle unit identity
RCOUNT -> RAUDIT if audit bundles split out
RAUDIT -> RCOUNT only if consuming count records; otherwise prefer RCTX/RHIST refs
RSTAT -> RPLAN/RCOUNT/RHIST/RCTX as inputs
RCASE -> all packages as child evidence
RMAP -> RCTX for rendered geography
```

Forbidden:

```text
RCTX -> RPLAN/RCOUNT/RHIST/RSTAT/RCASE
RHIST -> RPLAN/RCOUNT/RSTAT/RCASE
RMAP -> inventing canonical unit identity without RCTX
RSTAT -> changing verifier pass/fail status in RPLAN/RCOUNT
RCASE -> creating facts not present in child packages
```

## Current Work Alignment

RPLAN is already far enough along to remain the plan package. RCOUNT should
continue, but with two guardrails:

- keep precinct lineage records compatible with a future RHIST extraction;
- keep geography/context references compatible with shared RCTX, not
  RCOUNT-only `.rctxc` assumptions. The RCTX boundary spec recommends avoiding
  a permanent `.rctxc` dialect.

The immediate next base package should be **RHIST**, after the RCTX boundary is
recorded, with a tiny synthetic multi-cycle unit-lineage package and one
negative fixture. That will stabilize the dimension that both RPLAN and RCOUNT
need most.

Pulse 06 wires the first shared-reference example: RCOUNT's summary fixture can
declare the RCTX L0 context/crosswalk hashes and the RHIST L2 package hash, and
the district-aggregation RPLAN example carries the same package hashes in an
extension note. This is composition metadata only; it does not transfer RCTX
unit identity, RHIST lineage, or RCOUNT ledgers into RPLAN.

## RHIST Minimal Slice

The first RHIST slice should include:

```text
manifest.json
sources/source-index.json
units/cycles.ndjson
units/lineage-events.ndjson
units/crosswalks.ndjson
proofs/package-hashes.json
transcripts/verify-transcript.json
```

Minimal checks:

- every lineage event references known prior/current units;
- split events have one prior and multiple current units;
- merge events have multiple prior and one current unit;
- rename events preserve lineage identity;
- crosswalk weights for a prior unit sum to 1.0 where declared exhaustive;
- source hashes match preserved source files.

## Claim Boundary

This package family verifies declared evidence relationships. It does not
replace official legal certification, legislative authority, judicial findings,
or human chain-of-custody testimony.

Each package must say what it proves and what it merely references.
