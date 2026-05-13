# Goal: Finish The Civic Evidence R-Package Family

**Status:** Active goal  
**Track:** R-package family completion  
**Foundation:** [`2026-05-13-civic-evidence-package-family.md`](2026-05-13-civic-evidence-package-family.md),
[`2026-05-13-civic-evidence-layer-access-patterns.md`](2026-05-13-civic-evidence-layer-access-patterns.md),
[`2026-05-12-rcount-substrate.md`](2026-05-12-rcount-substrate.md),
[`2026-05-13-rhist-implementation.md`](2026-05-13-rhist-implementation.md),
[`2026-05-13-rcount-audit-algorithm-roadmap.md`](2026-05-13-rcount-audit-algorithm-roadmap.md)

## Goal

Drive the civic evidence package family from working slices to a coherent
package system. RPLAN and RCOUNT are the first mature members. RCTX and RHIST
are the base dimensions that prevent rewrite. RMAP, RAUDIT, RCERT, RSTAT,
RLOG, RCHAIN, RROLL, and RCASE should land only when their ownership boundary
and first real access pattern are clear.

The finish line is not "all possible packages exist." The finish line is:

- every active package has an owner boundary, manifest/hash shape, fixtures,
  verifier path, and claim boundary;
- RCOUNT/RPLAN consume RCTX/RHIST by stable references instead of duplicating
  geography or lineage semantics;
- audit/statistical layers can replay or report evidence without changing base
  package pass/fail meaning;
- every implemented package has positive and negative fixtures plus CLI or
  library verification coverage.

## Current Baseline

- [x] RPLAN exists as the redistricting plan package.
- [x] RCOUNT exists as the election count package.
- [x] `rctx-core` owns the first shared exact crosswalk verification slice.
- [x] RHIST has an implementation-ready package spec and verifier direction.
- [x] RCOUNT can reference RCTX and RHIST package hashes.
- [x] RCOUNT V.12 BRAVO and V.15 ALPHA replay algorithm transcripts.
- [x] RCOUNT V.14/V.18 comparison audit substrate includes overstatement,
  taint replay, batch comparison records, linkage, derivation helper, IO
  round-trip, and CLI replay coverage.

## Global Invariants

- RCTX owns canonical unit identity. No other package invents canonical ids.
- RHIST owns cross-cycle unit history. RCOUNT and RPLAN may reference it, not
  absorb it.
- RMAP owns presentation only. Rendered maps cannot rewrite machine context.
- RCOUNT owns current count ledgers, CVRs, canvass summaries, and embedded
  audit evidence until RAUDIT has a second independent consumer.
- RSTAT/W-series analytics inform investigation; they do not certify outcomes.
- RCASE composes child package hashes and claim boundaries; it creates no new
  domain facts.
- Every package hash must use a domain-separated prefix.
- Every implemented package must have at least one positive fixture and one
  negative fixture before being treated as active.
- Every milestone ends with focused tests plus the relevant full workspace
  suite.

## Stage 0 - Goal Setup

- [x] Create this active completion goal.
- [ ] Keep this checklist current as milestones land.
- [ ] Add a short pointer from the next-session handoff when the next major
  implementation slice completes.

## Stage 1 - RCOUNT Audit Algorithm Completion

Purpose: finish the V-series algorithms enough that RCOUNT can honestly replay
or boundary-report the audit methods it names.

- [x] V.12 BRAVO replay.
- [x] V.15 ALPHA fixed-bet bounded martingale replay.
- [x] V.13 Minerva/Athena package-level boundary runs.
- [x] V.14 Kaplan-Markov/MACRO comparison substrate and taint-product replay.
- [x] V.18 batch comparison records, overstatement verification, replay,
  linkage, derivation helper, IO round-trip, and CLI replay.
- [x] V.14 published MACRO Kaplan-Markov product primitive in `rcount-stats`
  with exact-rational no-error and overstatement fixtures.
- [x] V.14 package schema can carry MACRO `N`, `V`, and gamma design inputs,
  and `kaplan-markov-comparison-v1` replays with MACRO when they are present.
- [x] V.14 core verification rejects partial or invalid MACRO design fields.
- [x] V.14 has a reusable synthetic MACRO package fixture with core
  verification, IO round-trip, and CLI replay coverage.
- [x] V.14 external-public validation boundary is recorded: public replay needs
  `N`, `V`, gamma, and ordered overstatement categories.
