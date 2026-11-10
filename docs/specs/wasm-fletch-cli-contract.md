# FLETCH CLI report contract

Date: 2026-10-27. Status: bounded implementation and actual native/browser
evidence; parent and acceptance-v1 remain open.

`bisect fletch-sources` and `bisect fletch-cache-index` build the selected source
registry, write their reports, and only then apply the optional `--gate`. A
failed cache gate can therefore produce a valid diagnostic report. Missing
expected sources pass that native gate; unexpected or unverified indexed rows
fail it. Saved verification flags describe manifest evidence, not fresh file
hash checks. The separate browser object-verification workflow retains its own
contract.

The browser's explicit selection/Run/download path preserves these report and
gate outcomes. It does not discover native environment/default cache paths or
automatically replace an output file. Source CSV targets use portable slashes;
the Windows CLI emits backslashes. Only that target-path representation is
allowed to differ in the comparison. Source readiness is not acquisition or
plan validation.

Actual CLI comparison exposed two cache artifact mismatches. The browser report
download appended a newline absent from `write_fletch_cache_index`; it now emits
the exact native pretty-JSON bytes. The manifest reader silently stripped a
UTF-8 BOM, accepting a native-invalid source. It now preserves the BOM so the
native parser refuses it; the original bytes remain in the saved project and
worker failure request. Generic source readers and other report writers are
unchanged. Exact u64 values remain inside native report text.

The tracked `test_fletch_cli_contract.mjs` creates five source and eight cache
cases from tracked `data/manifest.json`, runs a copied actual CLI, and compares
original report bytes and exit outcomes with a copied actual WASM. It covers
single/all vintages, optional types, ignored/duplicate states, missing expected
entries, verified/unverified/unexpected evidence, gate on/off, u64 above the JS
safe range, BOM and malformed JSON. Native output before failed gates is checked.
`test_fletch_cli_browser.mjs` opens the original projects inertly, requires 13
explicit Runs through the real workbench worker, compares report downloads,
retains native failure inputs and verifies inert Save. External network requests
are refused in the isolated test context.

Evidence under `target/fletch-cli-contract`:

- Default native/WASM: `run-rNUNvl`; actual browser: `browser-FVg6Wa`.
- C-enabled native/WASM: `run-qQo4Ye`; final packaged browser: `browser-kpUdJR`.
- Old reader/native BOM divergence: `bom-before.json`; original output mismatch
  logs: `target/fletch-cli-before*.log`. Failed harness iterations are retained.
- Existing shared cache comparisons and the full project-handler regression pass
  after replacing the stale newline expectation with the actual CLI contract.

The new package `dist/wasm-fletch-cli-preview` passes 1,233 local dependency
references and 725 exact offline asset hashes/sizes, including 625 shell files
and 50 state/year packs. Its version is
`aad673d7e23b2ce807454fee93bdcf3618e5f3ff04a5a21b0d71531b2f560cf6`.
The packaged-browser checker verifies served source files against that package.
This is a JS-only overlay of the previous complete preview; engine bytes retain
their registered identities. Both original packages stay unchanged.

Both registered profiles and the final C preview builder now invoke these
checkers. Their scripts parse and the component invocations pass. The original component
checkpoint did not rerun the entire modified builders. No new whole-
pipeline, clean-checkout, all-state execution, automatic filesystem lifecycle,
provider authenticity or full desktop parity claim follows. Hashes identify
bytes and do not certify an election. Nothing is deployed.


The subsequent [complete preview checkpoint](wasm-report-format-full-preview-checkpoint.md)
passes both registered profiles and all final-package gates in run
`47641ab8-cb23-4d47-b138-9c9f4759d1dc`, with a versioned verified ZIP. It closes
this bounded full-pipeline condition; original component receipts/package remain
unchanged and broader native lifecycle/host/clean-checkout/formal scope stays open.
