# Vermont 2020 — Canonical Walkthrough

The Vermont 2020 walkthrough is the project's canonical end-to-end example. Per [SURVEY 2026-04-30](../../docs/superpowers/specs/2026-04-30-onboarding-and-tutorials.md), Vermont was chosen because:

- 1 congressional district → trivial population balance, no partisan controversy
- Fully-public Fekrazad election data + Census TIGER tracts → reproducible from scratch
- No live §2 litigation → not advocacy-aligned to any side a special master might encounter

For the post-Callais §2 evidence kit, see `examples/louisiana-callais-walkthrough/` (advanced).

## What this directory contains

| File | Purpose |
|---|---|
| `run.sh` / `run.bat` | The canonical pipeline invocation. Runs fetch → bisection → analyze → report → verify-manifest. |
| `checksums.json` | Pinned SHA-256 of every input + output. Schema: `tutorial-checksums v1`. Consumed by `redist doctor --check-tutorial-data`. |
| `pin.sh` | Helper that re-pins `checksums.json` from a successful local run. |
| `expected_outputs/` | (Reserved) sample output snippets for visual diff if a run produces unexpected results. |

## Running the walkthrough

Prerequisites: `redist` on `PATH` (run `bootstrap.sh` from the repo root if needed).

```bash
bash examples/vermont-2020-walkthrough/run.sh
```

Expected wall-clock: 2–5 minutes on broadband + a 4-core laptop. The TIGER fetch is the slowest step.

After the run completes, validate against the pinned checksums:

```bash
redist doctor --check-tutorial-data --tutorial vermont-2020
```

## Pinning (or re-pinning) checksums

The committed `checksums.json` ships with `PIN_ON_FIRST_RUN` placeholders. Until a maintainer runs `pin.sh` on a clean machine, every populated row will report `[FAIL]` from `redist doctor --check-tutorial-data`.

To pin from your own clean run:

```bash
bash examples/vermont-2020-walkthrough/run.sh
bash examples/vermont-2020-walkthrough/pin.sh
git diff examples/vermont-2020-walkthrough/checksums.json
git commit -m "Pin Vermont 2020 walkthrough checksums (build commit X)"
```

Re-pinning is appropriate when:
- The pipeline changes in a way that legitimately alters output bytes (new manifest field, etc.)
- The upstream Census TIGER URL ships a revised file
- The Fekrazad DOI publishes a new file revision

It is **not** appropriate to re-pin to silence a `[FAIL]` from a real bug. Investigate first.

## Cross-references

- Onboarding plan: `docs/superpowers/plans/2026-04-30-onboarding-and-tutorials.md`
- L2 acceptance test: `tests/acceptance/test_walkthrough_vermont.py`
- Doctor command: `redist doctor --check-tutorial-data --help`
- Schema source of truth: `redist/crates/redist-cli/src/doctor.rs::TutorialChecksums`

## Tract count

Vermont's 2020 TIGER tract count (as used by this project's adjacency build) is **193**. This matches the baseline in `tests/acceptance/test_pipeline_acceptance.py`. If your walkthrough produces a different count, the upstream TIGER vintage drifted — re-fetch with the pinned URL in `checksums.json` before re-pinning.
