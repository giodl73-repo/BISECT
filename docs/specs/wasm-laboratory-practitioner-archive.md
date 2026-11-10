# Full-state laboratory audit handoff

Status: focused native/export/project and two-profile browser/map checks pass;
complete build and final-package acceptance remain pending.

The first 50-state build attempt under `target/parity-full-build-20261028` failed
while exporting California's economic-character plan. All 9,129 tracts and 52
districts were retained. The selected character inputs are synthetic fixture
values, not observed economic data. The native plan text is about 4.6 MB, the
context about 6.0 MB and the combined audit handoff about 31.9 MB. The ordinary
project limit and the smaller exact-proof source limit both blocked the handoff.

`preparePractitionerProject` retains ordinary projects when they fit. A handoff
exceeding the 25 MB ordinary-project budget uses the existing workbench archive
format and its unchanged 128 MiB stored / 256 MiB expanded bounds. It retains the
original plan/context text and matching unit geometry. Larger exports offer an
explicit audit-archive download; direct ordinary-project transfer stays disabled.
The user opens the file with **Open workbench archive**. Opening grants no engine,
filesystem or provenance authority.

The RPLAN audit source has a separate 8 MiB limit, matching the existing context
budget. Typed RPLAN inspection/audit accepts that boundary while keeping 100,000
units / 500 districts. Generic exact proofs, profiles, certificates and lineage
keep their 3 MiB limits. Other operation-specific limits remain separate. No
districts, source attributes or geometry were removed to fit the export.

The focused California test matches the complete native Rust export adapter
reply, preserves original plan/context bytes through archive roundtrip and proves
ordinary projects still refuse oversized imports. An actual desktop `bisect
validate` accepts the same native file; both freshly rebuilt WASM profiles' typed
inspectors also accept it. Both accept an exactly 8 MiB padded plan, reject a
larger one and preserve/reject a native-invalid BOM. This does not establish all native CLI orchestration or source
authenticity. Project/UI regression checks include a >3 MiB plan request and
8 MiB pre-read refusal while preserving the profile's 3 MiB refusal.

Evidence and the original failure request/logs are retained under
`target/laboratory-practitioner-archive`. The failed complete build remains
failed. A new build attempt must capture the changed sources and use new output
directories. No deployment or full-parity acceptance follows from this fix.

The current focused run is `run-sz4zih/evidence.json`; its 31,874,318-byte archive
also has a saved laboratory input for the browser follow-up. The complete project
handler regression passes. `text-boundaries.json` records both rebuilt profiles'
large-plan checks and actual native CLI validation. The original character
fixture's 10% engine-test tolerance and synthetic inputs are unchanged; this is
not the separately evidenced 0.5% full-chamber compatibility campaign.

Follow-up `run-TaAgRp` also covers the ordinary value-count limit. Actual browser
checks retain the laboratory download, workbench Open/explicit Run/Save/reopen,
all original source bytes and the full 9,129-unit / 52-district map. The screenshot
was visually inspected. Laboratory and toolkit engines are pinned separately in
these compatibility checks; they are not claimed as a new final-package run.

The standard builder now registers the full CA handoff when that data is present;
the legacy profile gates register large typed-plan/native validity boundaries;
the final C builder registers the handoff with staged-source and stable-manifest
checks. Missing CA scope is explicitly not-run. PowerShell/embedded Python syntax
and ordinary ledger/practitioner validation pass. The
[six-lens review](../../signals/roles/check/wasm-laboratory-practitioner-archive-roles-check-2026-10-28.md)
keeps complete build, final-package and immutable evidence closure conditional.

R2 is under `target/parity-full-build-20261028-r2`, with 3,224 source/fixture
identities captured before execution and new standard/C output directories.
Inactive generated symbol caches and Cargo static archives older than seven days
were verified and evicted for validation-host disk space; their identities are
retained in its cache-eviction record. Sources, executable/dynamic binaries,
packages and evidence were retained. This is not a browser memory requirement.

R2 was deliberately stopped after an isolated run reproduced a stale legacy
handoff assertion: it required exactly plan/context and rejected the retained
geometry and native text sources. A second legacy Build check required every
export to fit an ordinary project. These tests now verify native fields, exact
text sources and appropriate ordinary/archive roundtrips. The corrected
three-state native audit handoff passes; a focused CA/RI Build at the original
0.5% tolerance passes retries, maps, inert reopening, tamper refusal and exports.
R2's generic failed status is qualified by `stop-reason.json`; it is not counted
as an algorithm failure or a passing build. A third complete attempt is required.

R3 under `target/parity-full-build-20261028-r3` failed at the subsequent
multiscale gate after completing all 500 real-state character project/export
replays and the registered CA archive native/browser checks. The character
matrix recorded 564 exact native/WASM cases, 66 matching generation failures,
42 balance misses and zero disconnected plans; these are not universal balance
successes. Its 3,224 captured source/fixture identities were unchanged at failure.
The native/browser archive receipts are `run-W5J7H3` and `browser-BuM2UL` under
`target/laboratory-practitioner-archive`.

The multiscale test still expected supported unweighted initialization to fail.
The corrected test covers all three resolution pairings, fixed/adaptive modes,
native/WASM equality and unweighted assignment invariance to physical edge
length changes. It passes 90 deterministic comparisons and 32 matching invalid
requests, with six balance misses and zero disconnected plans. The old test
bytes and failing request are retained with R3. A separate `remaining-gates`
diagnostic continues the unexecuted standard gates; it cannot change R3's failed
status or count as a complete builder pass. A new complete build and final
C package checks remain required.

Downstream diagnostics also exposed two obsolete boundary assumptions. The
selected-adjacency counter now observes the combined native source/compiler
operation and asserts one preparation, zero standalone preparations and one
partition; 11 complete native/WASM replies and independent native assignments
pass. Legislative ZIPs now verify the exact restored adjacency-intent argument,
both reviewed/original command hashes and `path_accessed: false`, retaining
source bytes and native exports. Four House/Senate Runs and their ZIP checks
pass, with four preparations/four partitions. Their prior failed diagnostic
receipts remain unchanged. `remaining-gates-3` continues later registered gates.
The [six-lens test review](../../signals/roles/check/wasm-build-gate-reconciliation-roles-check-2026-10-29.md)
and PP-73 retain complete-build/final-package/immutable-closure conditions.

Runtime follow-up: both normal builders now bind `BISECT_TEST_WASM` to their own
compiled module and restore the caller's environment afterward. The Fetch
acquisition test's historical default otherwise used an older module; an
explicit runtime-mismatch refusal is retained. A fresh six-case Fetch reference
and its actual browser handoff pass with the current module. Diagnostic 3's
separate missing-root refusal was caused by the extracted harness omitting the
prefix's reference variable, not by the normal builder.

Diagnostic 4 then exposed an intermittent report PDF file-read refusal. Two
reruns passed without a product change. A separate deterministic regression
found and corrected stale PDF-read cleanup clearing a newer pending selection;
the [ownership checkpoint](wasm-report-pdf-read-ownership.md) retains distinct
failure/recheck evidence, five genuine PDF cases and 28 inspected pages. Later
browser and standard gates continue in diagnostic 5. These local checks do not
turn any failed complete-build or diagnostic receipt into a pass.

The custom bound-workflow browser check subsequently waited for obsolete status
wording. Its corrected applied-binding/control-state wait passes all three
native-equal custom workflows and exports. The remaining registered browser
section passes under `remaining-browser-gates`; this is an explicitly bounded
diagnostic, not a complete standard build.

R4 is running under `target/parity-full-build-20261029-r4`, with 3,224 current
source/fixture identities captured before execution, fresh standard/C output
directories and browser checks enabled. Its terminal wrapper receipt remains
the authority for complete-build status. Starting it is not acceptance.

R4 terminated early at Fetch source coverage: the separate coverage catalog
still referenced the pre-PDF-ownership workbench hash and byte length. Its
current authority entry is reconciled, and the original catalog/failed receipt
remain retained. All 42 native planner and 42 FLETCH comparisons pass with no
acquisition claim; ordinary inventories pass. R5 under
`target/parity-full-build-20261029-r5` now captures 3,228 inputs, adding the four
current catalogs to the unchanged 3,224 code/fixture inputs. It requests the
complete 50-state standard/C build with browser checks. Its terminal result is
pending; no package/deployment acceptance follows from this correction.

R5 completed the 500 real-state character project/export replays and registered
browser section, then failed in `test_accepted_state_workflow.mjs`. The test
required the standalone analysis-package record to be byte-identical to the
containing workflow's record. The data files, including manifest, were identical;
the manifest step metadata intentionally describes reuse rather than standalone
generation. The corrected check keeps exact equality for every data file and
checks the package record against the explicit reused-manifest annotation,
including all other fields/steps and the workflow's stage record. All three
native/WASM replies, four maps/report, partial/failure retention and ZIP/project
checks pass. The original test bytes and failed full-build receipt are retained.

`target/parity-full-build-20261029-r5/remaining-gates` now executes the complete
remaining standard tail with the original 50-state scope and browser checks.
It is a diagnostic continuation, not a passing complete R5 build. Inactive,
older-than-seven-day, single-link Cargo static/symbol caches were evicted for
host headroom; 573 file identities are retained in `cache-eviction.json`.
Measured free space rose from about 0.79 GB to 3.41 GB, less than the estimate
from logical sizes. Sources, packages, executables/DLL/WASM and evidence remain.

The continuation passed recovery, batch output and multiscale gates, then failed
on a full national character project exceeding the generic 500k-value JSON cap.
The retained economic fixture has 1,274,114 values in 18.96 MB. Laboratory v1
projects now have a separate 2m-value budget, preserving their 25 MiB byte cap,
64-level depth and typed/security checks. Ordinary project limits are unchanged.
Laboratory downloads use compact JSON: indentation otherwise expands the valid
economic project to 43 MB. Actual 50-state economic/housing generation and
restoration pass 100 maps; actual browser Open/Save/reopen retains all inputs
and assignments in 18.96/10.04 MB downloads without a generation request.
Over-limit browser import preserves the open project. Resource bounds, existing
project regressions and both inventories pass. The area-tamper regression now
uses a 1% change because one square metre can fall within large-state relative
tolerance; the production tolerance is unchanged.

The source-overlay browser receipt is
`target/national-character-browser-dxfbS7/evidence.json`. The
[bounded-limit review](../../signals/roles/check/wasm-national-project-limits-roles-check-2026-10-29.md)
keeps final packaged browser/C checks and immutable closure open. Resource gates
are registered in both normal builders; the laboratory builder additionally
registers the packaged national character browser roundtrip. R5 and its failed
continuation remain failed, and previously staged/deployed packages are unchanged.

Separate experimental staging `dist/wasm-national-project-20261029` now passes
the require-package browser test with no source overlay: current served JS/CSS
identities are checked, both 50-state projects save/reopen below 25 MiB, and
over-limit import preserves the current data. Its receipt is
`target/national-character-browser-yYTu6T/evidence.json`. Verified static data
are shared read-only; changed code and other files are copied, with a staging
inventory retained under R5. Prior staging is unchanged. This is not a fresh
complete builder pass; offline manifest/final C/immutable closure remain open.
R5 `remaining-gates-2` continues the unexecuted tail against this explicit staging.

