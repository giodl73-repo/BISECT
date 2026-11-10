---
skill: roles-check
topic: wasm-desktop-parity-plan-handoff
date: 2026-10-23
depth: standard
roles_used: [meridian, benchmark, covenant, ledger, vault, trench, survey]
p1_count: 0
p2_count: 4
p2_open_implementation_count: 4
p3_count: 17
verdict: APPROVED-WITH-CONDITIONS
---

# Desktop-to-WASM plan: implementation handoff review

Artifact: [parity plan](../../../docs/specs/wasm-desktop-parity-plan.md).
Type: implementation plan, platform/workspace/evidence acceptance contract.
One agent applied the seven installed lenses below; these are not independent
reviewers or human approvals. This review assesses planning and inspected source
and existing records. It does not rerun runtime tests, accept pending code,
authorize deployment or establish full desktop parity.

## Role selection

The [catalog](../../../.roles/ROLE.md) and all seven selected profiles were read.

| Lens | Reason |
|---|---|
| [MERIDIAN](../../../.roles/meridian.md) | Actual method/backend usage, unchanged algorithm semantics. |
| [BENCHMARK](../../../.roles/benchmark.md) | Native ground truth, failure coverage and final-package gates. |
| [COVENANT](../../../.roles/covenant.md) | Source/binary custody and reproducible evidence. |
| [LEDGER](../../../.roles/ledger.md) | Project/receipt versioning and artifact interchange. |
| [VAULT](../../../.roles/vault.md) | Live authority, nonce lifetime and imported-record trust boundaries. |
| [TRENCH](../../../.roles/trench.md) | Stale closure claims and irreversible failure ordering. |
| [SURVEY](../../../.roles/survey.md) | Bounded next workflow and usable fallbacks. |

No new legal, statistical, certification or election-data claim is proposed.
Other catalog lenses remain applicable when those scopes change. Role prose is
not evidence of legal admissibility or current law.

## Findings

P2 means required before accepting the affected implementation. The amendments
make all four obligations explicit; none is closed by editing this plan. P3
items remain scoped implementation/release obligations. There are three findings
per lens, 21 total.

| ID | Lens | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|---|
| M1 | MERIDIAN | Pending target preflight rejects declared C/gpmetis backends without accounting for actual method use or one-district requests. The native State compiler explicitly distinguishes these cases. | P2 | WP-02/04; next checkpoint | Added requirement: use native method/count semantics, test unused declarations and genuinely required unavailable backends before target effects. Implementation remains open. |
| M2 | MERIDIAN | Seeded C matches under the named CRT profile do not establish all native runtime/build combinations. | P3 | WP-02 | Keep explicit profile and native build-feature limitations; no silent substitution. |
| M3 | MERIDIAN | A four-unit State result does not establish real-state balance, graph construction or national capacity. | P3 | WP-08 | Predeclare real-data/method rows and retain failed candidate outcomes separately. |
| B1 | BENCHMARK | New before-Run tests exist, but the inspected standard and workspace preview builders do not invoke them. The browser receipt names web/lab rather than a staged package. | P2 | Release gates; next checkpoint | Added final-package integration and worker gate requirements. Candidate implementation acceptance remains open. |
| B2 | BENCHMARK | The standalone reset matrix intentionally fails native graph loading; it cannot substitute for successful composed Run evidence. | P3 | WP-03/04 | Preserve its ordering claim and the separate successful CLI witness; never pool them as full workflow success. |
| B3 | BENCHMARK | OPFS and an injected directory picker exercise genuine handles but do not test OS picker UI or host permission revocation. | P3 | Compatibility | Name these limits and test advertised host paths separately. |
| C1 | COVENANT | New local target-action evidence lacks the final stable package/source/native-binary handoff required by the plan. Mutable target records alone are insufficient. | P2 | Evidence rules; next checkpoint | Added owned reference executable, raw outcomes and stable identity snapshot requirement. Implementation remains open. |
| C2 | COVENANT | An action receipt records removal, not successful partition, installed outputs or source authenticity. | P3 | WP-06 | Preserve distinct action/computation/install outcomes in project, UI and replay. |
| C3 | COVENANT | Optional C integration gates do not close dependency notices or obtainability/reproducibility of a release. | P3 | WP-10 | Keep notice and formal acceptance work open. |
| L1 | LEDGER | Pending workflow bindings add target_actions and derived argument records; old project semantics must survive. | P3 | WP-03/07 | Retain old-read and round-trip fixtures; original and derived commands remain separately named. |
| L2 | LEDGER | Manual installation, ZIP download and native automatic post-Run writing are different lifecycle claims. | P3 | WP-03/04 | Next checkpoint now explicitly excludes automatic output installation; keep exact output contracts. |
| L3 | LEDGER | Native artifact agreement does not establish external GIS schema compatibility. | P3 | WP-07 | Bind advertised external integrations to named schema versions and fixtures. |
| V1 | VAULT | Serialized receipts and flags must never recreate the live review/confirmation capability. | P3 | Workspace safety | Preserve private grants, current worker/request/nonce binding, one-use reviews and inert Open. |
| V2 | VAULT | A private JavaScript grant protects against imported data; it is not proof against privileged same-origin code. | P3 | Workspace safety | State this trust boundary; do not present receipt hashes as malware resistance. |
| V3 | VAULT | Source, target or operation changes during asynchronous review can retire authority while filesystem work is already in progress. | P3 | WP-03 | Preserve stale checks and explicit interrupted effects; do not promise atomicity or rollback. |
| T1 | TRENCH | The plan still described optional C gate wiring as candidate-only although the preview builder now invokes it. | P3 | Release gates | Amended current wording while preserving per-package rerun and standard-entry-point limits. Historical findings are not bulk-closed. |
| T2 | TRENCH | Backend/runtime preflight can conflate a declaration with actual consumption. | P3 | WP-04 | Apply existing declaration/effect pitfall controls (PP-25/28); M1 must gain a regression before closure. No duplicate pitfall is introduced. |
| T3 | TRENCH | Cancellation or failure after data removal cannot mean no action occurred. | P3 | Workspace safety; next checkpoint | Added explicit retained-outcome acceptance for post-removal failures/cancel. Keep PP-27/28 scoped and incomplete broader work visible. |
| S1 | SURVEY | The next checkpoint repeats delivered C/workspace slices instead of ranking the remaining user-visible work. | P2 | Next checkpoint | Replaced it with target-action completion, package/evidence handoff and ledger-led workflow ranking. Those deliverables remain open. |
| S2 | SURVEY | Unlabeled custom targets, bulk scheduling and acquisition recovery are broader than the pending single-State action. | P3 | WP-03/05 | Explicitly exclude these until separate contracts pass; retain portable output usability. |
| S3 | SURVEY | Parity work can drift into profiling while practitioners still lack composed workflows. | P3 | Objective / WP-08 | Keep functional parity first; use measured limits when they block an advertised operation. |

