# Structured WASM parity evidence

WP-01 is in progress. The existing capability and engine-option inventories now
contain a `parity_record` for all 69 command/screen and 240 field entries. This
initialization does **not** reconcile or verify their historical claims. Pending
rows say `unreviewed`/`unreconciled`; they do not mean an existing feature is absent.
No executable support percentage follows from these counts.

The current WP-07 Civic List/Show file-selection checks compare the actual
desktop command with Core and C browser workers. Desktop List scans immediate
child directories, skips malformed manifests and emits rows in filesystem
order. Browser List uses the selected file order and displays parse errors.
The check compares the header and the multiset of List rows, and compares
typed Show JSON byte for byte apart from the terminal CLI newline. Desktop
Show returned identical bytes after the validation log and normalized CSV
were removed from the test package; the command's help mentions the log, but
this path does not read it. The browser likewise inspects manifest records
without authenticating or rereading their underlying package artifacts.

Eleven desktop invocations, both Chrome profiles, and an independent archive
readback cover valid, malformed and overflowing manifests, an exact maximum
u64 counter, preserved input bytes, inert project Open and explicit replay.
See `target/civic-list-show-current-20261105-r1/independent-readback.json`.
The collection-folder check then exercised the actual Chrome directory input
in both profiles. The existing recursive picker retained a nested manifest
that desktop List skipped. An isolated UI candidate adds a separate desktop
`civic_inputs` picker for immediate package folders. Both profiles matched
desktop List rows and Show files, and the file-read trace confirmed that the
new picker skipped the nested manifest and unrelated files. In both folder
modes, saved source bytes, inert Open and explicit rerun were verified.
The candidate patch and independent ZIP readback are under
`target/civic-list-show-current-20261105-r1/native-root-candidate/`.
The separate `combined-r25-overlay-r1/` candidate preserves the pending Compare
changes and adds a self-contained Civic browser gate to both builders. The
gate passed in Chrome for both assembled profiles using the retained engines.
The draft reconciles 333 current source references across four catalogs and
preserves all acceptance records. Three accepted Config scopes bind changed
source bytes and require fresh evidence before this overlay can be adopted.
With the retained native and WASM binaries, the composed UI passed the original
26 Config New/Validate cases, 14 publication journeys on real OPFS, and 34
Windows directory facade cases. Independent checks read back 50 native YAML
observations and 30 OPFS observations, including collision, force, stale target
and interrupted writes. The same checks passed with the exact new Core/C
binaries, along with 64 State reset filesystem comparisons. The four existing
scopes were rebound to a new evidence archive and verified from an empty
restore root populated with pinned authority files. The Civic folder and
Compare changes are integrated in the local worktree. Broader host/capacity
checks, clean-checkout release verification and parent acceptance remain open.

WP-07 Civic Conflicts has a separate native command gap. Eleven current CLI
invocations confirm that Conflicts parses saved normalized CSVs, includes
retained warning rows, respects repeated labels and label order, and refuses a
single label. It does not read the original CSV, manifest or validation log.
The existing browser assessment's Advisory setting matches the two-file
output, but its default Strict setting excludes the warning rows; its input
contract also rejects repeats and allows a single file. A staged separate
adapter preserves desktop parsing and label order without reapplying ingestion
validation. It matched three desktop outputs and eight refusal controls in a
compiled Windows harness and a standalone Chrome WASM Worker. The staged
workbench now selects normalized files under explicit ordered labels, keeps
exact selected bytes in a saved project, and exposes a fresh desktop JSON
download. Project codec checks cover repeated labels, oversized file refusal
before reading, and rejection of undeclared path fields. Both locally linked
full toolkits passed Chrome Run/Save/Open: eight exact JSON exports total,
inert Open, stale file retirement and explicit replay. A source-owned gate
now regenerates the desktop fixtures and is registered in both candidate
builders. Its independent readback covered ten exact exports across the two
profiles, including explicit rerun after opening a forged saved result. The
draft source catalogs passed their maintained authority checks. The original
Config and State reset gates also passed on both linked Conflicts toolkits.
Both Cargo profiles passed the source-owned Conflicts browser gate and the
combined national archive check. The exact composed builds also passed the
Config, State reset and California/Texas Compare regressions. The worktree
now includes Civic Conflicts and the gzip preference for large saves. The
selected-normalized-CSV child is verified against the same native Core CLI
for both browser profiles. Native package discovery, broader host and capacity
coverage, clean-checkout release and parent acceptance remain open.
Evidence:
`target/civic-conflicts-native-contract-20261105-r1/candidate/conflicts-and-archive-overlay-r1/`.

WP-07 Candidate Race now has a verified selected-file child. A blank package
label resolves to the desktop state/year default; explicit labels are retained.
Both browser profiles matched the native CLI's four non-manifest ZIP members
byte for byte, and the manifest matched after normalizing its two clock fields.
Open restored the original CSV and attestation bytes, refused to export a
forged saved result, and left rerun explicit. The separate reviewed workspace
child now covers explicit selection, confirmation, create and replacement under
the native layout in real browser OPFS. Both profiles match the actual desktop
files with the same two clock-field allowances, preserve unrelated files, and
record the verified prefix if a write stops. Known directory conflicts are
refused before writing; the desktop can leave a partial package in that case.
Live OS picker, external writers, broader capacity and release remain open.
Evidence: `target/candidate-race-workspace-maintained-r1/independent-readback.json`.

Run/State/States now have source-pinned direct-read traces for their 152 fields,
covering each actual CLI entry branch and delegated configuration constructor.
The source check excludes inspector formatting, comments, strings and tests.
It records read sites, including wrapped member access and digits in field names;
it does not infer downstream behavior merely from a read. Changed source bytes
or a changed reviewed unused set invalidate the trace.