That continuation subsequently passed proportional-method comparisons, election
CSV and partisan TSV import/worker checks, then stopped at stale partisan option
rejections. A seven-option native/WASM probe agrees on five refusals and successful
AreaSection/Flip dispatch. `test_partisan_wasm.mjs` now retains the five refusals
and adds district-count/seed comparisons for both supported combinations, typed
project/evidence checks, and a requirement for successful cases in each. Native
AreaSection can reject an infeasible sampled root split; the new cases require
the same explicit feasibility refusal from WASM. An initial overstrict assertion
and its failed run remain in R5 `partisan-option-correction/attempt-1`.

The full existing partisan matrix and all 50 states/five searches run under
`partisan-option-correction`; unmodified BFS through convergence gates run under
`remaining-algorithm-gates`. Their terminal receipts govern the results; launch
alone establishes no pass. The separate 12-case partisan search-boundary gate
passes. No historical evidence was repinned. Inactive single-link compiler
archives/symbol caches older than 48 hours were reclaimed with per-file identities
in `cache-eviction-48h.json`; measured free space rose from 2.12 GB to 4.45 GB.
Sources, executables, WASM, packages and evidence remain. Complete fresh standard/C
build, final packaged checks and closure remain open.

The independent algorithm continuation has completed BFS (300 exact native/WASM
results and 50 project roundtrips), u64 seed handling (56 exact cases, four
matching generation failures, 18 invalid values and three projects), and N-way
tuning (24 exact cases, five refusals and three projects). Receipts are retained
under `remaining-algorithm-gates`. BFS reports 260 balance misses out of 300;
N-way reports eight. Both report zero disconnected plans. These results establish
matching behavior within those cases, not successful balance for every run.
Later tuning gates and the complete partisan run remain in progress.

`remaining-package-gates` now waits for both exact prerequisite process identities
to terminate with passing receipts before starting the integrated practitioner
builder and the unmodified remaining laboratory/offline gates. It checks free
space and refuses an existing toolkit output, captures current source/fixture/
catalog identities after the prerequisites finish, and verifies them at the end.
Waiting/launch is not acceptance, and sources captured by that live build must
remain unchanged. This continuation cannot turn failed R5 into a complete pass;
fresh complete standard/C acceptance remains required.

The corrected full partisan gate is now terminal passed in
`partisan-option-correction/status.json`: 900 whole native/WASM results with
independent weighted cuts, 10 AreaSection/Flip option successes and two matching
AreaSection feasibility failures, 9,900 evidence refusals, 13 invalid inputs/options
and all 250 state/search project/export roundtrips. It records one balance miss
and zero disconnected plans. The focused runner retains its failed first attempt,
final receipt, source identity and final reference request; post-run artifact
identities are explicitly separate from full build provenance/all-case raw output
retention. Other algorithm gates continue; the package runner still waits for them.

The full-CA VRA tuning case continues using CPU without a terminal reply.
`vra-ca-native-probe` runs the same retained graph, synthetic fraction profile,
52 districts, eight seeds per ratio, volume objective and three internal trials
in an isolated copy of the native reference executable. Its request and binary
identities are retained; it does not replace or interrupt the original WASM gate.
This is a native-only cost/progress diagnostic, not CLI orchestration, browser
acceptance or a completed build. The copied executable avoids locking/replacing
the normal reference binary during later builds. Await its terminal evidence
before attributing the long case to a target-specific stall or shared search cost.

The original first WASM call subsequently returned and its suite advanced to a
native comparison. The actual native request matches the isolated probe in every
decoded field (`vra-ca-native-probe/paired-request.json`); that transition alone
is not a successful case. Both native processes remain separate observations.
Five brief instruction-pointer samples of the owned isolated probe identify
`rgraph_core::connectivity::node_subset_connected` and heap allocation; the probe
was resumed before symbol resolution. This is a diagnostic sample, not a clean
timing benchmark or proof that every cost comes from that function.

Source inspection finds that each candidate boundary-move check rebuilds/sorts
the full undirected topology. An isolated copied-source experiment now retains
that topology and reusable query scratch, leaving the existing one-shot API and
all product sources unchanged. Native and WASM each pass 593,110 checks against
the original function and an independent weak-connectivity reference, including
small exhaustive directed/undirected graphs, reversed/duplicate node orders and
interleaved invalid/empty queries. The local 10,000-node/200-query cost is about
323/23 ms native and 141/24 ms WASM for old/prepared paths. Those measurements
are kernel-only and do not establish a VRA, full-engine or average-host speedup.
`target/connectivity-reuse-experiment-20261029/evidence.json` retains source,
lockfile and artifact identities. Caller integration, exact full-engine result
comparison, review and registered build acceptance remain required before adoption.

The isolated original-binary CA VRA native probe completed successfully at
2026-10-30T07:28:14Z. Its reply remains heuristic/unproved and reports
`within_requested_tolerance: false`; successful execution does not establish
balance feasibility. This is the sampled native adapter diagnostic, not clean
benchmark timing or whole browser acceptance. The original algorithm suite
continues separately against its original binaries.

`target/connectivity-caller-experiment-20261030` now contains a separate full
workspace with prepared-connectivity reuse only in the scalar boundary-repair
loop. Preparation is lazy, preserving singleton/no-repair bypasses. Candidate
order, score, seed schedule and the 200-iteration repair budget are unchanged.
The 890 captured baseline files match product source; all copied files except
the two declared isolated Rust edits match that baseline. Separate native and
WASM builds are running with their own compiler target directory. A continuation
queues 96 synthetic full-reply comparisons across Area/VRA, district counts,
search modes, objectives and exact seed widths, followed by native and WASM
comparison with the retained original CA VRA reply. Requests, replies and hashes
will be retained. These comparisons have not passed yet; no product Rust source,
normal reference executable or normal WASM module has been replaced. Review and
registered package acceptance remain required before adopting the optimization.

The first isolated caller build failed because the copied workspace omitted
`include!("../../scripts/rust/build_provenance.rs")`. Its failure, logs and
continuation status are preserved in the experiment's `attempt-1` directory.
That shared file was copied byte-for-byte, increasing the baseline to 891 files;
no product fix was needed. The retry reuses only this experiment's compiler cache
and the full-reply comparison remains queued until both builds pass.

The caller experiment's native build passed. A seven-lens single-agent review
records adoption conditions in
`signals/roles/check/wasm-connectivity-reuse-roles-check-2026-10-30.md`; it is not
independent signoff or package acceptance. AP-12 records repeated graph-topology
preparation as open in product. A complete copied-Rust scan now checks all three
literal include kinds: 121 references resolve, with computed paths explicitly
outside scope. Three proposed maintained regression tests pass against the
isolated checker with all six registry dependency versions/checksums matching
the project's lockfile. They cover exhaustive directed graphs through four
vertices, invalid/empty query histories, typed boundaries and owned snapshots.
`maintained-tests-evidence.json` pins the test, lockfile and log; the first test
run using newly resolved dependency versions is separately preserved. These
tests are not yet integrated into the product. WASM build, complete caller
comparison, fresh registered standard/C and package acceptance remain pending.

Both isolated caller builds now pass. The first comparison attempt stopped in
the harness because native diagnostic lines precede the terminal JSON error;
raw logs remain retained and the parser now requires and decodes only that
terminal adapter error while keeping all diagnostics. The next attempt matched
all 96 complete original/optimized native/WASM replies: 48 VRA successes and
48 matching Area rejections. Its success-coverage guard correctly refused an
Area acceptance claim from rejection-only examples. Both failed attempts are
preserved under `comparison-attempt-1` and `comparison-attempt-2`. The original
tight uneven-population matrix is unchanged; eight additional uniform-grid
Area fixtures from the existing tuning setup supply successful-case coverage.
The complete 104-case comparison and retained CA replay remain pending.

The corrected caller comparison now passes all 104 small cases: the unchanged
tight matrix has 48 exact VRA successes and 48 matching Area rejections, while
all eight added Area fixtures succeed. The four engine paths agree on whole
replies. Twenty-eight balance misses and zero disconnected plans are recorded;
parity is not feasibility. `small-cases-checkpoint.json` rechecks and pins all
208 request/reply files plus source and engine identities. The retained CA
replay continues separately, so the full experiment has no terminal pass yet.

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

The completed isolated builds' inactive `.rlib`/`.rmeta` caches were reclaimed
while the runtime comparison continued. No cargo/rustc process was live; the
927 single-link cache files were resolved within the two owned compiler target
roots and hash-recorded before removal. Executables, WASM, PDB/DLL, source and
evidence were retained. The active comparison's recorded native/WASM identities
matched before and after. `compiler-cache-eviction.json` records actual free
space rising from 2,371,485,696 to 3,402,518,528 bytes, above the current 3-GiB
package preflight. This is reclaimed cache capacity, not additional test acceptance.

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

## Post-build archive preparation — 2026-10-30

An independent continuation waits for R7's exact runner identity and terminal
whole-build pass before preparing any archives. It then checks all captured
source bytes and both final offline manifests, packages the unchanged tested
site trees, and reads every ZIP entry back against its original file hash/size.
It also preserves the captured source inputs and fixed build receipts in
separate archives. The candidate record explicitly leaves full desktop parity
unverified and deployment false; it cannot close canonical workflow/host/capacity
contracts. The archive step changes no site asset or source catalog.

The helper is syntax-checked and passes a separate container-level smoke test:
synthetic running-build status refuses before output creation, three known
UTF-8/binary/nested-path files survive exact ZIP readback, and five unsafe
archive-name forms refuse. No live build status is forged or altered.
`target/packager-smoke-jxpuc5ai/evidence.json` and R7's `packaging-recipe.json`
retain those helper identities. The actual R7 archive readback remains pending
behind standard/C completion, and no candidate package has been created yet.

## Predeclared Area/VRA regression controls — 2026-10-30

Before R7's Area/VRA gate, the retained 98 project baselines and two independent
pre-adoption native failure controls now form a 100-row expected-outcome matrix.
All 50 newly generated graph assets match the retained catalog byte-for-byte.
The matrix fixes graph identity/order, dimensions, exact assignment vectors,
population/district metrics and evidence before fresh results are observed.
Ninety-seven prior results pass their population check, one completed CA VRA
result misses it, and Nebraska/New Hampshire AreaSection have sampled-search
failures. These classes do not establish mathematical infeasibility or optimality.

The original native binary and the copied current R7 native/standard-WASM pair
produce the same exact two failure messages, with raw requests/replies, graph
and artifact identities retained in R7's `ratio-native-expectations`. The matrix
explicitly distinguishes successful baseline project/previous-assertion evidence
from retained raw native failure replies; it does not invent missing old native
success transcripts. Its scope is standard metis-core and Node WebAssembly, not
actual Chrome or explicit C execution. It supplements the canonical inventories
and creates no new feature/evidence status.

