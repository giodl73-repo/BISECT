# Native population-source behavior and browser work

Date: 2026-10-27. Status: source preparation, typed artifact binding, checked
original State jobs and explicit source-loading UI. Packaged portable-file browser
checks pass; actual OS folder permissions and broader acceptance remain open.

The State declaration describes a population basis, but current native execution
does not select a different balancing vector from this string. The runner stores
the result of `check_cvap_availability` in an unused variable. Successful native
RCTX populations retain the adjacency graph's weights. The manifest stores the
original requested string, including an unrecognized value; this argument is a
String without a Clap value enumeration.

There are two separate filesystem consumers:

1. The availability check acts only on the exact lowercase request `cvap`. It tests
   existence at `data/{year}/demographics/{raw_state_name}_cvap_{year}.csv` relative
   to the process working directory. A missing path emits a warning. It neither
   validates CSV data nor replaces the graph population vector.
2. Audit-sidecar construction independently searches for VAP/CVAP files, regardless
   of the requested population source. It normalizes the state name to lowercase
   with spaces replaced by underscores. Candidate order is normalized-name VAP,
   state-code VAP, normalized-name CVAP, then state-code CVAP. The first existing
   candidate is parsed using the shared native VAP reader and aligned to graph
   GEOIDs. Malformed input fails output generation; valid input supplies report
   demographics. This is separate from the graph's balancing populations.

The actual CLI reference at
`target/population-source/native-1793112696653683700/evidence.json` retains eleven
four-unit metis-core n-way attempts. Nine succeed with identical assignments,
ordered units and graph population vectors. Their requested population strings
survive in the manifest. Valid CVAP CSV populates report demographics even for a
`total` request. Empty-file and directory CVAP candidates suppress the availability
warning but fail during audit-sidecar loading. A raw spaced name demonstrates that
the availability and report loaders use different basenames. The first exploratory
attempt incorrectly expected an empty existing file to permit complete output;
its native failure logs remain under `target/population-source`.

Browser implementation must preserve these consumers separately. A checked State
workflow may retain the requested metadata and disclose the unchanged balance
vector, but dropping the flag alone is insufficient. Named auxiliary-source
discovery needs an explicit selected data root, bounded native-compatible CSV
reading, retained candidate precedence and source hashes, fresh rechecks, and
current authority for filesystem operations. A missing browser selection is not
proof that a native path is absent. Opening a project must not start discovery or
restore a folder grant. Portable explicit-file input remains necessary where the
directory API is unavailable.

Before exposing execution, compare actual native and WASM warnings, assignments,
RCTX populations/demographics and manifest strings for the retained cases. Add
candidate conflicts, changed/deleted files, basename normalization, source-root
changes, malformed CSV, inert Open and actual worker UI cases. Keep all fourteen
direct compiler orchestration guards until an explicit bound workflow owns the
corresponding behavior. A real alternate-balance-vector feature would require a
separate specification and changes to both desktop and WASM.

## Source-preparation implementation checkpoint

Native availability and report discovery now share pure basename helpers with
WASM, preserving their separate normalization and candidate order. Fresh native
reference `target/population-source/native-1793115189856167000/evidence.json`
retains all eleven outcomes. `target/state-report-sources/native-preservation.json`
compares it with the earlier executable: nine complete RCTX files are byte
identical, RPLAN plans and selected manifest fields agree, and the two expected
failure stages remain unchanged.

The internal `inspect-native-state-report-sources` operation plans those paths
without reading disk. Complete supplied missing/file/directory observations can
then produce native-compatible report counts in explicitly supplied unit order.
The first existing report candidate wins even if malformed; lower candidates
are not a recovery fallback. Original CSV bytes and exact requested population
strings are retained. Counts remain JSON text from Rust. Graph binding is
explicitly false at this stage, and no balancing vector is selected or changed.

