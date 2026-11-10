---
skill: roles-check
topic: wasm-adjacency-release-bridge
date: 2026-10-26
roles_used: [BENCHMARK, COVENANT, LEDGER, CONTOUR, VAULT, TRENCH]
p1_count: 0
p2_count: 6
p2_open: 1
p3_count: 12
verdict: APPROVED-WITH-CONDITIONS
---

# Private release bridge review

One coding agent applied six installed role lenses to the Worker, explicit client
opt-in and [contract](../../../docs/specs/wasm-adjacency-release-bridge.md).
These are review perspectives, not independent signatories. Approval applies to
the locally verified implementation; hosted acceptance remains open.

## BENCHMARK — real-provider and test fidelity

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| B1 | Controlled transport cannot prove the private CORS workaround. | P2 locally closed | Both sealed profiles pass actual private GitHub download/export/preparation through the production handler in local Chromium. Direct failure evidence retained. |
| B2 | Node-hosted Worker success does not prove Cloudflare deployment, limits or operator logging/rate controls. | P2 OPEN | No deployment claim. Verify intended host and operational controls before accepting hosted support. |
| B3 | The existing public relay could regress. | P3 bounded | Its full boundary regression passes for both staged profiles; public allowlist remains unchanged. |

## COVENANT — provenance and authority

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| C1 | A proxy can substitute bytes or stale asset selection. | P2 locally closed | Client checks original published size/digest after server re-discovers asset membership; exact original RI sources are checked in actual browsers. |
| C2 | Hash agreement does not authenticate the publisher. | P3 carried | Receipts retain source_authenticity_verified=false and disclose transport only. |
| C3 | Imported projects or receipts must not restore credential authority. | P3 bounded | Live selection WeakMap, explicit checkbox and existing inert Open/retirement remain; no bridge token is serialized. |

## LEDGER — formats and boundaries

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| L1 | Arbitrary request bodies could turn a bridge into a generic proxy. | P2 locally closed | Exact body keys, 2 KiB cap, fixed repository/tag and selected release membership; wrong scope/IDs/URLs refuse before binary fetch. |
| L2 | Transport should be visible without exposing secrets. | P3 bounded | Receipt records direct/site-bridge; exports contain neither token nor signed URL. |
| L3 | Static hosts cannot implement a POST Worker endpoint. | P3 carried | Direct/local import remains available; checkbox is explicit, absent endpoint fails rather than silently changing transport. |

## CONTOUR — input meaning

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| G1 | Successful acquisition could hide missing physical geometry. | P3 bounded | Preparation remains explicit and uses selected matching supplemental attributes in the actual RI tests. No geometry synthesis or override. |
| G2 | Opaque pickle/display/request/custom Census county files could become executable or physical inputs. | P3 bounded | Existing native-input name allowlist remains; these bodies are excluded. |
| G3 | RI 2020 cannot establish every vintage/resolution/state. | P3 carried | Exact scope documented; broader native lifecycle and matrix stay open. |

## VAULT — delegated credentials

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| V1 | The bridge expands who can receive the token. | P2 locally closed | Unchecked explicit option names this site's receipt of token; same-origin POST only, manual HTTPS CDN redirects drop auth; no server secret store/log calls. Host trust remains disclosed. |
| V2 | Signed URLs or provider headers could leak into browser artifacts. | P3 bounded | Worker constructs its own response headers and streams bytes only; real exported projects/receipts checked in memory before artifact write. |
| V3 | No-store cannot guarantee heap erasure or privileged-script resistance. | P3 carried | Narrow guarantees documented; contexts close, authority retires, no broader security claim. |

## TRENCH — failures and cancellation

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| T1 | Interrupted or malicious streams could partially replace accepted sources. | P2 locally closed | Server body bounds/deadlines/cancellation and client whole-bundle validation; overflow, short body, stalled transfer, cancel and abort tests pass. |
| T2 | Redirects may reach unintended hosts. | P3 bounded | Manual redirect limit and exact recognized HTTPS hosts; unsafe redirect refuses before destination fetch. |
| T3 | Same-origin checks are not global service rate limiting. | P3 carried | Caller supplies GitHub credential, fixed scope, bounded requests; operational rate controls remain part of B2 before hosted acceptance. |

Six roles /18 findings: zero P1, six P2 (five locally closed, one OPEN), 12 P3.
VAULT and COVENANT agree that token delegation must be explicit and grants no
authenticity. BENCHMARK and TRENCH agree local success cannot accept hosted service.

Amendments applied: (1) explicit unchecked credential-delegation option and fixed
scope; (2) independent client digest validation and secret-free response/receipt;
(3) retain real direct failure alongside successful live local bridge proof and
stream-failure tests. PP-49 records the new delegation/proxy pitfall. Full parent,
hosted release and maintainer acceptance remain open.
