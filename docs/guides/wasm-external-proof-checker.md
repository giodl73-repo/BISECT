# External proof checking in the browser

The standalone VeriPB page checks locally selected OPB model and PBP proof bytes in a fresh Web Worker. The native checker kernel, arithmetic, propagation and proof rules are retained unchanged from upstream **VeriPB 3.0.2**, commit `a50f0483d65aa019918d03ccc590b71dc23c666c`. Both legacy proof version 2.0, produced by RoundingSat, and proof version 3.0 are supported.

Open `http://127.0.0.1:4320/wasm-lab-build/toolkit/proof-checker.html` while the local preview server is running. Select the same OPB model used for generation, select its PBP proof, then press **Check proof**. Selecting files does not execute the checker. **Require an UNSAT conclusion** is enabled by default because the native decision proof stages require UNSAT. Uncheck it to inspect other conclusions, including SAT. A successful check enables **Save checked receipt**. Changing either source or the conclusion requirement retires that receipt. Cancel terminates the worker and discards its result.

A checked receipt records the exact model/proof hashes, accepted conclusion, proof version, checked rule count, upstream checker revision, vendored source provenance hash and actual checker WASM hash. It establishes the recorded conclusion for those formula/proof bytes. It does **not** establish that the formula matches a native model request, that every required decision stage/branch has been checked, or that a complete district plan is optimal. The separate [native proof pipeline](wasm-native-proof-pipeline.md) now performs fresh native request binding and checks all required decisions. Native recursive certificate adoption remains unfinished. The [RoundingSat page](wasm-external-proof-solver.md) continues to label its own outputs unverified until a separate checker runs.

## Source and build boundaries

`third_party/veripb/provenance.json` records the canonical Git blob SHA-256 and vendored SHA-256 for every included upstream source file. Checker, formula, rules and propagator sources are byte-identical to the pinned Git blobs. The adapted parser exposes the existing OPB parsing loop to an in-memory reader and the existing version 3 parser to a generic buffered reader. Native compression dependencies are removed; this port accepts plain text. AHash uses its supported fixed nonrandom seed configuration, avoiding an unsupported browser entropy dependency. This affects hash table implementation, not mathematical proof rules. The library distribution omits upstream native CLI and benchmark metadata.

The independent adapter lives under `web/proof-checker/`, with its own workspace and lockfile. It does not add dependencies to the main Rust engine or change that engine's WASM artifact. The build verifies every vendored checksum and requires unchanged kernel sources before compilation. Source attributes preserve canonical LF bytes across Windows and Linux checkouts. Upstream licenses accompany the preview assets.

```powershell
python scripts/web/build_proof_checker_wasm.py
python scripts/web/stage_proof_checker_preview.py
```

The first command builds the native adapter and WASM library offline with the committed lockfile, then records source/artifact identities in `target/proof-checker-wasm-build/wasm32-unknown-unknown/release/proof-checker-build.json`. Dependencies must already exist in Cargo's local cache. The second verifies the artifact and provenance hashes before staging assets into the existing local toolkit preview. Neither command publishes to Cloudflare.

To reproduce vendoring, obtain a clean upstream checkout at the pinned commit under `target/proof-toolchain/veripb-current/`, then run `python scripts/web/vendor_veripb.py`. It reads canonical Git blobs, applies only the documented boundary changes, and refuses to overwrite locally modified vendor files.

## Verification and limits

Run the RoundingSat tests first to generate the six source proof pairs. Build the independent unmodified native VeriPB executable at the pinned commit for comparison, then run:

```powershell
$env:BISECT_VERIPB = (Resolve-Path target/proof-toolchain/veripb-native-build/release/veripb.exe).Path
node scripts/web/test_proof_checker_wasm.mjs
$env:BISECT_PLAYWRIGHT_PACKAGES = (Resolve-Path target/browser-test-deps).Path
node scripts/web/test_proof_checker_browser.mjs
```

Ten complete WASM reports match the native adapter, with independent acceptance by the unmodified upstream executable: six version 2 proof pairs and four version 3 upstream conformance examples. Six truncated proofs and eight paired native/WASM refusals cover mismatched formulas, invalid references, missing contradictions, trailing junk, external output requests, timer rules, unchecked assertions and SAT proofs supplied where UNSAT is required. Additional checks cover source hash mismatch, unknown JSON fields, corrupted WASM identity, forged provenance and oversized inputs. Chrome checks three exact native receipts, inert selection, strict UNSAT policy, truncated proof refusal, changed-setting freshness, cancellation, fresh checking after cancellation and restrictive CSP. Evidence is under `target/proof-checker-tests/` and `target/proof-checker-browser/`.

Models are limited to **8 MiB**, proof text to **64 MiB** and encoded requests to **192 MiB**. The WASM artifact has a **512 MiB** linear memory ceiling and **8 MiB** stack. The page's **60-second elapsed watchdog** terminates long checks. This does not bound total browser process memory, and these small fixtures do not establish statewide proof-checking capacity. The module has no host imports: it cannot request network access or browser filesystem access. External-output and timer directives are explicitly refused by the adapter; unchecked assertions or weakened guarantees cannot yield an accepted report. Malformed inputs that trap or exhaust resources produce no checked receipt. Broader external-tool orchestration and full capability completion remain active work.
