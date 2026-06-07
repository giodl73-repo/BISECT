---
wave: track-y-bio-symmetry-spatial-factorization
date_open: 2026-06-06
status: active
source_goal: research/tracks/Y-bio-symmetry-spatial-factorization/README.md
---

# Track Y Bio-Symmetry Spatial Factorization

## Mission

Create a research and implementation path for cohesion-weighted,
symmetry-aware spatial factorization in BISECT.

The first executable direction is a Layer 2 weight profile:

```text
--weights-override cohesion
```

It should use local graph cycles, bridge-likeness, geometry, and population mass
to thicken or thin edge weights while preserving the existing structure and
search layers.

## Claim Boundary

This wave may define and later implement a research weight mode. It must not
claim legal fairness, public release readiness, or biological proof of civic
community. It must not use party, demographic, county, city, or land-use fields
unless a selected mode explicitly authorizes them.

The deferred Layer 1 idea:

```text
--structure cohesion-factor
```

must remain out of implementation scope until Layer 2 fixture and subset
evidence justify it.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Track Y research scaffold | DONE | Track Y README, Y.0-Y.5 plans, Y.1 implementation spec, `docs/PAPERS.md` index |
| 02 - Cohesion term L0 fixtures | DONE | `bisect-core::cohesion` with `CohesionParams`, cycle support, bridge-likeness, population mass, clamped mass factor, symmetric finite weights; focused tests pass |
| 03 - Layer 2 compositor wiring | DONE | `--weights-override cohesion`, YAML `weights: cohesion` plus non-default params, `WeightSpec::cohesion`, `CohesionWeighter`, first `cohesion.json` sidecar, manifest sidecar discovery, focused CLI/core tests pass |
| 04 - Synthetic integration checks | DONE | mesh/bridge, dense-core, relabeling, sidecar checks, and synthetic baseline comparison report landed |
| 05 - California subset research run | IN_PROGRESS | subset runbook plus preflight, fixture builder, run planner, refresh driver, comparison, status, and closure-gate helpers landed; execution blocked on available CA adjacency artifact |
| 06 - Close or promote | TODO | decide whether to close as research-only, continue to CA-52, or file Layer 1 follow-up |

## Design Spine

```text
topology finds the mesh
geometry measures physical connection
population supplies flow/mass
district count supplies factorization pressure
symmetry supplies canonical comparison and fairness tests
```

## Validation

Documentation pulses:

```powershell
git diff --check
```

Implementation pulses must add focused Rust validation before closing. The
expected first implementation validation is:

```powershell
cargo fmt
cargo test -p bisect-core cohesion -- --test-threads=1
cargo test -p bisect-cli cohesion -- --test-threads=1
git diff --check
```

Pulse 03 used the CLI cohesion filter above; broader integration belongs to
Pulse 04.
