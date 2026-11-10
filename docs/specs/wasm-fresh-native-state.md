# Fresh native State inputs in WASM

The workbench's **Fresh native State run from local adjacency** panel executes
ordinary and selected native64 multiscale State requests from selected RADJ bytes, optional original
GEOID/centroid companions and optional original COI, partisan, demographic,
economic and housing files. [Auxiliary input guidance](wasm-fresh-native-auxiliary-inputs.md)
describes native weighting flags, formats and bounds. A prior plan,
RCTX or replay receipt is not required or accepted as input authority.

The **Native build to compare** selector is independent of `--metis-engine` and
of the C-runtime loading checkbox. Omission defaults to portable `metis-core`.
Ordinary recursive METIS follows the selected native build, preserving desktop
flags that its runner ignores; prime-factor follows the configured flag and the
selected build's available backends. BFS growth, centroidal Voronoi, capacity
clustering, and ILP with `--ilp-fallback=error` need no METIS runtime. ILP with
METIS fallback still requires the selected backend. The Windows C comparison
uses `msvcrt-v1`; it does not represent every native platform.

Review shows configured, effective and unused backend information without
loading C. Selecting the runtime-loading checkbox authorizes C loading for Run;
leave it off for methods that do not need C. An unavailable required backend
refuses instead of falling back.
The selected native profile survives Save/Open and complete projects. Changing
it retires the review and current exports. A selected profile is a comparison
claim, not authentication of a desktop binary. Older saved inputs that omit it
default to portable core; an old C-loading choice does not infer a native profile.
Ship the matching UI and WASM together; older engines may reject the new field.

1. Select the raw files and their state/year claims. Supply the desired State
   arguments as a JSON array. The panel adds adjacency/COI flags from selected
   filenames. Include `--seed` and its decimal string to reproduce a run.
2. Review. The native parser prepares and checks the request without
   partitioning. The resolved seed and settings are visible.
3. Run the reviewed inputs explicitly. The worker uses native build/retry
   semantics. The accepted seed is shown, including the full unsigned 64-bit
   range. Edits and cancellation retire the review and current result.
4. Download the RPLAN, physical RCTX or complete run record. Matching WGS84
   GeoJSON can display assignments by exact unit ID. Display polygons do not
   supply native report geometry or prove the origin of the source data.
5. Save inputs to retain selected raw bytes, flags, metadata and optional display
   polygons. Open saved inputs is inert: Review and Run are required again.
   This panel's inputs-only file is separate from a complete project. The normal
   **Save project**, **Save workbench archive** and autosave flows now retain this
   panel too: draft settings, selected sources, display polygons, saved run data
   and engine identity. Open/Restore never executes or grants current exports.
   **View saved run (unverified)** can display retained assignments and maps;
   Review and Run are still required for current accepted outputs.

The optional project field `freshNativeStateSession` has its own versioned,
strictly typed envelope. Invalid draft arguments can be saved as bounded text;
native admissibility remains a separate explicit check. Older projects remain
readable. Older application versions may reject projects using the new field.
Saved-result viewing uses the retained-file 64 MiB byte bound and the existing
4,000,000-value workspace JSON budget, with depth, duplicate-key, unsafe-property
and exact-integer checks. Retained record bytes can be downloaded separately.
Ordinary project/autosave limits remain 25 MiB; larger sessions use a workbench
archive. The full-state examples below do not establish arbitrary device capacity.

The fresh Rust operations are `prepare-fresh-native-state-run` and
`run-fresh-native-state`. Both accept the same strictly typed input. Source names
and argument paths are metadata; the browser does not open filesystem paths.
The raw native loader and shared native plan/context factory preserve physical
Boundary edges, native unit-kind fallback, ordinal output labels and the
original present GEOID index. Absent or sparse source IDs are not invented for
COI weighting or county reporting. Generic Census/custom input checks remain
separate and unchanged.

Exports retain actual source hashes and versioned browser request/executed-request
hash roles. They do not invent a native original replay recipe digest. The
run record distinguishes the engine's Rust input identity from the exact
JavaScript input JSON hash and optional display-geometry text hash. These are
not signatures or upstream authenticity proofs.

Raw adjacency is bounded at 64 MiB; individual companions are bounded at
16 MiB and the prepared graph at 8 MiB. Selected display geometry is bounded
at 64 MiB. Combined saved inputs must fit 128 MiB to be reopened through this
panel. These are parser limits, not established full-state capacity guarantees.

The scoped implementation evidence includes four shared Rust tests, twelve
actual old/new native CLI comparisons, the previous Unicode county-prefix
panic correction, twelve fresh host cases and fifteen host refusal controls.
Both WASM profiles pass the maintained twelve fresh cases and fifteen refusals
using `metis-core`. Both actual Chrome panel packages pass twelve save/open/map
workflows with two explicit runs each, plus raw-file selection, cancellation,
inert malicious-context refusal, mobile control sizing and real workbench
registration. Display polygons in this fixed corpus are synthetic and are used
only to check rendering. No C-sidecar execution is claimed by these cases.

Broader fine/block/custom/native32 acceptance and optional native report/demographic/geometry
observations, the broader original project/state/national/lifecycle/capacity/
authenticity acceptance and release validation remain open. Scoped project
integration passes twelve complete workbench workflows per profile, ordinary
and plain/compressed archive round trips, actual autosave/restore, delayed-worker
Open refusal/cancel recovery, hostile retained data and atomic malformed input
refusal. See [the persistence review](../../signals/roles/check/fresh-state-projects-roles-check-2026-11-03.md). No original
acceptance parent is promoted by this feature. R18's full build remains failed;
its missing timestamped development CLI reference was diagnosed and corrected,
with seven focused cases and 22 recovery boundaries passing. This is not a
completed amended full build or a deployed release.

See the [parity plan](wasm-desktop-parity-plan.md) and
[role review](../../signals/roles/check/fresh-native-state-roles-check-2026-11-03.md).
Retained implementation evidence: `target/fresh-native-context-implementation-20261103/`.

Selected native64 BG/tract and adaptive source workflows, explicit sampler/C runtime
choices and physical/coarse hash roles are covered in the [multiscale guide](wasm-fresh-native-multiscale.md).


## Full-state projects and saved viewing

Additional NC/CA congressional tract checks retain requested seed42,0.5% tolerance
and100 iterations. NC accepts42; CA accepts46 under native retry order. Actual
State CLI, host and both browser profiles agree on the completed plans. The
selected real polygons are display inputs, not native report geometry or source
authentication. The panel uses selected filenames instead of filesystem path
labels; original files are never opened through those labels.

NC fits ordinary projects/autosave. The larger CA session needs **Save workbench
archive** and **Open workbench archive**; ordinary projects/autosave keep their
25MiB bound. Keep a downloaded archive for a portable backup. After Open, **View
saved run (unverified)** can draw the saved map without a worker. It uses bounded
large-workspace JSON parsing; malformed/deep/over-budget data still refuses.
Current exports and follow-ups require a fresh Review and Run. These two states
do not establish all-state, all-resolution or device-capacity support.


Explicit native auxiliary absence claims are also available in the private candidate. A named, explicit action records a claim for this run; unselected files imply nothing. Both WASM profiles preserve native COI/economic/housing fallback and required demographic/partisan Run failure, inert Save/Open and source edit retirement. The Windows host gate separately covers sharing-lock failure/retry, without browser folder-grant coverage. See [the auxiliary guide](wasm-fresh-native-auxiliary-inputs.md). Current full WP-01-WP-10, amended complete builders/release and all original parent acceptance remain open.
