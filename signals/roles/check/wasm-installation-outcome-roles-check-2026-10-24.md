---
skill: roles-check
topic: wasm-installation-outcome
date: 2026-10-24
roles_used: [BENCHMARK, COVENANT, LEDGER, VAULT, TRENCH]
p1_count: 0
p2_count: 3
p3_count: 12
verdict: APPROVED-WITH-CONDITIONS
---

# Installation outcome implementation review

Artifact: [bounded UI contract](../../../docs/specs/wasm-installation-outcome.md),
`native-state-output-ui.js`, the two extended browser gates, staging and validation
scripts. Type: code/workflow implementation. One coding agent applied installed
role lenses; these are not independent signatories or formal release acceptance.

BENCHMARK checks observable regression coverage, COVENANT checks custody and
reproducibility claims, LEDGER checks schema compatibility, VAULT checks imported-
data and capability boundaries, and TRENCH checks interruption/failure outcomes.
Geographic/statistical lenses are outside this UI-only change.

## BENCHMARK

| ID | Finding | Severity | Evidence / resolution |
|---|---|---|---|
| B1 | UI discarded actual interruption receipts. | P2 closed locally | Both staged batch gates download the exact cancelled on-disk receipt and completed recovery receipt. |
| B2 | Mocked picker is not a Windows permission test. | P3 carried | Contract identifies actual OPFS plus injected picker; real selected-folder hosts remain open. |
| B3 | Synthetic fixtures do not prove national installation capacity. | P3 carried | Three state/year fixtures regenerate; no national reacceptance claim. |

## COVENANT

| ID | Finding | Severity | Evidence / resolution |
|---|---|---|---|
| C1 | Missing final receipts could be mistaken for zero effects. | P2 closed locally | Null receipt, unknown completed writes and explicit interruption wording; both single browser gates assert it. |
| C2 | Browser observation is not independent native execution provenance. | P3 carried | Wrapper and contract explicitly distinguish native artifacts from browser outcome. |
| C3 | Session-only custody needs clear disclosure. | P3 carried | UI tells users to download before changing inputs/closing; no claim of automatic durable custody. |

## LEDGER

| ID | Finding | Severity | Evidence / resolution |
|---|---|---|---|
| L1 | New export must not silently change existing formats. | P3 bounded | Versioned wrapper, original nested receipt copied; installer and native formats unchanged. |
| L2 | Historical hashes must remain historical. | P3 bounded | Old active inventory retained, only 12 current-source/candidate references reconciled; old runtime bundles unchanged. |
| L3 | Completed/failed rows must not be summarized away. | P3 bounded | Complete nested receipt equals independent on-disk checkpoint, including ordered target rows. |

## VAULT

| ID | Finding | Severity | Evidence / resolution |
|---|---|---|---|
| V1 | Saved outcomes must never restore filesystem authority. | P2 closed locally | Session-only primitive metadata, no handle/review, no import consumer; inert Open clears outcome. |
| V2 | Local error/path information may be present in downloaded JSON. | P3 bounded | Explicit local download, no remote upload; error limited to 2,000 characters. |
| V3 | This does not defend against privileged same-origin scripts. | P3 carried | Existing private review/permission boundary and threat model remain; no stronger security claim. |

## TRENCH

| ID | Finding | Severity | Evidence / resolution |
|---|---|---|---|
| T1 | Disk failure can prevent writing a final batch checkpoint. | P3 carried | Wrapper permits null receipt and says completed writes are unknown; no synthesized success. |
| T2 | An old success could obscure a new failed attempt. | P3 bounded | New valid attempt retires old outcome; changed-folder refusal produces interrupted/null receipt, asserted in both single gates. |
| T3 | Outcome download itself must be inert. | P3 bounded | Download preserves exact target bytes and request/picker counts; zero recovery partitions. PP-45 records the general failure pattern. |

## Synthesis and amendments

Five roles, 15 findings: no P1, three P2 locally closed, 12 bounded/carried P3.
APPROVED-WITH-CONDITIONS for the two named local shells. Broader desktop/host,
durable cross-session custody, full acceptance-v1 and deployment remain open.
Top finding: an interruption needs an honest downloadable observation even when
no final receipt exists. COVENANT and TRENCH agree that null receipt never proves
absence of side effects.

Amendments applied: (1) version the passive wrapper and preserve exact original
receipts; (2) explicitly disclose unknown writes and session-only lifetime;
(3) test actual browser downloads, exact OPFS checkpoints, changed-target refusal,
inert Open and both staged shells. No Rust engine or installer semantics change.