The 22 no-read findings are:

| Command | Parsed fields with no direct read in this reviewed boundary |
|---|---|
| Run | `dpi`, `election_year`, `experiment_name`, `minimum_boundary_length`, `print_only`, `processing_mode`, `run_type`, `skip_analysis`, `skip_demographic`, `skip_political`, `stages` |
| State | `dpi`, `objtype`, `print_only`, `target_mm_districts` |
| States | `dpi`, `election_year`, `print_only`, `processing_mode`, `run_analysis`, `skip_demographic`, `skip_political` |

These are native limitations, not 22 absent browser algorithms. Their structured
records now distinguish this source finding from the still-unverified runtime
comparison. In particular, the native branches do not honor `print_only`;
browser no-execution intent is a separate adaptation. Native State `objtype`
is not evidence that it controls the engine's objective. Browser graph options
and explicit map/follow-up DPI belong to their own contracts.

The three command parents now have separate prepared-execution, selected-source
queue, workspace-lifecycle and unused-field/browser-intent child contracts.
They name current browser modules and candidate test sources, while leaving
execution evidence partial. Source-only reconciliation does not rerun those
historical tests or make their old output bundles current.

WP-03's first executable slice adds [reviewed single-output installation
recovery](wasm-native-output-recovery.md). It validates the exact interruption
journal/package, binds live folder authority, rejects changed/unrelated files
and resumes without another partition. Native/reference and browser tests pass;
formal release-bundle migration and broader workspace/queue recovery remain
open. The State parent has a separate scoped recovery child contract.

[Selected input workspaces](wasm-selected-input-workspace.md) now compose folder
upload, live-directory and bounded ZIP selection with native discovery, the
existing source queue, Run and reviewed output installation/download. The
source-generated four-unit RI case has module/native and actual browser evidence,
including permission/source changes before and during queueing, inert Open,
late picker retirement and installation recovery without repartitioning. Raw
ZIP bytes are retained as an optional inert project source. This adds a scoped
State child contract; the broad workspace parent and formal release acceptance
remain partial.

[Explicit native data reset](wasm-native-state-data-reset.md) now reproduces the
single-target native data-only removal and incomplete/labeled-collision ordering
through a separate reviewed GUI action. Thirty-two actual CLI filesystem cases,
25 refusals and actual packaged OPFS/browser checks cover that scope. Native
force is kept distinct from the older whole-target installation overwrite.
Engine-only reset/force remain refused. A later
[single selected-State before-Run connection](wasm-native-state-target-run.md)
now pauses for live target review/confirmation, preserves original flags and
exports action receipts. It has successful actual native CLI comparisons,
method-aware backend preflight and packaged worker/OPFS cancellation evidence.
Bulk/reprocess, cross-session failed-action custody and full lifecycle acceptance
remain explicit required gaps. An immutable local source/raw-outcome snapshot is retained
instead of promoting the scoped test into a full flag or parent completion.

[Reviewed native bulk workspaces](wasm-native-bulk-workspace.md) now compose
live canonical/historical completion observations, native Run/States selection,
scheduled target preflight and data-only Run reset before source execution.
Reprocess separately makes completed targets eligible. Six actual CLI cases
cover baseline, reprocess, incomplete-target continuation, historical completion,
States without reset and all three vintages on synthetic one-district fixtures.
Packaged worker/OPFS tests cover explicit review, stale targets, inert Open and
interrupted-action export after real removal. This adds scoped parent/flag
children with partial evidence. A later scoped saved-workspace continuation is
linked below; automatic installation and broader real-data/host/custody acceptance remain open. Existing
compute-only replay/Resume remains available.

[Paired source-acquisition recovery](wasm-state-acquisition-recovery.md) now
retains completed pairs across failure/cancel, resumes only unfinished pairs and
requires explicit saved-source consistency checks before imported continuation.
Raw archives are hashed/decoded and queued jobs rebuilt through the acquisition
constructor; saved markers alone grant no authority. Native-State and configured
YAML/form paths have packaged browser evidence, and a subsequent explicit source
Run matches six native adapter replies. This is a scoped browser adaptation,
not native CLI Fetch filesystem/release parity or full Verify. Vintages/profiles,
live providers, complete source-type reconciliation and full custody remain open.

[Batch output recovery](wasm-native-batch-output-recovery.md) now composes
live verification of completed targets, matching interrupted targets and missing
targets across a synthetic three-year fixture. Completed files are retained
without writes; copied reviews/old completion receipts confer no authority.
Nine module scenarios and real-browser recovery pass. This closes the scoped
batch installation recovery path, not unified workspace or queue recovery.

[Explicit custom county analysis](wasm-explicit-county-splits.md) now shares one
kernel across native CLI and WASM, validates matching contexts, retains the
effective county-map hash and preserves legacy Census report bytes. Custom State
follow-ups compute splits; missing optional election evidence still leaves a
partial workflow. Native CLI, browser and artifact tests cover this scope.
This addresses the identified shared analyzer limitation without closing all
Analyze commands, workspace or full-parity acceptance.

