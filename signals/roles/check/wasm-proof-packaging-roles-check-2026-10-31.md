---
skill: roles-check
topic: wasm-proof-packaging
date: 2026-10-31
roles_used: 5
p1_count: 0
p2_count: 5
p3_count: 10
verdict: APPROVED-WITH-CONDITIONS
---

# Standard WASM proof packaging review

Scope: the R8 standard toolkit staging change in `scripts/web/build_wasm.ps1`,
its existing three proof staging recipes, dependency validator, README,
source-UI test repairs and guarded whole-build sequence. This is one agent
applying five installed role lenses, not five independent reviewers. Algorithm,
legal compliance and ballot handling are unchanged and are outside this review.

Continue the captured validation run. Before claiming verified packaged proof
support or releasing this as fully supported, resolve the P2 items below.
A passing dependency check alone is insufficient. No deployment is authorized
or performed by this review. No new permission gate is imposed.

## Selected roles

- BENCHMARK: changed tests and dependency closure need trustworthy oracles.
- COVENANT: third-party artifact/source receipts and reproducibility boundaries.
- DATUM: distinguish asset presence, execution, capacity and acceptance claims.
- SURVEY: prerequisites and clean-machine build usability.
- TRENCH: prevent repeated omissions and preserve failed-build boundaries.

## BENCHMARK

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| B1 | Existing staging checks reject mismatched identities, but this integration has no registered mutation matrix demonstrating those failure paths. | P2 | Standard builder / staging recipes | Add focused missing/altered artifact and receipt checks before verified release; require failure before a candidate is accepted. |
| B2 | The 1,255-reference validator recognizes literal imports and URL constructors. Arrays passed through `new URL(path, ...)` and non-index proof HTML entry points are not exhaustively covered. | P2 | `test_toolkit_assets.mjs` | Validate an explicit proof runtime dependency manifest and every staged HTML entry point; retain literal-module checks. |
| B3 | Static inspection found another old `Saved batch verified` wait in `test_prepared_state_browser.mjs`. It was not found registered in the standard/C builder scripts. | P3 | Prepared State browser test | Correct and exercise this separately after the captured run; do not count it as fresh acceptance now. |

## COVENANT

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| C1 | Checker staging verifies WASM bytes and vendor provenance but does not compare the receipt's eight adapter/lock/native-source fields to current inputs. All eight happen to match today, as the retained readback demonstrates. | P2 | `stage_proof_checker_preview.py` | Enforce current-source correspondence in staging and reject a stale adapter/lock receipt. Do not describe current byte consistency as proof of reproducible compilation. |
| C2 | Package SHA-256 receipts establish selected-byte identity, not publisher authenticity or signatures. | P3 | Proof package receipts | Keep this boundary explicit in release and practitioner claims. |
| C3 | Native proof staging binds the engine checksum and solver/checker bytes; a C-profile refresh needs its own completed package execution receipt. | P3 | `stage_native_proof_preview.py` / C builder | Preserve the profile-specific dependency receipt and test it with that exact engine. |

## DATUM

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| D1 | The integration has passed asset closure, not actual proof-page execution from the final standard and C packages. Existing source-UI diagnostics do not supply that acceptance. | P2 | Packaged proof browser coverage | Execute retained valid/invalid proof fixtures through final package routes for both profiles, with exact asset identities and native/checker references. |
| D2 | The dependency count is a presence measurement, not a capability count or proof of complete desktop parity. | P3 | Archive / README | Keep closure, runtime execution and formal acceptance as separate claims. |
| D3 | Small proof fixtures cannot establish statewide automatic proof generation capacity or optimality of arbitrary generated plans. | P3 | Proof support scope | Preserve existing small-fixture and incomplete-proof refusal scope; report failures explicitly. |

## SURVEY

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| S1 | The newly required prebuilt proof prerequisites are checked near the end of the standard builder, after potentially long all-state tests. | P2 | Standard/laboratory build preflight | Add a read-only shared dependency preflight before costly work; retain final staged-byte checks. |
| S2 | README now names prerequisite builders, but RoundingSat also requires separately acquired pinned local toolchain sources. | P3 | README / solver builder | Link an exact bootstrap/acquisition procedure and keep external acquisition explicit. |
| S3 | Solver/checker WASM increases the static package payload even when the ordinary workbench does not execute proof workflows. | P3 | Static package distribution | Retain lazy runtime loading and publish measured final package sizes after the full builds. No performance claim follows from asset closure. |

