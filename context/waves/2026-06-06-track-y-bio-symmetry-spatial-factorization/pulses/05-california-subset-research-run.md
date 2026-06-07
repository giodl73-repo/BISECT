---
pulse: 05-california-subset-research-run
wave: track-y-bio-symmetry-spatial-factorization
status: in_progress
date: 2026-06-06
validation_level: L2
---

# Pulse 05 -- California Subset Research Run

## Goal

Exercise cohesion weighting on a bounded California tract subset before any
full CA-52 run or public-facing empirical claim.

## Subset Selection Criteria

Pick a subset that includes:

- one dense cycle-rich urban mesh;
- one sparse connector;
- one bridge-like boundary;
- tract population variance high enough to exercise mass clamping;
- enough adjacency complexity to distinguish `geographic` and `cohesion`.

The subset must be documented by source paths, tract count, edge count, total
population, and any excluded surrounding geography.

## Planned Deliverables

- `geographic` baseline run.
- `cohesion` run with the same structure/search/seed controls.
- `bisect.cohesion.v1` sidecar.
- summary comparison:
  - population deviation,
  - compactness proxy,
  - split counts where available,
  - selected cut cycle support,
  - selected cut bridge-likeness,
  - retained internal cycle support,
  - mass-factor range and median,
  - forbidden-fields-used row.

## Claim Boundary

This pulse may produce subset research evidence only. It must not make
California-wide, legal, public-release, or production-readiness claims.

## Validation Target

```powershell
cargo fmt
cargo test -p bisect-core cohesion -- --test-threads=1
cargo test -p bisect-cli cohesion -- --test-threads=1
git diff --check
```

Add the exact subset run command after the fixture is selected.

## Progress

- Added subset runbook:
  `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/california-subset-runbook.md`.
- Added subset preflight helper:
  `scripts/research/track_y_ca_subset_preflight.py`.
  Its manifest/path diagnostic self-test passes:
  `python scripts/research/track_y_ca_subset_preflight.py --self-test`.
  It also supports `--adjacency-override` for local CA graph artifacts outside
  the standard repo paths and emits `next_step`. It supports
  `--allow-blocked` so the current missing-data diagnostic artifact can be
  refreshed without failing automation.
- Added subset fixture builder:
  `scripts/research/track_y_build_subset_fixture.py`.
  It accepts RADJ `.adj.bin` and adjacency `.pkl` inputs. Its synthetic
  RADJ/pkl self-test passes:
  `python scripts/research/track_y_build_subset_fixture.py --self-test`.
  It now supports `--scan-seeds` to choose connected neighborhoods with cycle
  evidence, bridge evidence, population span, degree span, and higher induced
  edge density. It also emits `accepted_for_pulse_05` and
  `acceptance_reasons` so an underspecified subset cannot silently move to the
  mode-pair run, plus `next_step`. It supports `--allow-rejected` when the
  caller intentionally wants to persist a rejected diagnostic fixture.
- Added subset run comparator:
  `scripts/research/track_y_compare_subset_runs.py`.
  Its artifact-shape self-test passes:
  `python scripts/research/track_y_compare_subset_runs.py --self-test`.
  It emits `accepted_for_pulse_05` and rejects mismatched unit sets, different
  district counts, missing/invalid cohesion sidecars, missing manifest sidecar
  discovery, or forbidden-field use. It also emits `next_step`.
- Added subset run planner:
  `scripts/research/track_y_plan_subset_runs.py`.
  Its self-test passes:
  `python scripts/research/track_y_plan_subset_runs.py --self-test`.
  It emits paired `geographic` and `cohesion` commands only when preflight is
  ready and the subset fixture is accepted.
- Added Pulse 05 refresh driver:
  `scripts/research/track_y_refresh_pulse05.py`.
  Its self-test passes:
  `python scripts/research/track_y_refresh_pulse05.py --self-test`.
  It refreshes the preflight, run-plan, status, and closure-gate artifacts in
  one non-destructive command and supports `--allow-blocked` for the expected
  missing-data state.
- Added Pulse 05 status helper:
  `scripts/research/track_y_pulse05_status.py`.
  Its self-test passes:
  `python scripts/research/track_y_pulse05_status.py --self-test`.
  It summarizes the current stage and next action from the preflight, fixture,
  run-plan, and comparison artifacts.
- Added Pulse 05 closure gate:
  `scripts/research/track_y_gate_pulse05.py`.
  Its self-test passes:
  `python scripts/research/track_y_gate_pulse05.py --self-test`.
  It exits successfully only when comparison evidence is accepted, unless
  `--allow-blocked` is supplied to persist the current diagnostic gate result.
- Generated current status artifact:
  `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-status.json`.
  Current stage is `run_plan`; next step is
  `fetch_or_supply_adjacency_artifact`.
- Added current preflight evidence:
  `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-preflight.json`.
- Added current blocked run-plan artifact:
  `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-run-plan.json`.
  It emits no mode-pair commands while preflight is not ready, fixture evidence
  is missing, and no adjacency path is available. The run planner supports
  `--allow-rejected` so the blocked diagnostic artifact can be refreshed
  without failing automation.
- Added current blocked gate artifact:
  `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-gate.json`.
  It records `accepted_for_pulse_05: false` and keeps the next step at
  `fetch_or_supply_adjacency_artifact`.
- Checked the local repository for existing California run/build artifacts.
  Current checkout has policy metadata under `data/` and no usable California
  tract subset or prior run artifact under `outputs/`.
- Attempted the release-backed adjacency fetch:
  `cargo run -p bisect-cli -- fetch --year 2020 --states CA --type adjacency --release`.
  It failed because GitHub reported the manifest release `data-inputs-v1` on
  `giodl73-repo/REDIST` was not found.
- Verified the manifest release directly:
  `gh release view data-inputs-v1 --repo giodl73-repo/REDIST` also returned
  `release not found`.

## Current Blocker

Pulse 05 execution needs a local California tract adjacency/population artifact
or an existing CLI-supported subset mechanism. Until then, the runbook is ready
but no subset comparison can be honestly produced.

The preflight reports the next data command as:

```powershell
cargo run -p bisect-cli -- fetch --year 2020 --states CA --type adjacency --release
```

That command is currently blocked until the release manifest points at an
available adjacency release, or a local CA adjacency artifact is supplied.

Once a RADJ artifact is supplied, the next local command is:

```powershell
python scripts/research/track_y_build_subset_fixture.py `
  --adjacency outputs/V3/data/2020/adjacency/ca_adjacency_2020.adj.bin `
  --geoids outputs/V3/data/2020/adjacency/ca_adjacency_2020_geoids.json `
  --scan-seeds 500 `
  --max-vertices 250 `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-fixture.json
```

After paired `geographic` and `cohesion` subset runs exist, compare them with:

```powershell
python scripts/research/track_y_compare_subset_runs.py `
  --geographic-dir outputs/track-y-ca-subset/geographic `
  --cohesion-dir outputs/track-y-ca-subset/cohesion `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-comparison.json
```
