# Config CLI creation and validation contract

Date: 2026-10-27. Status: selected-source creation/validation implemented with
actual CLI/browser evidence. Config parents and formal acceptance remain partial.

Native `config new` maps its CLI defaults and empty option strings into the
shared template renderer. The CLI exposes no engine or fixed-seed argument for
creation. The browser's explicit engine/seed fields are extensions; choosing
native defaults preserves omitted engine/seed declarations. Generated YAML is
not a plan execution, and compositor validity is not runtime/capacity evidence.

The actual CLI writer creates output parents, refuses an existing file without
`--force`, overwrites when forced, and returns YAML without writing when
`--dry-run` is selected. The browser generates/downloads exact native
YAML bytes. The subsequent [selected-root publication workflow](wasm-config-publication.md)
implements the bounded destination lifecycle with separate native-reference and
real browser storage checks. A download or imported result alone grants no write.

Native `config validate` reads the original UTF-8 file and invokes the open
compositor. The browser's optional court filter comes from a separate native
build preflight, not a `config validate` CLI flag. It remains separate scope and
does not provide legal certification.

## Original YAML bytes

An actual CLI/UI-boundary probe found that generic browser decoding consumed a
BOM before validation. The pinned native parser's observed behavior depends on
the prefix: the tested BOM before `name:` refuses, while a BOM before the leading
generated comment is accepted and included in the file SHA. This is an observed
contract of the current native implementation, not a claim about all YAML parsers.
The old browser both accepted the native-invalid named-prefix case and changed
the identity of the accepted comment-prefix case.

`config-files.js` now uses the explicit `exactNativeUtf8Text` alias, preserving
the same byte sequence already used by the native JSON reader. Generic text
decoding and the existing JSON/ceremony APIs are unchanged. Inspection, open
validation and the shared configured/TUI Run request builders retain original
YAML. The source remains nonempty UTF-8, at most 256 KiB with a plain basename;
projects retain raw base64, and Open does not run or restore authority.

## Verification

`test_config_cli_contract.mjs` runs a real CLI, not a compositor example. Six
creation variants compare exact written YAML. Each also tests collision refusal,
forced replacement with changed worker bytes followed by restoration, and
dry-run/no-write behavior. A separate actual CLI case proves that dry-run does
not create a missing parent, while normal creation does. Seven validation variants compare original input,
native summary/errors and SHA, including both BOM prefixes, CRLF, malformed YAML,
an unsupported engine declaration and the exact maximum u64 seed. The shared Run
and TUI request builders preserve both BOM variants; the invalid prefix refuses
before partition. These wiring checks do not claim new CLI Run execution scope.

The real browser checker opens retained results inertly, selects original YAML
fixtures through the file input, requires 13 explicit Runs and compares source,
native outcomes, generated YAML/summary downloads and original-byte Save.
Imported results leave downloads disabled. The native console uses a full path;
browser summaries use the selected basename. CLI `println` adds one presentation
newline; downloaded generated YAML must match the actual native file exactly.

Current evidence:

- Default native/WASM: `target/config-cli-contract/run-dXAXKG/evidence.json`;
  actual browser: `browser-25j8r3/evidence.json` under `target/config-cli-contract`.
- C-enabled native/WASM: `run-gfSL1V/evidence.json`; final packaged browser:
  `browser-L9BLyE/evidence.json` under the same root.
- Original plain-prefix divergence: `target/config-cli-probe/before.json`.
  Original accepted-prefix byte mismatch: `target/config-cli-probe/contract-before.log` and
  `target/config-cli-contract/run-ODYj8M/progress.json`.
- Complete project-handler, 61 native validation comparisons and 80 configured
  execution/70 CLI assignment oracles pass; logs are under `target/config-cli-probe`.

The new local overlay is `dist/wasm-config-cli-preview`, version
`ba7d888aa95685fd62fe732f4c80aebdac606757b5e3a14930fb6daac105aeba`.
It passes all 725 listed offline hashes/sizes and 1,234 dependency references.
The C browser checker compares served source identities with this package. All
previous package asset hashes remain unchanged; shared sealed files use existing
storage, while changed files and the new manifest are independent.

Both registered profiles and the final preview builder now invoke these checkers;
registration is not a new complete-builder pass. The previous full d1274b1f2ce2
ZIP remains unchanged and predates this Config reader fix. No new deployment,
whole-build or formal parent acceptance is claimed here.

## Subsequent publication workflow

The subsequent publication implementation adds original generated bytes,
default `configs/<name>.yml` or reviewed relative output path, native collision
refusal, explicit force, dry-run/no-write, fresh recomputation, readback hashes,
cancel/interruption receipts and inert Open. A saved flag, path or receipt must
never grant a root or write capability. Native `std::fs::write` is not an atomic
no-clobber publication primitive; the browser must disclose its own guarantees.
Other host/capacity/method, extension and clean-checkout/formal scopes stay open.


Subsequent complete pipeline: [Config full preview checkpoint](wasm-config-full-preview-checkpoint.md)
passes the updated builder, both registered profiles and final packaged C,
version `7c67474c5785`. This closes the previously pending bounded pipeline
condition; broader native permissions, matrices, clean-checkout and formal
parent acceptance remain open. Original component receipts stay historical.
