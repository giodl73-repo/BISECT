# C-FFI WASM feasibility attempt

This is the **historical direct Rust-target attempt**, retained as investigation
history. The later [Emscripten sidecar experiment](wasm-cffi-sidecar-feasibility.md)
builds the same vendored C library, measures runtime RNG differences and tests
an explicit Windows-compatible profile in Node and a browser worker. The local
compiler obstacle below is no longer the latest feasibility result. Production
C-FFI browser integration was later added in a
[scoped optional extension](wasm-cffi-engine-integration.md); broad WP-02
acceptance remains open.

WP-02 remains open. On 2026-10-22, the isolated fixed-seed native probe compiled
and executed successfully; its `wasm32-unknown-unknown` build failed in the
vendored `metis-sys` C build because `clang` was not found. WASM execution was
**not run**. This is an observed local toolchain obstacle, not a determination
that C-METIS cannot run in a browser.

The production native backend uses `metis`/vendored C-METIS, while the current
browser build explicitly uses `metis-core` for methods requiring METIS. The
State prepared boundary refuses other backend choices for those methods and
multiple districts. `gpmetis` is a shared native stub, not another working native
backend to port. Keep those distinctions and the explicit browser backend.

The source-owned probe is in
[`scripts/web/fixtures/wasm-cffi-probe`](../../scripts/web/fixtures/wasm-cffi-probe/Cargo.toml).
Its independent workspace pins `metis` 0.2.2 and a retained Cargo.lock resolves
`metis-sys` 0.3.2. It does not change production dependency features or the root
Cargo.lock. The fixture is a four-cycle, two requested parts and seed 7, using
the dependency's vendored 32-bit index/real configuration. A small exported
function encodes the returned edge cut and assignment bits. The native result
was 521: cut 2, assignments `[1, 0, 0, 1]` in the retained vertex order.

Reproduce the attempt with cached dependencies and the Rust WASM target:

```powershell
python scripts/web/probe_wasm_cffi.py
```

The runner returns failure unless both native and WASM calls run. It retains
the exact argument vectors, toolchain output, source/lock hashes, executable
identity and raw logs under `target/wasm-cffi-feasibility/`. It records skipped
execution as not-run. The current local `evidence.json` is investigative output;
it is not a tracked acceptance prerequisite or a full-parity evidence bundle.
The tracked fixture and runner regenerate the attempt in a clean checkout.

The native host was `x86_64-pc-windows-msvc`, rustc 1.95.0. The checker identified
the missing `clang` tool during compiler-family detection and C compilation.
Common LLVM/Visual Studio Clang locations checked locally were also absent.
Production Cargo.toml, Cargo.lock, the WASM crate manifest and the production
WASM binary matched their before/after hashes.

Next, provide an appropriate cross C compiler and repeat the locked probe.
Then investigate any actual C header, libc, allocator, signal/error handling,
timing or linker/import requirements revealed by compilation. Do not assume
that installing Clang alone resolves those requirements. A successful module
must expose a callable backend with declared imports and compare against the
native fixed-seed result before BISECT integration. Full acceptance still needs
explicit backend UI/dispatch, weighted and multi-constraint cases, seed/index
semantics, failure behavior, the real-data matrix and unchanged `metis-core`
regressions. The four-cycle is only a feasibility witness.
