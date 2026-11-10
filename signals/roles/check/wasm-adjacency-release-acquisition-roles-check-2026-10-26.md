---
skill: roles-check
topic: wasm-adjacency-release-acquisition
date: 2026-10-26
roles_used: [BENCHMARK, COVENANT, LEDGER, CONTOUR, VAULT, TRENCH]
p1_count: 0
p2_count: 5
p2_open: 1
p3_count: 13
verdict: NEEDS-WORK
---

# Release acquisition implementation and acceptance review

Artifact: [contract/evidence boundaries](../../../docs/specs/wasm-adjacency-release-acquisition.md),
API client, UI, controlled browser/native tests and actual private-provider tier.
One coding agent applied six installed role lenses; no independent signatories.
BENCHMARK checks provider/test fidelity; COVENANT provenance; LEDGER schemas and
capabilities; CONTOUR source/geometry meaning; VAULT credentials; TRENCH failures.
NEEDS-WORK applies to full production/provider acceptance, not a claim that the
named passing local checks failed. The implementation remains evidence-partial.

## BENCHMARK

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| B1 | Real private asset responses lack browser CORS access. | P2 OPEN: reproduced | Authorized ephemeral-browser metadata succeeds; binary API-version preflight defect fixed, but actual asset GET fails MissingAllowOriginHeader. Final sanitized record: target/adjacency-release/private-browser-1793028279756/evidence.json. Use native download plus local import; a bridge and hosted-origin acceptance remain separate work. |
| B2 | Real private-provider bytes need an independent native consumer. | P3 bounded | Actual native CLI Fetch files equal adapter bytes; both native/WASM inspections, expected refusal and supplemented population builds match. |
| B3 | RI-only private assets do not prove other state/release capacity. | P3 carried | Explicit scope and bounded missing-binary errors; no invented all-state provider coverage. |

## COVENANT

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| C1 | Published digest consistency is not source/publisher authenticity. | P3 carried | Receipt/contract declare observations and source-authenticity false. |
| C2 | Native authenticated access must not export the CLI credential to JS. | P3 bounded | Live harness calls gh as its own authenticated client; no auth-token command or browser injection. |
| C3 | Full native wildcard/marker lifecycle differs from supported browser inputs. | P3 carried | Native reference retains pickle/done files; browser input path excludes those roles. Full lifecycle stays open. |

## LEDGER

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| L1 | Saved/copied discovery could become request authority. | P2 locally closed | Private live single-use WeakMap, active/copy/retire/reuse refusals; Open retires controls and grants nothing. |
| L2 | Missing provider digests must not be described as verified. | P3 bounded | Null digest + false verification flag, observed local SHA retained; dedicated test. |
| L3 | Reconciliation must not repin historical execution records. | P3 bounded | Source baselines/prototype bytes retained; current authority/candidate changes separate from old bundles. |

## CONTOUR

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| G1 | The actual release lacks geometry; weakening default validation would hide it. | P2 locally closed | Exact default refusal preserved; successful path uses an explicit hash-bound matching physical supplement, no optional-mode bypass. |
| G2 | Census county companions cannot override canonical GEOID prefixes. | P3 bounded | Both acquisition and local grouped import now exclude custom county bodies; regression asserts unread input. |
| G3 | A real supplemented two-district core build does not prove fresh derivation or all algorithms/backends. | P3 carried | Exact RI scope, seed, retained attribute provenance and core backend recorded; no C execution claim. |

## VAULT

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| V1 | Authorization or signed URLs could leak through redirects, HTTP cache or portable exports. | P2 locally closed | API-only header, browser-stripped cross-origin redirect header, no-store/cookie/referrer policy, no token/redirect URL in saved project or receipt; both shells tested. |
| V2 | Unsubmitted credentials could survive project Open. | P3 bounded | Revision change clears even a token without live discovery; actual browser assertion. Heap erasure/privileged script resistance not claimed. |
| V3 | Metadata endpoints must not become arbitrary network destinations. | P3 bounded | API/repository/asset-ID checks, constructed endpoints and recognized asset hosts; no generic server relay or policy widening. |

## TRENCH

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| T1 | A late failed/cancelled download could partially replace accepted sources. | P2 locally closed | Whole-bundle commit after hashes/size/validation; real browser bad-digest/cancel cases retain exact previous sources. |
| T2 | Pagination or mid-list changes could silently miss matching assets. | P3 bounded | Bounded complete pagination; repeated IDs/duplicate names and incomplete terminal page fail. |
| T3 | Provider quotas/transport failures are not successful acquisition. | P3 carried | Actual unauthenticated 404/403 retained; explicit errors, cancellation/deadlines and local import remain available. |

## Synthesis and amendments

Six roles /18 findings: zero P1, five P2 (four locally closed, B1 OPEN), 13 bounded/
carried P3. NEEDS-WORK for full production/provider acceptance. Top condition is B1;
BENCHMARK and COVENANT agree that authentication realms cannot substitute for one
another. Code and controlled/local scopes may be recorded as implemented/partial.

Amendments applied: (1) memory-only live authority with no-store and explicit
project-change retirement; (2) complete bounded metadata/asset validation and
atomic source commit; (3) preserve real native default refusal and require actual
matching physical attributes for the successful live build. Keep B1 and broader
source/native/host/full acceptance open. PP-48 records redirect/portable credential
leakage; no deployment or maintainer approval is implied.
