# Doctor manifest CLI contract

Manifest verification now preserves original UTF-8 bytes, including BOM and CRLF,
through Rust parsing and source hashing. This fixes the PP68 decoder class: BOM
JSON is native-invalid and must not become accepted by browser normalization.
The generic decoder used by other formats is unchanged.

The workbench offers structured comparison and Doctor text download. Text follows
native observations while explicitly identifying the actual WASM build. Native
executable metadata is never substituted. Missing/non-string versions warn with
comparison code 0; mismatch is 1; JSON parse failure is 2. These are reference
outcomes, not browser process exits. Recorded timestamps/paths/hashes are
informational. Neither matching versions nor literal hashes authenticate the
producer, original execution, sources, or legal compliance. Referenced adjacency
files are not opened or hashed, matching native Doctor.

Selection invalidates results before reading. Run and exports wait for completion
or explicit cancellation. A cancelled A cannot release B's read ownership;
reentrant reads refuse before ownership acquisition. Late callbacks after Open
cannot replace imported bytes. Save/Open are inert; exports require fresh Run.
Cancellation retires callback authority, without claiming physical IO abortion.

## Evidence and supported scope

`test_manifest_cli_contract.mjs` takes explicit CLI/WASM paths and pins an owned
native executable. Twenty actual CLI cases per profile cover arrays/scalars,
missing/non-string/exact/mismatched/untrimmed versions, optional field types,
Unicode, embedded/trailing CRLF, BOM, malformed/trailing JSON, duplicate keys and
1 MiB input. Full console bytes and exit status are compared. Only source-path
display, the explicit WASM heading/build block and its actual commit warning
differ; both build blocks are retained. Package versions must match first.
Embedded CRLF is preserved, not normalized.

The old-decoder counterexample records native code 2 versus old code 0 and
corrected code 2. Native empty, invalid-UTF8 and oversized outcomes are separate
guard observations. The browser preflight refuses these without inventing native
process output. Selected files support nonempty UTF-8 up to 1 MiB. Empty-file
execution remains missing through the WASM ABI. Arbitrary OS error strings,
unrestricted paths and actual OS permission grants are not accepted here.

| Profile | Actual CLI records | Real browser records |
|---|---|---|
| Default | `target/manifest-cli-contract/run-drCjqu/evidence.json` | `target/manifest-cli-contract/browser-gfVB9u/evidence.json` |
| C-enabled | `target/manifest-cli-contract/run-SrNrMH/evidence.json` | `target/manifest-cli-contract/browser-EBJeJD/evidence.json` |

Each browser selects all twenty actual files, performs explicit worker Runs,
compares exact requests/results and full text/JSON downloads, and checks original
project bytes and inert Open. Controlled A/cancel/B/reentrant/late-A/Open/late-B
tests callback ownership. The actual worker-loaded WASM identity is measured and
compared; native executable equivalence is not inferred. Prototype test failures
remain diagnostics, not accepted records.

`dist/wasm-manifest-cli-complete` has version
`a8b363875a8e6e7e39c4f15f4d506ec8f1b51877387d0931c55e6c881339a83e`,
737 assets, 637 shell assets and 50 state/year packs. The C browser binds served
modules to staged bytes. Engine bytes are unchanged. Both builders register the
new checks; registration is not a whole-builder pass. Doctor location workspace,
wide plan metadata, tutorial reconciliation, empty execution, clean-checkout,
host/capacity matrices, deployment and full-parent acceptance remain open.

The earlier 90a54 overlay and its closure are retained as a superseded prototype: its workbench alias was stale and its root landing page was replaced. Final acceptance additionally checks all three workbench aliases and preserves the laboratory landing page. The prototype closure is historical evidence, not an accepted final package.

## Retention pointer

`target/manifest-cli-contract/closure-final.json`, SHA-256
`54b9e7998be27d0575852cc56d31d591e199dd42a5f08d54a54cedcc9bc5364d`,
retains 1,841 mappings /1,187 identities with 5,637,023 new bytes.
Independent `target/manifest-cli-contract/readback-final.json`, SHA-256
`56126a4d7adce34b68c7dbbc9f3eef098fd5cf89973ca6f9d12bc60a5868490b`,
verifies all mapped blobs and the unchanged earlier ZIP. The snapshot retains
the pre-pointer text and labels the earlier prototype as superseded.
Project regression, 42 Fetch planning/handoff comparisons per profile, ledger/
practitioner validation, builder syntax, 1,248 dependency checks and all 737
offline asset hashes pass. No acquisition or new full builder is claimed.