The State `metis_engine` field has its first source-traced child contracts:
explicit `metis-core`, missing C-FFI, and the shared `gpmetis` stub. Historical
metis-core tests have not yet been migrated to the new acceptance format. C-FFI
production integration and full native comparisons remain open. The
[historical direct Rust-target attempt](wasm-cffi-feasibility.md) failed because
`clang` was unavailable. A later [Emscripten sidecar experiment](wasm-cffi-sidecar-feasibility.md)
compiles the same vendored C library and runs in Node and an actual browser
worker. Its unmodified runtime differs from Windows-native seeded outcomes;
an explicitly named Windows CRT RNG profile matches the scoped 24-case fixture,
320 measured RNG values and error/recovery probes. That original experiment is feasibility evidence. The subsequent
[optional Rust/C engine integration](wasm-cffi-engine-integration.md) now has 52
native State-compositor assignment matches, four matching native failures and
an actual packaged workbench prepared run. The C declaration requires the named
runtime and explicit loading/Run; ordinary requests keep the Rust kernel.
Broader method/workflow coverage, normal release gates and acceptance-v1 bundles
remain incomplete. Historical unavailable-C descriptions above are retained as
history; the current C child contract is partial, not wholly missing.

The optional C preview now runs the affected seven legacy configuration/State
suites on both actual build variants before staging. A fresh local package,
`dist/wasm-cffi-gated`, also passes the native matrix using packaged modules and
WASM, offline/asset checks and the real browser worker. Current source and
result identities are retained in the C field's `build_gate_followup` record;
the earlier `integration_attempt` remains historical. Capability-dependent
presentation differences are named and retain both replies; unloaded-runtime
refusals are distinct from native algorithm failures. This closes the scoped
legacy-gate condition, not clean-checkout fixture portability, remaining native
workflows, real-data capacity or formal acceptance-v1/full-parity claims.

Run the source/inventory/identity checks with:

```powershell
python scripts/web/wasm_parity_ledger.py
python scripts/web/test_wasm_parity_ledger.py
python scripts/web/test_wasm_native_consumption.py
```

Both WASM build scripts run these gates before building. The default ledger
check permits explicit unresolved rows, so it does not prevent a scoped browser
preview. It is a structural/evidence check, **not preview release approval**.
An attempted full-parity claim uses:

```powershell
python scripts/web/wasm_parity_ledger.py --full-parity
```

That command currently fails, as required. It checks native declaration drift
before validating claims, so removing unsupported parents cannot close parity.
To see all the remaining closure conditions, run:

```powershell
python scripts/web/wasm_parity_ledger.py --audit-full-parity
```

The audit prints JSON to standard output and sends the source checks to standard
error. It validates the existing evidence before listing each unresolved parent
and required child, with its stated remaining work and candidate test sources.
An audit can finish successfully while `ledger_conditions_satisfied` is false.
Use `--full-parity` for the strict pass/fail gate and the separate release checks
for the actual packages.

Each record separates implementation state from evidence state, pins the
declaration bytes, and records assignee, review lenses, priority, dependencies,
native consumer, child contracts and an open issue. Historic `operation`, `ui`,
`verification`, `review`, `evidence` and `limitation` fields are preserved as
research history; reconcile their scope into child contracts before treating
them as current support authority. Never infer success from those free-text
fields. Inventory generators preserve reviewed records; changed declarations
require a new review rather than silently updating the old identity.

A verified execution child requires a hash-bound `bisect-parity-acceptance-v1`
bundle matching its exact ID, scope, comparison, claim kind and backend. The
checker requires native CLI or independent-kernel reference evidence, the
browser action/source, fixture inputs, raw outputs, comparison results, test
sources, build identities and passing required checks. Expected native failure
is distinct from successful support. Source, executable, WASM, fixture and
evidence bytes must still match their hashes. Generated `target/` outputs alone
cannot constitute a reproducible fixture. The portable mutation tests use
synthetic files to exercise rejection logic; they are not native/WASM execution
evidence and are not inserted into the real inventories.

Evidence manifests are assertions checked against retained bytes, not signatures
or an independent test runner. Reviewing test adequacy, enumerating all required
scopes, recording complete build/fixture identities and running real acceptance
tests remain necessary. No current row is promoted to verified by this change.
WP-08 still needs its predeclared real-data matrix and WP-10 must bind those
results to the actual release package. PP-25 remains open until that integration
is complete.

The next three slices are ranked below. The assignee is Codex implementation;
the BISECT maintainer accepts each milestone. These are source-backed work
priorities, not a claim that all rows have been reconciled. The live completion
and target-action portion of the first ranked bulk slice is now delivered above.
Paired acquisition recovery, composed Verify and saved-workspace continuation
now have scoped delivery too. Fetch/FLETCH reconciliation and the predeclared
real-data matrix now have a [bounded source/evidence tier](wasm-fetch-source-reconciliation.md): all 12 Fetch types are reconciled, and RI/NC/CA retained-graph cases ran against both builds and the packaged GUI. RI succeeds; NC/CA preserve expected native failures. Missing acquisition transitions, successful larger cases, fresh derivation and broader matrix workflows remain next. Broader recovery must preserve the no-repeat boundary. Preserve
existing working paths while identifying exact missing behavior in each parent.

The [archive follow-up](wasm-fetch-archive-acquisition.md) now delivers bounded
school/EIA acquisition or local import, retained source/cache/project handoff and
native-layout extraction/download. Both actual builds, successful native function
comparisons and final packaged GUI evidence cover that scope. Shared zone joins,
full Fetch orchestration/provider/capacity and other adapters remain open. The
practitioner/TUI action inventory is the next reconciliation output; preserve the
delivered archive path while ranking its remaining broader lifecycle conditions.

