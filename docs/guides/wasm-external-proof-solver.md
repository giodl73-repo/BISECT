# Experimental external proof solver

RoundingSat now has a standalone browser runner for one OPB decision model. It runs in a fresh Web Worker, exports a PBP proof log and records the exact input, proof and solver WASM SHA-256 hashes. Selecting a file is inert. Changing the file or time limit retires the previous exports. Cancel terminates the worker; subsequent runs create fresh modules because the upstream solver has process-global state.

The page deliberately reports **solver-reported SAT/UNSAT** and **proof unverified**. A solver verdict is not independent proof verification. The page does not adopt a certificate, assert district-plan optimality, or run commands embedded in imported files. A [separate VeriPB WASM page](wasm-external-proof-checker.md) now checks a selected OPB/PBP pair. The [native proof pipeline](wasm-native-proof-pipeline.md) combines compilation, generation and checking with fresh native request/instance/discovery binding across all required compact decisions.

The existing native compact proof-model ZIP supplies suitable OPB files. This runner accepts one selected file at a time; it does not orchestrate all population branches, boundary and canonical stages. A population decision may need both low/high branches to establish the incumbent bound. Completing one decision does not establish the whole proof package.

## Build and local preview

The build uses clean [RoundingSat source](https://gitlab.com/MIAOresearch/software/roundingsat) at commit `d4edbf7908a9bb951fd181940919e0f3ac7ab1ee`, Emscripten **6.0.12**, and Boost **1.86.0** headers. No upstream source patches are applied. A small linked clock adapter maps `getrusage` timing to Emscripten's monotonic elapsed clock because its normal syscall is a stub; timing fields labeled CPU time therefore report elapsed time in this port. The adapter's source hash is recorded. The compiler selects upstream's existing Boost multiprecision integer fallback because libc++ does not provide `std::abs(__int128)`. SoPlex/LP, GMP and compressed input/output support are disabled, matching the repository's `--lp=0` proof smoke workflow.

Acquire the following under `target/proof-toolchain/`:

- `roundingsat/`: the clean checkout at the pinned commit above.
- `emsdk/`: the official [Emscripten SDK](https://emscripten.org/docs/getting_started/downloads.html), with `emsdk.py install 6.0.12` and `emsdk.py activate 6.0.12`. Activation uses the SDK-local configuration; no permanent system activation is needed.
- `boost_1_86_0.tar.gz`: the [official Boost archive](https://archives.boost.io/release/1.86.0/source/boost_1_86_0.tar.gz), SHA-256 `2575e74ffc3ef1cd0babac2c1ee8bdb5782a0ee672b1912da40e5b4b591ca01f`.
- `boost/boost/`: the extracted archive's `boost/` headers; `boost/LICENSE_1_0.txt`: its license.

From the repository root:

```powershell
python scripts/web/build_roundingsat_wasm.py
python scripts/web/stage_roundingsat_preview.py
```

The first command builds into `target/roundingsat-wasm/` and records options, versions, artifact hashes and sizes in `build-receipt.json`. The second installs the experimental page, solver assets and licenses into the existing `dist/wasm-lab-build/toolkit/` preview. It verifies artifact checksums before copying and records all packaged hashes. Neither command publishes a site or modifies the Rust engine build.

The local page is `http://127.0.0.1:4320/wasm-lab-build/toolkit/proof-solver.html` while the existing preview server is running. The Cloudflare deployment has not been updated with this experiment.

## Limits and validation

Input is strict UTF-8, nonempty, NUL-free and at most **8 MiB**. Time limits are **1–120 seconds**; the page also terminates the worker after the selected limit plus 30 seconds of elapsed time. WASM linear memory has a **512 MiB** ceiling and an **8 MiB** stack. Proof writes are limited to **64 MiB**, with bounded diagnostic output. The adapter stops before a virtual proof-file write crosses this bound, returns `solver-proof-output-limit` with `exit_code: null`, and retains only an unverified prefix. This interrupts the fresh module; it is not a native SAT/UNSAT or timeout conclusion. The result records the byte budget, actual retained bytes and truncation flag. These are application limits; the memory ceiling does not bound the browser's total process memory. Solver requests use fixed virtual filenames and arguments, with no shell execution or imported command execution. Worker asset hashes are checked before solving. The page's CSP permits WASM compilation without JavaScript `unsafe-eval`.

The independent validation checker is [VeriPB](https://gitlab.com/MIAOresearch/software/VeriPB) **3.0.2**, Rust upstream commit `a50f0483d65aa019918d03ccc590b71dc23c666c`. Its parser supports the solver's legacy proof version 2.0. This is a different checker revision from the historical Python/Cython smoke checker; evidence records its actual executable hash and identity.

Build the native checker at that exact commit using its checked-in lockfile, then set `BISECT_VERIPB` to its executable. After the compact model tests have generated their fixtures, run:

```powershell
$env:BISECT_VERIPB = (Resolve-Path target/proof-toolchain/veripb-native-build/release/veripb.exe).Path
node scripts/web/test_roundingsat_wasm.mjs
$env:BISECT_PLAYWRIGHT_PACKAGES = (Resolve-Path target/browser-test-deps).Path
node scripts/web/test_roundingsat_browser.mjs
```

Six WASM solves are independently checked: the two committed smoke models, three freshly compiled population/boundary/canonical models, and a satisfiable model. The checker rejects all six truncated proofs, a mismatched satisfiable formula/UNSAT proof pair and an invalid conclusion reference. The runner refuses corrupted WASM bytes, oversized inputs, invalid time limits and malformed models. A separate pairwise pigeonhole model exercises the actual one-second solver timeout, including native exit code 4; its partial proof log remains unverified. Chrome checks four exact proof downloads against the same native checker, inert selection, export freshness, cancellation, a fresh run after cancellation and the restrictive script CSP. Evidence is in `target/roundingsat-wasm-tests/evidence.json` and `target/roundingsat-browser/evidence.json`.

These small tests establish the port and proof format interoperability. They do not establish statewide proof-solving capacity, optimality of a full district plan, or completion of the remaining WASM capability inventory.
