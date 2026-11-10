# Native State identity workflow — implementation in progress

This is the next WP-04 workflow after the fresh-engine national checkpoint.
It does not advertise `--state-name` execution support. The existing direct
prepared-run refusal remains in place, and accepted preview packages are unchanged.
The [five-role review](../../signals/roles/check/wasm-native-state-identity-roles-check-2026-10-26.md)
approves implementation planning and guarded inspection with conditions; the
complete consumer chain remains open.

Native `main.rs` applies `state_name` only when `adjacency` is present. Explicit
names are passed verbatim. Otherwise a registered policy name is lowercased and
spaces become underscores. An unknown code with adjacency falls back to lowercase
code without replacing spaces; the non-adjacency branch normalizes spaces, though
unknown locations are rejected earlier. The ordinary State path ignores an
explicit name. `resolve_state_name` in `bisect-core::state_catalog` now expresses
these exact branches and is consumed by the native CLI and WASM inspection.

The internal `inspect-native-state-identity` operation uses the actual shared
State parser and embedded policy. It reports requested/effective name, source and
ignored/effective override, preserves raw strings, and grants no path or execution
authority. An inspected unsafe name remains data. No named file is opened.
The operation does not validate the complete native location/year/district scope.

## Observed downstream consumers

| Native consumer | Name behavior | Browser mapping still required |
|---|---|---|
| `StateConfig::effective_label`, `runner/mod.rs` | Explicit label wins; default label lowercases the effective state name and replaces spaces. | Generate the same default label without changing the retained raw name. |
| Unlabeled writer, `runner/mod.rs` | `{year}/states/{state_name}` uses the raw effective name; explicit labels use the plans namespace. | Distinct checked output target and matching RPLAN alias; refuse unsafe paths before effects. |
| `check_cvap_availability`, `runner/mod.rs` | CVAP availability checks a filename containing raw state name. | Match the selected original source and native fallback/warning behavior. |
| Population source loading, `runner/support.rs` | VAP/CVAP loader lowercases and replaces spaces before composing its candidate filenames. | Keep this normalization separate from the raw CVAP availability check. |
| Demographic graph/analysis loading, `runner/mod.rs` and `runner/support.rs` | Multiple lookup sites interpolate the raw name. | Bind the correct source at each preparation/follow-up stage. |
| LODES/housing character loading, `runner/support.rs` | Receives the effective state name before invoking the named source loaders. | Trace each loader's own normalization and selected-source association. |
| Manifest adjacency and multiscale GEOID names, `runner/mod.rs` | Generated basenames use raw effective name; BG and tract suffixes differ. | Preserve source provenance and expected basename, without opening the recorded path. |
| Completion filtering, `runner/support.rs` and `bisect-core::state_catalog` | Canonical state-name namespace precedes historical lowercased-code namespace; incomplete markers stop fallback. | Apply reviewed effective identity to canonical guards and retain historical fallback rules. |

This table is a consumer trace, not evidence that the mappings have been
implemented. In particular, raw CVAP presence checks and normalized population
loader candidates must not be merged into one synthetic filename convention.

## Required work before browser execution exposure

1. Bind original flags and embedded/selected policy identity to a fresh workflow
   review. Distinguish effective custom name from ignored ordinary-State name.
   Preserve the 14-field direct compiler guard; strip a workflow field only after
   its effects have an explicit checked mapping.
2. Connect effective name to native default label and unlabeled writer namespace,
   including case and raw-name behavior. Portable browser paths must refuse unsafe
   names rather than sanitize them into a different native target. Explicit labels
   retain native precedence. Do not use state code as a substitute for custom name.
3. Reconcile demographics, VAP/CVAP, LODES/housing and generated adjacency basename
   consumers. Native normalization is consumer-specific. Preserve selected source
   bytes and make missing named dependencies explicit before execution where the
   native workflow requires them. Naming alone must not authorize file discovery.
4. Carry identity through completion/incomplete guards, collision review,
   Force/reset, export aliases, map/report follow-ups and directory installation.
   A changed name invalidates prior target/source review and grants. Imported
   projects restore no authority and remain inert until explicit verification.
5. Compare actual native CLI output trees and effective labels on known/custom
   codes, absent/explicit name, mixed case/spaces, explicit/default label, ignored
   override, changed binding and unsafe path cases. Check accepted assignments
   independently. Then exercise actual browser Run, Save/Open/replay, export,
   target/source refusal and recovery on both sealed profiles.
6. Run applicable registered/profile/package gates, role review, inventory and
   negative full-parity checks. Freeze source preimages, native/WASM identities,
   raw outputs and failures before adding scoped support evidence. Preserve the
   existing national snapshots; a later engine needs its own regression evidence.

This inspection foundation closes no full parent, host/API, matrix or release
acceptance requirement. It is not an alternative definition of `--state-name`
support. The implementation target remains the complete consumer/workflow chain.

## Inspection foundation verification

The shared naming-rule test and two adapter tests pass. The native CLI release
compile check passes. A freshly built default WASM passes nine actual ABI cases,
including ignored overrides, registered/unknown fallback, raw mixed-case/space,
Unicode, empty and unsafe names, parser refusal and the retained execution guard.
Raw outcomes are in `target/state-identity/wasm-1793052276097/evidence.json`.
Two failed test expectations (policy hash prefix and refusal display spelling)
are retained in separate logs; engine behavior was unchanged for the rerun.

Active native source authorities and direct-read traces were reconciled for the
shared-rule refactor. These are source-only records: support classifications and
historical execution identities are unchanged. The ordinary ledger passes.
Actual new CLI output-tree behavior, C profile, browser UI/workflow, wider native
consumers, final packages and immutable full-workflow acceptance remain open.

## Native naming and output-preview checkpoint

Both fresh default and C-enabled WASM builds now report effective native year,
raw state name, native default/explicit label and separate source basename
intents. The pure default-label helper is shared without filesystem features;
its native public exports remain compatible. A named-target inspector applies
portable component validation and existing incomplete/collision guards while
preserving case and internal spaces. Unsafe target components refuse without
sanitizing the original observation. ASCII portable target constraints remain;
Unicode names can be inspected but are not accepted as output components here.

The actual State command view shows name selection, label, relative writer folder
and recorded output-root intent. It explicitly says no folder was observed or
opened. Native year failures and unsafe proposed folders remain visible refusals.
Save/Open is inert; explicit reinspection recomputes the view. No flag is stripped
and `--state-name` remains in the 14-field direct execution refusal boundary.

`target/state-identity/output-preview-validation/evidence.json` passes both exact
staged previews, SHA-256
`d3713f11f16bd0bf71bdc6553eaea732c2191c1930451434e4e6283e05f8d8ac`.
Four actual two-district native CLI runs independently match raw output folders,
labels and manifest adjacency names on each profile. Nine ABI cases per profile,
seven actual browser cases per profile, eight current Rust tests, package
asset/offline checks and before/after package pin verification pass. The final
390px preview was visually inspected; separator wrapping retains the exact text.
The earlier preview and filesystem-feature compile failure remain retained.

The [six-role review](../../signals/roles/check/wasm-state-identity-output-preview-roles-check-2026-10-26.md)
approves this read-only local slice with conditions. Inspector tests are now
registered in laboratory and dual-profile runners. Full registered runs and
execution-workflow package acceptance remain pending; registration is not a run.
The earlier full national/timing packages remain separate immutable evidence.

The next implementation is to bind named auxiliary source choices and effective
identity to the original command, compiled export label, reviewed output target,
Force/reset, recovery and installation. This preview does not satisfy those
remaining consumer or full State flag requirements.
