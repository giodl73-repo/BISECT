# Spec: RCOUNT Incubation And Election Count Audit Substrate

**Status:** Draft architecture proposal  
**Date:** 2026-05-12  
**Scope:** Reproducible election counting, canvass reconciliation, and plan-linked vote audit packages  
**Related specs:** [`2026-05-10-rplan-incubation.md`](2026-05-10-rplan-incubation.md),
[`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md),
[`2026-05-10-plan-audit-certificates.md`](2026-05-10-plan-audit-certificates.md)

## Decision

Create a new generic crate family, **RCOUNT**, for reproducible election count
and canvass audit records.

RCOUNT is a sibling of RPLAN, not a submodule of BISECT and not a redistricting
algorithm crate. RPLAN answers: "what district assignment was adopted, and can
we verify the plan?" RCOUNT answers: "what votes, ballots, eligibility records,
precinct totals, and canvass adjustments produced this reported election total,
and can we reconcile them?"

The public expansion of the name is **Reproducible Count**. The name `ecount`
was considered, but `rcount` is a better companion to `rplan`: both are neutral
interchange and audit substrates that may later stand outside this repository.
The `r` does not need to mean the same word as RPLAN; the parallel is in the
role, not the acronym.

## Fixed Point

Until the count-audit boundary is settled, treat this as the fixed point for
downstream work:

- The public name is **RCOUNT**.
- Count ledger files use `.rcount`.
- Optional count-context files use `.rctxc` until a better extension is chosen.
- The generic crate family is `rcount-*`.
- `rcount-*` crates may not depend on `bisect-*`.
- `rcount-*` crates may depend on `rplan-*` only in adapter/audit layers that
  explicitly need plan-aware district aggregation.
- `rplan-*` should not depend on `rcount-*`.
- `bisect-*` may depend on `rcount-*` only for reporting, package verification,
  and plan-linked election analysis.

RCOUNT must be useful without a redistricting plan. A countywide ballot-count
audit, a precinct canvass reconciliation, or a tabulator-to-hand-count audit
should not require RPLAN. RPLAN becomes relevant when precincts or vote units
are aggregated into districts.

## Names

- **Spec name:** RCOUNT
- **Primary file extension:** `.rcount`
- **Optional context extension:** `.rctxc`
- **Crate prefix:** `rcount-*`
- **CLI command:** `rcount`

The context extension is intentionally provisional. If the first implementation
finds that count context should be embedded in `.rcount`, or that `.rctx` can
support both plan and count contexts without confusion, update this spec before
implementing public fixtures.

## Crates

```text
rcount-core   # generic election count, ledger, canvass, and reconciliation model
rcount-io     # RCOUNT read/write, canonical JSON, schema migration
rcount-audit  # reconciliation checks and count audit certificates
rcount-rplan  # optional plan-aware district aggregation bridge
rcount-cli    # binary package; command name `rcount`
```

Initial dependency direction:

```text
rcount-io -> rcount-core
rcount-audit -> rcount-core
rcount-audit -> rcount-io
rcount-rplan -> rcount-core
rcount-rplan -> rcount-audit
rcount-rplan -> rplan-core
rcount-rplan -> rplan-io
rcount-cli -> rcount-io
rcount-cli -> rcount-audit
rcount-cli -> rcount-rplan

bisect-* -> rcount-*
bisect-* -> rplan-*
rcount-rplan -> rplan-*
```

Forbidden direction:

```text
rcount-* -> bisect-*
rplan-* -> rcount-*
rcount-core -> rplan-*
rcount-io -> rplan-*
rcount-audit -> rplan-*
```

This keeps the core count model independent from redistricting plans. The only
crate that knows about both count ledgers and district plans is the optional
bridge crate `rcount-rplan`.

## Core Model

RCOUNT models an election count as a reconciliation ledger, not only as a final
vote table. The model should be able to represent partial evidence. A public
report may have precinct totals but not ballot-level cast vote records; the
certificate should say exactly which checks were possible and which were not.

Core concepts:

| Concept | Description |
|---------|-------------|
| Election | Jurisdiction, date, contest set, ballot style universe, and source identity. |
| Unit universe | Precincts, vote centers, batches, scanner batches, or imported reporting units. |
| Eligibility ledger | Registration or eligibility counts by unit and ballot style when available. |
| Ballot ledger | Ballots issued, returned, cast, rejected, provisional, cured, and spoiled. |
| Tabulation ledger | Candidate/choice totals by contest and unit. |
| Canvass ledger | Corrections, adjudications, late-arriving ballots, and official certification deltas. |
| Audit sample | Hand-count/RLA sample units, sample size, sampled ballots or batches, and comparison outcomes. |
| Aggregation map | Optional mapping from reporting units to counties, precincts, districts, or RPLAN units. |
| Certificate | Machine-checkable reconciliation result with hashes, warnings, failures, and unsupported checks. |

The first implementation should support aggregate ledgers. It should not require
ballot-level cast vote records.

## RCOUNT File Sketch

A `.rcount` file is a count-ledger artifact. It is not required to contain
geometries, district plans, or private voter records.

```json
{
  "rcount_version": "0.1",
  "election": {
    "schema_version": "election-count-v1",
    "jurisdiction": "Example County, ST",
    "election_date": "2026-11-03",
    "election_type": "general",
    "source": {
      "producer": "county-canvass-system",
      "source_id": "official-canvass-v1",
      "retrieved_at": "2026-11-20T18:00:00Z"
    }
  },
  "units": {
    "unit_kind": "precinct",
    "canonical_order": "explicit-unit-ids",
    "unit_ids": ["PCT-001", "PCT-002"],
    "unit_universe_hash": "sha256:..."
  },
  "contests": [
    {
      "contest_id": "us_house_district_01",
      "office": "U.S. House",
      "district_label": "1",
      "choices": ["A", "B", "write_in"]
    }
  ],
  "ledgers": {
    "eligibility": [],
    "ballots": [],
    "tabulation": [],
    "canvass": [],
    "audit_samples": []
  },
  "provenance": {},
  "extensions": {}
}
```

## Reconciliation Checks

The audit engine should expose checks as named constraints, mirroring the
clarity of `rplan-audit`.

Minimum phase-1 checks:

| Check | Requirement |
|-------|-------------|
| Unit universe | Every ledger row references a known unit id. |
| Contest universe | Every vote total references a known contest and choice. |
| Ballot conservation | Cast plus rejected/provisional/spoiled categories reconcile to returned/issued categories when those fields are available. |
| Tabulation non-negativity | Vote counts and ballot counts are non-negative integers. |
| Canvass arithmetic | Certified totals equal tabulator totals plus recorded canvass adjustments. |
| Aggregation arithmetic | Jurisdiction and district totals equal the sum of their lower-level units. |
| Hash binding | Certificate hashes bind the exact `.rcount` document and any external context inputs. |

Checks must distinguish:

- `pass`: evidence is present and reconciles.
- `fail`: evidence is present and does not reconcile.
- `unsupported`: evidence needed for the check is absent.
- `not_applicable`: the check does not apply to this election/package.

This distinction is crucial. A small package with precinct tabulations can still
be useful if the certificate honestly says that eligibility and RLA checks are
unsupported.

## Plan-Aware Bridge

RCOUNT should support district-level vote auditing without making RPLAN a core
dependency.

The `rcount-rplan` bridge should accept:

```text
plan.rplan
context.rctx
count.rcount
optional-unit-crosswalk.json
```

and produce:

```text
district-count-summary.json
count-audit-certificate.json
```

The bridge is responsible for questions such as:

- Do all precinct or reporting units map to plan units or districts?
- Are split precincts explicitly represented rather than silently rounded?
- Do district-level vote totals equal the sum of assigned unit totals?
- Does the reported district outcome change under any unresolved split-unit
  allocation?
- Are district totals tied to a specific plan hash and context hash?

This is the direct connection to BISECT and RPLAN: once a plan is a certified
assignment artifact, vote totals can be audited against that assignment instead
of treated as disconnected tables.

## Election Audit Principles

RCOUNT should encode the same principle that made U.20 useful: every public
claim should be backed by a lower-level manifest.

For election counting, that means a reported total should be traceable through:

```text
eligibility universe
  -> ballots issued
  -> ballots returned/cast
  -> rejected/provisional/cured/spoiled categories
  -> tabulator or batch totals
  -> canvass adjustments
  -> certified totals
  -> audit samples or hand-count comparisons when available
```

For plan-linked district reporting:

```text
RPLAN/RCTX plan package
  + RCOUNT count package
  + unit crosswalk
  -> district totals
  -> district count certificate
```

The public doctrine is simple: show the ledger, not just the headline number.

## Privacy Boundary

RCOUNT must not require personally identifiable voter records. The default
model is aggregate and unit-level.

Allowed in phase 1:

- precinct/unit-level counts
- contest totals
- ballot category totals
- batch or scanner totals
- public audit sample summaries
- public canvass correction records

Out of scope for phase 1:

- voter-level registration records
- ballot images
- ballot-level cast vote records that may create privacy risks
- signature records
- personally identifiable provisional-ballot records

Later versions may support hashed or privacy-preserving references to more
granular evidence, but that requires a separate privacy review.

## Acceptance Stages

- [ ] Stage 0: approve this incubation spec and name decision.
- [ ] Stage 1: write RCOUNT v0.1 schema spec with concrete JSON fields.
- [ ] Stage 2: add `rcount-core` aggregate domain types.
- [ ] Stage 3: add `rcount-io` canonical read/write and fixture round trips.
- [ ] Stage 4: add `rcount-audit` reconciliation checks and certificate model.
- [ ] Stage 5: add `rcount-cli verify` for standalone count packages.
- [ ] Stage 6: add `rcount-rplan` bridge for plan-linked district aggregation.
- [ ] Stage 7: add one tiny public synthetic package with passing and failing
      fixtures.
- [ ] Stage 8: update paper/docs language that discusses election audit and
      certification.

## First Golden Fixture

The first fixture should be small enough to inspect by eye:

```text
two precincts
one contest
two candidates
one canvass correction
one certified total
one negative fixture with a one-vote mismatch
```

The plan-aware fixture can then add:

```text
four plan units
two precincts
one split precinct or explicit no-split declaration
two districts
district totals derived from the count ledger
```

This keeps the implementation honest: it must prove ordinary arithmetic before
attempting full election administration complexity.

## Open Questions

1. Should `.rctxc` exist, or should count context be embedded in `.rcount` until
   files become large?
2. Should `rcount-cli` expose `verify`, `audit`, or both?
3. Should ballot-style information be required in v0.1, or optional until a
   real multi-style fixture is added?
4. What is the minimal public fixture that avoids jurisdiction-specific legal
   claims while still resembling a real canvass?
5. Should the first plan-aware bridge operate at precinct-to-district level
   only, or also support precinct-to-RPLAN-unit crosswalks?

## Goal Prompt

```text
/goal Build the RCOUNT incubation path described in docs/specs/2026-05-12-rcount-incubation.md: approve the naming and crate boundary, write the v0.1 schema, implement the aggregate count core/IO/audit crates with tiny passing and failing fixtures, add a CLI verifier, then add the optional RPLAN bridge for district-level count reconciliation, verifying and committing stage by stage.
```