The browser source helper loads only these named candidates from an explicitly
provided working-folder handle. It reads the snapshot twice and refuses changed
or deleted bytes/types, changed flags, cancellation and permission failures. It
never creates directories or returns a handle. Portable file bundles require an
explicit choice to treat unselected candidates as absent in that bundle; they
make no claim about files elsewhere on the computer. The current bounds are
8 MiB/file, 16 MiB total, 100,000 CSV lines and 100,000 supplied unit IDs, with
portable ASCII name components. These are capacity limits, not native limits.

Both exact default/C-enabled WASMs pass eleven fresh native comparisons and the
read-only Windows facade/adversarial tests. Two further real CLI probes per build
verify duplicate/numeric/missing-unit defaults and first-existing precedence,
including failure despite a valid lower-priority source. Both still refuse direct prepared
`--state-name` and `--population-source` orchestration. The new API performs no
partition or filesystem operation; selected-folder reads occur only in the
explicit caller-side helper. No C partition execution or actual browser picker
permission test is claimed.

The first parallel harness run collided in a shared millisecond-named fixture
directory. Failed logs and its source preimage remain retained; atomic unique
test directories remove that interference. See
[PP-61](../../design/pitfalls/pitfalls-pipeline.md#pp-61-concurrent-tests-share-a-timestamp-named-fixture-directory).

Next, bind the prepared source snapshot to the actual compiled graph and accepted
RPLAN/RCTX; regenerate hashes/provenance and follow-up analyses without feeding
report counts into partition weights. Then connect explicit folder/file loading,
source review/reapplication, original flags and warning/manifest metadata to the
State UI and safe local projects. Actual worker UI, lifecycle/recovery, packaged
regressions and full flag/parent acceptance remain open. The preparation tests
must not be promoted to an executed State workflow claim.

The [six-lens preparation review](../../signals/roles/check/wasm-report-source-preparation-roles-check-2026-10-27.md)
approves the internal foundation with those integration conditions. Current
development evidence is recorded at `target/state-report-sources/evidence.json`;
it is not a final release package or acceptance-v1 bundle.

## Accepted-artifact binding checkpoint

The internal `bind-native-state-report-sources` operation now reads matching
RPLAN/RCTX text through Rust, checks complete ordered unit identity, graph and
populations, and requires the native name/label/state/year/chamber and any explicit
district count to match. It recomputes counts from the original snapshot in the
actual artifact unit order. Successful binding replaces only report demographics,
adds an explicit source association and regenerates native context/provenance
hashes. It does not rerun generation or change assignments, graph or populations.

The versioned association lets generic manifest preparation retain the original
population-source request. It must match the artifact scope and current context/
source hashes; stale provenance refuses. Absent associations preserve the legacy
manifest branch. Raw artifact text stays in typed Rust so large i64 populations
and maximum unsigned seeds survive. Added browser source provenance means full
RCTX bytes/hashes are not claimed equal to native CLI output; report counts,
requested metadata and preserved generation fields have separate comparisons.

The prepared-result adapter preserves original input objects and accepted request/
assignments/replay data. Native report load failure keeps that generation but
removes the completed export and records export_error. Explicit rebind is allowed;
previous hashes remain in lineage. Current-generation and abort callbacks prevent
late publication, and inputs are captured before asynchronous work.

Both exact WASMs pass eleven accepted prepared-run comparisons apiece and eighteen
independent ZIP/context/manifest/seed checks. Result-only archive roundtrips,
source/policy/scope/provenance refusals, cancellation and named-State regressions
pass. The test prepares portable engine controls separately: it does not establish
original State population-source flag execution or a GUI handoff. New source input
adoption, queued jobs, follow-up/installation/recovery, actual worker UI and full
package/parent acceptance remain open. See the
[binding review](../../signals/roles/check/wasm-report-export-binding-roles-check-2026-10-27.md)
and `target/state-report-binding/evidence.json` for this in-progress component.

## Original State workflow checkpoint

A retained `stateReportSourceSnapshot` can now be applied with the original State
arguments. The binding preserves the original tokens, source bytes, observation
realm, candidate precedence and native identity/policy receipt. Only this explicit
binding permits removal of `population_source` from engine-only arguments. An
unbound explicit flag still refuses execution; the fourteen direct compiler
orchestration guards remain intact. The existing Apply action can consume this
input; no dedicated source picker has been exposed yet.

Queued selected-adjacency, graph and prepared-template State jobs recheck supplied
bytes, flags and native identity before preparation or partitioning. This recheck
does not parse the CSV early: malformed first-existing sources still fail after
generation. The accepted export binder runs before optional follow-up and output
installation. Failure retains the generation and replay, records `partial-output`,
removes the completed export and skips follow-up. Missing-CVAP warnings use a
separate warning event so position 999 cannot hide them.

Optional analysis/report stages use the same typed native manifest as final
selected-adjacency output. The containing workflow passes its generated manifest
into analysis before file hashing and report assembly, avoiding a conflicting
generic manifest or a later text patch. Raw JSON text retains exact integer digits.

`test_native_report_state_workflow.mjs` compares eleven retained native cases,
including requested strings, demographics/populations, successful assignments and
ordered units, exact maximum-u64 manifest text, completed ZIPs and expected partial
failures. It also checks graph/prepared jobs, saved files reopened from disk,
changed selections, forged policy/source receipts, position 999 warnings, and
reviewed reset/force with preserved source bindings. Imported bindings authorize
no target action; reset requires current separate review and preserves old exports.
Native and browser source IDs/context provenance differ by design; whole-artifact
byte identity is not claimed.

Development results live under `target/state-report-workflow`. This checkpoint
does not accept real browser picker permissions, automatic source discovery,
dedicated file/folder UI, worker cancellation/recovery presentation, C partition
dispatch, broad method/capacity matrices, final packages or full State/parent
parity. Earlier component evidence continues to identify its own older engines.

## Explicit source-loading UI checkpoint

The State command panel now reviews native candidate paths and offers a read-only
working-folder snapshot or portable CSV selection. File bundles require an explicit
choice that unselected candidates are absent within that bundle. The checkbox is
not saved and resets when project inputs change. Empty bundles are explicit;
unexpected/duplicate filenames and oversized sources refuse. Selected handles are
local to one read, never stored or imported as authority. Picker invocation occurs
in the click's user activation; worker/path inspection does not invoke a picker.

Loading restores the original command for fresh Inspect and Apply, retains raw
source bytes in the project, invalidates fresh results and starts no generation.
The guard worker uses the loaded WASM identity. Cancel, changed flags, project
revision, busy state or runtime changes retire the read/worker; a late picker
result cannot publish a selection. Detach restores bound original flags when they
are still the active engine command and preserves independently edited flags.

The panel displays retained observation realm, source hash, requested string and
missing-CVAP warning. A failed report export explicitly retains generation and
offers save/retained-run download while completed output and audit export are
disabled. Corrected sources need a fresh explicit Inspect/Apply/Run.

`test_native_report_source_ui_browser.mjs` exercises ten portable-file/empty-file
native cases per actual packaged Chrome build, with guard workers, original flag
Apply, generation, report counts/failures, inert Open and explicit replay. It also
checks absence consent, unexpected filenames, saved source bindings, retained
generation download and stale/cancelled delayed picker results. A directory source
cannot be represented by the portable file picker; its failure remains covered by
the eleven-case job/API suites. The delayed directory handle is injected and does
not prove actual OS permission behavior. Both packages pass assets/offline gates.

Evidence is retained under `target/state-report-ui`; earlier snapshots are not
repinned. Actual native picker permission/readback, folder errors during an active
read, complete provider/host/recovery matrices, registered final release gates,
C partition dispatch and full flag/State/parent acceptance remain open.

## Folder API and recovery checkpoint

`test_native_report_folder_browser.mjs` now exercises all eleven retained native
cases through the packaged folder UI with actual Chrome File System Access
directory/file handles backed by origin-private storage. The test picker returns
an explicitly seeded fixture root; it does not select an external OS folder or
grant a permission. Native missing-file and directory/type-mismatch behavior,
source observation hashes, unchanged source trees, worker runs, requested metadata
and report counts/failures agree on both exact staged profiles.

Nine additional scenarios per profile cover permission/unreadable/picker-denial
faults, file deletion/replacement/byte changes between reads, oversize refusal
before byte reading, paused-read cancellation and project replacement. Permission,
unreadable and pause failures use an instrumented wrapper; deletion, replacement
and byte changes mutate actual owned OPFS entries. They are not evidence of OS
permission denial. Failed selections preserve the prior snapshot and start no
partition. A corrected root requires explicit loading, Inspect/Apply and Run;
recovery then binds counts successfully. Late paused reads make no further handle
calls and cannot publish into the changed project.

The first harness attempt timed out waiting for an Open status; its source/logs
remain retained. Autosave replaces that status after 300 ms. The harness now
observes the exact import transition before starting Open rather than depending
on a polling frame during that short interval. The original timeout's specific
cause is not proven by this adjustment. Final default/C-enabled runs pass.

`check_native_report_folder_permission.mjs` prepares a separate temporary visible
Chrome session and owned public fixture root for a user-operated native picker
and read grant. It never accepts a permission prompt. A pending/live session is
not passing permission evidence; only its terminal checks and retained artifacts
can support that claim. See `target/state-report-folder` for this checkpoint.
External OS permissions, other active host errors and full matrix/release/parent
acceptance remain open unless separately accepted.

## Registered dual-build regression checkpoint

Registered run `70800c35-7864-4544-aac9-57de84971f19` passes the complete
dual-build/browser runner, including the newly registered report-source suites.
Both fresh WASM hashes match the earlier source/job/UI engines. Preparation,
accepted export binding, original State jobs and packaged portable/folder API
journeys now run alongside configuration/CLI assignment oracles, composed Verify,
label/report workflows, workspace recovery and Fetch/FLETCH comparisons.

`check_native_report_regression.mjs` generates fresh actual CLI reference cases
and a fresh source-preparation snapshot for each profile. It supplies that exact
evidence to subsequent tests, verifies retained native bytes and ignores inherited
report-fixture selections. Local replay may explicitly consume retained evidence
through the common reference helper. Browser staging separately requires a
hash-verified accepted package base; this is not a clean-room packaging claim.

The first registered attempt stopped on a portable-browser Open-status polling
timeout. Its terminal log, receipts, source preimages and partial results remain
retained. Both portable and folder suites now share a preinstalled observer for
the exact expected Open transition, then independently check saved project data
and absence of implicit worker/picker activity. Autosave cannot itself satisfy
the success predicate. The succeeding run starts with fresh not-run receipts.

Current validation/evidence lives under `target/state-report-regression`. The
registered runner keeps actual C execution/runtime checks separate: these new
report fixtures use metis-core on both builds. OS picker grants, broader
method/data/capacity/recovery matrices, full child/parent support and release
acceptance remain open. No deployment occurred.

## Actual C State/report checkpoint

The same retained-source workflow now has an explicit Windows-CRT C backend
check. Fresh C-enabled CLI cases and the independent C oracle are built with
`--no-default-features --features c-ffi-engine`. Eleven direct State jobs match
native C outcomes; successful browser plans also compare native assignments and
ordered IDs, not only counts. Ten portable-file and eleven OPFS folder journeys
use explicit C selection and retain runtime hashes through save/Open/replay.
Nine folder failure/recovery scenarios pass. A cold runtime refuses execution.

The [C integration checkpoint](wasm-cffi-engine-integration.md#current-report-source-integration-checkpoint--2026-10-27)
records the repeatable dedicated gate and separate metis-core control. It covers
the four-unit RI 2020 N-way/report intersection and existing 64/300-node C kernel
grids, not national C or a full backend/method/capacity matrix. Native build-feature
selection caveats and actual external-folder permissions remain open. Detailed
evidence is under `target/report-cffi`; nothing was deployed.
