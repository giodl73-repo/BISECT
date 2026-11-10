---
skill: roles-check
topic: wasm-command-comparison
date: 2026-10-31
roles_used: [benchmark, covenant, datum, survey, trench]
p1_count: 0
p2_count: 11
p3_count: 10
verdict: APPROVED-WITH-CONDITIONS
---

# Command comparison candidate review

This is one agent applying five installed role lenses, not five independent
reviewers. Artifact: the unapplied command-comparison code and validation
candidate under `target/parity-full-build-20261031-r9/`. The review permits
continued local integration after the captured build and dependent queues are
terminal. It does not approve release, canonical parity closure or deployment.

BENCHMARK reviews test sensitivity and builder registration; COVENANT reviews
raw evidence and source identities; DATUM checks claim/evidence scope; SURVEY
checks practitioner/browser coverage; TRENCH checks structural recurrence and
adoption failure modes. Their installed `lens.verify` definitions were read.

The v6 helper passes one complete isolated 29-caller default-argument run:
1,503 independently read raw pairs, one native image, one WASM image, three
actual Chromium journeys and 185 corruption controls. All 3,243 captured R9
source/fixture/catalog inputs remain unchanged at that checkpoint. Candidate
v7 preserves that helper byte-for-byte and adds an approximately 80 KB compressed
fixture, a pipeline-independent corruption gate and three builder registrations.

| ID | Finding | Severity | Evidence / section | Recommendation and current disposition |
|---|---|---|---|---|
| B1 | Corruption controls initially existed only in ignored run artifacts; ordinary positive equality tests would not prevent recursive erasure returning. | P2 | v6 regression versus v7 `test_native_command_comparison.mjs` | Candidate-fixed: retain twelve real reply fixtures and 127 controls, register all three builders; maintained adoption still pending. |
| B2 | The amended builders have not executed as full maintained-source builds. | P2 | v7 guard verification; live R9 uses captured earlier sources | Run required maintained/default/C checks after adoption. Source registration and PowerShell parsing are not full builder execution. |
| B3 | The fixture gate is pipeline-independent, while the 29 integration callers still require their native/source prerequisites. | P3 | v7 clean-CWD baseline; final 29-caller run | Preserve both tiers and prerequisite documentation; do not describe the integration campaign as clean-checkout acceptance. |
| B4 | The existing closure gate accepts a synthetic verified bundle with a pending, missing or unversioned comparison when the bundle repeats that definition. | P2 | `unfinished-comparison-probe/`; separate closure candidate | Candidate-fixed: reject invalid closure definitions and preserve unresolved ordinary rows. Four retained counterexamples now refuse; maintained adoption is pending. |
| B5 | A generic all-green check list can omit a comparison's mandatory workflow stage unless required check IDs are enforced. | P2 | proposed twelve contracts; policy candidate v2 | Candidate-fixed: each declared stage must appear exactly once. All 177 per-contract omission controls refuse; actual stage evidence remains pending. |
| C1 | Changing 29 tests and three builders leaves canonical source references needing reconciliation, including the new helper/fixture/test. | P2 | v7 manifest versus captured inventories | Candidate-prepared: 336 current references updated across 180 contracts, with helper/guard dependencies added. Historical receipts and every scope/state remain unchanged; real adoption remains open. |
| C2 | Content hashes identify retained bytes; they do not authenticate a publisher, upstream Census provenance or a historical execution. | P3 | v6 final readiness and raw receipts | Keep the explicit scope. No signed-release, source-authentication or certification claim follows. |
| C3 | The native evidence is an observed AMD64 Windows image; it is not a clean-machine build attestation. | P3 | measured PE32+ width and executable SHA; Node/Chromium identities | Retain exact platform/toolchain facts and fail closed on an unrecognized executable target. Do not extrapolate to other native hosts. |
| C4 | A versioned non-placeholder comparison is a structural prerequisite, not proof its semantics or required oracle were exercised. | P3 | new closure definition guard; original twelve command contracts | Finalize comparison semantics and stage-specific native authority separately. A valid dictionary must not substitute for raw workflow evidence. |
| C5 | The globally allowed independent-kernel reference cannot establish CLI scheduling, filesystem lifecycle or native ignored-flag behavior. | P2 | per-contract `reference_kinds`; native command branches | Candidate-fixed policy: prepared execution admits a kernel reference; the other nine command contracts require CLI authority. Nine insufficient-authority synthetic controls refuse. |
| D1 | Native/WASM equality does not establish population feasibility or algorithm optimality. | P2 | unchanged R9 miss/failure evidence; comparison scope | Preserve feasibility failures and original metric flags in all reporting. Full-plan acceptance remains separate. |
| D2 | The source audit identifies 29 pointer-width callers, not every possible normalization pattern in the project. | P3 | `acceptance-readiness/comparator-callers.json` | Keep the quantified claim tied to that search and those source hashes. |
| D3 | The v7 helper matches the v6 tested helper, but three newly amended builders are additional unexecuted code paths. | P3 | helper SHA equality; 35-file candidate manifest | Carry forward helper evidence only; validate new builder integration separately. |
| S1 | Default-argument candidate coverage and three Chromium journeys do not close C-profile, required argument variants or desktop orchestration. | P2 | final run scope; original twelve canonical comparison contracts | Complete original required profile/workflow scopes. Do not shrink the plan around this campaign. |
| S2 | Automated browser journeys exercise selected UI paths, not all browser/OS capabilities. | P3 | three retained Chromium 155.0.8059.85 journeys | Preserve the tested scope; do not reopen the user-abandoned manual folder/private-release exercise. |
| S3 | The new compressed test fixture is small, but raw differential evidence is intentionally retained in larger local directories. | P3 | 79,661-byte fixture; unique comparison directories | Keep the fixture outside shipped web assets and preserve historical/failed artifacts. No cleanup is authorized here. |
| T1 | Recurrence prevention is not installed until the shared helper and durable gate are actually adopted. | P2 | PP-39 follow-up; six negative mutants | Leave the failure class open. Candidate tests are mitigation, not a SOLVED maintained implementation. |
| T2 | Same-invalid-schema pairs can evade a test that changes only one side, because ordinary deep equality already rejects one-sided changes. | P3 | paired-schema controls; removed-schema mutants | Candidate-fixed: test invalid command and envelope schemas on both sides; both weakened-schema mutants fail. |
| T3 | A text-only adoption omits the binary fixture; Git line-ending conversion can also change the expected helper bytes. | P2 | disposable adoption staging | Candidate-fixed recipe: validate all payloads before staging, apply text with `core.autocrlf=false`, copy the pinned absent-destination fixture, verify all 35 files. Real adoption remains pending. |
| T4 | Placeholder closure can recur unless its regression lives in the maintained closure-test entry point. | P2 | two new tests in candidate `test_wasm_parity_ledger.py` | Candidate-fixed: 22 invalid forms tested in ordinary/full verified modes; three unresolved forms retain ordinary acceptance and full refusal. Keep PP-25 open until adoption and broader closure work. |
| T5 | Structural controls using synthetic source/build files must never be promoted as native execution evidence for real contracts. | P3 | per-contract omission/reference controls and their explicit synthetic scope | Keep synthetic fixtures separate, preserve actual scopes/backend gaps/states, and require real stage evidence before verified claims. |

