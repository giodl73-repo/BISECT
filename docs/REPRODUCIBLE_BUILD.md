# Reproducible Build of `redist`

**Status:** v1 (2026-04-29) — Plan 01 Task 7 deliverable.

This document defines the procedure for building the `redist` Rust binary from source in a way that can be independently verified. Required for court admissibility (a special master must be able to reproduce the binary that produced a contested map) and for the binary-provenance block embedded in output files.

## Inputs

| Input | Source | Verification |
|---|---|---|
| Source code | This repository at a tagged commit | `git rev-parse HEAD` |
| Rust toolchain | Pinned via `redist/rust-toolchain.toml` (if absent, use the version recorded in CI) | `rustc --version` must match |
| Cargo lockfile | `redist/Cargo.lock` (committed) | `--locked` build flag fails on lockfile mismatch |
| METIS C library | System install (apt/brew/winget); subprocess'd at run time, not linked | `gpmetis --version` |

## Build

```bash
git clone <repo-url>
cd apportionment
git checkout <tag-or-commit>
cd redist
cargo build --release --locked
# Binary: target/release/redist (or redist.exe on Windows)
```

The `--locked` flag ensures `Cargo.lock` is honored exactly. If the lockfile is missing or stale, the build fails rather than silently resolving newer versions.

## Verification

### Same source → same binary?

Rust binaries are not byte-deterministic by default. Three sources of nondeterminism on most platforms:

1. **Build timestamps** in debug info or PE/ELF headers
2. **Path-dependent debug info** (absolute paths to source files baked in)
3. **Parallel-compilation order** affecting code-section ordering in some cases

To get byte-identical builds, either:

- Build in a clean container with a fixed source path (e.g., `/build`) and `SOURCE_DATE_EPOCH=0` set
- Strip non-determinism after build with `strip` + `--remap-path-prefix` flags

For most court purposes, **functional equivalence** is sufficient: two binaries produced from the same source + lockfile produce identical outputs for identical inputs. Verify by running `redist state --state VT --year 2020` from each binary and comparing `final_assignments.json` byte-for-byte (METIS is deterministic given the same edge weights and seed; the Rust kernel is deterministic given the same input).

### Verifying a published release

A release commit is tagged `v0.X.Y`. To verify a released binary matches its source:

```bash
# 1. Build from source at the tag
git checkout v0.X.Y
cd redist && cargo build --release --locked

# 2. Run the binary on a known input
./target/release/redist state --state VT --year 2020 --output-dir /tmp/verify

# 3. Compare to a reference output (if published) or to the official release binary
diff /tmp/verify/v1/states/vermont/data/final_assignments.json \
     reference/vt_final_assignments.json
```

## Provenance Block

Every output JSON file produced by `redist` SHOULD embed a provenance block (TODO: implement, currently a spec-level requirement only):

```json
{
  "redist_version": "0.1.2",
  "redist_build_commit": "<git sha>",
  "redist_build_date": "2026-04-29T00:00:00Z",
  "rustc_version": "1.84.0"
}
```

This is what `redist doctor --verify-manifest <output.json>` (planned) will check.

## Toolchain Pin

If `redist/rust-toolchain.toml` does not yet exist, create it with the rustc version recorded in CI:

```toml
[toolchain]
channel = "1.84.0"
components = ["rustfmt", "clippy"]
```

Without this file, builds may use whatever stable rustc the developer has installed, which can cause subtle drift in compiler-emitted code over time.

## Linkage

- `redist` binary is statically linked Rust dependencies; only system libraries (libc, libm) are dynamic.
- METIS is invoked as a subprocess (`gpmetis`), not linked. Pin via `gpmetis --version` if reproducibility across machines matters.
- PyO3 bridge (`redist_py`) is built separately and is not part of the production binary.

## Open Items

- Pin `rustc` toolchain in a committed `rust-toolchain.toml` (Plan 01 Task 7.1)
- Embed provenance block in output JSON files (Spec v2 §Provenance)
- `redist doctor --verify-manifest <path>` subcommand (Spec v2 §Provenance)
- Deterministic-build recipe for byte-identical reproduction (out of scope for v1; functional equivalence sufficient)
