# Native location Doctor snapshots

Location preflight now offers an optional **Use native location Doctor snapshots**
mode. The advanced controls remain collapsed until enabled. Select primary,
alias and/or manually supplied home policy files, optional manifest sources,
then review outputs and workspace roots and explicitly Run. The output prefix
is display text only. Existing explicit policy/RCTX preflight stays separate.
The native diagnostic never generates a plan or changes Run settings.

The new Rust operation `diagnose-native-location-workspace` uses the shared
registry lookups and tolerance function. It reproduces the actual native command:

- Registry policy tries primary, alias, home, then embedded; parse success stops
  fallback even for a scalar or unknown location. Unknown location/year exits
  before count/tolerance/path observations.
- Tolerance independently tries primary, alias and embedded, never home.
- Congressional manifest fallback uses the native typed manifest with read-only
  `u64` district values. Primary configuration stops the manifest chain even on
  read/parse failure; home is used only when primary is absent. Policy counts
  win. District strings retain values above JavaScript's exact integer range.
- Census adjacency tries V3 then V4. Block-group requests try both bg paths
  before tract fallback. Other resolutions, including block, use the native
  tract filename. A Census template is used only after all conventional paths
  fail. Missing conventional paths without a template produce the native
  resolution warning, not an invented missing-file result.
- Custom relative templates use the selected workspace root. TIGER checks the
  outputs V3 directory for any entry, including an unrelated file/subdirectory.
  File/directory presence remains distinct from readable/valid graph or geometry.

The full native stdout is shown and downloadable as text; JSON retains source
selection/refusal attempts and observations. The visible source summary separates
registry, tolerance and manifest selection. Native stderr is retained in tests;
typed input attempts do not forge platform-specific stderr warnings.

## Workspace and project boundary

Folder review enumerates names/kinds only; it never reads adjacency, TIGER or
other artifact bodies. Supported snapshots contain at most 20,000 portable ASCII
relative entries and 8 MiB of path text. Live review stops beyond 32 levels.
Windows ASCII case comparison matches the current 64-bit native reference.
Aliased paths, missing directory ancestors and traversal refuse. Query paths
outside this domain, or roots not reviewed, return an explicit unverified result
with null reference CLI status. They are not reported as native absence/failure.

Source bytes are bounded at 512 KiB per policy and 2 MiB per manifest. Empty,
BOM and invalid UTF-8 bytes are preserved for native selection semantics. A failed
browser read or **Record unavailable** creates an input observation, not proof of
a live native path failure. Home files are selected manually; no environment,
actual home directory, arbitrary path, URL or network resource is probed.

Open/Save retains bytes and metadata snapshots, never handles or read authority.
Open is inert; fresh export requires explicit Run. Native code 0 can include
warnings/missing artifacts and does not establish legal readiness or source
authenticity. Changes retire exports; invalid drafts block Run. Read ownership,
cancellation, retryable file pickers and revision checks prevent late A from releasing B or overwriting
Open. A changed project retires pending ownership immediately; it does not wait
for the old IO to resolve. Native aliases/case-sensitive field text survive imported dropdown values.
Folder/data snapshots describe supplied state, not the current native filesystem.

## Evidence

Both newly built WASM profiles pass 43 actual CLI stdout/exit comparisons and
43 workbench/worker Runs. CLI fixtures isolate HOME/USERPROFILE and overrides
inside owned directories; they never inspect the user's real home policy.

| Profile | Native CLI | Final browser |
|---|---|---|
| Default | `target/doctor-workspace/run-RD9iKS/evidence.json` | `target/doctor-workspace/browser-uvyQTc/evidence.json` |
| C-enabled | `target/doctor-workspace/run-E4RbIm/evidence.json` | `target/doctor-workspace/browser-oiFdWQ/evidence.json` |

Coverage includes source precedence and early stops, home/tolerance divergence,
manifest refusal/large counts, V3/V4/bg/tract/custom paths, Windows case matching,
directory-as-adjacency, TIGER file/empty/junk/subdirectory, chamber aliases/case,
year rules and nesting. Console comparison adapts separators only in adjacency/
TIGER path lines. The collector's body-read counter remains zero.

The browsers select actual source files, review real OPFS roots through an
injected picker, compare exact requests/results and text/JSON downloads, and
exercise mode/prefix edits, invalid drafts, A/cancel/B/reentrant/late-A/Open/
late-B. An actual OS picker grant is not claimed. Native link/permission and other
OS-specific existence behavior is not established by these fixtures. Boundary records
`target/doctor-workspace/boundaries-Abk4Pp` and `boundaries-1o4CYb` also refuse
malformed imported paths and failed/cancelled enumeration. An owned native
absolute-path success is retained as an explicit unsupported browser domain.

Fresh build source identities are retained under `target/doctor-workspace/build`
and checked after both builds. The C module separately passes 52 native assignment
matches/four matching failures. Project regression passes. Prototype dropdown and
package-binding observations remain diagnostics; an early C browser run bound the
previous manifest during staging and is not accepted as final package proof.
The final checker requires an unchanged manifest from start to finish.

The staged overlay `dist/wasm-doctor-workspace-sealed` has version
`e1109bde436c47943a9675d53369c5bf4bdb93b75672b2d8cd03a0fd7033c58a`, 741 assets, 641 shell
assets, 50 state/year packs and 1,251 local dependencies across 316 toolkit files.
The laboratory landing page and all workbench aliases are checked. Both builders
register CLI, boundary and browser checks; registration is not a new whole-builder
pass. Wider path/OS/capacity matrices, wide plan-manifest metadata, clean-checkout,
deployment and formal full-parent acceptance remain open. Earlier missing native
location-workspace notes are superseded only for this selected-snapshot scope.

The compiled embedded manifest content is additionally checked after both builds
against current `data/manifest.json` bytes in
`target/doctor-workspace/embedded-manifest-identity.json`. This supplements the
prebuild Rust/policy pin list; it does not rewrite that list or assert native
executable identity from a matching WASM data hash.

## Retention pointer

`target/doctor-workspace/closure.json`, SHA-256
`ec2ce210df8f182373dab125e814e4d4b86e8402ffbb40dc592d4ab1028417ce`,
retains 2,395 mappings /1,483 identities. Independent
`target/doctor-workspace/readback.json`, SHA-256
`8a37094fa1d304e5ce54a0f6f86cc1d853927748653c16b2143b67964440398d`,
verifies every mapped blob and the unchanged earlier release ZIP. The snapshot
retains the pre-pointer text. Its local store contains 67,757,107 logical bytes,
including 50,737,347 bytes linked from immutable owned WASM artifacts. Allocation
counters describe the resumed invocation and are not physical storage estimates.

One verified generated-symbol cache deletion recovered 188,428,288 measured
physical bytes after the reserve stop; sources, executables, test evidence and
archives were unchanged. Use new output/build directories for sealed artifacts.
Final project, ledger/practitioner, dependency, offline and builder syntax checks
pass. Both profiles also pass 15 Policy source-chain /68 strict backward cases
and 42 Fetch planning/handoff comparisons without acquisition. No new full
builder, deployment or formal parent acceptance is claimed.
