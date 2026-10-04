---
skill: roles-check
topic: precomputed-web-laboratory
date: 2026-10-03
roles_used: 12
p1_count: 3
verdict: APPROVED-WITH-CONDITIONS
---

# Precomputed web laboratory role review

Reviewed artifact: [implementation plan](../../../../docs/specs/2026-10-03-precomputed-web-laboratory.md).
This is a design review using the installed roles' verify lenses, not an
independent certification or a claim that the implementation has passed yet.

## Role selection

BOUNDARY and WARD review legal framing and jurisdictional scope; CONTOUR reviews
Census joins; MERIDIAN reviews executed algorithms; BENCHMARK reviews test
instruments; SCALE and DATUM review statistical claims and coverage; COVENANT
reviews provenance and disclosure; LEDGER reviews formats; TRENCH reviews
failure prevention; COMMONS reviews the public interpretation; SURVEY reviews
delivery and operational feasibility. CANVASS and TALLY are excluded because
this catalog contains no election canvass or ballot accounting. VAULT's ballot
privacy questions are not triggered; public export sanitization is covered by
COVENANT and TRENCH. The role files' general legal assertions are not adopted as
authoritative current legal guidance.

## Findings

| Role | Finding | Severity | Plan section | Recommendation |
|---|---|---|---|---|
| BOUNDARY | Experimental tolerance can be mistaken for legal sufficiency. | P2 | Browser behavior | Label tolerance as an experiment setting. |
| BOUNDARY | Completed runs do not certify VRA compliance. | P2 | Browser behavior | Preserve heuristic and unproved labels. |
| BOUNDARY | No racial voting analysis is supplied by this catalog. | P3 | Scope | Do not imply a VRA comparison. |
| WARD | Chamber allocations and tolerances must not be conflated. | P2 | Catalog contract | Include chamber and district count in matching. |
| WARD | Split counts are not state-law compliance decisions. | P3 | Browser behavior | Describe the measured county metric. |
| WARD | The first catalog cannot claim legislative nesting support. | P3 | Scope | Start with congressional runs; expose no nesting claims. |
| CONTOUR | A map joined to the wrong tract vintage is a false result. | P1 | Catalog contract | Require exact unique GEOID coverage and year identity. |
| CONTOUR | Derived graph hashes do not establish upstream Census custody. | P2 | Catalog contract | Disclose the provenance boundary. |
| CONTOUR | Reusing geometry is valid only for identical display inputs. | P2 | Catalog contract | Reject conflicting geometry within a state and year. |
| MERIDIAN | Accepted CLI flags may not be executed by a structure. | P1 | Scope | Validate compatibility and exclude misleading historical requests. |
| MERIDIAN | Inactive budgets can produce false configuration distinctions. | P2 | Browser behavior | Define effective configuration matching. |
| MERIDIAN | Saved heuristic endpoints do not prove a best cut. | P2 | Browser behavior | Keep optimality unproved. |
| BENCHMARK | Local data cannot be required for CI tests. | P2 | Acceptance checks | Use complete small fixtures. |
| BENCHMARK | Wrong joins and asset corruption need failing tests. | P2 | Acceptance checks | Exercise both rejection paths. |
| BENCHMARK | Root-path testing misses GitHub project-path failures. | P2 | Acceptance checks | Test the site beneath a nested URL. |
| SCALE | A selected set of seeds is not a calibrated distribution. | P2 | Computation | Record explicit seeds; do not invent uncertainty estimates. |
| SCALE | National averages can hide state deviations and missing states. | P2 | Browser behavior | Show coverage and worst-state deviation. |
| SCALE | A curated grid does not establish a universally superior method. | P2 | Scope | Compare measured tradeoffs, not universal rankings. |
| DATUM | All-options language exceeds a finite experiment grid. | P2 | Scope | Publish exact configurations and missing combinations. |
| DATUM | Failures omitted from the catalog create selection bias. | P2 | Computation | Retain sanitized state failure records. |
| DATUM | Engine and dataset differences confound comparisons. | P2 | Catalog contract | Preserve hashes and build provenance. |
| COVENANT | Older records may lack an executable digest. | P2 | Catalog contract | Disclose absence; hash executables for new computations. |
| COVENANT | Raw records contain local paths and process context. | P1 | Catalog contract | Export an allowlist; omit logs, command vectors and tokens. |
| COVENANT | SHA-256 is integrity evidence, not a digital signature. | P3 | Catalog contract | State the limited guarantee. |
| LEDGER | Browser assets need a versioned schema. | P2 | Catalog contract | Reject unknown versions. |
| LEDGER | Assignments and geometry require explicit formats. | P2 | Catalog contract | Document GEOID keys and GeoJSON coordinate order. |
| LEDGER | Leading zeros can be lost through numeric normalization. | P2 | Catalog contract | Keep GEOIDs as strings. |
| TRENCH | Partial exports must not appear complete. | P2 | Catalog contract | Refuse populated destinations and write manifest last. |
| TRENCH | Missing combinations could silently load nearby results. | P2 | Browser behavior | Require exact effective matches and show missing status. |
| TRENCH | Regeneration can leave stale assets in place. | P2 | Catalog contract | Build into a fresh directory. |
| COMMONS | Compactness and county splits do not describe all communities. | P3 | Browser behavior | Avoid fairness or representation claims. |
| COMMONS | Expensive states could disappear from a public demonstration. | P2 | Scope | Disclose successful, failed and absent state coverage. |
| COMMONS | Users need to distinguish a saved plan from a live run. | P2 | Browser behavior | Rename action to View saved results. |
| SURVEY | Publication should not require downloading raw Census files. | P2 | Delivery | Deploy an already prepared archive. |
| SURVEY | Large assets can exceed static hosting capacity. | P2 | Catalog contract | Share geometry and enforce a 900 MB budget. |
| SURVEY | Browsing must require no credentials or engine install. | P2 | Delivery | Use only relative static asset reads. |