Roles reviewed: 5. P1 blockers: 0; P2 conditions: 11; P3 notes: 10.

Verdict: **APPROVED-WITH-CONDITIONS** for continued local integration only.
Top finding: corruption sensitivity must survive outside temporary evidence
directories. BENCHMARK and TRENCH agree on the durable gate; COVENANT and
TRENCH agree on exact source/binary payload binding; DATUM and SURVEY agree
that this campaign cannot replace the original parity scopes.

Three amendments:

1. Candidate v7 adds a Node-only guard using twelve retained native/WASM pairs,
   127 corruption checks and three early builder commands with failure guards.
   A clean-CWD baseline passes; five broken helper variants and an altered
   fixture fail. Builder commands are source-checked and PowerShell-parsed.
2. Disposable staging validates every payload before applying the text patch
   and pinned binary fixture. Missing/altered binary payloads fail before
   staging. An initial automatic line-ending conversion failure is retained;
   the corrected recipe yields all 35 expected file hashes and a passing guard,
   with all 32 original source files unchanged in the real worktree.
3. Keep source-pin reconciliation, real maintained-source adoption, complete
   amended builder/profile runs and original contract closure open. None of
   these conditions requires asking the user to repeat prior authorization.

Source-pin preparation follow-up: the initial recursive scan found 391 matching
references, including historical reconciliation receipts. A guard refused to
rewrite those receipts. The corrected preparation enumerates only current
declaration/consumer/browser/test reference fields: 336 updates and two new
helper/guard dependencies on 180 affected contracts. It reverses just those
allowed edits and compares the entire inventory structures to prove that all
other data, including histories, scopes, statuses and bundles, remains unchanged.
The structured validator functions pass all 309 parents on a materialized
source overlay; full parity still refuses the unverified Run parent. The fixture
guard also passes in that overlay. This does not execute the scanner CLI or
amended builders and does not adopt either inventory.

