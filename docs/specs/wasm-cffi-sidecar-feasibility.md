# C METIS sidecar feasibility

This records the original isolated experiment. A subsequent
[optional Rust-engine/workbench integration](wasm-cffi-engine-integration.md)
now executes scoped BISECT methods. Its broader acceptance remains open; the
standalone experiment below is retained as historical baseline evidence.

WP-02 has a viable experimental C execution path. The same vendored C METIS
builds with Emscripten and runs in Node and a dedicated Chrome module worker.
An explicit Windows CRT RNG compatibility profile matches the native reference
on the scoped matrix below. **Production C-FFI backend integration remains
missing.** No browser controls, production dispatcher, Cargo features or deployed
assets change in this experiment; `metis-core` remains available as before.

## What was compared

The source-owned [isolated fixture](../../scripts/web/fixtures/wasm-cffi-probe/Cargo.toml)
pins `metis` 0.2.2 and `metis-sys` 0.3.2. The runner checks the crate archive
SHA-256 `769ee6be814b21c52afcc631f07051f578487d642f3e89a45f2a086fd94fffa9`
and compares every archive file to the extracted Cargo-cache file before
compiling. It derives the C source lists from the upstream `build.rs`, using
32-bit indices/reals and the native release defines. Windows-only platform
adaptation code is not used on the Emscripten target. No Cargo-cache source is
patched. The [older direct Rust-target failure](wasm-cffi-feasibility.md) remains
historical evidence; a sidecar is a different integration architecture.

Native reference: the real `metis::Graph` API links vendored C under
`x86_64-pc-windows-msvc`. Its source-owned matrix emits exact CSR, weights,
targets, seed, status, cut and assignments. The sidecar calls the C API with
those same inputs, with separate C memory. Both set ufactor 30, ten refinement
iterations, contiguity and minimum connectivity. No comparison renumbers
districts or normalizes assignments.

| Witness | Scope |
|---|---|
| CRT sequence | Five unsigned seeds: 0, 1, 7, 2147483647, 4294967295; 64 measured native `rand()` values per seed. |
| Graph matrix | 6×5 and 20×15 grids; unweighted, weighted vertices/edges, and two population constraints; four seeds each, 24 cases. Targets are 0.4/0.35/0.25 per constraint. |
| Independent check | JavaScript sums every crossing weighted CSR edge and checks twice the returned cut; labels must be in range. This does not certify balance, optimality or real geography. |
| Original witness | Four-cycle, k=2, seed=7; exact packed cut/assignments. |
| Failure and cleanup | Disconnected graph with contiguity returns -4; invalid zero refinement count returns -2; a successful cycle follows each error. |
| Browser | The same comparisons run inside a real dedicated module worker, fetching the exact generated JavaScript/WASM assets from a private localhost server. |

The 300-vertex fixture exercises a larger graph than the initial cycle; it is
still synthetic and establishes no state-scale capacity. Native wrapper
comparison proves these C-kernel calls, not CLI orchestration or BISECT dispatch.

## Explicit runtime profiles

The vendored GKlib build does not enable its portable `USE_GKRAND` generator.
It delegates to libc `srand`/`rand`. Emscripten's musl implementation and the
measured Windows CRT implementation differ. Using the same source and seed
therefore does not guarantee the same native result.

- **`unmodified`** uses the normal Emscripten runtime. It builds and executes,
  but the RNG sequence and most matrix assignments differ from Windows native.
  The cycle encodes 518 versus native 521; disconnected/error cleanup matches.
  The runner exits 1 and retains all mismatches. This is an expected observed
  mismatch, not an accepted exact-parity result.
- **`msvcrt`** explicitly links the source-owned
  [compatibility shim](../../scripts/web/fixtures/wasm-cffi-probe/msvcrt_rng.c)
  and renames `rand`/`srand` calls at compilation. Its 32-bit state evolves as
  `state = state * 214013 + 2531011` modulo 2³², returning
  `(state >> 16) & 32767`. The sequence is checked against the actual linked
  Windows CRT, not assumed from the formula. All 320 values, 24 graph outcomes
  and error/recovery witnesses match native in Node and Chrome's module worker.