## TRENCH

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| T1 | The missing-worker-dependency failure class needs a durable pitfall entry with narrowly qualified status. | P3 | Pipeline pitfalls | Record PP-74 as mitigated, with asset-check evidence and pending dynamic/runtime coverage. |
| T2 | Staging recipes can leave a partially written output if a later recipe fails. Current build status/packager guards stop acceptance, but filesystem presence alone remains unsafe. | P3 | Builder failure lifecycle | Preserve unique outputs and the terminal successful-build requirement; consider staging into a temporary package before promotion. |
| T3 | Several package entry points copy all frontend JavaScript; future proof module additions can create another dependency omission. | P3 | Package construction | Reuse a single declared dependency inventory across standard/C/incremental packagers and test it against actual worker loads. |

## Synthesis

Roles reviewed: 5. P1 blockers: 0. P2 issues: 5. P3 notes: 10.

Verdict: APPROVED-WITH-CONDITIONS for continuing validation, not for a
verified-support release. Top finding: byte-checked asset staging and dependency
presence do not yet establish current-source enforcement plus final packaged
proof execution. BENCHMARK, COVENANT and DATUM agree that those boundaries must
remain explicit. No existing parity inventory status is promoted.

## Three amendments

1. Add shared source/receipt/prerequisite preflight to standard and C build paths,
   and invoke it before costly tests. Add negative cases for missing, altered and
   stale inputs. Recheck final staged bytes before promotion.
2. Describe proof dependencies explicitly, validate dynamic worker paths and all
   HTML entry points, and run valid/invalid proof fixtures from exact final
   standard/C package routes with source/native references.
3. Record PP-74 and preserve the scoped dependency evidence. Keep runtime and
   formal acceptance pending. Amend captured code only after the active run is
   terminal so its source-stability evidence remains meaningful.

Evidence: `target/parity-full-build-20261030-r8/proof-package-correction` retains
before/after builder source and the focused package/three stage logs; diagnostic
15 passed its final packaging section and 3,234-input source stability check.
`target/parity-full-build-20261030-r8/packaging-role-review/checker-source-readback.json`
retains the eight current source/receipt matches. R8's complete standard build
is still running; C has not started. Historical failed attempts stay unchanged.

## Follow-up evidence from the active validation window

An isolated candidate now addresses B1/C1/S1 in executable form: nineteen
refusal tests pass and a six-file integration patch is prepared but unapplied.
D1 has narrower new evidence: the complete native-proof browser test passes
against a frozen completed diagnostic package using fresh native/upstream
references. It is not the final standard/C pair, so D1 remains open at that
scope. B2's dynamic dependency manifest remains open. T1 is recorded as PP-74.
All three release amendments retain their original required scope; no P2 is
silently marked resolved. Detailed raw artifacts and limitations are retained in
`target/parity-full-build-20261030-r8/proof-hardening-candidate/evidence.json`.

The isolated v2 follow-up adds B2's explicit dynamic dependency inventory and
seven actual damaged/undeclared-package refusals. Its 221-file closure matches
186 assets served across three actual proof page tests. Solver/checker pages
now have four separately checked proof downloads and three fresh native checker
receipt comparisons. Native-tree is static-only here; final standard/C browser
execution and source integration remain pending. The nine-file v2 patch is
checked and unapplied, with full scope in `proof-hardening-candidate/evidence-v2.json`.
These results strengthen B1/B2/D1 evidence without changing their final-release
scope or marking the original P2 findings resolved.

The v3 evidence closes the diagnostic package's former static-only tree gap:
fresh native tree/plan/map/model references and the complete tree Chrome test
pass, including the bounded real RI proof-output refusal. All four diagnostic
page runs bind 194 served identities to the 221-file closure. D1's final
standard/C scope remains open; a separate process/receipt/source-guarded queue
will run these checks against each actual completed final package. Its current
state is queued, not successful. Integration and formal parity claims remain
separate.

The v4 integration evidence executes the actual candidate scripts in a disposable
installed-layout replica: valid stage bytes/receipt match, ten stale/changed
inputs refuse before package writes, and all three builders stop at the initial
shared preflight before outputs. The nineteen prerequisite refusal regression
also passes there. Ten real repository source/artifact hashes remain unchanged.
This verifies the proposed integration behavior, not its adoption into the
captured source tree. B1/C1/S1 final-release integration conditions remain open.

B3 now has isolated actual-browser evidence: the exact current replay-message
wait passes alongside sixteen native comparisons, independent module/browser
ZIP readback, native national mapping, stale queue edits and cancel/Resume.
The prepared workbench served 247 package-bound identities. The production-test
status patch remains unapplied and this sequence is not registered in the
complete builders. This resolves the test-expectation diagnosis at diagnostic
scope, not final standard/C or full-plan acceptance. See
`target/parity-full-build-20261030-r8/prepared-browser-candidate/evidence.json`.

## Post-adoption follow-up and fresh R9 validation

