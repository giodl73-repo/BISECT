# Reproducible Build And Reference Replay

**Status:** v2 candidate, 2026-07-09
**Claim posture:** Functional assignment reproducibility; not byte-identical
executable reproduction, legal certification, or full national-standard
conformance

This document defines how to build the `bisect` Rust binary and reproduce a
named reference assignment from public source, pinned tools, and hashed inputs.

## 1. Pinned Inputs

| Input | Pin | Verification |
|---|---|---|
| Source | Named git commit | `git rev-parse HEAD` |
| Rust toolchain | `rust-toolchain.toml`, exact `1.95.0` | `rustc --version` |
| Dependencies | Committed `Cargo.lock` | `cargo build --locked` |
| Default METIS engine | Vendored/static C FFI through the Rust workspace | Manifest engine field and source/dependency lock |
| Portable engine | `bisect-metis` / pure Rust | Explicit engine selection; comparative output unless named by the profile |
| Census and geography | `data/manifest.json` plus replay-record hashes | SHA-256 and source URLs |
| Algorithm profile | Committed YAML under `configs/` | Raw-file SHA-256 |

The default `c-ffi` engine is embedded in the built binary. Production runs do
not invoke a system `gpmetis` subprocess. The reserved `gpmetis` engine is not
implemented and must not be described as an available verification path.

## 2. Build

From a clean checkout at the declared commit:

```powershell
rustc --version
cargo --version
cargo build --release --locked -p bisect-cli --bin bisect
```

Expected compiler family for this version:

```text
rustc 1.95.0
cargo 1.95.0
```

The executable is:

```text
target\release\bisect.exe
```

Use `cargo +1.95.0` explicitly if a shell is outside the repository and does not
honor `rust-toolchain.toml`.

## 3. Build Provenance

`crates/bisect-cli/build.rs` captures:

- git commit, with `-dirty` when applicable;
- build timestamp;
- `rustc --version`; and
- package version.

`crates/bisect-cli/src/provenance.rs` exposes these values through
`Provenance::current()`. Plan manifests and provenance sidecars also record the
running executable SHA-256, algorithm parameters, adjacency hash, TIGER source,
and related chain fields.

Inspect a generated plan manifest with:

```powershell
target\release\bisect.exe doctor --verify-manifest <path-to-manifest.json>
```

This command verifies the manifest structure and available provenance/hash
links against the running binary. It does not certify the plan's legality,
fairness, VRA compliance, or external admissibility.

For label-pipeline runs, verify the config-to-build-to-analysis-to-report chain:

```powershell
target\release\bisect.exe label-verify <label> --year 2020
```

## 4. Functional Versus Byte Reproducibility

Rust executables are not guaranteed to be byte-identical across operating
systems, linkers, source paths, or build environments. The current release gate
therefore uses functional assignment equivalence:

1. build from the same source commit with the pinned toolchain and lockfile;
2. use the same reference-engine profile and hashed inputs;
3. run the same command;
4. canonicalize the assignment JSON; and
5. compare SHA-256 values.

Matching assignment hashes establish that the tested executions produced the
same unit-to-district mapping. They do not establish byte-identical binaries.

A future byte-reproducible binary profile would additionally need a fixed
container image, target triple, linker, source path, `SOURCE_DATE_EPOCH`,
path-remapping flags, and executable normalization procedure.

## 5. NRS v0.1 Reference Replay

The committed reference profile is:

```text
configs\nrs_reference_v0_1.yml
```

Reference scope:

| Field | Value |
|---|---|
| State | Rhode Island |
| Census year | 2020 |
| District count | 2 |
| Structure | `standard-bisect` |
| Weights | `geographic` |
| Search | `single` |
| Seed | `424242` |
| Balance tolerance | `0.5` percent |
| Engine | `c-ffi` |
| Resolution | Tract |

Rhode Island is small enough for rapid independent replay but exercises an
actual two-district METIS partition. Because the candidate national standard
selects census blocks as normative, this tract fixture is a reproducibility
reference, not full NRS v0.1 conformance.

Run:

```powershell
target\release\bisect.exe build nrs_reference_v0_1 `
  --year 2020 --states RI --workers 1 --force --no-interactive
```

Then verify:

```powershell
target\release\bisect.exe label-verify nrs_reference_v0_1 --year 2020
```

The reference evidence record under `docs/fixtures/nrs-reference-v0.1/`
contains the declared source commit, compiler, dependency, config, data,
adjacency, binary, assignment, and manifest hashes.

## 6. Strict Clean Replay Harness

For a release-subset label replay, use:

```powershell
python scripts\maintenance\dcr007_clean_replay.py `
  --label nrs_reference_v0_1 --year 2020 --states RI --workers 1
```

The strict launcher refuses any visible tracked or untracked worktree change.
Ignored census and generated-output paths may be provisioned in the clean
checkout, but their source and hashes must appear in the replay record.

`scripts/maintenance/dcr007_release_subset_replay.py` writes the ignored JSON
execution record under `reports/vtrace/` by default. Promote only a reviewed,
redacted record whose artifact paths and hashes conform to the applicable
custody policy.

## 7. Failure Interpretation

| Failure | Meaning |
|---|---|
| Compiler version mismatch | Toolchain is not the pinned reference |
| `--locked` failure | Dependency lock and source do not agree |
| Config hash mismatch | A different algorithm profile was used |
| Input/adjacency hash mismatch | The run is not the same data experiment |
| Assignment hash mismatch | Functional replay diverged and requires investigation |
| Manifest verifier failure | Provenance or recorded hash links are inconsistent |
| Binary hash mismatch with matching assignments | Functional match; executable bytes differ |

No mismatch should be silently normalized or reported as success.

## 8. Remaining Gates

- Implement the NRS manifest-derived seed rather than the fixed reference seed.
- Implement the block-level national benchmark path and block adjacency profile.
- Publish a fixed container/target profile before claiming byte-reproducible
  executables.
- Obtain non-author replay before promoting the reference beyond internal
  engineering evidence.