## Synthesis

Roles reviewed: **7**. **0 P1, 4 P2, 17 P3**.
Verdict: **APPROVED-WITH-CONDITIONS for the amended plan**. The four P2
implementation obligations remain open. This is not approval of the new target
action for release, full parity or certification.

Top finding: make the next milestone reflect the actual dirty implementation,
then test its final packaged bytes. MERIDIAN/BENCHMARK/TRENCH agree that declared
backend availability must follow native consumption rules. BENCHMARK/COVENANT
agree that source-tree results cannot close final-package acceptance.

Three amendments applied:

1. Updated current C gate status and separated delivered scoped records from
   pending target-action code and historical evidence.
2. Required method/count-aware preflight and retained post-removal outcomes
   before accepting the selected-State target action.
3. Replaced stale next work with three ordered deliverables: finish that action,
   package and bind its evidence, then reconcile/rank remaining workflow children.

## Inspected evidence and limits

Baseline HEAD: `383db340235e21d752a35610aab28e2948b73e57`; dirty working tree.
The review inspected the plan, prior current-plan review, parity ledger,
standalone-reset specification, optional-C/workspace builders, standard builder
test registrations, pending target-run module and native State compiler.

`crates/bisect-wasm/src/native_state_run.rs` tests `districts > 1 && uses_metis`
before requiring an available METIS backend. The pending JS target preflight
checks the backend declaration without this condition. This supports M1's
behavioral discrepancy; no newly executed failure witness is claimed.

Existing `target/native-state-target-run/evidence.json` reports a successful CLI
Run, exact assignments, original arguments, source-bound authority and force-only
preservation. Existing browser evidence names `web/lab`, worker/OPFS checks and
Chrome 155.0.8059.85. These records were inspected, not rerun or independently
accepted. They do not establish final-package execution or reproducibility.

The amended plan and inspected critical sources are pinned in the companion
[review baseline](wasm-desktop-parity-plan-handoff-baseline-2026-10-23.json).
These hashes identify inspected bytes, not build provenance or authenticity.
Only the plan and this review's report/baseline were changed. Documentation
links, finding counts and whitespace were checked; no engine build, runtime
suite, commit, publication or deployment was performed by this review.
