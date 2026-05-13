# Role Review: Civic Evidence Package Family

reviewer: ROLE PANEL  
date: 2026-05-13  
scope: `docs/specs/2026-05-13-civic-evidence-package-family.md`  
roles: BOUNDARY, COVENANT, CONTOUR, LEDGER, BENCHMARK, SCALE, TRENCH, CANVASS,
TALLY, VAULT, IDENTITY, LINEAGE, CARTOGRAPHY, CUSTODY, STATS, COMPOSITION

## Verdict

Approved as an umbrella architecture and ordering guide. Do not treat it as a
crate implementation spec yet.

The central decision is right: keep canonical machine context, rendered maps,
unit history, plans, counts, audits, statistics, and evidence bundles in
separate package layers. The most important sequencing decision is also right:
stabilize the RCTX boundary and build a minimal RHIST package before expanding
multi-cycle RCOUNT.

## Role Scores

| Role | Score | Finding |
|---|---:|---|
| BOUNDARY | 3.5/4 | Clear non-certification language and package ownership boundaries. Needs per-package non-goal text when each package spec lands. |
| COVENANT | 3.3/4 | Source hashes, package hashes, and child-package composition are central. Needs canonical hash prefixes for RHIST/RCTX/RMAP. |
| CONTOUR | 3.4/4 | RCTX/RMAP split is correct: machine context versus cartographic presentation. Needs concrete geometry/projection policy. |
| LEDGER | 3.2/4 | File roles are plausible. Needs versioned manifest shape and package-family hash vocabulary. |
| BENCHMARK | 3.0/4 | RHIST positive/negative fixtures are named. Needs an explicit L0/L1/L2 fixture ladder. |
| SCALE | 3.2/4 | Keeps RSTAT/W-series separate from verifier claims. Needs uncertainty-propagation story later. |
| TRENCH | 3.1/4 | The build order reduces rewrite risk. Needs migration policy from existing RCOUNT lineage records to RHIST. |
| CANVASS | 3.0/4 | RCOUNT remains current-count owner while RHIST owns unit history. Certification events may later pressure RCERT. |
| TALLY | 3.1/4 | Count/CVR/manifest ownership stays in RCOUNT. Good separation from ROLL/RCHAIN. |
| VAULT | 3.0/4 | Privacy boundaries are preserved at family level. RHIST/RMAP small-cell and map disclosure risks need later review. |
| IDENTITY | 3.5/4 | RCTX owns canonical unit identity; RMAP cannot invent it. This is the right base rule. |
| LINEAGE | 3.7/4 | RHIST-before-deeper-RCOUNT is the right order. This is the strongest part of the spec. |
| CARTOGRAPHY | 3.5/4 | RMAP as presentation and RCTX as machine context avoids a common trap. |
| CUSTODY | 2.8/4 | RCHAIN/RLOG are correctly deferred, but custody source examples are needed before modeling. |
| STATS | 3.3/4 | RSTAT/W-series is explicitly non-certifying. Good guardrail for anomaly analytics. |
| COMPOSITION | 3.2/4 | RCASE-last is correct. Needs child-package boundary preservation rules. |

Overall: 52.8 / 64. Architecture accepted; implementation specs required for
RCTX and RHIST.

## P0 Decisions

### P0-A: Create The Overlay Roles

Yes, create package-family overlay roles. The existing panel can review most
claims, but the package family needs sharper lenses for identity, lineage,
cartography, custody, statistics, and composition. Add them as review lenses,
not as standalone process overhead.

Canonical overlay-role doc:

```text
docs/specs/reviews/civic-evidence-package-family-roles.md
```

### P0-B: RCTX Is Not RMAP

RCTX must own canonical unit identity, graph/context identity, and crosswalks.
RMAP must own map presentation: rendered layers, styling, projection metadata,
labels, and visual outputs. RMAP may reference RCTX; RCTX must never depend on
RMAP.

### P0-C: RHIST Comes Before Multi-Cycle RCOUNT Expansion

RCOUNT can keep current lineage records, but the next base package should be a
minimal RHIST slice. Otherwise RCOUNT will become the accidental owner of all
historical unit semantics, and RPLAN will need a parallel history model later.

## Required Follow-Ups

### F1: RCTX Minimal Boundary Spec

Write a short spec that defines:

- unit id namespace;
- canonical unit order;
- graph/source hash;
- crosswalk record shape;
- relationship to existing `.rctx`;
- relationship to RMAP.

This can be a spec before any new crate.

### F2: RHIST Minimal Package Spec

Write an implementation-ready RHIST spec before crate work:

- package directory shape;
- cycle/unit/lineage/crosswalk record schemas;
- source-index and hash rules;
- verifier equations;
- fixture ladder.

### F3: Migration Note For Existing RCOUNT Lineage

Document how `RcountPackage.lineage` maps to RHIST concepts. The current field
can remain, but it should be treated as a local embedded lineage subset rather
than the final cross-package history model.

### F4: Package-Family Hash Vocabulary

Each package needs domain-separated hash prefixes. Do not reuse RPLAN/RCOUNT
prefixes for RHIST/RCTX/RMAP.

### F5: RCASE Boundary Rule

When RCASE lands, it must compose child package hashes and claim boundaries. It
must not create facts unavailable in the child packages.

## Approved Build Order

1. RCTX boundary spec.
2. RHIST minimal package spec.
3. RHIST tiny positive/negative fixtures.
4. Continue RCOUNT using RHIST-compatible lineage references.
5. Defer RCHAIN/RLOG until real custody/log artifacts exist.
6. Build RSTAT/W-series analytics after enough RCOUNT/RHIST inputs exist.
7. Build RCASE last.

## Decision

Proceed with the package-family plan, but gate implementation on RCTX and RHIST
minimal specs. The plan is strong precisely because it resists overbuilding:
RMAP, RCHAIN, RLOG, RSTAT, and RCASE all have homes, but only RHIST is urgent
enough to build next.