A separate owned continuation waits for a fresh R7 ratio report, then will compare
all 98 completed projects against those fixed expectations, independently derive
population/connectivity from each hashed graph and assignments, and freeze the
new project bytes. Its two failure rows use the separately retained raw probes
and the main gate's aggregate; they are not represented as per-case transcripts
of the main gate. The audit is pending and is not a whole-build or full-parity
acceptance bundle. Root build source, catalogs and running tests are unchanged.

## Fresh staged national roundtrip snapshot — 2026-10-30

R7 passed its staged laboratory browser check without source overlays: economic
and housing experiments cover 50 states each, 100 maps, preserved original
inputs/assignments/configuration, inert Open/reopen, zero generation requests,
and atomic over-budget refusal. Economic/housing downloads are 18,962,582 and
10,035,621 bytes respectively. The producer's original input projects still
match the preserved download contents in those declared fields.

`target/parity-full-build-20261030-r7/national-roundtrip-snapshot/snapshot.json`
freezes all 249 observed served assets, browser outputs, original project inputs
and relevant test/source files. Its SHA-256 is
`b4c486e4d10db93691f5f8886c83aacd9a8bce0f364c98f67e594682887f6331`.
The standard blob reader independently verifies 264 mappings, 261 unique blobs
and 123,488,718 logical bytes; 176 digest-verified immutable objects were reused.
The readback receipt is retained alongside it. Mutable source/package files are
not linked into the snapshot.

This preserves the current staged UI result, not completed final/C package
acceptance, native partition replay or directory-picker permission behavior.
The browser receipt did not record an exact browser runtime version, so none is
inferred. Source generation and final host/release obligations retain their
separate evidence. Root build sources and catalogs remain frozen.

Compiler-cache compression was considered when free space dropped. By the time
the guarded helper ran, free space was already above its 4-GiB threshold; it
changed zero cache files. The content-preservation receipt records that no-op.

## Paired State flag inspection and preservation — 2026-10-30

While the unchanged R7 Area/VRA gate runs, a separate actual Chrome check uses
only its staged standard workbench assets. Seven inspections cover baseline,
DPI, print-only, target majority-minority districts, objtype, their combined
values and invalid DPI. Six valid cases retain identical effective algorithms;
all seven match native-tool inspection replies and retain original arguments
through inert Save/Open. The comparison permits only native/WASM pointer width
and the separately labelled, read-only browser identity preview. It does not
remove parser or effective-configuration differences.

The check records Chrome 155.0.8059.85 and its executable digest, the copied native
tool and current WASM identities, raw replies, exact served assets and downloaded
projects. Its separate actual-CLI witness matches the current CLI executable and
retains all seven native cases and output files. The first browser attempt used
the final toolkit URL before that alias existed in staging; the second exposed
the browser-only preview augmentation. Both attempts are preserved alongside the
corrected, passing experiment.

`target/parity-full-build-20261030-r7/state-flag-browser/snapshot-scope/snapshot.json`
has SHA-256
`751fe1690633caa9b1cf237b6ab3a68e249ea2ffc8fbb5fcaf97caee2744d7d1`.
The standard reader verifies 352 mappings, 311 unique blobs and 90,724,654 logical
bytes, including inherited digest-verified staged asset blobs. This is bounded
inspection/preservation evidence. Browser partition Run, print-only adaptation,
C execution, whole-builder/final-package acceptance and complete field contracts
remain separate. No captured R7 inputs or canonical ledger states were changed.

## Paired State flag execution and refusal — 2026-10-30

The staged standard workbench now also executes five selected-adjacency State
Runs: baseline, DPI 300, target-MM 1, objtype volume and their combined non-print
form. Every ordered assignment matches the corresponding current actual CLI
output exactly on the four-unit RI 2020 N-way/metis-core/seed-42 fixture. The
combined native witness additionally contains print-only, which is unused by
that CLI; this is disclosed rather than treated as a byte-identical command.
Original command recipes, source bytes and separately selected DPI intent remain
in the saved results. Reopening the completed files performs no work and retires
their downloads until a fresh explicit execution.

Two print-only variants queue without execution, then fail on explicit Run with
the expected `--print-only` workflow refusal. This is an explicit browser policy
difference from the native CLI, which still produces assignments. Independently
executing those same queued jobs with the current actual WASM facade reaches
only State inspection calls before the refusal: no source preparation or
partition operation occurs. The test does not infer native print-only behavior
from the browser help text.

All five downloaded ZIPs and their nested ZIPs pass an independent Python reader:
100 entries have valid CRCs, exact declared byte counts, unique safe paths and
bounded nesting. This is archive integrity, not whole native ZIP byte equality.
The immutable snapshot at
`target/parity-full-build-20261030-r7/state-flag-run/snapshot-scope/snapshot.json`
has SHA-256
`bb7a25fab895fbd180161fef6b2f4808c80a01ce37727c2ef3e84f30d382f7ed`.
Its reader verifies 391 mappings, 350 unique blobs and 93,369,930 logical bytes,
including the prior native/inspection witness through verified inheritance.
The original test attempts are preserved: one expected string seed 42 rather
than its valid numeric representation, and one expected refusal during queueing
rather than explicit Run. No engine/UI change was needed for either correction.

This is execution/refusal evidence for the stated small fixture, not complete
field, method, backend, host, filesystem permission or release acceptance. Frozen
R7 code/catalog/fixture inputs and canonical contract states remain unchanged.

## Fresh Area/VRA regression completed — 2026-10-30

The unchanged R7 standard gate completes 288 exact synthetic Area/VRA cases,
3,168 evidence refusals and 98 real-state project/export roundtrips, with two
matching real-state sampled-search failures. Its corrected feasibility report
records zero synthetic population misses, one real-state miss and zero
disconnected plans. Fresh GeoSection subsequently passes 192 exact comparisons,
2,496 evidence refusals and six projects, with no reported population miss or
disconnected plan.

The independently predeclared 100-outcome Area/VRA regression audit now passes.
All 98 completed projects preserve expected original configuration, graph/order,
exact assignments and selected metrics; independently derived district populations,
unit counts and weak-undirected components agree. CA VRA retains its population
tolerance miss, and the two separately captured raw negative controls retain
their exact sampled-search errors. The successful baseline uses retained old
project/previous parity assertions, not invented old native success transcripts.
This is standard metis-core/Node-WASM regression evidence, not actual-browser,
explicit-C, optimality or full acceptance.

`target/parity-full-build-20261030-r7/ratio-regression-audit/audit.json` has SHA-256
`f50fe6975536346825ecbedca5ad31bba6b6cd4a9514bfd2e4665ec33ac766e9`.
The 98 fresh projects are retained under content-addressed filenames and all
their hashes were read back. `phase-receipts/ratio-geosection-checkpoint.json`
preserves the two fresh phase summaries and audit identity. The standard builder
continues; these phase results do not promote the whole run or canonical ledger.

Fresh recursive (96 exact/six projects), percentile (48 exact/three projects and
32 independent candidate replays) and tuned ensemble (256 exact/three projects,
six matching refusals) then pass. Together with fresh GeoSection, that is 592
exact comparisons and 18 real-state projects with zero reported population misses
or disconnected plans. Their current summaries are copied and hash-verified in
`phase-receipts/remaining-tuning-checkpoint.json`. Convergence candidate replay,
distinct limit/threshold stopping, no-feasible refusal and secure Save/Open also
pass. These are fresh standard gate results, distinct from the earlier focused
retained-engine diagnostics and still short of complete standard/C acceptance.

Free disk space subsequently fell below 1 GB. The guarded NTFS compression
continuation compressed 1,312 inactive regular single-link compiler `.rlib` and
`.rmeta` files and rehashed all 4,325,091,333 logical bytes unchanged. Three
protected runtime identities also agree. No source, evidence, executable or WASM
file was compressed or removed. Its receipt is
`target/parity-full-build-20261030-r7/cache-compression-reserve2/evidence.json`.
Observed free space increased from 665,579,520 to 4,634,992,640 bytes during this
operation; that observation does not attribute concurrent system disk changes to
compression. The earlier zero-file compression receipt remains unchanged.

## Tutorial dispatch harness correction — 2026-10-30

R7 stops after the laboratory algorithm gates, inside the nested toolkit build's
`test_tutorial_wasm.mjs`. Its isolated dispatch test strips imports and evaluates
the worker as a plain VM script; the worker now contains a legitimate module-only
`import.meta.url` for the optional C sidecar. The resulting syntax error is in
the test harness. R7's final check confirms all 3,232 captured source/fixture/
catalog files unchanged. The original failed status, stack/logs and refused
package continuation are preserved; the C build was not reached.

The dispatch harness now injects the actual worker module URL alongside its
already-injected dependencies. Its static wiring checks require a versioned
module worker and versioned engine import, replacing two obsolete exact cache
tags. No production worker or engine behavior changes. The corrected test passes
13 native/WASM tutorial reports, seven actual CLI checks, eight streamed-hash
cases and seven matching refusals. R8 preserves the previous test and temporary
tutorial outputs, plus the focused passing log and report.

Before another complete all-state attempt, R8 runs a separate diagnostic of the
unchanged toolkit gates after that boundary, with BrowserChecks enabled and a
new output directory. It captures the same 3,232 source paths; only the tutorial
test differs from R7. The diagnostic's source builder and generated continuation
are pinned in `target/parity-full-build-20261030-r8/toolkit-tail/recipe.json`.
It is not a passing whole build, C check, package candidate or deployment.

R8's `continue_after_tail.ps1` is separately running under an owned PID. It waits
for the diagnostic process and exact passing receipt, rechecks the diagnostic's
captured sources, captures/verifies the new complete build inputs, and only then
launches unique all-state standard/C outputs. Its separate local packager waits
for both full builds to pass. Failure at the diagnostic/source boundary prevents
that launch. The full build is queued, not currently accepted; no publication or
deployment is scheduled. The focused tutorial correction receipt also verifies
that its WASM bytes are identical to R7, with native references and before/after
test identities retained in `tutorial-correction-evidence.json`.

## Configured-export chamber expectation correction — 2026-10-30

The first R8 toolkit diagnostic passes the native TUI, civic, suite, weighting
and configuration checks through configured UI/prepared-graph scope, then stops
at a stale configured-export rejection of chamber `oops`. The shared current
configuration/export implementation accepts custom chamber names of 1–200 UTF-8
bytes without controls. Its final source check retains all 3,232 captured inputs;
the failed diagnostic and blocked full-build continuation remain failed.

A separate eight-case native-tool/WASM probe retains individual requests and raw
stdout/stderr: ordinary custom names, Unicode and 200 ASCII bytes succeed; empty,
control, 201 ASCII bytes and 202 Unicode bytes reject on both runtimes. The
maintained export test now requires successful custom exports with exact chamber
metadata and unchanged assignment arrays, plus the actual empty/control/byte
limits. Its full suite passes 27 exact native/WASM results and ten matching
refusals, including five custom exports and the inclusive 200-byte Unicode case.
This is shared native-tool comparison, not independent CLI chamber orchestration.
The full suite retains aggregate assertions; the separate probe retains per-case
raw replies. Original and corrected test/output files and identities are kept in
`target/parity-full-build-20261030-r8/config-export-correction/evidence.json`.
No production code changes; current WASM bytes still match R7 exactly.

