---
skill: roles-check
topic: wasm-native-state-identity
date: 2026-10-26
roles_used: [benchmark, covenant, ledger, survey, trench]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# State identity plan and inspection foundation review

Artifact: `docs/specs/wasm-native-state-identity.md` and shared naming/inspection
code. One coding agent applies five installed roles, not independent signatories.
BENCHMARK checks parity evidence; COVENANT checks custody and source identity;
LEDGER checks native interchange; SURVEY checks the practitioner workflow;
TRENCH checks refusal boundaries. Approval covers implementation planning and
guarded inspection only. Execution exposure remains unaccepted.

| ID | Role | Finding | Severity | Required action / status |
|---|---|---|---|---|
| B1 | BENCHMARK | Name resolution has distinct adjacency and registry fallback branches. | P2 | Shared Rust rule; unit tests include unknown-code spaces and raw overrides. Locally addressed. |
| B2 | BENCHMARK | Identity inspection does not prove full native output behavior. | P2 | Actual CLI label/tree comparisons and browser Run/replay/export remain required. Open. |
| B3 | BENCHMARK | Rust host tests do not prove the WASM ABI path. | P2 | Nine fresh default WASM ABI cases pass; parser errors and all 14 direct compiler refusals remain guarded. Locally addressed for this foundation only. |
| C1 | COVENANT | Source policy affects registered names. | P2 | Retain embedded policy hash/source kind; selected-policy matching remains required before wider exposure. Partial. |
| C2 | COVENANT | Historic accepted binaries cannot stand in for a new engine. | P2 | Preserve old sealed packages; fresh integration/profile/package evidence is required for the completed workflow. Open. |
| C3 | COVENANT | Source preimages are useful but do not authenticate upstream origins. | P3 | Keep developer evidence separate from source authenticity and legal acceptance. |
| L1 | LEDGER | Raw aliases cannot be silently normalized into different native targets. | P2 | Inspector preserves exact text. Future portable-path refusals must preserve original intent. Partial. |
| L2 | LEDGER | An explicit label has precedence over the generated label. | P2 | Trace accepted exports, aliases and manifests before mapping the flag. Open. |
| L3 | LEDGER | Demographic and character-source names use consumer-specific normalization. | P3 | Enumerate all basename consumers; do not reuse a universal slug rule. Open. |
| U1 | SURVEY | A human-readable name can change data and output choices. | P2 | Present a concrete source/target review before Run; invalidate stale review on changes. Open. |
| U2 | SURVEY | Imported projects must not restore file authority. | P2 | Explicit verification and re-selection before continuation/installation. Open. |
| U3 | SURVEY | Internal inspection should not be advertised as completed support. | P3 | No new UI control or README support claim; direct refusal remains. Locally addressed. |
| T1 | TRENCH | Unsafe paths can be useful observations but cannot grant writes. | P2 | Raw inspection grants no path authority; execution must retain independent portable path validation. Partial. |
| T2 | TRENCH | Removing the compiler guard before consumer mappings would silently drop effects. | P2 | Keep all 14 direct refusals; strip only after checked workflow binding. Locally addressed. |
| T3 | TRENCH | Unknown location naming does not validate year or district allocation. | P3 | Inspector explicitly disclaims complete native preflight; future execution must preserve it. Open. |

Roles reviewed: 5. P1: 0; P2: 11; P3: 4.
Verdict: APPROVED-WITH-CONDITIONS for the plan and guarded inspection, not the
complete feature. The top finding is that naming must reach every real consumer
before the execution refusal is lifted. LEDGER, SURVEY and TRENCH agree.

Three amendments incorporated into the plan: preserve the adjacency-only raw
override and separate fallback branches; enumerate labels/auxiliary sources/target
review as implementation requirements; require actual native/browser/profile
evidence and immutable custody before support registration. Broader parent,
matrix, host permissions and release acceptance remain open.