- [x] V.13 has a scoped round-one two-candidate Minerva replay primitive with
  exact binomial tails.
- [x] V.13 has a reusable synthetic Minerva replay package with core
  verification, IO round-trip, CLI pass coverage, and declared-risk drift
  coverage.
- [x] V.13 audit sample steps can carry explicit `round_index` values, and the
  synthetic multi-round Minerva fixture replays cumulative round one as
  continue and round two as stop.
- [x] V.13 Athena has a documented boundary fixture with core verification, IO
  round-trip, and CLI replay coverage.
- [x] V.17 stratified/hybrid coordinator schema can preserve strata, component
  run ids, ballot counts, and combining rule ids.
- [x] V.17 has a two-stratum synthetic fixture, missing-component negative
  coverage, IO round-trip, and CLI boundary replay.
- [x] V.17 records nuisance parameters and stratum allocation ppm values, and
  rejects flattened one-stratum coordinator misuse.
- [x] V.19 RAIRE/AWAIRE boundary fixtures preserve reported IRV elimination
  order and ranked sample choices, with malformed ranked-choice negative
  coverage.
- [x] V.20 Bayesian tabulation boundary fixture preserves prior/likelihood ids,
  posterior probabilities, deterministic simulation metadata, and analytic
  claim separation, with impossible-posterior negative coverage.
- [x] V.21 SOBA observable-ballot boundary fixture links sampled openings to
  privacy-safe inclusion proofs, rejects missing openings, and reports
  comparison-risk replay as a boundary.
- [ ] Validate V.14 MACRO replay against external public audit fixtures once a
  source exposes those fields.
- [ ] Validate V.13 Minerva replay against an external public fixture that
  exposes per-round ballot observations or tallies.
- [ ] Add Athena risk replay only after method-specific inputs and public
  validation fixtures are available.
- [ ] Normalize V.12/V.14/V.15/V.18 under the V.16 SHANGRLA assertion language
  where practical.
- [ ] Add V.17 combined-risk math after nuisance/allocation transcript fields
  are explicit.
- [ ] Keep V.19 RAIRE/AWAIRE risk replay staged until ranked CVR semantics and
  IRV tabulation fixtures are stable.
- [ ] Add V.20 hand-computable posterior replay only after conjugate-model
  inputs and reporting language are fixed.
- [ ] Add V.21 CVR/human-observation mismatch fixtures after the comparison
  audit linkage shape is ready to consume them.

## Stage 2 - RCTX Minimal Shared Context

Purpose: stabilize the machine geography layer that RPLAN and RCOUNT both use.

- [x] Boundary spec says RCTX owns unit ids, unit order, graph identity, source
  universe, and crosswalk context.
- [x] `rctx-core` verifies exact rational crosswalk rows and source refs.
- [x] RCOUNT district aggregation can preserve declared RCTX reference and
  crosswalk hash.
- [x] Add a minimal RCTX package directory fixture with manifest, source index,
   units, graph/crosswalk records, package hashes, and verify transcript.
- [x] Defer `rctx-io` until the package directory needs independent loading;
  pulse 04 uses `rctx-core` fixture helpers and docs fixtures only.
- [ ] Add `rctx-cli verify` only after the fixture shape is stable.
- [x] Add RCOUNT consumer coverage that preserves the minimal RCTX fixture
  context and crosswalk hashes during district aggregation.
- [ ] Update RPLAN examples to reference the same RCTX fixture where possible.

## Stage 3 - RHIST Minimal Lineage Package

Purpose: prevent multi-cycle unit history from becoming accidental RCOUNT or
RPLAN state.

- [x] RHIST implementation spec and review record exist.
- [x] RHIST-compatible references exist in RCOUNT.
- [x] RCOUNT transitional lineage can map toward RHIST-compatible events.
- [ ] Lock the L0 RHIST positive fixture for rename/split/merge lineage.
- [ ] Lock the L0 RHIST negative fixture for missing unit or bad crosswalk
  weight.
- [ ] Ensure `rhist-core`, `rhist-io`, and any CLI verify surface remain tiny
  and package-boundary focused.
- [ ] Add an RCOUNT consumer test that references an RHIST fixture by package
  hash rather than embedding new history semantics.
- [ ] Add an RPLAN consumer note or fixture for historical plan comparison.

## Stage 4 - RCOUNT Source Adapter And Data Validation Ladder