The new `toolkit-tail-2` diagnostic starts at the registered configured-build gate
after this corrected test, with BrowserChecks and original optional defaults.
Its capture differs from R7 in exactly the tutorial and configured-export tests.
`continue_after_tail2.ps1` waits for its matching passing terminal receipt and
stable sources before capturing/launching the complete all-state standard/C build.
The original diagnostic, continuation and logs are preserved. This second
diagnostic is running, not complete build or package acceptance.

## TUI mixed-progress test correction — 2026-10-30

R8's second diagnostic stops at the TUI Run test's exact mapping of all progress
events to workflow stages. Native round-monitor messages now interleave with
the wrapper's build/export/complete messages and have no `stage` field. Its final
source check passes; that diagnostic and its refused full-build continuation stay
failed and are retained.

The corrected test asserts the same exact workflow stage order and first/last
boundaries, validates every other event with the existing strict native round
progress validator, and requires those events to occur only inside engine build.
It also requires actual native event coverage, so filtering cannot silently omit
the new stream. Four successful native form/YAML/accepted-attempt assignment,
export and report-manifest cases pass across the existing structures and exact
seeds, with 16 validated monitor events. Existing geometry, source/hash,
scope/schema, ZIP and inert-project checks remain. No production behavior changes.

The old test and outputs, new outputs, raw progress arrays and focused passing
log are retained under
`target/parity-full-build-20261030-r8/tui-progress-correction`, with identity pins
in `evidence.json`. This is focused native/Node-WASM evidence, not actual browser
or whole-build acceptance. The third captured downstream diagnostic starts at
the accepted-plan workflow gate with BrowserChecks enabled. Its capture differs
from R7 in exactly the tutorial, configured-export and TUI Run tests. The owned
`continue_after_tail3.ps1` waits for its matching passing receipt and stable
sources before the complete all-state standard/C launch; previous failures are
not overwritten or promoted.

## Remaining workflow analysis record comparisons — 2026-10-30

The third diagnostic stops in the optional accepted-plan workflow test at the
same manifest-reuse metadata distinction corrected earlier in the separate
accepted-workflow gate. Whole native-host/WASM replies already agree. Standalone
analysis generates the manifest; the containing workflow supplies the identical
manifest and records reuse. The failed diagnostic, source-stability receipt and
refused full-build continuation are retained.

The two remaining workflow tests now share a test-only comparator. It requires
all constituent file bytes to match, except for the explicitly derived
`analysis-package.json` manifest step. It verifies the standalone manifest's
exact expected key set, computed status, evaluated population check and original
plan/context source hashes; every other metadata field and containing analysis
stage must match exactly. Unknown metadata is not stripped. Four mutations prove
that changed source hashes, unrelated election scope, unexpected manifest fields
and changed summary bytes still reject.

The synthetic suite passes ten whole native-host/WASM receipts, exact accepted
Run, analysis/four maps/report outputs, hash/CSV/seed/project/ZIP guards and
partial/failed-stage preservation. The retained original 792-unit RI block-group
test also passes its assertions. Its result remains partial: the native map
validator refuses a self-intersecting/touching ring and required analysis/report
inputs remain incomplete. This is not a completed map/report claim or newly
established independent CLI capacity result.

Before/after tests and outputs, raw workflow receipts and focused logs are kept
under `target/parity-full-build-20261030-r8/workflow-analysis-correction`, with
identities in `evidence.json`. The shared helper is
`scripts/web/fixtures/assert-workflow-analysis.mjs`. No engine or UI source changes.
The fourth diagnostic now captures 3,233 inputs (five changed tests plus this
added test helper relative to R7) and resumes the remaining registered gates with
BrowserChecks. Its owned continuation requires matching passing receipt and
unchanged sources before the complete all-state standard/C launch. Previous
diagnostics and their failed continuations remain unchanged.

## Label checkpoint and native progress streams — 2026-10-30

The fourth diagnostic passes browser proof/context, precinct/imported and four
prepared Census unit-kind workflows, then stops at the label progress test's
count of 14 total notifications versus five expected checkpoints. The stream
contains the same five ordered label checkpoints plus nine native observations.
The current UI already routes native round progress before checkpoint retention;
its production code is unchanged. The diagnostic's final check confirms all
3,233 captured inputs unchanged, and its failed status/continuation are retained.

The checkpoint tests now use a shared test-only discriminator that recognizes
label checkpoints and strictly validates native round messages; unknown schemas
reject. Five checkpoint sequence numbers, common request identity, exact state/
year deltas, callback failure recovery, reentrancy/cancellation, mutation isolation
and mixed worker relay pass. The two isolated worker harnesses also bind their
module URLs rather than evaluating unbound `import.meta.url` as plain script.
Checkpoint files pass eight malformed retention cases, exact final/cancelled
project and ZIP checks. Archive/source-start checks pass eleven archive refusals
and four pre-engine source refusals respectively.

The related captured-batch test passes complete/no-event metadata, exact native/
WASM two-year output, accepted-attempt/final-plan binding, capture-on/off equality,
legacy/invalid-flag guards and independent ZIP CRC/path/hash readback. Mixed RI/IA
VRA source-policy execution also passes its whole native-tool comparison and
per-state provenance/identity guards. The national callback has the same stream
correction, while its fresh all-50 run remains pending; neither that run nor the
optional CA/TX/RI capture subset is claimed by these focused checks.

Before/after test/output files, raw fourteen-event trace and focused logs are
preserved in `target/parity-full-build-20261030-r8/label-progress-correction`,
with scope/identity pins in `evidence.json`. The new helper is
`scripts/web/fixtures/label-build-progress.mjs`. The fifth diagnostic resumes
registered gates after those focused checkpoint/archive/source-start checks.
Its capture has 3,234 inputs: eleven changed tests and two added test helpers
relative to R7, with no engine/UI changes or canonical evidence promotion.
`continue_after_tail5.ps1` retains the matching-pass/source-stability boundary
before any complete standard/C launch. Earlier diagnostic failures remain failed.

## Fifth diagnostic analysis/report checkpoint — 2026-10-30

The current continuation passes six combined native analysis steps, individual
label analysis (nine full native/WASM cases), reports (five cases), registry
updates and multi-year analysis/report workflows (eight/nine cases respectively).
Independent ZIP checks cover exact native file sets, raw index chains, CRCs,
digests and retained failed/cancelled-year receipts. These are the declared small
synthetic year/state fixtures, not new original national or host capacity proofs.

Five freshly written analysis/report summary receipts are copied, JSON-parsed
and hash-verified under
`target/parity-full-build-20261030-r8/toolkit-tail-5/analysis-phase-receipts`.
Its `checkpoint.json` binds the current 3,234-input capture and WASM identity,
and explicitly records that summaries are not full per-case transcripts or
whole-build acceptance. Native checkpoint verification/resume regressions are
compiling. No new source changes or canonical evidence promotion follow; the
continuation still requires the diagnostic's final pass and source stability.

Checkpoint verification/resume subsequently passes four complete native/WASM
results, verified one/two-year prefixes, 16 tamper/source/settings refusals and
exact complete-run equality. Its independent ZIP reader checks 122 exact files,
CRCs, digests and original year-index chains. Actual plan-hub, label import,
imported-plan and multi-year analysis/report browsers also pass, including exact
downloads and inert Open/handoffs. RPLAN package verification passes 32 native/
WASM results, 21 actual CLI cases and six boundary refusals; manifest certificate
checks pass 37 native/WASM results, five CLI cases and four boundary refusals.
Their declared fixture scopes and failed/unrecorded-audit distinctions remain.

Read-only owned-process observers now record future diagnostic/full-build Chrome
executable identities, version metadata, creation/ancestry and associated test
script hashes. The diagnostic observer uses process polling because WMI start
events are unavailable. No URLs, command arguments, credentials, environment or
profile contents are retained; polling may miss short-lived processes. These
observations are neither retroactive browser witnesses nor proof of the exact
loaded browser engine or action acceptance. The source/copy identities and scope
are recorded in `toolkit-tail-5/runtime-observer-recipe.json`; a separate guarded
waiter starts the full-build observer only after an actual matching build launch.
No source inputs, final build status or canonical acceptance states change.

## Native loopback HTTP fixture mode correction — 2026-10-30

The fifth diagnostic stops in the retry/timeout comparison: browser retry succeeds
while the native local reference intermittently reports a Windows connection
abort. All 3,234 captured inputs match at failure; that run, observer and refused
full-build continuation retain their terminal receipts. Its observer recorded
zero future browsers before termination, so no runtime identity is inferred.

Instrumented baseline replay reproduces native client error 10053
(`ConnectionAborted`) alongside the same fixture handler's error 10035
(`WouldBlock`). The nonblocking accept loop's socket was used by blocking
`read_exact`/`write_all`. The selected fix explicitly sets the accepted socket
blocking before I/O. Handler join results and the native error chain now remain
visible in the reference output. The instrumented baseline and selected candidate
differ only by that socket-mode statement and its comment; the client/library,
retry counts and timeout policies are unchanged.

Thirty candidate replays cover 60 complete HTTP handlers with exact scripted
attempt counts and no handler errors. All original nine native/browser HTTP,
short-body, timeout and retry cases pass. The maintained driver now requires
successful ordinary handlers, allowing the intentional late disconnect only for
the scripted delay case, with unchanged attempt/outcome assertions. Secret-free
errors, no-retry guards, cancellation and hung-transport deadline also pass.

Original failures, binaries/sources, raw baseline errors, replay records, build
logs and focused passing outputs are retained under
`target/parity-full-build-20261030-r8/fetch-network-investigation`, with selected
identities in `evidence.json`. A broader lifecycle candidate was investigated and
rejected; its artifacts stay separate. The final code keeps the original server
structure with blocking handlers and diagnostics. No other nonblocking Rust
loopback accept fixture was found in `crates`/`scripts`.

The sixth diagnostic resumes after that gate with BrowserChecks and the same
3,234 captured paths: twelve changed tests, one native test-fixture correction
and two added test helpers relative to R7. Engine/UI/download-library behavior
is unchanged. New owned observers retain the prior read-only scope; old failed
observers/continuations are preserved. A passing diagnostic and source check
still gate the complete all-state standard/C launch.

The sixth continuation then passes raw staging (nine actual native HTTP downloads),
retry-history retention, eleven independent cache-ZIP checks, Doctor/Compare/
session/library browsers, state completion/batch and binary/auxiliary/adjacency
source handoffs with seeded Run and inert saved Open. Actual cache restore/backup
and pruning UI/worker checks pass atomic merge, conflict/force, corrupt/cancelled
preservation and orphan-only removal without changing referenced objects. The
current process observer records owned Chrome executable/test-source witnesses;
the observed executable product version is 155.0.8059.85. This remains executable
metadata, not proof of exact loaded-browser-engine identity, every action or
broader host capacity. The diagnostic and complete build acceptance remain open.