| Rank / parent | Native authority and existing browser baseline | Next acceptance boundary |
|---|---|---|
| 1. `bisect.Run/workspace-lifecycle` and `bisect.States/workspace-lifecycle` (WP-03/04), scoped preflight and continuation delivered | `main.rs` filters completion before parallel dispatch; `runner/support.rs::filter_incomplete` honors reprocess. `apply_run_args` propagates reset; `apply_states_args` does not invent reset. The live workspace composes selection/target actions with source queues, output recovery and verified saved continuation. | Preserve the tested skip/reprocess/incomplete/cancel/stale-root and no-repeat continuation contracts. Automatic post-Run installation and broader source/layout/host/custody cases remain open. A verification record alone grants no authority; each continuation needs its own fresh live review. |
| 2. `bisect.Fetch` / acquisition handoff (WP-01/05), paired recovery delivered | `main.rs::Commands::Fetch` validates vintage, expands all years, builds item lists and chooses check-only or download. `fetch-acquisition.js` already handles bounded acquisition/retry/cache; `state-source-acquisition.js` already stages paired sources and serial batches with checkpoints. | Preserve the delivered paired recovery contract. Finish source-type/consumer reconciliation, other vintage/profile rows and streaming/disk recovery; add only missing transitions with deterministic retained-body and stale/cancel/partial cases. Keep live-provider availability separate; saved local inputs stay usable. Do not relabel paired/batch acquisition as absent. |
| 3. `bisect.Verify` / composed evidence workflow (WP-01/06; consumes source/output identities), scoped delivery recorded | `verify.rs::run_verify` branches into package verification, assignments-only and replay, checking audit evidence before comparison. The later composed selected-output workflow and saved continuation now have scoped delivery records. | Preserve those tested contracts and distinguish native executable identity from loaded WASM and action receipts from verified installed bytes. Broader CLI branch/lifecycle acceptance stays open; do not repeat delivered composition as missing work. |

Relevant native files: [entry branches](../../crates/bisect-cli/src/main.rs),
[completion filter](../../crates/bisect-cli/src/runner/support.rs),
[Run/States configuration](../../crates/bisect-cli/src/runner/mod.rs),
[verification](../../crates/bisect-cli/src/verify.rs).
Existing candidate suites include `test_native_run_queue.mjs`,
`test_state_completion_wasm.mjs`, `test_state_source_acquisition.mjs`,
`test_state_source_acquisition_browser.mjs`, `test_package_verification_wasm.mjs`,
`test_manifest_replay_wasm.mjs` and `test_tui_verify_browser.mjs`.
This ranking inspects sources; it does not rerun or promote those historical
tests. Optional-C method/custody/notice work and WP-08's real-data matrix proceed
under their existing contracts without blocking unrelated workspace work.

The complete scope and acceptance conditions remain in the
[desktop-to-WASM plan](wasm-desktop-parity-plan.md).

The next Verify slice now has a
[selected-output inspection/replay implementation](wasm-native-output-verification.md).
It composes sidecar prechecks, exact native graph/manifest/index binding and
explicit portable replay without filesystem effects. Source-tree Node/worker
checks and four native-host inspection comparisons supplement two actual CLI
dry-run precheck cases. They do not establish full CLI replay equivalence.
Both actual builds and the final local package now pass their registered checks;
the child remains evidence-partial because broader workflow and formal release
acceptance remain open. A 145-file immutable snapshot binds this scoped delivery.
Active source/candidate pins have been reconciled after affected
workspace/project regressions; historical evidence and snapshots are unchanged.
The ordinary ledger check passes again, while the full-parity gate still requires
unverified parents and broader acceptance work to close.

[Saved native-workspace continuation](wasm-workspace-continuation.md) now has
separate Run/States children with partial evidence. Completed original-source
results are replayed and inspected before a private fresh-root review; accepted
rows remain exact and only unfinished states execute. Data-absent prior actions
do not repeat reset; reappearing data blocks that target. Both engine variants,
default and final-package browsers, three-year order, RI two-district partitioning
and interrupted real OPFS removal are covered by a 180-file local snapshot.
Explicitly C-backed prepared batches now propagate runtime availability, compare
loaded identities before dispatch and retain actual C identities. Native bulk
defaults still select metis-core. No parent or formal acceptance-v1 promotion is
made; broader matrix, acquisition/consumer reconciliation, custody and release
work remain required.

[Reviewed label repository application](wasm-label-repository-installation.md)
now has separate evidence-partial LabelAnalyze/LabelReport child contracts.
Fresh native merge recomputation precedes live-root review; exact current and
absence expectations precede writes, stage indexes follow ordinary artifacts,
and registry writes/readback are last. Explicit continuation retains exact
intended bytes and refuses unknown partial/concurrent edits. Both actual engines,
actual native CLI consumption/tamper refusal and final-package worker/OPFS checks
pass. This does not close whole native label filesystem orchestration, OS picker
coverage, formal acceptance-v1 or release/full-parity parents. The practitioner
source inventory is checked separately and remains a mapping/ranking record,
not a competing runtime acceptance tracker.

[Browser PDF/print delivery](wasm-report-pdf-delivery.md) now has scoped Report
and LabelReport child evidence: both actual engines, five genuine PDF cases per
run, five exact native-host replies, two actual native CLI missing-tool outcomes
and final-package regressions. Independent PDF text/page/media/image/action
checks and visual QA support the named Chrome renderer. The immutable inherited
snapshot verifies 1,461 mappings without recopying prior engines. Print events
and user-selected PDF bytes confer no successful-save/content-match claim.
Parents and formal acceptance-v1 remain partial. Native LabelReport HTML/JSON generation has the later scoped delivery below; live application remains a separate required missing child. Reviewed rename and import now have later scoped deliveries; native-summary application now has its own later delivery; build application remains missing.

