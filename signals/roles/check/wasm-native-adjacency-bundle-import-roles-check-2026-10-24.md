---
skill: roles-check
topic: wasm-native-adjacency-bundle-import
date: 2026-10-24
roles_used: [BENCHMARK, COVENANT, LEDGER, CONTOUR, VAULT, TRENCH]
p1_count: 0
p2_count: 4
p3_count: 14
verdict: APPROVED-WITH-CONDITIONS
---

# Grouped local adjacency implementation review

Artifacts: [bounded contract](../../../docs/specs/wasm-native-adjacency-bundle-import.md),
helper/workbench, unit/browser tests and dual-shell packager. One coding agent
applied installed role lenses, not independent signatories or full release acceptance.
BENCHMARK checks observable proof; COVENANT provenance; LEDGER compatibility;
CONTOUR geographic/physical source semantics; VAULT imported-data boundaries;
TRENCH asynchronous replacement and failure modes.

## BENCHMARK

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| B1 | A filename-only successful import does not prove the resulting native input. | P2 locally closed | Both profiles compare the complete shared native-host/WASM preparation reply and exact source bytes; browser explicitly prepares afterward. |
| B2 | Synthetic local files do not prove release provider/capacity coverage. | P3 carried | No remote acquisition claim; external real-data tier remains open. |
| B3 | Metadata path typo prevented a passing package record. | P3 closed locally | Original failure retained; corrected path preflight and fresh v2 package verification succeed. |

## COVENANT

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| C1 | Release-style names and observed hashes do not authenticate publication. | P3 carried | Contract and helper declare local selection/source authenticity false. |
| C2 | Downloading files and importing them are different workflows. | P3 carried | Native gh/private auth and automatic release discovery stay missing in coverage ledger. |
| C3 | Rewriting JSON companions would change source identities. | P3 bounded | Base64 retains original bytes; hashes/project roundtrip exact. |

## LEDGER

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| L1 | A new project schema would need another compatibility surface. | P3 bounded | Existing typed native input/settings fields reused; no new persisted receipt/grant. |
| L2 | Binary-only replacement must not retain unrelated companions. | P3 bounded | Fresh source dictionary replaces the entire selection; actual browser checks null companions. |
| L3 | Active source reconciliation must not rewrite historical acceptance. | P3 bounded | Prior inventory retained; source/candidate pins only; historical bundles stay unchanged and parents unresolved. |

## CONTOUR

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| G1 | Display geometry was incorrectly bound as physical source geometry. | P2 locally closed | Actual native guard refused first attempt. Corrected import leaves display unread; exact native physical preparation passes. |
| G2 | Declared state/year names cannot establish the actual unit universe. | P3 carried | Native content preparation remains explicit; no filename-based certification or cross-vintage equivalence. |
| G3 | Tract release scope does not cover county/block/custom imports. | P3 carried | Group import refuses non-tract settings; existing explicit individual controls remain separate. |

## VAULT

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| V1 | Pickle/request selection must not become code or execution intent. | P2 locally closed | Unit and browser instrumentation prove ignored files are never read; zero import engine dispatch. |
| V2 | Wrong/unsafe/oversized selections should refuse before reading. | P3 bounded | Complete metadata preflight with bounded count/size; nine refusal classes and duplicate/mixed scope checks. |
| V3 | Local source bytes must not restore folder authority. | P3 bounded | Typed source-only project fields, inert Open, no handles or installation behavior. |

## TRENCH

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| T1 | A late file read could undo Clear or replace newer inputs. | P2 locally closed | Revision checks before/after asynchronous reads; actual browser paused read then Clear preserves the cleared state. |
| T2 | An import error could partially replace old sources. | P3 bounded | Commit after all validation; actual failed import keeps exact previous source dictionary. |
| T3 | Low host space can derail evidence production. | P3 carried | 200 MiB check-host reserve, ephemeral browser contexts and shared immutable base bytes; no product limit change or deletion. |

## Synthesis and amendments

Six roles, 18 findings: zero P1, four locally closed P2, 14 carried/bounded P3.
APPROVED-WITH-CONDITIONS for local tract release-file import only. Top finding:
display geometry and physical source measurements have different authority.
CONTOUR and COVENANT agree that selected filenames cannot establish source truth.

Amendments applied: (1) leave display/pickle/request bodies unread; (2) preflight
all names/sizes and atomically commit original bytes; (3) native exact comparisons
and both actual staged browsers, including Clear/failed/binary-only cases. Broader
release acquisition/provider authenticity/native parent and acceptance-v1 stay open.
PP-47 records the display/physical-source conflation failure. No deployment is implied.