## Census source browser and archive capacity reconciliation — 2026-10-30

The sixth diagnostic stops at the Census browser's outdated hidden-ZIP-controls
assertion. Current source UI intentionally keeps ZIP alternatives visible and
collapsed beside selected TIGER/PL94 files. The corrected test checks visibility,
collapse and no engine dispatch on menu toggles, preserving the original exact
source/preparation, wrong-state/freshness, handoff/seed, Run/map/analysis/report
and native YAML assignment comparisons. Four 2020 county/tract/block-group/block
fixtures pass, with one/two/two/two units respectively. This is not statewide or
final package/C-profile acceptance.

The maintained browser test now records actual Chrome 155.0.8059.85, all 247
served asset hashes, source/test/WASM/native-oracle identities, worker operations
and four raw native oracle replies. The immutable snapshot at
`target/parity-full-build-20261030-r8/census-browser-correction/snapshot-scope/snapshot.json`
has SHA-256
`4f54621fc3c2cb72b73d1365ed4826d225c771a6c36931f067b2fd8f8de97d51`.
Its standard reader verifies 291 mappings/unique blobs and 35,137,018 logical
bytes using verified inherited storage. No browser profile contents are copied.
Original failed test/artifacts and current evidence are preserved alongside it.

The seventh diagnostic stops at the prepared-capacity test's old four-million-
value archive rejection. Workbench archives already use eight million values;
ordinary JSON and prepared-graph readers retain their separate smaller limits.
The corrected test asserts that existing public limit, accepts 7,999,999 scalar
values plus the containing array (eight million total), rejects the next scalar,
and enforces the existing 128-MiB archive byte cap. Existing generic/prepared
limits, integer/prototype/duplicate/escaped-key/depth/UTF-8 guards, dense archive
roundtrip and county-only ZIP allowance still pass. The 256-MiB expanded profile
constant is checked; this test does not claim a new browser memory/host trial.
Before/after test and focused log/report are retained under
`target/parity-full-build-20261030-r8/prepared-capacity-correction`.

Both failed diagnostics and source-stability receipts remain failed/preserved.
The eighth continuation resumes registered remaining gates with original optional
defaults and BrowserChecks. Its 3,234-path capture differs from R7 in fourteen
tests and one native HTTP fixture, plus two added test helpers. Engine/UI/import
policy code is unchanged. The complete all-state standard/C launch remains gated
on its passing receipt and source verification; no canonical evidence promotion.

## Composed verification worker harness — 2026-10-30

The eighth diagnostic passes auxiliary replay bindings (32 native/WASM cases),
original replay bindings (18) and manifest/replay composition (11), then stops
when the composed binary advisory test evaluates `import.meta.url` in its plain
VM worker harness. Its 3,234-input source check passes; original failure, observer
and blocked continuation receipts remain retained.

The final worker harness now binds the actual worker module URL, matching the
already-passing runtime-artifact test pattern. Nine whole native/WASM comparisons
and six refusal cases pass, including literal/sentinel/skip behavior, worker-owned
loaded fingerprint overriding an incoming claim, and manifest prechecks stopping
before binary advisory/replay. No worker or engine code changes. Before/after
test/output files, focused report/log and identities are kept in
`target/parity-full-build-20261030-r8/verify-worker-correction`.

A scan confirms every plain-VM harness evaluating the shared WASM worker now
binds its module URL. The ninth captured continuation resumes TUI Verify and
subsequent registered toolkit/browser gates with the original flags. Its capture
differs from R7 in fifteen tests and one native HTTP fixture, plus two added test
helpers (3,234 total inputs). The guarded complete standard/C launch remains
pending diagnostic/source acceptance; no whole-build or canonical promotion.

## Prepared state geometry alias probe correction — 2026-10-30

The ninth diagnostic passes captured-batch, antimeridian, national-stream and
native map browsers, then stops at a rejection fixture that adds uppercase
`GEOID` without removing the original lowercase alias. The strict native/WASM
decoder rejects that ambiguity before reaching the intended topology failure.
Its final 3,234-input source check passes; failure/continuation receipts stay
preserved and no complete build starts.

The corrected test leaves hash-verified original geometry properties intact and
checks two deliberate duplicate-alias refusals separately. RI's 250-unit display
geometry retains its self-intersection refusal; IA's 896-unit display geometry
retains its cross-unit overlap refusal, with matching native reason classes and
no WASM trap/native panic. The same WASM instance then accepts explicitly synthetic
separated squares at both original unit counts. Whole native tool results and
independent native compactness report bytes match; these squares are not real
state boundaries or proposed plans, and local timings are diagnostic only.

Per-state requests, native error text, positive WASM/native replies, independent
reports, original asset copies/hashes and native executables are retained in
`target/parity-full-build-20261030-r8/state-geometry-correction`, with scope and
identities in `evidence.json`. Production geometry validation is unchanged.
The tenth registered continuation resumes after this probe with original flags
and BrowserChecks. Its 3,234-path capture differs from R7 in sixteen tests and one
native HTTP fixture, plus two added test helpers. Complete standard/C launch and
canonical acceptance remain gated on diagnostic/source success.

## Full-source batch default and large profiles — 2026-10-30

The tenth continuation passes the existing full-source RI balanced/spatial and
IA bounded-rejection/recovery gates, then stops at the old batch test's expected
success for ordinary-profile IA. Instrumented probes show RI250/83,797 positions
succeeds, while IA896/454,150 positions reaches the same default 50-million work
refusal in WASM, the native tool and independent native compactness executable.
All captured sources match at failure; no whole build starts from that receipt.

The existing explicit `source-exact-large` root profile permits 200 million work.
A separate IA probe succeeds on all three references with that profile. The
maintained test now requires the original IA default refusal and absence of an
independent output report, confirms the same WASM instance recovers with the
earlier RI request, then runs IA with the explicitly recorded large profile.
Removing the inserted root marker restores the original geometry byte-for-byte:
no source coordinates or other metadata are simplified or rewritten.

RI default and IA opt-in both match whole native results and independent native
report bytes. The original independent shoelace/compensated-sum area oracle passes
all six district checks at its unchanged tolerance. The saved IA project retains
the actual large-profile geometry; original/executed geometry hashes remain
separate in the evidence. This verifies the existing profile boundary, not a
raised policy limit, default IA support or broader host/backend capacity.

Before/after source and outputs, both baseline/large probes, raw native errors,
native executables, saved projects and source/area checks are preserved in
`target/parity-full-build-20261030-r8/full-batch-investigation`, with scope and
identities in `evidence.json`. The eleventh diagnostic resumes the remaining
registered toolkit/browser gates. Its 3,234-path capture differs from R7 in
seventeen tests and one native HTTP fixture, plus two test helpers. Production
engine/UI/policies remain unchanged; diagnostic/source success still gates the
complete all-state standard/C build and no canonical evidence state is promoted.

### Plan-map geometry capacity diagnostic — 2026-10-31

The eleventh diagnostic stopped at a stale plan-map test that expected an
8 MiB geometry limit. The existing parser uses 16 MiB by default and permits
64 MiB only with an explicit root geometry profile. Production limits are
unchanged. The corrected focused test accepts the inclusive 16 MiB ASCII
boundary and rejects oversized characters and UTF-8 bytes. Native/WASM map
validation, ten geometry refusals, three prepared-state maps and four malformed
plan refusals pass. The existing large-profile test also passes its root opt-in,
duplicate/nested marker, 64 MiB bound and compactness handoff checks.

Before/after test source and focused logs are preserved under
`target/parity-full-build-20261030-r8/plan-map-capacity-correction`, with a scoped
`evidence.json`. The twelfth diagnostic resumes the registered toolkit/browser
gates after this test, with a new capture of 3,234 inputs. Neither this focused
success nor source stability establishes complete standard/C, packaged browser
or canonical parity acceptance. The guarded full build remains conditional on
all remaining diagnostic gates passing; no deployment is performed.

### Saved batch browser replay status — 2026-10-31

The twelfth diagnostic passed actual browser source acquisition, custom source
and House/Senate workflows, native Run/States queues, OPFS output installation
and independent output readback, then stopped at an outdated status-string
assertion in the saved batch recovery test. The UI already says that saved
batch replay finished; the test expected the older verified wording. The test
now checks the exact current message, completed-state count and continuation
instruction. Production UI and execution behavior are unchanged.

The complete focused browser test passes: native-equal parallel/serial runs,
exact seeds, cancellation, explicit Resume without reexecuting completed states,
failure retry, inert reopening, fresh saved-result replay, engine fingerprint
refusal and raw Census recovery. Independent ZIP readback passes CRCs, unique
members and 22 exact state files/nested native archives. Actual replay status
and browser version are recorded. Before/after tests, failed probes, focused
logs and before/after outputs (excluding browser profiles) are retained under
`target/parity-full-build-20261030-r8/batch-replay-status-correction`, with scoped
identities in `evidence.json`.

The twelfth diagnostic ended with all 3,234 captured inputs unchanged. Its
read-only observer recorded 16 owned browser processes; this is executable
identity evidence, not complete browser-action acceptance. The thirteenth
diagnostic resumes remaining registered gates from the recovery ZIP check.
Its capture changes nineteen tests and one native HTTP fixture relative to R7,
plus the two test helpers. Complete all-state standard/C builds remain guarded
on a successful diagnostic and stable sources. No canonical parity record is
promoted and no deployment is performed.

### COI browser receipt fingerprint — 2026-10-31

Diagnostic 13 passed recovery ZIP readback, native target browser checks,
electoral controls and multiscale seed display, then stopped because the COI
browser test compared its worker receipt to a direct-ABI fixture with no loaded
WASM identity. The worker correctly supplies the measured artifact fingerprint.
The corrected test independently hashes the served WASM bytes, supplies that
identity to four fresh native reference requests, retains raw requests/replies,
and requires complete native/browser equality. It additionally requires that
the native reply differs from the original fixture only in the supplied loaded
WASM identity. No receipt field is dropped from comparison.

All four actual browser cases pass: engine and adaptive block-group/tract
replay, their changed-COI pre-execution refusals, inert Open, download freshness
and exact selected-source retention. Raw receipts, actual browser version,
before/after tests and outputs (excluding profiles) are preserved in
`target/parity-full-build-20261030-r8/coi-browser-fingerprint-correction`, with
scoped hashes in `evidence.json`. Production behavior is unchanged.

Diagnostic 13 ended with its 3,234 captured inputs unchanged. Diagnostic 14
resumes the remaining registered gates after the COI browser check under a new
capture: twenty changed tests and one native HTTP fixture relative to R7, plus
two test helpers. Complete standard/C builds and formal acceptance remain
pending; no canonical state is promoted and no deployment is performed.

### Standard toolkit proof dependency staging — 2026-10-31