Purpose: make RCOUNT useful on real public artifacts without weakening claim
boundaries.

- [x] Statement CSV and NIST CDF imports exist.
- [x] Rhode Island Rep. 28 import preserves source hashes and emits Minerva
  boundary evidence.
- [ ] Add one public audit/report adapter that supplies sampled-batch order and
  calls `derive_batch_comparison_algorithm_run`.
- [ ] Add validation-data index entries for each public fixture, naming what
  can and cannot be verified from source bytes.
- [ ] Add negative adapter fixtures for source hash drift, manifest total drift,
  and missing sampled-unit evidence.

## Stage 5 - RMAP Boundary

Purpose: keep cartographic presentation useful without contaminating RCTX.

- [ ] Write implementation-ready RMAP boundary spec.
- [ ] Define minimum rendered-product record: projection, layer inputs, style
  hash, output hash, RCTX reference.
- [ ] Add one tiny fixture that references RCTX but does not create unit ids.
- [ ] Add negative fixture where RMAP attempts to override canonical unit
  identity.

## Stage 6 - RAUDIT / RCERT Extraction Decision

Purpose: avoid premature package sprawl while preserving future split points.

- [x] Current decision: RAUDIT remains embedded in RCOUNT until there is a
  second independent consumer.
- [ ] Revisit RAUDIT after V.13/V.14/V.15/V.18 replay surfaces stabilize.
- [ ] Write RAUDIT extraction criteria: reusable transcript bundle, RCOUNT
  references, source hashes, and claim boundary.
- [ ] Keep RCERT as a reader of RCOUNT/RAUDIT/RLOG/RCHAIN, not an arithmetic
  owner.

## Stage 7 - RSTAT / W-Series Analytics

Purpose: add forensic analytics only after enough normalized data exists.

- [ ] Define RSTAT report boundary: investigation aid, not certification.
- [ ] Add W.01 outlier/anomaly report fixture after RCOUNT has enough public
  normalized examples.
- [ ] Add precinct-lineage-aware analytics only after RHIST fixture references
  are stable.
- [ ] Add report tests that prove RSTAT cannot change RCOUNT/RPLAN verifier
  pass/fail status.

## Stage 8 - RLOG / RCHAIN / RROLL

Purpose: reserve homes for logs, custody, and eligibility universes without
inventing source detail.

- [ ] Add RLOG only when real public event/EMS/admin logs need normalization.
- [ ] Add RCHAIN only when source custody/seal/observer records are available.
- [ ] Add RROLL only for aggregate registration or ballot-style universes that
  can be safely published.
- [ ] Keep private voter-level data out of public package fixtures.

## Stage 9 - RCASE Composition

Purpose: make public evidence bundles that compose child packages without
creating new facts.

- [ ] Write RCASE boundary spec after RCOUNT/RPLAN/RCTX/RHIST claim boundaries
  are stable.
- [ ] Define child package hash list, claim matrix, verifier result summary,
  and source disclosure index.
- [ ] Add negative fixture where RCASE asserts a fact not present in children.

## Final Acceptance

- [ ] RCOUNT current-count and audit algorithm roadmap reaches its declared
  boundary.
- [ ] RCTX and RHIST have independent minimal package fixtures and verifier
  coverage.
- [ ] RCOUNT and RPLAN can both reference RCTX/RHIST without duplicating their
  ownership semantics.
- [ ] RMAP, RAUDIT, RCERT, RSTAT, RLOG, RCHAIN, RROLL, and RCASE each have a
  written defer/activate decision.
- [ ] Every active package has a claim-boundary file or documented equivalent.
- [ ] The docs/specs review records reflect the final build order.
- [ ] Full focused workspace suites pass for all implemented crates.

## Suggested `/goal`

```text
/goal Finish the civic evidence R-package family using
docs/specs/2026-05-13-r-package-completion-goal.md as the active checklist.
Work stage by stage without collapsing package boundaries: finish RCOUNT audit
algorithm replay, then RCTX minimal package fixtures, then RHIST minimal lineage
fixtures and RCOUNT/RPLAN references, then only activate RMAP/RAUDIT/RCERT/RSTAT
and later packages when their first real access pattern exists. For each
milestone, update the checklist/docs, add positive and negative fixtures, run
focused tests plus the relevant full workspace suite, and continue until the
final acceptance checklist is complete.
```
