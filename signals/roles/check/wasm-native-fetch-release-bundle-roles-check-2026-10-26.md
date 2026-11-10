---
skill: roles-check
topic: wasm-native-fetch-release-bundle
date: 2026-10-26
roles_used: [BENCHMARK, COVENANT, LEDGER, CONTOUR, VAULT, TRENCH]
p1_count: 0
p2_count: 7
p2_open: 1
p3_count: 11
verdict: APPROVED-WITH-CONDITIONS
---

# Native wildcard release export review

One agent applied the six installed lenses to the [contract](../../../docs/specs/wasm-native-fetch-release-bundle.md),
client purpose, Worker boundary, ZIP packager and native/browser evidence. These
are perspectives, not independent reviewers or maintainer release acceptance.

## BENCHMARK

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| B1 | A mocked provider cannot prove real private wildcard export. | P2 locally closed | Both exact browser profiles compare ZIP bytes with fresh actual native CLI/gh Fetch outputs. Controlled native CLI/CRC tests are separate. |
| B2 | Local Worker/CLI build success does not prove deployed Cloudflare host controls. | P2 OPEN | Dry-run only; actual origin/runtime/rate/log policy and wider provider/matrix acceptance remain open. |
| B3 | A transient browser failure must not become an unreported successful retry. | P3 bounded | First connection-closed preflight failure retained, alongside independently completed C and default retry records. |

## COVENANT

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| C1 | A generated done marker could be mistaken for trusted completion. | P2 locally closed | Exact native marker bytes disclosed; receipt separately records primary presence, installed=false and authenticity=false; no filesystem authority or engine execution. |
| C2 | The ZIP might omit original opaque sources. | P3 bounded | Complete bounded wildcard discovery; actual native pickle/binary/GEOID bytes and marker all match. |
| C3 | Observed hashes cannot establish publisher identity. | P3 carried | Published/observed hashes remain distinct; source authenticity and signed custody stay unclaimed. |

## LEDGER

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| L1 | Input import and complete Fetch export have different contracts. | P2 locally closed | Explicit purpose captured in live selection/receipt and rechecked by server; default import unchanged and separately regressed. |
| L2 | Native data belongs under outputs/V3, irrespective of manifest's schema version. | P3 bounded | Fixed native-source layout, actual CLI file-tree comparison and independent ZIP read prove the relative directory. |
| L3 | Export is not skip/force/install lifecycle parity. | P3 carried | Receipt and UI explicitly say no installation; selected-root review/lifecycle remains separate work. |

## CONTOUR

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| G1 | Pickle or display bytes could become physical graph data. | P2 locally closed | Bundle path returns opaque byte arrays; no native import/preparation callback or engine request. Invalid synthetic pickle survives byte-exactly without parsing. |
| G2 | Filename matching could silently narrow the native wildcard. | P3 bounded | Preserve prefix wildcard, including longer suffixes, empty files and request/display assets; unsafe names fail explicitly instead of omission. |
| G3 | One RI release does not validate all states or missing geometry. | P3 carried | No derivation/preparation claim; wider releases and geometry remain separate. |

## VAULT

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| V1 | Broader asset membership could expand credential destinations. | P2 locally closed | Same explicit bridge opt-in, fixed server repository/tag, bounded membership, constructed headers and no auth on asset hosts; real exports checked for token/signed URL. |
| V2 | Opaque JSON assets differ from API metadata masquerading as a download. | P3 bounded | Permit JSON only for recognized final asset responses in bundle mode; API JSON still refuses. Files remain unexecuted. |
| V3 | Arbitrary filenames could escape ZIP extraction paths. | P3 bounded | Portable flat names, no path/colon/control characters or trailing dots; generated fixed directory and non-executable ZIP attributes. |

## TRENCH

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| T1 | Case aliases or a provider marker could overwrite another entry. | P3 bounded | Case-fold duplicate and marker-collision refusal before downloads; supplemental name tests pass. |
| T2 | Cancel after acquisition could still emit a ZIP while package hashing finishes. | P2 locally closed | Real-browser fault injection reproduces one ZIP on the prototype, zero on final v2; per-hash signal/revision checks and a final offer guard close the window. Completed native inputs remain unchanged. |
| T3 | Marker without pickle could be reported as ready. | P3 bounded | Binary-only test records primary=false, whereas pickle-only remains opaque. No filesystem readiness claim is issued. |

Six roles /18 findings: zero P1, seven P2 (six locally closed, B2 OPEN), 11 P3.
COVENANT and LEDGER agree marker-byte correspondence is not installation or trust.
CONTOUR and VAULT require opaque retention without deserialization or credential
destination widening. The retained package remains implemented/evidence-partial.

Amendments applied: (1) separate explicit purpose and whole-bundle validation;
(2) fixed native layout/marker with source-preserving, non-executing ZIP export;
(3) retain actual provider/native comparisons, first failure and narrow hosted
boundary. PP-51 records marker/opaque-file authority confusion. Full parent,
deployment and maintainer acceptance remain open.