Diagnostic 14 passed history verification after VRA audit, plan export, election
imports and district aggregation, then found a real packaging omission. The
standard toolkit copied proof-related JavaScript but omitted the referenced
solver/checker artifacts. The registered standard builder now invokes the
existing RoundingSat, VeriPB and native-proof staging recipes before checking
asset closure. Those recipes verify artifact identities and vendor/source
provenance and include license files. No engine, UI, proof kernel, solver or
runtime limit changes are made. The build requires the separately built local
pinned solver/checker prerequisites; missing or mismatched inputs stop it.
README now documents these prerequisites.

Before/after builder source, stage logs, preserved focused package and receipts
are under `target/parity-full-build-20261030-r8/proof-package-correction`, with
scoped identities in `evidence.json`. Focused dependency closure passes 312
root files and 1,255 local module/worker/HTML references. This does not establish
packaged proof execution or statewide proof capacity.

Diagnostic 15 independently reran the registered final packaging section and
passed, including source stability for all 3,234 captured inputs. The capture
contains twenty changed tests, one native HTTP fixture and one standard build
recipe relative to R7, plus two test helpers. Its guarded continuation may now
capture and start the complete original all-50-state standard/C pipeline.
Only that complete pipeline can establish whole-build success; formal parity
acceptance and deployment remain separate and are not claimed.

### Isolated proof prerequisite hardening and packaged browser evidence — 2026-10-31

While the captured full R8 standard build remains active, the packaging review
follow-ups were developed under
`target/parity-full-build-20261030-r8/proof-hardening-candidate`. Captured source
and live engine/native artifacts were not edited. The read-only candidate
preflight accepts the actual checker/solver prerequisites and rejects nineteen
missing, altered or stale copies, including all eight recorded checker source
fields, schemas/revisions, artifact bytes, solver glue, vendor provenance and
an unexpected solver inventory member. Five original prerequisite files retain
exact before/after hashes. A six-file integration patch is prepared and passes
`git apply --check --ignore-space-change`; it is not applied. It adds early
standard/laboratory/C preflight, a refusal regression and current-source
correspondence enforcement in checker staging. Integration remains pending a
terminal captured run and its own final validation.

A frozen copy of the completed diagnostic-15 toolkit package contains 326 files.
The first actual browser run correctly exposed old comparison fixtures naming
a different engine hash. That failure and the old frozen fixtures are retained.
Fresh references were then generated in isolation from the exact frozen package,
not by replacing old fixture hashes. All thirteen decisions across the two
complete split fixtures and the canonical SAT counterexample match independently
compiled native model/request bytes and pass the separate upstream VeriPB CLI.
An actual 100-unit timeout retains only a checked prefix and refuses complete
export. Two saved-result rechecks and two complete native/WASM external split
certificate comparisons provide fresh browser recovery references.

Chrome 155.0.8059.85 now passes the full native-proof browser test against these
frozen package routes: exact whole receipts and ZIPs, inert saved-session Open,
fresh replay, rehashed forged-claim refusal, checked SAT preventing complete
export, actual timeout, visible discovery seeds, input freshness, cancellation
and a fresh run after cancellation. All 180 served asset hashes match the frozen
package manifest, and all 326 frozen files remain unchanged. Raw requests,
replies, reference executable copies, current fixture outputs, snapshots, failed
and passing browser logs and scoped `evidence.json` are preserved. Profiles are
excluded from evidence mappings. The separate VeriPB executable and checkout
metadata are retained; this does not claim a fresh upstream compilation or
build attestation.

This is successful packaged-browser evidence for the completed local diagnostic
package. It does not establish final complete standard/C package execution,
statewide proof capacity, recursive tree adoption, legal validity, reproducible
compilation or formal parity acceptance. The shared preflight/source enforcement
still needs integration; dynamic dependency inventory and final profile-specific
proof browser checks remain open. No capability state is promoted or deployment
performed.

### Proof dependency inventory and standalone package pages — 2026-10-31

The isolated hardening candidate now declares four proof HTML entry points,
dynamic runtime artifacts and distribution/license files. Its validator walks
all four entry-point module graphs and checks solver/checker/package/native-proof
receipt bindings: 221 exact files, including dynamic worker assets omitted by a
literal-only scan. Seven physically damaged or undeclared package variants are
refused: missing solver WASM, checker receipt and license; corrupt checker WASM;
wrong native engine binding; an auxiliary HTML script omission; and a newly
observed runtime file outside the declared closure. The frozen baseline remains
unchanged after mutation tests. This is a build-time dependency contract, not
runtime or global proof correctness by itself.

Two additional actual Chrome 155.0.8059.85 tests run exclusively from the frozen
diagnostic package: standalone RoundingSat delivers four byte-exact proof
files that the separate retained VeriPB CLI accepts; standalone VeriPB produces
three receipts matching fresh native checker replies. Both exercise inert file
selection, setting/export freshness, cancellation, a fresh run after cancellation
and restrictive script CSP. Checker coverage also includes strict UNSAT policy
and truncated-proof refusal. Exact input/proof bytes and raw native checker
requests/replies are retained. Across these two pages and the earlier native
proof page, all 186 distinct served asset identities match the four-entry-point
dependency closure. Native-tree entry coverage remains static-only in this
follow-up; no tree browser execution is inferred.

`proof-hardening-candidate/integration-v2.patch` preserves a nine-file integration
proposal alongside its complete overlay. It adds the explicit inventory,
validator and mutation regression to the earlier preflight/source enforcement
proposal and registers standard/C dependency checks. The original six-file
patch remains unchanged. The v2 patch passes `git apply --check
--ignore-space-change`; PowerShell overlay files parse, and the dependency
regression passes when run from the generated integration overlay. The patch
is unapplied so the active full build keeps its exact captured inputs. Fresh
integration, final standard/C package browser runs and formal acceptance remain
pending. `evidence-v2.json` retains the original evidence hash plus the new
sources, logs, snapshot identities and raw standalone references. No browser
profiles are mapped into that evidence, capability states promoted or deployment
performed.

### Complete frozen-package tree browser and guarded final-profile queue — 2026-10-31

The remaining native-tree page now runs from the same frozen completed
R8 diagnostic package, using fresh isolated reference generation rather than
historical engine identities. Five reference checks pass: four native/WASM
2/3/4/5-seat trees with forged lineage/graph/schedule/leaf/missing-node refusals;
three whole native plan/audit packages with exact ZIPs and source/profile
refusals; integer and real RI model preparation with no silent rounding;
three whole native map renders and geometry refusals; and two/four-seat
proof-tree generation with a three-seat checked counterexample stop.

The complete original browser assertions pass in Chrome 155.0.8059.85 against
those frozen routes: exact tree/plan/map/model ZIP downloads, schedule/leaf
presentation, inert Open/reload, explicit fresh reverification, forged-parent
refusal, closed-key/UTF-8 rejection, automatic discovery/proof generation,
setting freshness, stopped-split delivery, inert opening in the individual
proof page and cancellation/recovery. The real retained 250-unit RI context
uses its actual 30-second solver setting and stops at the enforced proof
output bound: 67,106,426 proof bytes, below the 64 MiB ceiling. The resulting
partial evidence remains unverified and no complete tree certificate/bundle is
available. This is verified refusal behavior, not successful statewide proof
generation.

All four proof pages now have actual browser coverage for this diagnostic
package. Their combined 194 served asset identities match the candidate's
221-file dependency closure. All 326 frozen package files remain unchanged.
`proof-hardening-candidate/evidence-v3.json` links the previous evidence hash,
all five raw reference logs and outputs, browser source/recipe, exact downloaded
artifacts and combined asset readback. Its 88 tree-scope file mappings exclude
browser profiles. The v2 integration patch remains unapplied and no formal
capability status changes.

A separate guarded continuation is queued at
`target/parity-full-build-20261030-r8/final-proof-profiles`. Its live guard waits
on the actual full-build PID/script identity, requires the matching terminal
successful complete standard/C receipt, checks available headroom, then runs
source stability before freezing either exact final toolkit. The recipe pins
33 templates/input files. Each profile receives its own frozen assets, retained
native executables, fresh proof/recovery/tree references, four actual browser
page tests, combined dependency identity check and final unchanged-asset readback.
The runner checks captured source stability again before success. Syntax and
recipe paths are checked; queued final-profile execution itself is not yet run.
A failed/incomplete full build prevents these checks from starting. This queue
does not apply source patches, deploy or establish formal complete desktop
parity, and does not replace explicit C partition-backend acceptance.

### Installed-layout hardening execution and character-phase readback — 2026-10-31

The unapplied nine-file hardening integration now has actual entry-point
execution in a disposable repository replica, beyond syntax/apply checks. The
replica contains exact prerequisite/source copies and a clean shared local
clone of the pinned solver checkout. The installed-layout preflight regression
passes all nineteen refusal cases. Valid checker staging produces ten files
whose bytes/lengths match the staged receipt. Eight stale source-field claims,
an altered adapter source and altered checker WASM all stop before any package
writes. The standalone, laboratory and C builders each reject a stale source
at their shared initial preflight, with no output directory created. Ten actual
repository source/artifact hashes remain unchanged. Logs, raw executed overlays,
case records and source identities are linked by
`proof-hardening-candidate/evidence-v4.json`. No integration patch has been
applied to captured inputs.

The full R8 build has completed its character phase and continued into native
State/States and actual browser workspace/recovery checks. The phase reports
564 exact native/WASM results, 564 independent weighted replays, 6,204 evidence
tamper refusals, 66 matching generation failures, 24 invalid requests and 500
real prepared-state project/export replays. Its 42 balance misses and zero
disconnected counter are synthetic-only: source inspection confirms those
counters do not accumulate the real-state loop. The original summary remains
unchanged and must not be presented as an all-case feasibility count.

An independent readback predeclared exactly fifty states x two synthetic
character inputs x five searches before freezing all 500 projects and fifty
prepared graphs. Every original file was rehashed during capture; each project
binds the frozen raw graph hash. The audit uses Python integers/rationals to
recompute population totals and exact maximum absolute relative deviation from
equal population, and its own per-district adjacency traversal for connectivity.
All unit identities/labels are complete and all recorded district populations,
unit counts, component counts and overall metrics agree. All 500 cases meet
the requested 10 percent tolerance; none is disconnected in its selected graph.
This is synthetic economic/housing data on real prepared state graphs, not
observed character data, land-only contiguity, legal validation, global
feasibility or optimality. Scope, predeclared cases, content-addressed frozen
projects/graphs, source test, original summary, audit code and raw outcomes are
retained under `target/parity-full-build-20261030-r8/character-readback`, with
identities in `evidence.json`. No fresh generation is claimed by that readback.

The complete standard/C build, guarded final proof profiles, candidate source
integration and formal acceptance remain pending. Source coverage and these
phase-specific successes do not promote any canonical parity status.

### Prepared-state workbench candidate coverage — 2026-10-31

The roles review's separate stale prepared-state browser wait is now exercised
in isolation. The only proposed production-test change replaces its old saved
verification message wait with completion plus the exact current replay message
and completed-state count. The status-only patch passes an apply check and is
unapplied. No source/UI/engine behavior is changed during the captured run.

