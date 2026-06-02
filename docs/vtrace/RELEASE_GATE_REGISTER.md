# Release Gate Register

## Scope

This register converts the remaining S6 release-grade blockers into an
operator-facing gate list. It does not close any DCR, select a public release
candidate, publish generated artifacts, or upgrade the current
`internal_engineering_baseline_only` posture.

Source controls:

- `READINESS_DECISION.md`
- `BASELINE_HANDOFF.md`
- `DCRS.md`
- `TRACE.md`

## Current posture

| Field | Value |
|---|---|
| S6 decision | `internal_engineering_baseline_only` |
| Register status | active control aid |
| Release readiness | blocked |
| Public evidence publication readiness | blocked |
| Legal/court/filing readiness | blocked |
| External-user readiness | blocked |
| Clean reproducibility readiness | blocked |

## Gate list

| Gate | Current evidence | Required next evidence | Review lane | Claim unlocked only after pass |
|---|---|---|---|---|
| External-user usability | DCR-003 L1 role simulation, external-operator packet, and record helper. | Real non-author operator or external reviewer completes the packet; all friction is fixed, deferred, or accepted with rationale. | COMMONS / operator review | Non-author usability or public operator readiness for the declared workflow. |
| Clean reproducibility | DCR-007 data-dirty candidate replay, clean replay packet, and strict clean-run launcher. | Clean checkout and data-backed replay record accepted by MERIDIAN/COVENANT, with VAULT disposition for any promoted artifacts. | MERIDIAN / COVENANT / VAULT | Clean release-subset or full-scale reproducibility for the declared scope only. |
| Concrete public evidence bundle | DCR-004 L1 evidence-package contract and internal checklist. | Named bundle with manifest, hashes, limitations, non-claims, and custody disposition reviewed against `BISECT-EVIDENCE-PACKAGE-v1`. | DATUM / SCALE / COMMONS / VAULT | Public evidence-package publication readiness for that bundle only. |
| Legal/court boundary | DCR-006 L1 court/legal packaging boundary and checklist. | Jurisdiction-specific human/legal authority review for the intended use and language. | BOUNDARY / WARD / COMMONS | Court-ready, filing-ready, official, statutory, approved, or certified language for that reviewed scope only. |
| Public import interoperability expansion | DCR-001 L2 fixtures for the named CSV, GeoJSON, RPLAN, and shapefile/DBF import-label fixture set; DCR-005 L1 matrix. | Additional fixture and matrix rows for any broader external-tool, round-trip, version, or export claim. | LEDGER / package owners / VAULT when externally sourced | Compatibility claims beyond the named fixture-backed import-label set. |

## Stop rules

Stop and update the relevant DCR, trace, review, and custody records before:

1. Publishing generated reports, dashboards, maps, packages, or bundles as public
   evidence.
2. Describing BISECT outputs as legal, court-ready, filing-ready, official,
   approved, statutory, or certified.
3. Describing quickstarts or CLI workflows as externally validated or
   non-author ready.
4. Describing any run as clean, reproducible, release-subset reproducible, or
   full-scale reproducible.
5. Expanding interoperability claims beyond the named fixture-backed import
   scope.

## Closure rule

This register is closed only when every gate above is either:

- passed with evidence and review disposition,
- explicitly marked out of release scope, or
- superseded by a newer S6 readiness decision.

Until then, use this register as a routing surface for release-grade work, not
as release evidence.
