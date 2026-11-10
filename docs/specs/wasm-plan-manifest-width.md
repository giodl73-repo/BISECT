# Read-only plan manifest count width

Doctor's selected-plan reader now accepts the native 64-bit unsigned domain for
all thirteen pointer-sized manifest fields. The shared schema is generic over
count type: `PlanManifest` remains the native/execution `usize` alias, while
`ReadOnlyPlanManifest` uses `u64`. Doctor formats counts as exact decimal strings
and retains original manifest bytes. It never uses these values to allocate a
graph, partition, replay or increase execution limits.

The fields are num_districts, seats_per_district, total_seats, split_seeds,
spectral_iters, ilp_solve_report_count, flip_steps, flip_visited_count,
flip_selected_plan_rank, burst_length, n_bursts, selected_burst_idx and n_units.
Defaults, missing/optional fields, unknown fields and duplicate-field rejection
remain shared with the native schema; this is not a JSON-number sanitization pass.
Invalid label manifests still use adapted diagnostics with null reference status.
Native completion code 0 can still include missing/failed analysis checks.

## Evidence

Both fresh default/C-enabled modules pass 27 actual native CLI cases and 27 real
workbench/worker Runs. Cases cover each wide field, boundaries around 2^32 and
2^53, u64::MAX, all fields at maximum together, defaults, negative/fractional/
overflow/string values and duplicate fields. Full successful all/label console
text matches, with portable path separators only in the label Dir line. Directory
inventory order and presence sets are canonicalized for source reload; source
bytes, hashes, values, requests/results and report text remain exact.

| Profile | Actual CLI | Browser |
|---|---|---|
| Default | `target/plan-manifest-width/run-M1hoJE/evidence.json` | `target/plan-manifest-width/browser-i4pirY/evidence.json` |
| C-enabled | `target/plan-manifest-width/run-QMDw7f/evidence.json` | `target/plan-manifest-width/browser-9xMexP/evidence.json` |

The existing eight-case Doctor command suite also passes per module, with its
former unsupported-width observation now a matching successful report. Original
home-loader and dispatch observations remain separate. Projects reopen inertly;
explicit Run enables fresh text/JSON exports. Folder selection uses injected OPFS
handles, not an actual OS permission grant. No map/audit/RPLAN body is interpreted.

Three native schema tests pass for identical serialization/defaults, full unsigned
range and rejection behavior. Full native CLI `cargo check --release` passes.
The first test build exhausted disk while compiling dependencies; its log is
retained. Verified generated incremental/symbol cache cleanup preceded the
successful retry with incremental compilation disabled. No native executable,
source, evidence or release archive was deleted. The new C module separately
passes 52 native assignment matches/four matching failures.

`target/plan-manifest-width/build/source-inputs.json` retains the prebuild WASM/
policy input list. Shared report files were additionally hashed during the default
build in `shared-source-review.json` and verified unchanged after both builds and
native checks. This supplement is not relabeled as a prebuild capture or complete
reproducible-build proof.

`dist/wasm-plan-manifest-width` retains the existing UI, 741 assets, 641 shell
assets, 50 state/year packs and 1,251 dependencies across 316 toolkit files.
The packaged C browser check matches staged source bytes. Both builders register
the new width checks; no new whole-builder, clean-checkout, deployment or formal
parent acceptance is claimed. Historical unsupported-width receipts remain unchanged.

## Other reader gaps remain explicit

`audit_manifest_width_readers.mjs` records four current WASM differences:
certificate verification, Verify prechecks and assignment discovery reject wide
metadata before their main work; multi-label summary silently marks its manifest
unavailable. Evidence is in `readers-upUVNh` and `readers-dCHnm9` under
`target/plan-manifest-width`. This is a source/WASM audit, not native Verify/Report
orchestration acceptance. These gaps are separate required Verify/Report ledger
children; Doctor success does not close them.

Interchange, improve, producer/binding and replay consumers require individual
review because counts may control execution or allocation. A PlanManifest use
inside state_report_binding tests is not classified as a production gap. Wider
path/OS/capacity matrices, actual folder grants and complete release acceptance
also remain open.

## Retained checkpoint

`target/plan-manifest-width/closure.json` has SHA-256
`c29d5083cf7b7fce287280934349d6220919f490e7c883393fe60fc995edd349`.
Independent readback verifies all 2,214 mappings / 1,475 blob identities and the
unchanged previous release archive. The staged package version is
`929203e2dc794f54eb864eb88f43a61d9abefe84dd146ebc3f6b192c47e246c7`.
Both new engine profiles also pass 15 native policy comparisons plus 68 strict
cases and 42 Fetch planner/handoff comparisons. Fetch acquisition is not exercised
by those comparisons. PowerShell and embedded Python builder syntax checks pass;
registration remains distinct from a complete builder run.

This pointer was appended after the immutable checkpoint; its retained document
copy intentionally precedes this pointer. Nothing was deployed.
