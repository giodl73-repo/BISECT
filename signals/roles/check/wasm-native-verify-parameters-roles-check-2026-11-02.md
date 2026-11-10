---
skill: roles-check
topic: wasm-native-verify-parameters
date: 2026-11-02
roles_used: [benchmark, covenant, ledger, survey, trench]
p1_count: 3
verdict: NEEDS-WORK
---

# Native Verify parameter experiment review

Artifact: target/native-verify-parameters-20261102, based on the complete current 38-file candidate. All installed role files were inventoried; five lenses apply to executable replay, evidence, manifest compatibility, practitioner use and failure prevention. One agent applies the lenses, without delegation or independent human signatories.

| Lens | Finding | Severity | Evidence and action |
| --- | --- | --- | --- |
| BENCHMARK | Three original multi-district Verify cases still fail strict assignment equality. | P1 | strict-replay-recorded-tolerance.log preserves seed 0, seed 17 and max-u64 failures. Do not substitute one district, lower similarity or alter the original single-search scope. |
| BENCHMARK | Actual CLI controls isolate a missing search setting. | P3 | Exact recorded seed/tolerance reach the candidate child; separate direct State calls with original single search reproduce all three original assignments. Those controls are not passing Verify calls. |
| BENCHMARK | Existing bounded regressions remain intact. | P3 | Three actual output-root/precheck/mismatch controls pass; 58 native Verify unit tests pass, with the one ignored environment case separately executed successfully against the original real ILP/RPLAN package. |
| COVENANT | The manifest does not retain enough general engine configuration to reproduce this original run. | P1 | Original single and default multi-search can have otherwise equal manifest settings yet different assignments. A bound original execution recipe and explicit legacy evidence limits are needed. |
| COVENANT | Legacy negative seeds cannot be authenticated or safely recovered as a guessed unsigned value. | P2 | Baseline State wraps max-u64 to -1. Candidate preserves new unsigned values in existing engine_seed and explicitly refuses a negative legacy value without it. Historical records remain unchanged. |
| COVENANT | Local identities establish the observed experiment, not execution authenticity. | P3 | Owned binaries, original producer bytes in v2+, raw invocations, source preimages, failed attempts and dependency receipt are retained; no attestation or legal certification is inferred. |
| LEDGER | Existing unsigned seed schema must keep precedence and signed compatibility. | P2 | Candidate uses engine_seed first, nonnegative legacy seed second, and preserves absent seed. WASM's existing unsigned consumer remains unchanged; wider consumers and profiles still require acceptance. |
| LEDGER | Exact recorded tolerance differs from reconstructing the user's earlier decimal. | P2 | Native State already records 7.125 as 7.124999999999999 after percent conversion. The corrected reader checks exact recorded bytes/value and keeps the original requested value separately; first assertion failures remain. |
| LEDGER | No new manifest schema is introduced by the two native edits. | P3 | Producer uses the existing optional engine_seed field; 3,771 other copied source inputs were byte-exact before the new standalone regression was added. |
| SURVEY | This is an unadopted native experiment with no new browser acceptance. | P2 | Keep it separate from the tested 38-file candidate and from full Verify support. No README, deployment, C-profile or package claim is advanced. |
| SURVEY | The parameter probe is a small synthetic grid. | P2 | Sixteen units exercise actual multi-district CLI behavior; they do not replace original max-u64 browser, real-data, method/backend, workspace or host obligations. |
| SURVEY | Absent recorded seed behavior is preserved explicitly. | P3 | The one-district absent-seed control passes without synthesizing a fixed seed. It is not substituted for the three failing multi-district cases. |
| TRENCH | A failing strict gate must not be registered as completed builder acceptance. | P1 | New standalone maintained-relative probe remains failed and unregistered; complete replay work must make its original assertions pass before closure. Original R13 remains frozen and advancing. |
| TRENCH | Precheck failure and output-root behavior remain structurally tested. | P3 | Existing actual CLI regression retains original trees and stops on corrupted certificate; explicit negative seed control has no child effects. |
| TRENCH | Preparation and source-copy ordering need durable receipts. | P2 | Copy completed and all 3,773 inputs verified before edits/build. Two preparers refused before writes, and one wrong-workdir retry ran the unchanged probe. Preserve these failures; use absolute preparation paths and separate dependent commands. |

Five roles, fifteen findings: three P1, six P2, six P3. Verdict: NEEDS-WORK. The experiment improves actual parameter routing and new unsigned seed metadata, but complete native replay still fails and must stay open.

Amendments: preserve and replay a versioned, source-bound original engine recipe; retain exact original multi-district tests and recorded-tolerance semantics; only then reconcile source catalogs/builders and complete core/C/browser/package acceptance. No approval pause, adoption, commit, push or deployment is implied.
