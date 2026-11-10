# Native label/year summary delivery

Date: 2026-10-23. Status: fresh native/default/C engines and full dual-build
regressions pass. The final local package passes all 21 verification stages,
including native summaries, actual PDFs, TUI/composed Verify, loaded C,
workspace, project and mutation checks. Immutable evidence and scoped review
registration follow the execution-byte freeze. Whole native filesystem,
host/capacity, WP-07/WP-10 and full-parity acceptance remain open.

## Native contract

`analyze_label.rs::run_label_report` validates the label/registry/analyzed year,
requires the analysis directory, reads only its `index.json` if present, writes
one `{label}_{year}_report.html` or `.json`, then writes a report index and marks
the registry reported. Missing analysis index is a successful empty-state
fallback with an empty upstream SHA chain. Invalid JSON is an error. The raw
index is arbitrary native JSON, not necessarily a typed `AnalysisIndex`.
Per-state commission report sources are not required for this command.

Native HTML and JSON renderers now live in `bisect-report::label_summary`, and
the desktop wrapper calls that shared implementation. Original desktop output
and source/binary identities were captured before editing. Source-owned golden
fixtures retain exact original bytes. JSON creation time and report-index
creation time are the only predeclared runtime comparison allowances; all
other bytes, filenames, registry transitions, exit states and diagnostics are
compared independently against the retained original executable. Full original
CLI comparison is an explicit integration tier; missing binary is not a pass.

The original native renderer interpolated unescaped text. A new adversarial
regression fails on that original implementation. The shared renderer now
escapes label, year, state and status text for both targets; JSON preserves the
original values. This is an explicit shared security fix, not a WASM-only parity
claim. Fresh native/default/C builds and actual HTML/download/browser checks
pass after that shared fix. Browser preview remains script-free and
source-bound; imported HTML or JSON never grants execution or write authority.

## Native core and browser adapter

`run-native-label-summary` executes in the fresh Rust/WASM engines with explicit label/year,
raw registry and optional raw analysis index, directory-presence observation,
format and generated time. It checks native prerequisites, uses the shared
renderer, returns original default-layout report/index/registry file strings
and hashes, and writes no disk files. Its output distinguishes `native-summary`
from the existing per-state commission reports. Both engines pass thirteen CLI/file/failure comparisons. The explicit native-summary UI and read-only intake are source-tested; final-package acceptance remains separate.

The worker adapter adds original registry ordering, including repeated years,
and safe relative output overrides with native last-writer and partial-prefix
semantics. This portable preparation does not close unknown existing-output
collisions, native OS locking/atomicity, reviewed summary installation, other
hosts, original-state capacity or release acceptance.
Directory presence is a selected-snapshot observation, not a saved-path grant.
Registry completion records command completion even when summary state statuses
include failures; it is not successful analysis or official certification.

## Acceptance and remaining scope

Delivered checks include original/rebuilt CLI comparisons, fresh host/default/C
engines, exact native files/failures, readonly source-preserving UI, escaped
HTML/precision checks, native year/output sequencing, final-package regressions
and verified immutable evidence. Preserve those gates for subsequent changes.

Reviewed live summary installation remains a required separate child: current
ZIP preparation does not grant root authority or resolve unknown existing output
collisions, native OS locking/atomicity, absolute paths, other host/API behavior
or original-state capacity. Full parent, WP-07/WP-10 and release acceptance stay
open. Native reported status is command completion, not official certification.


## Delivered UI and sequencing

The report panel now has an explicit desktop-summary type while retaining the
commission-report default. Read-only selected-root intake collects the original
registry and analyzed-year indexes, preserving absent directory versus missing
index. Imported source snapshots stay inert. One explicit year or the registry's
original all-year order is used, including repeats; generation stops on the first
failure and keeps a completed prefix. Relative output overrides, including the
selected workspace root, retain native last-index-writer behavior. If an output
overwrites a later analysis index, subsequent work consumes those new virtual
bytes as native CLI would. No actual repository files are written by this flow.
Unknown existing output collisions, native OS locking/atomicity, absolute paths
and automatic installation remain separate filesystem-adaptation obligations.