The profile is experimental, serial and module-local. It is not a claim about
Linux/macOS libc, all Windows library versions, threads, other index widths or
every GKlib utility. It does not enable `USE_GKRAND` or change the native engine's
seed semantics. A production backend must expose and retain the runtime profile
identity explicitly rather than silently substituting it or `metis-core`.

## Reproduce

Requirements: native Rust/C build tools, cached locked fixture dependencies,
Node, an Emscripten SDK, and Chrome/Playwright for the browser tier. These are
developer tools; users of a future packaged backend would download its built
assets rather than install this SDK.

The tested SDK is Emscripten **6.0.13**, emsdk revision
`d7b4130688bfa348392b921ab04015aa15c4f154`, release tools hash
`eb132757325583b787fe9414b09afea9442b72ad`. The SDK-local configuration references
its bundled Node 24.19.0 and Python 3.13.3. Node comparison runtime is recorded
separately; it need not be the bundled compiler-runtime Node. The source-owned
runner records the actual emcc, native Rust, Node and SDK revision outputs and
hashes SDK configuration/compiler files. It does not treat transport or hashes
as a signature or source-authenticity proof.

Install in an ignored local tool directory, without changing the permanent
system environment:

```powershell
git clone https://github.com/emscripten-core/emsdk.git target/toolchains/emsdk
git -C target/toolchains/emsdk checkout d7b4130688bfa348392b921ab04015aa15c4f154
python target/toolchains/emsdk/emsdk.py install 6.0.13
python target/toolchains/emsdk/emsdk.py activate 6.0.13
cargo fetch --manifest-path scripts/web/fixtures/wasm-cffi-probe/Cargo.toml --locked
python scripts/web/probe_metis_c_sidecar.py --rng-profile unmodified
# Inspect evidence.json: comparison mismatch is expected, exit 1.
python scripts/web/probe_metis_c_sidecar.py --rng-profile msvcrt
# Expected exit 0 only when native/matrix outcomes match exactly.
$env:BISECT_PLAYWRIGHT_PACKAGES='<directory containing the Playwright package>'
node scripts/web/test_metis_c_sidecar_browser.mjs
```

An alternate SDK location uses `--emsdk`; `--output` must resolve beneath this
repository's `target/` directory. Outputs default to
`target/metis-c-sidecar-matrix/{unmodified,msvcrt}/`, with per-command raw logs,
exact arguments, source/native/SDK/artifact hashes, reference data, comparisons
and before/after production hashes. Browser evidence is
`target/metis-c-sidecar-matrix/browser-evidence.json`. Browser tests reject stale
source/artifact identities and compare their results to the recorded Node
outcomes, including the unmodified mismatches. No ambient browser tabs are used.
Missing tooling is a not-run/setup failure, not passing support. Local target
evidence regenerates from tracked source; it is not a committed acceptance
prerequisite. This optional integration tier is not required to rebuild the
unchanged production `metis-core` browser package.

## Next implementation boundary

Specify and validate Rust-to-C memory transfer, array lengths, index/float
encodings, option mapping, allocation failure, ownership and result/error
transfer. Then connect explicit backend selection to all actual BISECT native
consumers, retaining current browser defaults. Load and hash-bind both sidecar
assets only through an explicit execution path; project Open stays inert.

Acceptance still requires actual native BISECT CLI/dispatch comparisons,
backend controls and saved-profile semantics, memory/worker cancellation tests,
unchanged `metis-core` regressions, broader real-data/browser coverage and exact
release/offline package identities. This experiment resolves the immediate
question of whether the C library can execute in a browser; it closes none of
those integration or full-parity requirements by itself.