CLI and combined-staging follow-up: the actual scanner/ledger CLI now passes
all 309 records in the materialized source tree. Its full-parity command
refuses specifically at the unverified Run parent after the source scanners
pass; the standalone corruption guard passes 127 cases. A second disposable
repository then applies both the code text patch and inventory patch with
`core.autocrlf=false`, installs the pinned binary fixture, and verifies all 37
payload files byte-for-byte. The same actual CLI/default/full-refusal/guard
checks pass there. All 34 real original source/inventory files remain unchanged
and the three new real destinations remain absent. This closes preparation and
CLI-validation uncertainty for C1's candidate; real adoption and full amended
builder/profile execution remain conditions. Evidence is under v7
`source-pin-reconciliation/cli-evidence.json` and
`combined-adoption-20261101T030526620473Z/`.

Closure-definition follow-up: reviewing the twelve unfinished command contracts
finds a separate acceptance-gate defect. The current gate accepts all four
retained synthetic counterexamples when contract and bundle agree on a pending,
missing or unversioned comparison. The isolated two-file
`closure-comparison-candidate/` requires a positive integer version, non-placeholder
nonempty mode and explicit list allowlist before any execution contract can
close. Explicit partial rows remain valid ordinary metadata. All ten unit tests
pass, including 22 invalid forms in both verified modes and three preserved
partial forms; all four retained counterexamples refuse under the new gate.
The actual ordinary scanner/ledger CLI still passes 309 records on the combined
source overlay, and full parity refuses unverified Run. This is a separate
unapplied candidate, not part of the already-staged 37-file manifest. It does
not finalize the twelve semantic contracts or promote any real row. Evidence is
`closure-comparison-candidate/verification-20261101T031813441529Z/`.

Semantic-definition follow-up: twelve comparison proposals now preserve the
original scope, backend descriptor, implementation/evidence state and history
while specifying native authority, exact width exceptions, raw preservation,
matrix/outcome policy and 177 required stage summaries. Prepared execution
admits native CLI or independent-kernel authority; queue/workspace/unused-field
contracts require native CLI. State's per-location/chamber/custom rules remain
distinct from Run all-vintage and States bulk policies. Provisional backend
descriptions remain unresolved and cannot be treated as executable identities.

`closure-policy-candidate-v2/` extends the two-file gate correction to enforce
declared reference kinds and exactly-once required stage IDs. All twelve unit
tests pass. Per-contract synthetic controls reject 177 omitted stages and nine
insufficient references; three prepared-kernel controls remain structurally
valid. These are deliberately synthetic controls, not native executions or real
acceptance bundles. An isolated 39-file composition applies the policy gate and
twelve definitions on top of the prior code/fixture/source-pin candidate: exact
bytes, ordinary inventory CLI (309 records), expected full refusal, twelve unit
tests and the 127-case comparator guard pass. All other inventory data is
unchanged. Evidence is under `command-comparison-contract-candidate/`, including
`policy-verification-20261101T033715771898Z/` and
`overlay-verification-20261101T034000288506Z/`. Maintained adoption, actual stage
evidence, backend/profile reconciliation and full-plan acceptance remain open.

Evidence: `comparison-integration-candidate-v6/final-readiness.json`,
`comparison-integration-candidate-v7/manifest.json`,
`comparison-integration-candidate-v7/guard-verification-20261101T025031075198Z/evidence.json`,
and `comparison-integration-candidate-v7/adoption-stage-20261101T025227747189Z/evidence.json`,
all under `target/parity-full-build-20261031-r9/`. The live R9 run and its
captured sources are preserved; this review attaches no acceptance-v1 bundle.


Evidence addendum — 2026-11-01 UTC: the actual CLI unused-field matrix and three isolated headless browser journeys now pass within the one-fixture scope recorded in the plan follow-up. Raw output order differences and the max-u64/signed-manifest seed representation remain visible. Independent audits retain 25 CLI invocations, 95 output files, 22 unused-field pairs, three controller boundaries, three UI journeys and 429 unchanged package hashes. This addendum records evidence against existing review conditions; it is not a new independent role review, changes no finding counts/verdict, and closes no full-profile or acceptance-bundle condition. See `target/parity-full-build-20261031-r9/native-unused-matrix-20261101T040323555984Z/browser-policy/independent-audit.json`.


Evidence addendum — same-input executions and lifecycle witnesses: 22 WASM jobs and their saved project/ZIP artifacts now independently match actual CLI assignments on the four-unit fixture; three same-input browser baselines retain recipes and exact seeds. Full context identities differ and are expressly not accepted as equal. Eighteen additional actual native CLI lifecycle scenarios and 29 WASM snapshot-driven operations agree on scheduling/refusal decisions. Five refusals and ten unchanged no-execution snapshots are independently checked (215 files). Raw native nanosecond metadata is retained byte-for-byte after the first decision mirror failed precision audit. See the plan's 2026-11-01 same-input/lifecycle follow-up for immutable paths and limits. This is evidence against existing conditions, not a new independent role review, a finding-count change, a whole-context exception, or full-profile/bundle closure.
