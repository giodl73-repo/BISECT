---
name: r-pulse
description: Execute one apportionment wave pulse end to end with scout, implementation, docs, and validation.
user-invocable: true
---

# r-pulse

Execute a pulse from the active wave.

## Usage

```text
/r-pulse 04
/r-pulse next
```

## Procedure

1. Resolve active wave from `context/waves/PHASES.md`.
2. If `next`, choose the first pulse with `status: todo`.
3. Read the pulse file completely.
4. Run every command in `Pre-implementation Scout`.
5. Implement deliverables using the repository's existing patterns.
6. Update docs and the pulse checklist.
7. Update `WAVE.md` pulse table.
8. Run validation from the pulse.
9. Run `git diff --check`.

## Default Validation For R Packages

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-stats -p rcount-core -p rcount-io -p rcount-audit -p rcount-district -p rcount-cli
git diff --check
```

## Completion Report

Report:

- pulse number and title
- files changed
- gates completed
- validation commands and result
- carry-forwards

## Rules

- A boundary fixture is allowed only when its docs say what is not replayed.
- A new package reference must have a hash shape and a verifier path.
- A new active package needs positive and negative coverage.