Source CLI comparisons for six all-year/output cases and actual UI tests pass
on both fresh post-security-fix engines. The new batch result distinguishes command completion from state
analysis outcomes. Save/Open retains original source bytes and settings; Open
requires fresh Run before export/preview. Input/output edits retire results,
and superseded folder reads cannot replace a new project's source/status.


## Retained failures and evidence boundaries

The first package attempt correctly failed because a copied catalog still named
the old engine. The new staging helper updates only the served-engine identity;
historical run producer identities and state-pack hashes remain unchanged.
The next candidate passed summary/report checks but exposed an old TUI expected
fixture containing the previous loaded WASM identity. That fixture is retained;
expected receipts are regenerated by running the actual new engine, not by
patching hashes or weakening equality. A fresh verification pass over unchanged
staged runtime bytes preserves both earlier failures.

A portable API-absence case was added after the original default browser gate.
The original default gate record and original browser root remain retained;
a separate affected-browser rerun binds the current script and engine. This is
explicit follow-up evidence, not a claim that the earlier run covered that case.
The C gate executes the extended case directly. Original CLI comparison remains
an explicitly declared integration tier with a digest-pinned retained binary;
small goldens and ordinary current-CLI cases regenerate from source-owned inputs.


## Tested local package

Package: `dist/wasm-native-label-summary-validated-v2`.
Execution record:
`target/native-label-summary-delivery/package-1792804770898803600/package-evidence.json`,
SHA-256 `2677f39eddf905a69f9770341596b259916fa2c7f5e48fd85ef4fbd793722400`.
Offline version:
`0f0ddf1121b55bc730185ed7954d11751b92c870c56e769736f754a2d3cb7069`.
Default WASM: `1652ccf3bdd6d1d7277117b800af71147c0c7ba6e873e1e6a876fcbcc0d7905c`.
C-enabled WASM: `e964b062ad562483040fc62310d94f3a6d514cc72327fb9dd0ad250859358544`.

Seven shared-renderer/security tests pass. Each fresh engine has thirteen
actual CLI direct cases and six all-year/output comparisons, including a later
analysis index changed by the chosen output. Original/rebuilt CLI comparison
retains six unmodified baseline cases. Both source browsers and the final
packaged browser exercise actual workers/owned OPFS, readonly intake, source
precision, native order/fallback, escaped downloads, restricted preview,
portable saved-source execution without directory API, inert Open and late-read
protection. The final package includes fresh engine bytes and current UI;
historical catalog run identities and source packs remain unmodified.

The initial catalog mismatch, stale TUI fixture and stale C-batch fixture are
retained failures. The successful record explicitly re-verifies unchanged staged
runtime bytes after actual fixture generation, preserving earlier failure
records. Identity assertions remain exact. Source-tree injected pickers and
Chrome API-absence simulation do not establish OS picker behavior or other
browser/host support. Prepared summary ZIP delivery does not itself install a
live repository or resolve concurrent output collisions.


## Scoped evidence closure

The verified snapshot is
`target/native-label-summary-delivery/package-snapshots/wasm-native-label-summary-validated-v2/snapshot.json`,
SHA-256 `eaa99fdca9ca5cce61ce4eb19e4f09dd8d821a1a36e0a21da93b0b4ccb16def6`.
It retains 3,187 mappings / 1,360 unique identities, with 789 new blobs /
211,030,165 bytes for 697,727,821 logical mapped bytes. Its inherited base is
an explicit dependency and must remain resolvable. All referenced bytes verify.

The [eight-role review](../../signals/roles/check/wasm-native-label-summary-roles-check-2026-10-23.md)
approves this local scope with carried conditions. The archive preserves the
execution-stage document; final closure links, support/ledger registration and
source-inventory re-ranking follow the freeze and change no runtime bytes.
The capability child remains evidence-partial and `acceptance_v1` is false.
Reviewed live native-summary installation, unknown output collisions, OS/API
coverage, capacity and full parent/release acceptance remain open.