[Reviewed RCOUNT transcript application](wasm-rcount-transcript-installation.md) now has an implemented, evidence-partial child under `rcount.Verify`. Actual default/C engines match three on-disk CLI transcripts, including failed verification; both source browsers exercise fresh review and confirmed writes. Both-engine/final-package regressions, the 1,869-mapping verified immutable snapshot and scoped review now support this local delivery. Broader host/capacity and formal release acceptance remain open. Native status and supplied lineage are preserved; verification grants no certification authority.

[Native label/year summaries](wasm-native-label-summary.md#scoped-evidence-closure) now have scoped generation evidence registered under LabelReport: fresh default/C engines, exact CLI comparisons, final-package browser checks and a 3,187-mapping inherited snapshot. The generation child remains evidence-partial; reviewed live-summary application has the later scoped delivery below. This does not close broader filesystem, host/API/capacity or formal release acceptance.


[Reviewed label rename](wasm-label-rename-application.md#scoped-evidence-closure)
now has implemented/evidence-partial selected-root application and shared native
case-alias safety children under Mv. Five kernel tests, 31 serial native tests,
four actual CLI alias-preservation cases, fresh default/C engines with eleven
CLI cases/forty-one interruption boundaries per engine, all 25 final-package gates
and the 9,790-mapping verified inherited snapshot support the local delivery.
Root authority remains private/single-use; Open is inert and prior in-session
receipts are visibly historical for changed preparations. Whole-parent,
OS/API/capacity and formal acceptance-v1 remain open. Current missing-implementation
ranking is build application; successful medium/large and national WP-08 coverage remain the next separate verification tasks.


[Reviewed native label import](wasm-label-import-application.md#scoped-evidence-closure)
had an implemented/evidence-partial child in the original package. Both
original engines passed 14 measured CLI outcomes and four recovery boundaries;
all 26 final-package gates and the verified 6,153-mapping inherited snapshot
support the bounded selected-root adaptation. Exact source/companion and registry
binding, private single-use review, inert saved baselines and registry-last
readback preserve the scope.

The current child is verified for reviewed assignment application. Both builds
preserve exact unsigned 64-bit IDs in assignments and downloads, while plan
conversion retains its context and district checks. Each matched twelve new
width cases and reran fourteen established CLI cases and four recovery
boundaries. Real Chrome workspace readback, ZIP, and inert Open passed, and the
three Config contracts that share the parser were reverified on the same builds.
The seven-shard archive restored all 972 evidence files into an empty root.
Malformed registries remain outside successful parity: the browser refuses
before writes, while desktop may leave partial effects. Broader workflow,
OS picker, capacity, locking, and release remain unresolved.

The external label handoff is a separate verified LabelAnalyze child. Both
desktop builds select the assignments named by the active external import
index. Both Chrome profiles read the real desktop analysis index and match
the native summary report, including the portable ZIP and reviewed workspace
application. The existing LabelReport child was rerun on the same builds.
This proves the handoff; whole analyzer and filesystem parity remain open.

Imported-plan graph contiguity is now a separate verified LabelAnalyze child.
Both native and browser profiles match exact report and analysis index bytes.
The fixtures cover connected and disconnected plans, three guarded refusals,
and 37,700 units across four retained graphs. Real Chrome tests verify exact
ZIP output and inert archive Open for California tracts and Rhode Island
blocks. The same five-shard restore preserves the nine related scopes.


[Reviewed ordered native-summary application](wasm-native-summary-application.md#scoped-evidence-closure)
now has an implemented/evidence-partial LabelReport child. Twenty fresh engine
stages include 42 native report tests; each profile has nine CLI outcomes,
nine exact fixed-clock native outcomes, 31 interruption boundaries and actual
worker/OPFS. Both registered profiles, all 27 staged-package gates and the
8,787-mapping inherited snapshot verify. Original live-clock self-overlap hash
differences remain differences; fixed-clock comparison is supplemental native
orchestration, not a relabeled ordinary CLI run. Per-year writes, exact observed
prefix continuation, inert baselines and completed-prefix status retain their
boundaries. Whole-parent/OS/API/capacity and formal acceptance-v1 remain open.


Current scoped Build delivery: [reviewed native-order application](wasm-label-build-application.md#scoped-evidence-closure)
now has fresh CLI/default/C, exact root/effect and failure-prefix comparisons,
received-state cancellation retention, passive project/archive recovery, both
registered regressions, all 28 staged gates and a verified inherited snapshot.
The child is implemented with partial evidence; its parent and acceptance-v1
remain unresolved. Earlier ranking text is historical: the bounded application
shortlist is delivered. Full-chamber NC/CA now has a separate successful CLI/
WASM/GUI tier; broader source/map/report/host/recovery, national and native
lifecycle/source-consumer reconciliation remain the next open work. This
registration changes no tested runtime bytes and deploys nothing.


National retained-data closure: [queue and recovery](wasm-national-browser-capacity.md#scoped-evidence-closure--2026-10-24), [national display map](wasm-national-retained-map.md) and [large-context follow-ups](wasm-large-context-followup.md) now have three implemented/evidence-partial child contracts. Both actual staged profiles, fresh engines, registered regressions and all 29 package outcomes pass; the 9,586-mapping immutable snapshot fully verifies. Run parent state remains partial; Map/TUI parent states remain unreviewed. The practitioner source inventory binds 110 sources. Full source/lifecycle/host reconciliation and acceptance-v1 remain open.


[Passive installation outcomes](wasm-installation-outcome.md#scoped-evidence-closure)
now have implemented/evidence-partial Run/State/States child records. Both staged
shells pass exact cancelled/completed receipt downloads, missing-receipt disclosure,
inert download/Open, OPFS recovery and asset/offline closure. The 11,076-mapping
inherited snapshot fully verifies; existing parents remain partial. Practitioner
source coverage is now 111 pins. This browser adaptation closes no native CLI,
real selected-folder host, automatic cross-session custody or acceptance-v1 gap.


[Bounded local installation history](wasm-installation-outcome-local-history.md#scoped-evidence-closure)
has implemented/evidence-partial Run/State/States children, separate from session
outcome downloads. Nine staged outcomes pass across both existing engine profiles,
including actual reload mid-write, inert stored history, stale-tab final-save/
Forget refusal and actual storage-unavailable session fallback. The standalone
1,508-mapping scope snapshot fully verifies. Practitioner source coverage binds
112 sources; current candidate references were reconciled separately from old
execution bundles. Parent/native/host and acceptance-v1 gaps remain unresolved.


[Grouped local adjacency inputs](wasm-native-adjacency-bundle-import.md#scoped-evidence-closure)
now have an implemented/evidence-partial State child. Ten dual-profile stages pass
exact native-host/WASM preparation and staged atomic source import, plus 42 native
Fetch planner/handoff source cases without acquisition. The standalone 1,795-mapping
scope snapshot fully verifies. Practitioner inventory binds 114 current sources.
The Fetch adjacency row still marks release network/authentication behavior missing;
no parent or acceptance-v1 is promoted by this local-file adapter.


[Explicit private release bridging](wasm-adjacency-release-bridge.md) has an
implemented/evidence-partial State child. Both actual private-provider browser
shells pass original-byte digest checks, secret-free exports and explicit matching
supplemental preparation; 18 package stages and stream-fault checks pass. The
independent 1,549-mapping snapshot fully verifies. Practitioner coverage binds
119 current source identities. Direct private CORS remains unsupported; hosted
Cloudflare runtime/origin/rate/log and full native/parent acceptance stay open.


[Explicit raw election source mapping](wasm-election-source-format.md) has an
implemented/evidence-partial Analyze child. Thirteen staged checks pass with
independent Python CSV bytes, complete native/WASM standalone and combined analysis,
raw-source/mapping/seed project retention and both actual browser profiles. The
1,555-mapping snapshot fully verifies and practitioner coverage binds 121 sources.
Native/direct provider probes return HTTP403; actual publication, broader acquisition,
Parquet/national/host and full parent acceptance remain open.


[Native Fetch wildcard ZIP](wasm-native-fetch-release-bundle.md) has an implemented /
evidence-partial Fetch child. Sixteen final staged checks and both actual private
browser/native gh file-tree comparisons pass, including original opaque artifacts
and exact marker bytes. A real-browser post-acquisition cancel test fails the
prototype and passes final v2. The 1,610-mapping snapshot fully verifies; current
practitioner coverage binds 122 sources. Export does not implement target skip,
force, installation or recovery. Hosted/parent/acceptance-v1 remain open. Stale
Build/Run/States parent prose now acknowledges prior scoped deliveries without
changing their historical evidence identities or full acceptance states.


[Reviewed Fetch folder application](wasm-native-fetch-folder-install.md) has an
implemented/evidence-partial Fetch child. Eighteen final stages, ten supported
actual CLI file-tree comparisons, both injected-picker UI profiles and both real
Chrome OPFS checks pass. A late-directory cancellation test fails the prototype
and passes v2 before file creation. The independent 1,823-mapping snapshot fully
verifies; practitioner coverage binds 124 source identities. Actual external native
permission/revocation, close/reload/durable custody, wider hosts/scales and full
parent/acceptance-v1 remain open. OPFS success does not replace the not-run native
picker gate.

## Actual Windows directory check — 2026-10-29

The completed user-operated Chrome 155 directory check uses a real external OS
directory handle and granted read/write permission, with no OPFS substitution.
One supplied accepted Run produces output matching an actual native child reply;
review writes nothing, installation retains exact generated bytes and the existing
witness file, and completion has a final manifest without an incomplete marker.
Outcome download and project Open pass without a picker, execution or restored
installation authority. No partition was run. This closes that specific observed
Run-output grant/write/readback case; it does not prove Fetch/report folder paths,
revocation, storage exhaustion, other hosts, a final package or full parent parity.

The terminal receipt is
`target/real-directory-grant-20261029/run-1793317667170/evidence.json`.
The separately read-back-verified snapshot is
`target/real-directory-grant-20261029/snapshot-scope/snapshot.json`, SHA-256
`5092ce5dc7a6190f6648950551c78874318f85159048f37cf1db2fda9bca1040`:
280 mappings, 279 content identities, 65,507,840 logical bytes. It retains all 247
served source assets at their observed hashes, the native/WASM bytes, supplied
project, native request/reply, output files, screenshots and observation script.
Earlier timeout attempts remain retained. The browser is closed; no further user
folder interaction is required. Existing parent/acceptance-v1 states are unchanged.

## Run option source reconciliation — 2026-10-29

Seven previously unreviewed Run options now distinguish actual native consumers
from their existing browser mappings: year, version, output_dir, workers, debug,
partition_mode and states. The review traces the CLI Run branch, shared default-mode
compositor, StateConfig handoff and browser compilation/queue/streaming adapters.
Native worker caps remain separate from browser CPU/selected-state limits. Raw
state filters cannot turn an unmatched selection into all states. Output paths
remain intent until an explicit selected-root action. Debug is assigned but has
no native runner consumer; it is a shared limitation, not a missing browser log mode.

Each row has a required not-run workflow contract with current source and candidate
test references. No runtime evidence or acceptance bundle is created. Historical
review/evidence fields, direct-read traces and all other 233 option rows are
unchanged. The preserved preimage, exact changed-row record and passing ordinary
ledger/native extraction logs are under `target/run-option-source-reconciliation-20261029`.
There remain 199 unreviewed engine-option records. This reduces a source-review
gap without establishing executable support or closing any parent acceptance.

## YAML option source reconciliation — 2026-10-29

All 22 YAML document/algorithm records now carry current native consumer and
browser mappings, with required not-run workflow contracts. The review traces
shared parsing/resolution, CLI `NativeAlgorithmConfig` conversion, actual Build
workers/years/tolerance consumers, and configured browser dispatch. The raw YAML,
wide integer/seed values and declared backend remain distinct from execution
availability and source authenticity.

The source review preserves several native distinctions: Build uses its CLI label
for output namespaces; name/description are also inspection metadata. Build workers
and years do not schedule work in a single-graph Run. `resolved_analysis_types`
provides metadata/defaults but native `run_build` does not call it to launch
analysis. Native search consumption depends on structure; MKA's actual alternative
metric token is `polsby`, while its documented `polsby-popper` spelling does not
select that branch. No silent backend or metric substitution is introduced.

Preimage/after-image, changed-row identities and passing ledger validation are
under `target/yaml-option-source-reconciliation-20261029`. All other 218 records,
including the preceding Run reconciliation, and all historical evidence fields
remain unchanged. There are now 177 unreviewed option records. Source mapping
does not promote any workflow, backend, method, host or full-parity acceptance.

## State option source reconciliation — 2026-10-29

The 67 remaining unreviewed State fields now identify native consumers, current
browser mappings and required not-run workflow contracts. The review follows
location/year/chamber/district resolution, adjacency bypass, actual preset then
structure/weight/search overrides, StateConfig handoff and typed browser State
preparation. Its findings distinguish configuration from actual downstream use:
some presets reset seeds; a structure may consume only a search count or ignore
the search; geographic CVD requires centroids; multiscale needs aligned source
levels and a matching request; ILP fallback/completion is not an optimality proof.

Version/output/label/manifest fields retain separate explicit workflow/output
actions rather than disappearing from the prepared compiler's refusal list.
Partisan/COI paths bind selected source identities and never authorize implicit
file access. Electoral metadata, chamber policy, source authenticity and browser
resource bounds remain distinct requirements.

Preimage/after-image, changed-row identities and passing ledger validation are
under `target/state-option-source-reconciliation-20261029`. Other 173 records,
all historical review/evidence fields and native direct-read traces are unchanged.
The 67 required workflow contracts remain not-run, and no acceptance bundle or
new runtime result is minted from this review. There remain 110 unreviewed
option records; full parent/method/backend/host/release acceptance stays open.

## Bulk States option source reconciliation — 2026-10-29

The 46 remaining States fields now have current native-consumer/browser mappings
and required not-run workflow contracts. The review uses the actual States branch,
`StatesConfiguration`, native completion/scheduler helpers and browser bulk
compilation/queue/workspace paths. States requires year, version and output_dir;
it does not inherit Run's all-year/default-output policy or single-State's
constructor. Bulk GeoSection/AreaSection budgets, search count defaults and the
strictly-positive alpha override remain distinct. CVD overrides are traced through
the current shared bulk constructor.

Reprocess reaches native completion filtering, separately from reset and install
overwrite. Browser workers retain requested counts while using explicit browser
CPU/selected-state caps. Debug is copied into StateConfig but has no native runner
consumer; no debug-log feature is invented. Source paths remain intent until a
selected-root action. These findings do not prove a complete States CLI workflow.

Preimage/after-image, changed-row identities and passing ledger validation are
under `target/states-option-source-reconciliation-20261029`. Other 194 records,
historical evidence fields and native direct-read traces remain unchanged.
All three CLI option families and YAML now have source reconciliation, while
64 prepared-graph/engine-option records remain unreviewed. No new runtime bundle,
verified parent or release acceptance follows from this metadata checkpoint.

## Prepared-graph source reconciliation — 2026-10-29

All twelve prepared-graph fields now carry native payload/source/kernel references,
current browser mappings and required not-run workflow contracts. These are
browser JSON envelope fields, not automatically native CLI flags: schema_version
has no corresponding CLI flag or native binary-version identity. Its native
references identify the decoded payload counterpart only. Native graph loaders,
explicit RPLAN unit ordering/subdivision context and native geometry/method
consumers remain distinct from browser import safety and capacity checks.

The review preserves complete ordered population/topology vectors, canonical
nonnegative physical edges including zero lengths, optional empty geometry arrays,
method-specific area/centroid requirements and explicit custom county identities.
Legacy omitted unit_kind and explicitly declared Census/custom kinds retain their
different validation paths. Declared source scope is not source authenticity.

Five earlier current MKA source findings now explicitly disclose the native
Polsby-enum-to-Reock-scoring fallback. This clarifies the requested enum versus
actual scoring; it introduces no algorithm or true perimeter-scoring support.
Preimage/after-image, changed-row identities and passing ledger validation are
under `target/graph-option-source-reconciliation-20261029`. Twelve new graph
contracts remain not-run, five MKA source findings are clarified, and all other
223 rows plus historical review/evidence/native traces remain unchanged. There
remain 52 unreviewed portable engine options; no runtime acceptance is promoted.

## Portable option and inactive-flag contract checkpoint — 2026-10-29

The remaining 52 portable engine options now map to actual native method,
parameter or observer APIs and existing browser actions. These references do not
assert identical CLI flags/defaults or complete orchestration. Findings retain
method-specific seed/budget interpretation, unavailable-backend refusal, bounded
round/ILP observation, explicit source/fallback selection, profile-filter limits
and the native MKA scoring fallback. Required contracts remain not-run.
Preimage/after-image, changed-row identities and passing ordinary ledger validation
are under `target/portable-option-source-reconciliation-20261029`.

A subsequent audit adds required comparison contracts to the 22 already-reviewed
parsed-but-unused Run/State/States fields. Both original argument vectors and
parser records must survive a no-effect comparison: their raw text/provenance
need not be identical. Browser print-only/DPI follow-ups are explicit adaptations;
the portable objective API does not implement unused native State objtype.
Primary native-consumer links for Verify and state_name now match their existing
child references, without modifying those child contracts or their evidence.
Verify's current open issue acknowledges the previously retained read-only
manifest native/browser checkpoint while keeping complete workflows open.

The latter preimages/after-images and passing ledger/eight guard tests are under
`target/inert-flag-source-contracts-20261029`. All 309 records now have primary
source links and explicit contracts; no engine-option record remains marked
unreviewed. This is structural source/contract coverage only. The ordinary ledger
permits unresolved records, all newly declared contracts stay not-run, and no
runtime bundle, verified parent, complete compatibility matrix or release
acceptance is created by these source reviews.

## Parsed-unused State native witness — 2026-10-30

The maintained `test_native_state_inert_flags.py` passes seven actual native CLI
cases on the retained four-unit custom adjacency: baseline, DPI=300, print-only,
target-mm-districts=1, objtype=vol, all four together, and parser refusal of
DPI=600 before output creation. The successful variants preserve exact assignment
and RCTX hashes and the declared state/seed/label/source manifest fields. Raw
arguments, native logs and complete output files are retained; whole manifests
and all methods/backends are not claimed identical. Native print-only still
executes in this fixture; browser no-execution intent remains an explicit
adaptation.

The test is registered after native CLI compilation in standard/laboratory and
C-legacy build paths. `target/state-inert-flags/1793348324642079700/evidence.json`
retains actual binary/input/test identities and outcomes, with complete build
provenance explicitly unproved. Four current contracts gain a candidate test;
their required paired workflow evidence stays not-run. Only 24 current candidate
source pins were refreshed. Historical evidence and all other catalog semantics
are preserved by `target/state-inert-flags/registration-1793348561492404700/preservation.json`.
Ordinary validation passes 309 records and full-parity still refuses unverified
parent evidence. Whole builders have not rerun; no field or parent is closed.

## Connectivity adoption and tuning continuation — 2026-10-30

The isolated caller comparison passed 104 small cases and full CA VRA native
and WASM replies against the retained original native reference. CA remains
contiguous with a population-balance miss, exactly as before. The two tested Rust
files are integrated without changing candidate order, score, global-ID tie
break, seeds or repair iteration budget. The original one-shot function is
unchanged. Three maintained independent graph/snapshot/query-history tests pass
and are registered in both normal builders and C legacy gates. AP-12 is
mitigated for this caller; wider callers and final package acceptance remain open.

The original algorithm continuation passed 288 synthetic Area/VRA cases, 98
real-state project/export roundtrips and two matching real-state failures, then
failed on a one-district GeoSection tamper assertion. Its package continuation
refused that failed prerequisite. These failed statuses remain unchanged. The
same test assumption was corrected in recursive, percentile and ensemble
project checks. All four focused corrections pass using the retained pre-adoption
engine pair: 592 exact cases and 18 project roundtrips, including six single- and
twelve multi-district cases. The diagnostics do not accept a newly built engine.

Original balance-miss counters covered only synthetic cases. Independent
arithmetic from all 98 retained projects confirms one real balance miss (CA VRA)
and zero recorded disconnected plans. Raw projects are now copied and
hash-verified in `retained-projects.json`; the original-path readback is preserved
in `retained-ratio-feasibility.json`. Five tuning reports now record separate
synthetic/real-state counters and aggregate totals. Historical receipts are not
rewritten or promoted. Current source/candidate pins are refreshed with inverse
checks preserving all other catalog semantics and historical evidence.

R6 (`target/parity-full-build-20261030-r6`) captures 3,232 inputs and preserves the
original 50-state standard/C build settings. Its first wrapper launch preceded
completed source capture and was refused before any build. That failed preflight
is preserved in `preflight-attempt-1`; the retry waited for completed capture and
verified the original engine identities before focused tests. Those passed and
the complete standard build is now running; C/package acceptance is pending.
No new package is deployed and no full parent or WP item is closed.

## C gate preflight correction and R7 — 2026-10-30

R6 was deliberately stopped during compilation after identifying that the newly
registered connectivity Cargo test preceded the C legacy disk-capacity guard.
Only its verified compiler process subtree was stopped; the wrapper retained its
failed status and confirmed all 3,232 captured inputs unchanged. Its interruption
reason and old gate source are preserved. The regression now runs after existing
capacity and supplied-record checks. The current candidate pins were refreshed;
historical evidence and all other catalog semantics remain unchanged.

R7 (`target/parity-full-build-20261030-r7`) captures and verifies source inputs
synchronously before launching its runner. It preserves the same 50-state and
full standard/C browser-check commands. Compared with R6, only the C legacy gate
and the two catalogs' current candidate pins changed; 3,229 of 3,232 inputs are
identical. The four already-passing focused tests have unchanged source bytes
and their diagnostics are reused, while every registered fresh standard/C gate
still runs. The standard build is active; no complete package or full parity
has been accepted or deployed. R5, its failed continuations and R6 remain distinct
historical attempts.
