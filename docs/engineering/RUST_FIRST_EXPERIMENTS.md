# Rust-First Experiments

All new computational work in this repository starts in Rust.

This includes experiment runners, research orchestration, analysis,
verification, evidence generation, benchmarks, and production workflows.
Python is not an acceptable prototyping path for those responsibilities.

## Permitted Python Boundary

Python may be changed only in these compatibility areas:

- `python/bisect_py/`: the maintained Python language binding;
- `scripts/data/`: external-data download and normalization adapters; and
- `setup_data.py`: the repository data bootstrap adapter.

The pre-commit hook rejects added or modified Python elsewhere. Deleting
legacy Python remains allowed.

The same policy is available without a shell hook:

```text
cargo run -p bisect-ops -- audit-python --staged
cargo run -p bisect-ops -- audit-python --base origin/main
```

## Migration Policy

Existing Python is legacy, not precedent. Migrate active code in this order:

1. experiment controllers and resumable batch runners;
2. evidence builders and independent verifiers;
3. research analysis and benchmark drivers;
4. historical figure and paper-support scripts when those studies are rerun.

Do not rewrite sealed forensic material under `archive/`. It remains evidence
of prior execution and is not an active entry point.

## Native Operations Crate

`bisect-ops` owns Rust-native experiment operations that coordinate the core
BISECT engine. Its first migration is the nationwide operational recursive-tree
builder previously implemented in
`scripts/research/build_operational_recursive_tree*.py`.

The migration preserves deterministic seed ordering, per-seed timeouts,
resumable evidence, arithmetic-floor classification, recursive context
projection, connected-leaf verification, and package custody. Once parity is
certified, active documentation and batch entry points must use `bisect-ops`.

```text
cargo run -p bisect-ops -- build --bisect target/release/bisect \
  --context <state.rctx> --out-dir <package> --districts <n> --max-seed 32
cargo run -p bisect-ops -- batch --bisect target/release/bisect --max-seed 32
cargo run -p bisect-ops -- verify <package>
```

Completed Rust packages contain `builder-source.rs`, an immutable snapshot of
the exact builder source hashed by the manifest. This avoids the historical
failure mode where a manifest pointed at a mutable repository script.

## Legacy Inventory

The initial 2026-07-15 inventory found 429 tracked Python files outside
`archive/`: 213 under `scripts/`, 104 under `research/`, 101 under `tests/`,
and 11 elsewhere. These files are migration inventory. They may be deleted or
replaced by Rust, but they may not be extended as active experiment machinery.
