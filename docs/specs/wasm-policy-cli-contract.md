# Policy CLI and selected-file byte contract

Date: 2026-10-27. Status: byte-preserving explicit-file inspection implemented;
Policy parent and formal acceptance remain partial.

The actual `policy` command dispatches to `crates/bisect-cli/src/policy.rs`.
It resolves case-insensitive codes first, then a unique case-insensitive name
substring. Test entries are available by explicit code but excluded from fuzzy
name matches. JSON uses the shared pretty formatter followed by one `println`
newline. Every non-`json` format string selects the native table formatter.
The browser offers both formatter outputs; JSON download excludes the console
presentation newline, while table download matches native stdout exactly.

## Original bytes

The old browser's generic decoder consumed a leading UTF-8 BOM from an explicitly
selected override. The actual native parser rejects that prefix. A custom XX
entry therefore appeared in the browser despite native parse refusal, with a
different source hash. The actual CLI reproducer is
`target/policy-cli-probe/before-1793149600146/evidence.json`.
The new actual CLI regression also fails before the fix, retained at
`target/policy-cli-probe/contract-before.log`.

`policy-files.js` now uses `exactNativeUtf8Text` for validation and request
construction. It preserves original UTF-8, BOM and CRLF instead of normalizing
them. A malformed selected source reaches the real Rust parser unchanged and
refuses. Valid source hashes include all original bytes. Raw JSON downloads
retain exact large integers; the browser's separate display uses decimal strings
outside JavaScript's safe integer range. Existing 512 KiB/UTF-8/plain-basename
and JSON complexity/property guards remain in force. Native input is not subject
to all of those browser resource constraints.

Save/Open retains original base64 and query data. Open and file selection do not
run the worker, apply policy to a plan, select another database or restore fresh
export authority. Explicit Run is required. No current legal accuracy, automatic
plan policy application or legal certification is established.

## Native fallback at the selected-file checkpoint

The CLI first tries `BISECT_LOCATION_POLICY`; a successful JSON parse wins even
when the alias exists. If reading/parsing fails, it tries `BISECT_STATE_POLICY`,
then the embedded database. The primary warning can say “embedded” before the
alias is actually consulted; tests retain the original text and observed result.
At that selected-file checkpoint the browser inspected one explicitly selected
source or the embedded database. The subsequent [fallback workflow](wasm-policy-fallback.md)
implements the source chain over explicit bytes/observations, with separate evidence.

For example, malformed primary JSON followed by a valid alias selects the alias
in the CLI. A malformed explicit browser file refuses. These outcomes are not
equated. That gap was recorded as the required `native-override-fallback` child. The
subsequent implementation now provides explicit source-selection/fallback
semantics; the Policy parent still requires broader acceptance. Browser environment/path discovery cannot be inferred from
a successful single-file inspection or from selecting embedded policy manually.

## Verification and package

`test_policy_cli_contract.mjs` runs a fresh retained copy of the actual executable,
not a renderer example. Per profile it checks 68 embedded/override/query cases,
including exact JSON/table bytes, large integers, both BOM refusal contexts,
CRLF, malformed and empty databases. Six further actual CLI observations check
primary/alias precedence, missing/invalid source fallback and non-JSON formatting.
Embedded and accepted selected-file source hashes are checked against raw bytes.

Current passing records under `target/policy-cli-contract`:

- Default native/WASM: `run-wIXI3P/evidence.json`; real browser:
  `browser-oyQvlw/evidence.json`.
- C-enabled native/WASM: `run-jSM2b5/evidence.json`; staged packaged browser:
  `browser-nkMFXq/evidence.json`.

Each browser checker performs ten explicit Runs, including actual source input
selection, original worker requests, native JSON/table downloads, fresh-only
exports and inert original-byte Save/Open. The existing Policy regression passes
63 native/WASM comparisons, 127 example-reference CLI checks and 17 rejections;
the complete project-handler regression also passes. Those older example checks
supplement the new actual command observations.

The overlay is `dist/wasm-policy-cli-preview`, offline version
`ef28c618c389af71595bb0cd96c30aa7b09e9c4ed7784b687ea4887e8bf3df61`.
All 729 asset hashes/sizes and 1,240 dependencies pass. The packaged C checker
compares served modules against this overlay. The earlier complete 7c674 ZIP
is unchanged and still contains the old reader disclosed in its support notes.
Both registered profiles and the final preview builder now invoke the new tests;
registration is not another complete-builder pass. No new ZIP or deployment is
claimed for this overlay. Broader host/capacity, clean-checkout and formal parent
acceptance remain open.


## Post-freeze retention pointer

`target/policy-cli-contract/closure.json`, SHA-256
`a30a81270734d217da7d350b2500a13d8374983dba4d12ed5d796aa4adbdb6dd`,
retains 1,235 source/evidence/package mappings with 8,188,272 new blob bytes.
`target/policy-cli-contract/readback.json`, SHA-256
`96330f81b52eda4c844285171f309c6ff521ee17645a7a822556727cdc2cc47f`,
verifies 657 blob paths and independently confirms the earlier complete ZIP
hash is unchanged. Original pre-fix evidence and metadata preimages are retained.
The snapshot holds this document before this post-freeze pointer. This bounded
checkpoint does not repair or reclassify older nested historical retention gaps,
implement browser fallback, or close broader parent/release acceptance.


Subsequent implementation: [native source fallback](wasm-policy-fallback.md)
advances the previously missing child to implemented/partial evidence. The earlier
selected-file receipts and ef28 overlay retain their original identities.
