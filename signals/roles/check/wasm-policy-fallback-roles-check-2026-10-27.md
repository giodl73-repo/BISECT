---
skill: roles-check
topic: wasm-policy-fallback
date: 2026-10-27
roles_used: 6
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

Standard single-agent application of installed lenses, not independent signatories.
Artifacts: optional Rust fallback input/selector, closed recipe, guided controls,
selection/refusal tables, actual CLI/worker checks and staged package. BENCHMARK:
independent observations; COVENANT: provenance; LEDGER: interchange; VAULT:
authority; SURVEY: practitioner outcomes; TRENCH: failure structure.

| ID | Lens | Finding | Severity | Resolution or remaining condition |
|---|---|---|---|---|
| B1 | BENCHMARK | Matching a second implementation alone does not prove CLI source precedence. | P2 | Actual command reads valid/invalid/empty/BOM/UTF-8/missing files in 15 cases per profile. |
| B2 | BENCHMARK | A historical temporary reference would break fresh registered runs. | P2 | Explicit CLI reference parameter; builders pass the reference they just generated. |
| B3 | BENCHMARK | New selector evidence could imply a full pipeline pass. | P3 | Component/profile/package checks only; new whole builder remains open. |
| C1 | COVENANT | Imported unavailable observations could appear authenticated. | P2 | Recipe/result/UI label them as data without live path authority. |
| C2 | COVENANT | Formatter output/source kind could change during fallback. | P2 | Exact native bodies/hash checks and embedded source-kind assertion. |
| C3 | COVENANT | Old package receipts might be repinned to new binaries. | P3 | New owned binaries/overlays; original runtime receipts stay historical. |
| L1 | LEDGER | Large integer policy data can be rounded by JS parsing. | P2 | Native formatter strings compared exactly; display retains existing decimal-string boundary. |
| L2 | LEDGER | Empty bytes and valid empty JSON have different native meanings. | P2 | Zero-byte parse failure continues; valid `{}`/array/scalar stops before lookup refusal. |
| L3 | LEDGER | New optional recipe could break old projects/requests. | P3 | Existing shape retained; 68 strict CLI cases and full project regression pass. |
| V1 | VAULT | Imported recipe might trigger file/env probing or execution. | P2 | Closed byte/observation inputs; no ambient IO; Open/select/Save are inert. |
| V2 | VAULT | Browser guard rejection could fall back to an unintended database. | P2 | Parse selects first; shape/complexity/property/lookup errors never try another source. |
| V3 | VAULT | Cancellation could imply physical OS IO termination. | P3 | Only callback authority is retired; underlying IO abort not claimed. |
| S1 | SURVEY | Lookup failure could be hidden as successful policy inspection. | P2 | Distinct refusal schema/table; policy exports disabled; native failure recorded separately. |
| S2 | SURVEY | A stale file-picker value prevents reselecting the same strict file after mode change. | P2 | Picker cleared on mode switch; real file adoption/reset tests. |
| S3 | SURVEY | Correct inspection could be mistaken for applying policy to a plan. | P3 | Read-only role and legal/source limits remain explicit. |
| T1 | TRENCH | Old delayed read can overwrite Open or release a new read's busy state. | P2 | Token/revision checks and ownership cleanup; controlled A/cancel/B/late-A/late-B/Open journey passes. |
| T2 | TRENCH | Rejecting reentrant reads inside cleanup can release another operation. | P2 | Early guards; browser directly invokes both handlers during delayed read. |
| T3 | TRENCH | New codegen could break optional C execution despite pure policy changes. | P3 | New C binary passes 52 actual native assignments/four failures; wider matrices still open. |

Roles reviewed: 6. P1: 0; P2: 12; P3: 6.
Verdict: APPROVED-WITH-CONDITIONS for explicit browser source-chain adaptation.
Top finding: parse success stops source fallback independently of lookup success.
LEDGER/SURVEY agree on empty-vs-valid JSON semantics; VAULT/TRENCH agree on
retiring read authority without replay or cross-operation cleanup.

Amendments applied: actual CLI source-chain comparisons; visible typed selection
and refusal outcomes; closed inert recipes with callback ownership and explicit
scope. Ambient discovery, broader host/capacity, new full builder/clean-checkout
and acceptance-v1 remain separate conditions.