Fresh isolated prepared-batch fixtures use frozen input projects, retained native
reference executables and the exact diagnostic package engine/modules. They pass
sixteen complete native tool comparisons, including exact numeric-u64 request
text, original options/retry modes, serial/parallel identity, mixed/failure cases,
state/national rendering, cancellation/Resume and explicit saved verification.
Independent Python readback verifies nine nested module-output archives: CRCs,
paths and exact original request/native plan/context/replay/geometry bytes,
including no fabricated physical export when fine-boundary edges are absent.

The complete prepared-state Chrome 155.0.8059.85 test then passes against the
frozen package workbench: typed review/queue without partition, stale pending
queue-hash edit refusal, full numeric-u64 sources, three actual prepared workers,
explicit retry modes, worker termination, interactive selected-state map, ZIP
delivery, inert national handoff and exact native national render, inert Open
with disabled map/download/Resume, explicit saved replay, and cancellation with
unfinished-only Resume. These are synthetic prepared cases labeled RI/IA/VT,
not original all-state source acquisition. A further independent readback binds
the browser ZIP's full receipt to the saved/native-matched result and verifies
all three nested archives' CRC/unique/safe paths, source/seed, accepted replay,
assignments, native plan/context and geometry bytes.

All 247 served workbench asset identities match the frozen package manifest;
all 326 package assets remain unchanged. Recipe, before/after test, unapplied
`status-fix.patch`, input snapshots, native binaries, module/browser sources,
logs, native-matched outputs, ZIPs and scoped `evidence.json` are retained at
`target/parity-full-build-20261030-r8/prepared-browser-candidate`. Browser
profiles are excluded from evidence mappings. The prepared browser sequence
remains unregistered in the complete standard/C builders; this is diagnostic
package evidence, not final-profile or formal parity acceptance.

The active full R8 build has separately passed the national character browser:
two fifty-state experiments and 100 saved/restored maps, with exact synthetic
character input retention and inert reopening. It has moved on to proportional
algorithm validation. Whole standard/C success, final proof-profile execution,
source integration and formal acceptance remain pending.

### Complete proportional outcome accounting — 2026-10-31

The fresh R8 proportional gate passes 270 exact synthetic native/WASM results,
180 matching synthetic failures, 4,710 tamper refusals, fifteen invalid inputs,
three practitioner exports, 120 real-graph projects and thirty real-graph
failures. Its printed zero balance misses/disconnections counts synthetic cases
only. Stored real diagnostics already include larger deviations; the original
report is preserved and must not be interpreted as universal feasibility.

The independent follow-up predeclares all fifty states x three method/search
combinations before freezing the 120 projects selected by the fresh phase report,
not by filesystem existence. Fifty immutable prepared graphs and the current
catalog bind those inputs. Exact integer/rational population and independent
per-district adjacency traversal find two produced plans outside the requested
10 percent tolerance: CA proportional-section/single has maximum deviation
47.1193988663 percent; TX proportional-section/multi has 54.0460321411 percent.
Both remain connected in the selected graph. All recorded district and aggregate
metrics, including failed-balance flags, match the independent audit.

The remaining thirty outcomes are freshly replayed with the original seed/options
and synthetic census-scaled election counts, using a retained native executable
and the exact current standard WASM identity. Every native/WASM error string
matches. These are bounded sampled-search failures, not infeasibility proofs.
Every raw request, native stdout/stderr and comparison is retained. The same
WASM instance subsequently reproduces a retained RI success exactly. Positive
and negative sets are disjoint and cover the declared 150 cases once.

`target/parity-full-build-20261030-r8/proportional-readback/evidence.json` binds
the ledger, snapshots, 120 content-addressed projects, fifty graphs, original
summary/source, independent audit, native binary, raw negative replays and
standard engine identity. No original report or execution claim is rewritten.
A separate two-test `counter-scope-fix.patch` adds synthetic/real/total counters
to character/proportional summaries; syntax and apply checks pass, but it is
unapplied during captured validation. It changes reporting only and has no
algorithm or feasibility improvement claim. The active full build has advanced
to partisan validation. Standard/C completion, source integration and formal
acceptance remain pending.

### Paired native ignored-field and browser print-only adaptation — 2026-10-31

The existing Run/States print-only contracts distinguish current native behavior
from browser intent: the reviewed native entry/configuration ignores the parsed
field, while the browser explicitly keeps print-only inert. This follow-up tests
the declared adaptation without changing the engine, silently substituting a
native effect or treating it as complete CLI filesystem equivalence.

Four supplied-graph baseline/print-only pairs execute through the independent
native Run/States parser/configuration/dispatch references: eight native process
invocations. Configurations, seeds/retries, assignments and remaining output
fields match exactly. Run's parser-debug field is separately retained and checked
for the intentional parsed flag change; it is the sole excluded comparison
field, as allowed by the current contract. States replies remain whole-equal.
Original argument vectors, requests, stdout/stderr, native executables and their
current authority/frozen-legacy source bodies are preserved.

Both complete original browser suites then pass against the frozen diagnostic
workbench in Chrome 155.0.8059.85. Their print-only cases preserve exact user
argument JSON text and parsed vectors, retain parsed true, report no execution
or filesystem reads, and retire run downloads. Other original Run/States
compile/explicit-run, source retention, maps/output, inert Open and mobile checks
remain intact. Run's inspector explicitly records that native print-only was not
applied. Each browser serves 247 package-bound asset identities and records
actual loaded standard-WASM fingerprints (nine Run and ten States observations).
All frozen package files remain unchanged. Source/argument retention plus these
separate behaviors do not establish automatic native OS discovery, actual native
Run/States filesystem orchestration or complete final-profile parity.

`target/parity-full-build-20261030-r8/print-only-pairing/evidence.json` and
`evidence-v2.json` bind the four native pairs, actual browser receipts, selected
inputs, raw outputs, binary/source snapshots, measured browser/module identities
and their two stable contract IDs. Browser profiles are excluded. No source or
canonical evidence state changes during captured validation; formal acceptance
remains pending.

The live whole build has separately completed partisan (900 exact cases,
250 project/export/native comparisons), search-boundary, BFS (300 exact results,
260 reported balance misses), exact-u64 (56 cases/four matching failures), and
N-way gates. Printed partisan/proportional feasibility counters must retain their
source-defined scope; their passing parity does not establish universal balance.
Fresh partisan summary/source snapshots are preserved in `fresh-phase-receipts`.
The build is now running ratio-constraint validation; C and final-profile proof
checks remain queued behind successful complete builds.

### R8 ratio completion and independent positive-project readback — 2026-10-31

The fresh ratio gate completed 288 exact synthetic cases, 3,168 evidence
refusals, 98 real-state project/export roundtrips and two matching real-state
generation failures. `target/parity-full-build-20261030-r8/ratio-readback`
freezes its summary, test source, historical predeclared matrix, current catalog,
50 graph files and all 98 fresh project files, with SHA-256 identities.

`audit_ratio_readback.py` independently derives population using exact rational
arithmetic and connectivity using the undirected selected graph. It checks unit
coverage, district metrics, recorded feasibility flags, configurations and exact
assignment hashes against the earlier matrix. All 98 positive cases match;
CA VRA is the sole balance miss (16.654499 percent versus 10 requested), and
none are disconnected. The two negative outcomes are supported by the fresh
gate's aggregate matching-failure count; this audit does not retain or compare
individual fresh raw negative replies. The result is positive-project regression
and readback evidence, not actual-browser, explicit-C or full-parity acceptance.
The live standard build advanced to label import/rename/recovery workflows.

### Reproducible prepared-state package gate candidate — 2026-10-31

`target/parity-full-build-20261030-r8/prepared-registration-candidate` contains
an unapplied nine-file integration patch. Three fixture files retain only
synthetic prepared source/settings/geometry and configured input, without prior
engine results. The original batch assertions now import the selected package's
modules and WASM and use explicit native-oracle, fixture and output paths. The
browser serves the selected package's `index.html`; default source testing still
uses `workbench.html`. Its earlier saved replay status correction is included.
The ZIP audit also accepts the explicit output path.

The new package runner refuses existing evidence paths and outputs within or
above the package, pins package/input/native/test hashes, retains every command
and log, runs batch/independent ZIP/optional browser checks in order, and checks
the complete package is unchanged. Standard and C builder hook candidates build
the portable native tool oracles and run this gate against their toolkit, using
a fresh evidence directory and forwarding `BrowserChecks`.

`gate-run-v2/evidence.json` records a passing complete diagnostic gate: 16 exact
native tool comparisons, nine nested ZIP audits, actual browser review/queue,
worker generation, selected-state map, national map, exact downloads, inert Open,
explicit saved verification, cancellation and unfinished-only resume. Earlier
attempts are retained: the initial browser used the source-only entry filename;
the first combined gate lacked the required Playwright environment variable.
The runner now checks that variable before creating an output when browser
checks are requested. Patch applicability and both PowerShell recipe parses
pass. Builder hooks are unexecuted and current captured sources stay fixed.
Adoption must replace the previous status-only patch and combine these builder
edits with the separate proof hardening changes; no explicit C backend or formal
acceptance status is promoted by this diagnostic result.

### Combined adoption candidate — 2026-10-31

`target/parity-full-build-20261030-r8/adoption-candidate-v3` merges all 18
pending source files, including overlapping standard/C builder edits, in an
isolated overlay. It supersedes the earlier 12-file candidate and includes the
full prepared-state registration rather than its status-only correction.
The failed `adoption-candidate-v2` merge is retained: newline expansion in the
candidate recipes made their patch overlap the entire files. Normalizing reads
before writing produces the corrected `registration-v4.patch` and clean merge.
Current repository sources are unchanged.

The resulting combined patch passes applicability; JavaScript syntax, Python
compilation, JSON parsing and all PowerShell recipe parses pass. Its separate
`diagnostic-binding.json` verifies the exact seven prepared test/input files
match those exercised by `gate-run-v2`, and pins the earlier proof and counter
readback evidence. These identities preserve the scope of diagnostic checks;
they do not prove execution of the changed builders. Integration waits for the
captured build and queued package/proof checks to terminate and their original
evidence to be retained, then requires fresh affected validation.

The final combined patch is `combined-v2.patch`, recorded by `evidence-v2.json`.
A second isolated materialization checks all 18 resulting files byte for byte.
This retains the tested prepared source/fixture bytes across mixed Windows line
endings; `diagnostic-binding.json` now passes all seven exact identities. Earlier
patches and the initial failed identity check are retained without promotion.

### Terminal R8 preservation, integration and laboratory correction — 2026-10-31

R8 standard reached its final laboratory gates after 50 real-state catalog
runs/maps/projects and the successful national label build. The latter has 50
accepted outputs, 153 ZIP entries with matching native plan/context/assignment
sets, and an 87,289,596-byte source/output archive. It then stopped because
`test_wasm_lab.mjs` unconditionally required every single-attempt candidate to
meet population tolerance. Standard remains failed; C, packaging and queued
final-proof checks did not launch. Final source verification confirms all 3,234
captured inputs unchanged.