This follow-up applies the same five installed lenses with one agent. The
original finding severities and review counts above describe the initial review;
they are not new independent reviewer votes. The release verdict remains
APPROVED-WITH-CONDITIONS. No canonical parity contract is promoted.

R8 terminated failed after an unconditional laboratory balance assertion; its
source guard confirms all 3,234 inputs unchanged. Terminal logs, pre-adoption
sources, binaries and successful scoped national artifacts are preserved under
`target/parity-full-build-20261030-r8/terminal-and-adoption`. Only after all three
guarded runners terminated was the 18-file combined patch applied byte-exact.
The later laboratory test corrections change assertions, not production engine
behavior or budgets. Fresh R9 captures 3,243 current inputs and runs both complete
builders; packaging and four-page proof checks await its success receipt.

| Finding | Current evidence and boundary |
|---|---|
| B1 | Adopted prerequisite regression passes 19 stale/missing-input refusals; current package regression refuses seven damaged/undeclared packages. Full registered R9 execution remains pending. |
| B2 | Adopted explicit runtime manifest/verifier covers four proof entry points and 221 dependency files, including dynamic worker assets. Diagnostic browser closures are retained; final standard/C loaded-asset checks remain queued. |
| B3 | Corrected replay wait, three maintained input fixtures and explicit package/native/output paths are integrated. The current-source package gate passes batch, nine nested ZIP checks and the actual browser suite. Standard/C builder hooks are integrated; complete-builder execution remains pending. |
| C1 | Current checker staging enforces all eight source claims. Byte-exact adopted scripts pass valid staging and ten stale/changed-input refusals in a disposable installed-layout copy. This establishes recorded-source correspondence, not reproducible compilation. |
| C2 | Hash receipts still identify bytes rather than publisher authenticity or signatures. No stronger authenticity claim is introduced. |
| C3 | Standard and C packages retain separate proof dependency receipts and will receive separately frozen browser checks. Their actual completed-package evidence remains pending. |
| D1 | Four diagnostic proof pages have native/upstream and served-byte evidence. The new queue is tied to R9's exact PID/terminal success and freezes each final package separately; neither final profile has passed yet. |
| D2 | The plan continues to distinguish comparison counts, feasibility, dependency closure and full acceptance. The corrected laboratory gate reports 29 native-equal population misses rather than assuming every execution is accepted. |
| D3 | Small proof fixtures and the retained RI output-limit refusal retain their stated limits. No statewide complete-proof generation or global optimality claim follows. |
| S1 | Shared read-only preflight is integrated in all three builders. Current adopted entry scripts demonstrably refuse stale prerequisites before output directories in the disposable replica. R9 starts from passing actual prerequisites; whole-builder results remain pending. |
| S2 | README retains the separately acquired pinned solver/checker prerequisites. A clean-machine bootstrap run is still unproved; local retained toolchains do not establish it. |
| S3 | Lazy proof loading is retained. Final package/archive measurements depend on successful guarded packaging and are not inferred from the diagnostic file count. |
| T1 | PP-74 remains mitigated. Adopted structural guards have focused regression evidence; final runtime/profile coverage remains pending, so this does not mark the pitfall solved. |
| T2 | Failed R8 packages remain failed and preserved. Fresh output names, exact-process guards and successful terminal receipts prevent their acceptance; partial directory presence still conveys no success. |
| T3 | Standard/C builder integration now invokes the common dependency manifest/verifier and package mutation test. Additional or future copying entry points still require their own guard review; no universal packager coverage is inferred. |

BENCHMARK and DATUM also reviewed the two laboratory test repairs: the original
RI/IA/NC balance and memory/map/export/cancellation assertions remain; independent
population/components/metric checks and whole native/WASM replies replace the
incorrect universal single-attempt acceptance assumption. The CA/TX retry
assertion now applies to those named fixtures, while accepted balance is still
required for every selected native-build state. Both corrected all-state gates
pass: 102 whole native replies in the candidate diagnostic, then 50 balanced
native-build outputs with export/audit/certificate verification. These are scoped
functional checks, not complete CLI orchestration or final package acceptance.

The remaining-laboratory diagnostic separately passes nine gates, including
7,150 one-district assignment/project/map cases, 21,450 tamper refusals and
offline packaging, with 3,243 captured inputs unchanged. It uses retained
compiled engines. R9's fresh complete builders must independently pass.

Evidence: `terminal-and-adoption/prepared-source-check/evidence.json`,
`adopted-proof-entrypoints/source-binding.json`,
`lab-balance-investigation/all-50/evidence.json` and
`lab-balance-investigation/native-build-all-50-v2/evidence.json` under the R8
root; `target/parity-lab-tail-20261031-r1/status.json`; and fresh build/queue
receipts under `target/parity-full-build-20261031-r9`. Original receipts remain
unchanged. The three amendments above retain their full release scope.
