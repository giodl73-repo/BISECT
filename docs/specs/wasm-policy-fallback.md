# Policy override fallback in the browser

Date: 2026-10-27. Status: explicit source-chain workflow implemented and checked
against actual CLI outcomes; Policy parent and formal acceptance remain partial.

The Policy tool now offers **Use native override fallback**, with primary and
alias file inputs. On explicit Run, it tries primary, alias, then the embedded
database. A successful JSON parse selects that source immediately. A missing
location, invalid selected database shape or browser guard refusal does not cause
another fallback. For example, `{}` selects primary and then refuses lookup;
a zero-byte file fails parsing and continues to alias. BOM-prefixed JSON fails
parsing; invalid UTF-8 follows the native read-failure branch.

This follows the precedence in native `policy::load_policy`: primary is
`BISECT_LOCATION_POLICY`, alias is `BISECT_STATE_POLICY`. The browser consumes
explicit bytes or unavailable-source observations. It reads no process environment
and probes no arbitrary path. File import is preparation; no selection/lookup runs
until Run. A valid primary prevents alias decoding in the Rust selector.

The existing explicit single-file mode is retained. Enabling fallback adopts a
previously selected strict file as primary; disabling it selects embedded policy
and clears stale file-picker values. Query changes retain the source recipe.

## Original bytes, outcomes and authority

The optional closed `policyInputs.fallback` recipe contains `primary` and `alias`.
Each is null, `{status: "file", name, base64}`, or
`{status: "unavailable", name}`. Empty and binary files are retained as original
bytes. Input files are limited to 512 KiB each, names/query to 200 UTF-8 bytes.
The existing selected-policy JSON depth/node/entry/property guards remain in force.
Those guards refuse a selected source; they never silently choose another one.

`inspect-location-policy` accepts the optional fallback input only when its strict
source fields are null. Existing requests/results remain unchanged. Successful
lookup keeps the native JSON/table formatter bodies and adds a selection record:
selected role/name/SHA-256 and ordered attempted-source statuses, reasons and
available original byte hashes. Embedded source kind remains `embedded`.

After parse success, a lookup refusal returns a diagnostic
`bisect-location-policy-fallback-v1` result with `status: lookup-refused`, the
selection trace and lookup error. The UI visibly labels refusal and disables
policy JSON/text exports. This is a retained diagnostic, not successful policy
lookup or plan computation. Successful JSON download differs from CLI stdout by
the one console `println` newline; native table bytes match exactly.

The source-selection table is visible for both success and lookup refusal.
Generic result/project export retains the diagnostic and recipe. Imported results
do not restore fresh export authority. Unavailable-source observations are data,
including after Open; they are not authenticated read failures or live file grants.
No source hash establishes legal accuracy, source authenticity or certification.
Inspection does not apply policy to a plan.

## Read lifecycle

File reads lock conflicting controls and Run. Read failure records an unavailable
observation, without persisting a platform exception/path. Over-limit or changed
files refuse before replacing the earlier recipe. Cancellation retires the pending
callback and restores controls; it does not claim to abort underlying OS IO.
Late data cannot overwrite an opened project or release a newer read's busy state.
Reentrant handlers return before acquiring another operation's cleanup ownership.
Open/Save restores bytes and observations only, with no automatic WASM Run or source
selection. No file handles or environment capabilities are serialized.

## Verification

Fresh default and C-enabled Rust/WASM builds retain their exact source inputs in
`target/policy-fallback/build/source-inputs.json`; the runner checks them again
after both builds. Development Rust unit checks exercise precedence, parse/read
refusal, embedded selection and stopping on valid JSON. Final artifact acceptance
uses the following actual CLI and browser checks against the built binaries.

Per profile, `test_policy_fallback_wasm.mjs` compares 15 real CLI source-chain
cases, successful JSON/table bytes, native lookup failures and byte identities.
It also replays 68 earlier actual CLI cases to check strict-mode compatibility.
The reference is passed explicitly; registered gates use their newly produced
Policy CLI record rather than a fixed historical temporary directory.

Current evidence under `target/policy-fallback`:

- Default: `run-a8onfP/evidence.json` and real browser
  `browser-XP4m0w/evidence.json`.
- C-enabled: `run-oAfDmM/evidence.json` and final packaged browser
  `browser-EcL2cK/evidence.json`.

Each browser checker performs 15 explicit Runs through real file inputs and the
actual worker. It checks source-selection/refusal tables, native formatter
downloads, inert original-byte projects, mode adoption/reset, controlled unreadable
files, two delayed/cancelled reads, reentrancy and late callbacks after Open.
Controlled IO faults do not claim actual OS permission or exhaustion acceptance.
The new C engine separately passes 52 native assignment matches/four matching
failures; the complete project-handler regression also passes.

The final staged overlay is `dist/wasm-policy-fallback-final`, version
`66cade656ea4a2e0283ae7476118404cae6ea66e3316f51506dcfd88f8e58d8f`.
It contains 733 listed assets, 633 shell assets and 50 state/year packs, with
1,244 local dependencies across 312 toolkit files. The packaged C browser checker
compares served modules with these staged bytes. Earlier overlays and the complete
7c674 ZIP remain unchanged. Both profile and final preview builders register the
new checks; registration is not a new complete-builder pass. Broader host/capacity,
ambient source discovery, clean-checkout and formal parent acceptance remain open.


## Post-freeze retention pointer

`target/policy-fallback/closure.json`, SHA-256
`428ec13528afc7a9b61df92189609aa03b36fdcf5da322dfaad3d253279eca45`,
retains 2,005 mappings /1,310 blob identities with 66,520,319 new bytes.
`target/policy-fallback/readback.json`, SHA-256
`63fdc021c0080425b82621dbcf206efe29781a2cfac6b5f415335878dc157534`,
independently verifies all mapped blobs, the retained generic C test source and
the unchanged earlier ZIP. The snapshot retains the pre-pointer support text;
original execution receipts were not repinned to later source or documentation.
Older nested historical retention gaps remain historical and are not repaired
or reclassified here. No new whole-builder, deployment or full-parent acceptance
is claimed.