`terminal-and-adoption/preservation.json` retains terminal logs, original source
capture/settings, pre-adoption source files, native/WASM binaries and scoped
national inputs/ZIP/archive evidence. After all three guarded runners terminated,
the combined 18-file patch was applied byte-exact. Existing failed packages and
earlier artifacts remain. `adoption.json` records integration, not acceptance.

`lab-balance-investigation` retains the exact failing California request and
whole native/WASM replies. Both produce the same connected plan with 73.460542
percent deviation and correctly failed requested 5 percent tolerance. The test
now independently derives population, selected-graph connected components,
district metrics and feasibility flags; it retains raw native/WASM comparisons
for every graph/options execution, original pilot balance checks and all prior
map/export/memory/cancellation checks. Engine behavior, constraints and budgets
are unchanged. Corrected pilot and all-state gates pass. The all-state gate has
102 exact replies, 29 misses (two single-bisection and 27 N-way), zero disconnected
plans. Those are candidate-quality diagnostics, not accepted-plan claims.

Fresh adopted-source proof checks pass both real prerequisites and 19 stale/
missing-input refusals, plus seven damaged/undeclared package refusals across
four proof entry points/221 dependency files. The adopted prepared package gate
passes its batch, nine archive checks and actual browser suite against the
retained diagnostic package with its complete package unchanged. These focused
results do not establish fresh full-builder or final standard/C acceptance.
The separate all-state native-build gate still requires balanced accepted retry
results and now passes, including all practitioner exports, audits/certificate
verification and inert saved reopening. Its first run exposed an assertion
intended for CA/TX but applied to every selected state; restricting that retry
assertion to the two named fixtures preserves its original intent. CA accepts
seed 46 after four retries; TX accepts seed 45 after three. Other states can
succeed immediately. No production retry behavior changes.

Twenty-six current candidate-source contract pointers are reconciled, with old
pointers and reconciliation notes retained; historical receipts and all
implementation/evidence states stay unchanged. The ordinary ledger validates all
309 records. A captured remaining-laboratory diagnostic now runs in a separate
copy at `dist/wasm-parity-standard-20261031-tail1`, with status/logs under
`target/parity-lab-tail-20261031-r1`. The failed R8 package is preserved. This
diagnostic uses retained compiled engines; fresh full builds and formal closure
remain open.

### Adopted proof guard execution — 2026-10-31

`target/parity-full-build-20261030-r8/adopted-proof-entrypoints` copies 23
current source/artifact files byte-exact into a disposable installed-layout
replica. Its original 14-case integration exercise passes against the adopted
scripts: valid checker staging produces ten exact files; all eight stale source
claims, changed adapter bytes and changed checker WASM are refused before package
writes; standard, laboratory and C build entry points stop at shared preflight
without creating output directories. All 23 active input hashes remain unchanged.
`source-binding.json` binds current files to the retained exercise evidence.
These checks establish adopted early-guard behavior, not execution of complete
builders or final standard/C proof pages. The separate remaining-laboratory
diagnostic continues its one-district matrix with captured source fixed.

### Remaining laboratory gates pass; fresh R9 launched — 2026-10-31

`target/parity-lab-tail-20261031-r1/status.json` is passed for all nine remaining
gates, with 3,243 captured current inputs unchanged. The one-district matrix
passes 7,150 native/WASM assignment cases and Save/Open/map roundtrips across
all 50 states, supported structures/searches, legacy/explicit options and three
weights; 21,450 tampered projects are refused and Open executes no engine.
Project, demographic CSV/worker, real-input chains, 24 VRA project cases/48 tamper
refusals, and offline manifest/package checks pass. Character/demographic inputs
are explicitly synthetic where declared; no legal-contiguity finding follows.

Fresh complete standard/C R9 builds launch at 2026-10-31T22:09Z under
`target/parity-full-build-20261031-r9`, with unchanged 50-state/2020 settings.
Its source capture includes the adopted proof/package/fixture/counter changes,
two laboratory test corrections and reconciled current candidate pointers.
`scoped-prerequisites` freezes the preceding one-district matrix and laboratory,
native-build, prepared and adopted-proof summaries before temporary paths are
reused. Original failed R8 and separate diagnostic packages remain preserved.

R9 has guarded packaging and final-proof queues tied to the exact new build PID
and terminal success receipt. Four proof-page checks will run against separately
frozen completed standard/C packages; optional C partition execution remains a
separate contract. Launch checks and initial connectivity/ZIP gates pass. Full
standard/C execution, final profile checks, versioned package readback and formal
contract closure remain pending. Nothing is deployed or marked full parity.

### Fresh phase readback collection — 2026-10-31

R9's read-only `collect_phase_readbacks.py` holds a handle to the original live
build process and waits for fresh completed character/proportional summaries.
It retains the fixed earlier 500/150-case matrices, requires current project
timestamps and graph identities, freezes positive projects/graphs/source/report
bytes, then runs independent integer/rational population and graph-component
audits. Copied audit scripts use the newly explicit synthetic counters; measured
real misses/disconnections must match the current phase report. Proportional
positive/negative classification changes require review rather than selection
by existing filenames. Individual fresh raw failure replay is not claimed.

The collector has its own launch/status/logs under
`target/parity-full-build-20261031-r9/phase-readbacks`; it is currently waiting,
not a completed audit. It changes no captured source or build output. Browser,
explicit C, original-source provenance and formal acceptance remain separate.

### Fresh R9 character phase and frozen readback pass — 2026-10-31

The fresh R9 character phase passes 564 exact native/WASM cases, 564 weighted
replays, 6,204 evidence tamper refusals, 66 matching generation failures, 24
invalid requests and 500 real-state project/export roundtrips. Its adopted
counter split reports 42 synthetic balance misses, zero real-state misses and
zero disconnections. The report separates candidate quality from execution.

The collector freezes all 500 fresh projects, all 50 graphs, the current test
source/catalog and completed summary under
`target/parity-full-build-20261031-r9/phase-readbacks/character-readback`.
Independent exact-integer/rational population and selected-graph component
readback passes: zero population misses, zero disconnections, no recorded metric
mismatches. `audit.json` SHA-256 is
`97717b4f937f6297244685c9e56923ca8c8e4bb4384aba4f6498fc83b097f8c6`.
Character values are synthetic; graph connectivity does not establish land-only
or legal contiguity, upstream source authority, actual-browser/C coverage or
full parity. The collector continues waiting for the fresh proportional phase.
The whole build continues with practitioner workflow/browser checks; final
standard/C, proof pages and package readback remain pending.

### Fresh national character browser checks pass — 2026-10-31

R9's national project gate passes two 50-state experiments and 100 saved/restored
maps. The actual packaged national character browser test then passes at
`target/national-character-browser-0114uh`: no source overlay, package source
identity checks, complete source preservation and inert Open/reopen for economic
and housing experiments. Retained project downloads are 18,962,582 and
10,035,620 bytes respectively; maps and the refusal-retention project also remain
in that unique evidence directory. These bytes are recorded measurements, not a
general project-size guarantee. Character inputs remain declared synthetic.

Actual character CSV workers pass conversion/raw retention, integrity, UTF-8,
scope/duplicate/year, pre-read bound, cancellation and termination checks. The
whole build advances to proportional validation; the read-only collector waits
for its completed summary. Fresh complete standard/C builders, final proof-page
profiles, packaging/readback and formal contract closure remain pending.

### Fresh proportional phase and independent positive readback pass — 2026-10-31

R9 proportional validation passes 270 exact native/WASM cases, 180 matching
failures, 4,710 tamper refusals, 15 invalid inputs/options and three exact
practitioner exports. It records 120 real-state projects and 30 real-state
generation failures. The adopted counters report zero synthetic/two real balance
misses, zero synthetic/real disconnected plans. Generation failure is not an
infeasibility proof; comparison success is not candidate acceptance.

The fixed-matrix collector freezes and independently checks all 120 fresh
positive projects and their 50 graphs. Exact integer/rational population and
selected-graph component readback confirms the CA section/single and TX
section/multi misses, zero disconnections and no stored metric mismatches.
Evidence is under `target/parity-full-build-20261031-r9/phase-readbacks/proportional-readback`;
`audit.json` SHA-256 is
`5a50d6f2bab5d10f1d717efc71b50185b035d4b2f643087700444347ff56d1ae`.
Positive/negative classifications match the earlier fixed matrix. Individual
fresh negative replies were not re-audited or preserved by this collector.
Its two-phase status is passed; all original snapshots remain unchanged.

Subsequent election CSV and partisan TSV gates pass 13 exact comparisons apiece,
31/27 malformed or tamper refusals, six RI/IA/NC run/Save/Open/map cases apiece,
and actual conversion/retention/scope/UTF-8/cancellation worker checks. Votes and
shares are synthetic. The whole standard build continues; C/final proof/package
execution and formal contract closure remain pending.

### Partisan independent readback queued — 2026-10-31

Current partisan source accumulates its existing balance/disconnection counters
across synthetic cases, supported-option cases and saved real-state projects.
It does not have the earlier character/proportional omission. A new read-only
collector under the R9 root retains the exact fixed 50-state x five-search matrix
from that source, checked against its pre-build source hash. This is an input
matrix derived from the already pinned test, not a separate experimental
preregistration or selection by output success.

`partisan-readback/status.json` currently waits for a fresh completed phase
summary. The collector will freeze all 250 fresh projects and their graphs,
independently derive exact-integer/rational population and selected-graph
components, check recorded metrics, and label any synthetic count derived by
subtracting real misses from the combined report as inferred. Active build
source and output files are unchanged. No partisan audit, browser/C or full
acceptance is inferred from starting this collector.

### Fresh partisan phase and frozen readback pass — 2026-10-31

The fresh partisan phase passes 900 exact native/WASM cases and independent
weighted-cut checks, ten supported-option comparisons, two matching feasibility
failures, 9,900 evidence refusals, 13 invalid inputs/options and all 250 real-state
native/project/export roundtrips. Its combined miss count is one, with zero
disconnections. Shares are synthetic, not observed election results.

`target/parity-full-build-20261031-r9/partisan-readback/status.json` is passed.
All 250 fresh projects and 50 graphs are frozen, with exact integer/rational
population, selected-graph component and district-metric readback. One real
miss is FL single (44.03780597 percent versus 10 requested); the plan is connected
and correctly records failed tolerance. No recorded/computed metric mismatch or
selected-graph disconnection occurs. Audit SHA-256 is
`f12e1f82a2ce91634b299ffe0a08c5a14e035bc42713df4e476d48b256c9d0b8`.
Subtracting that real miss from the combined reported one implies zero synthetic
misses; this is labeled inferred, not an independent synthetic-case readback.
No actual-browser/C, land-only/legal contiguity or full parity claim follows.

The build subsequently passes twelve partisan boundary cases, 300 exact real-
state BFS results/50 projects (260 balance misses), 56 exact-u64 cases/four
matching failures/18 unsafe values, and 24 N-way cases/five invalid requests
(eight balance misses). Candidate quality stays separate from parity. Fresh
ratio-constraint validation is now running. Complete standard/C, final proof
pages, package readback and formal closure remain pending.
