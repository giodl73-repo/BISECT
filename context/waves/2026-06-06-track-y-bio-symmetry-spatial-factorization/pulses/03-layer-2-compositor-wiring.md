---
pulse: 03-layer-2-compositor-wiring
wave: track-y-bio-symmetry-spatial-factorization
status: done
date: 2026-06-06
validation_level: L1
---

# Pulse 03 -- Layer 2 Compositor Wiring

## Goal

Expose cohesion as an experimental Layer 2 weight profile while preserving the
existing structure and search layers.

## Delivered

- Added `--weights-override cohesion` to the Layer 2 CLI compositor.
- Added `WeightSpec::cohesion` dispatch in the runner.
- Added `CohesionWeighter` as a composable edge-weight transform over the
  existing geographic base weights.
- Added a geography-ready `CohesionWeighter::try_new_with_geography`
  constructor for future terrain/river fixtures. The current CLI override still
  passes empty geography.
- Added first `cohesion.json` sidecar writer for cohesion runs with schema
  `bisect.cohesion.v1`, including cut-edge low-cycle and bridge-likeness
  diagnostics.
- Added manifest discovery field `cohesion_sidecar_path`.
- Added YAML config support for `weights: cohesion`.
- Added YAML config support for non-default cohesion parameters and propagated
  them to the runner sidecar.
- Added L0/L1 tests for parsing/config dispatch and cycle-vs-bridge edge
  weighting behavior.

## Deferred

- Manifest fields for selected parameters and default policy beyond sidecar
  discovery.
- Real-data physical geography parameter presets.

## Non-Goals

- No `--structure cohesion-factor`.
- No module affinity unless Pulse 02 evidence makes the first formula too weak.
- No forbidden field access in geographic mode.

## Validation Target

```powershell
cargo fmt
cargo test -p bisect-core cohesion -- --test-threads=1
cargo test -p bisect-cli cohesion -- --test-threads=1
git diff --check
```

## Validation Result

- `cargo fmt` passed.
- `cargo test -p bisect-core cohesion -- --test-threads=1` passed
  (5 tests; existing non-snake-case warning in `partisan_weights.rs`).
- `cargo test -p bisect-cli cohesion -- --test-threads=1` passed
  (5 tests; existing workspace warnings).
- `cargo test -p bisect-report cohesion_sidecar -- --test-threads=1` passed
  (1 test).
- `cargo test -p bisect-cli weights_cohesion -- --test-threads=1` passed
  (3 tests).