## Synthesis and amendments

Roles reviewed: 12. P1 blockers: 3. P2 issues: 28. P3 notes: 5.
Verdict: APPROVED-WITH-CONDITIONS.

The top finding is that a public result must describe the algorithm actually
executed on the exact geographic units displayed. CONTOUR, MERIDIAN, BENCHMARK
and COVENANT agree that joins, effective configuration and provenance must be
machine-verifiable. DATUM, SCALE and COMMONS agree that catalog coverage must
not be confused with comprehensive evidence or legal validity.

The plan now includes three amendments: exact coverage with explicit missing
results; strict shared geometry joins and hashed, versioned assets; and a public
field allowlist with manual publication separate from computation. Conditions
are discharged only by implementation tests and the real static browser check.

## Implementation disposition — 2026-10-03

The three plan conditions are satisfied for the local pilot artifact:

- Exact effective settings and explicit unavailable status passed Rust and
  browser-adapter tests, plus a browser check using an uncomputed county
  strength. National totals disclose their three-state coverage.
- Versioned, hashed assets and strict GEOID joins passed corruption, duplicate,
  coverage and export tests. The complete real artifact passed the independent
  Node verifier; geometry is shared across six experiments.
- Public field sanitization passed tests with injected private process fields.
  Publication is a separate manual workflow. It preserves a supplied existing
  static site and installs only the laboratory beneath `laboratory/`.

Validation: 11 Rust tests, 4 JavaScript tests, 18 successful real state results,
and a static browser check of restoration, maps, comparisons and missing
settings. The artifact is 8,564,750 bytes; its release archive is 1,934,404 bytes.
Browser downloads were initiated without confirmation of the saved file;
exported map/assignment contents are verified independently. Full national
generation and the first GitHub Pages deployment remain unverified.

Verdict for the tested local pilot: READY FOR PUBLICATION PREPARATION. This is
the primary agent applying the installed role lenses, not an independent audit
or a certificate of algorithmic optimality or legal compliance.
