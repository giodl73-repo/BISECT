---
track: Y.1-cohesion-weighted-bisection
date: 2026-06-06
status: pending-data
scope: california-subset-research
---

# California Subset Runbook

Pulse 05 should exercise `cohesion` on a bounded California tract subset before
any CA-52 or public-facing empirical claim.

This runbook records the intended subset protocol. It is pending local tract
artifacts; the current repository checkout has only policy metadata under
`data/` and no California build artifact under `outputs/`.

## Preflight

The preferred diagnostic refresh command is:

```powershell
python scripts/research/track_y_refresh_pulse05.py --allow-blocked
```

It refreshes the preflight, run-plan, status, and closure-gate artifacts without
building a subset fixture or running BISECT. Pass
`--adjacency-override C:/path/to/ca.adj.bin` to test a local graph artifact
through the same chain.

Local refresh validation:

```powershell
python scripts/research/track_y_refresh_pulse05.py --self-test
```

Expected output:

```text
[OK] track_y_refresh_pulse05 self-test passed
```

Run the local preflight before attempting a subset comparison:

```powershell
python scripts/research/track_y_ca_subset_preflight.py `
  --allow-blocked `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-preflight.json
```

If the CA adjacency artifact exists outside the standard repo paths, pass it
explicitly:

```powershell
python scripts/research/track_y_ca_subset_preflight.py `
  --adjacency-override C:/path/to/ca_adjacency_2020.adj.bin `
  --allow-blocked `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-preflight.json
```

Current preflight evidence:

- `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-preflight.json`
- `ready: false`
- `error: No California adjacency artifact found in this checkout.`

The preflight accepts native `.adj.bin` artifacts and release `.pkl` adjacency
fallbacks. Native RADJ remains preferred, but a release `.pkl` is enough to
start a bounded research run if the local Python shim is available.

Local preflight validation:

```powershell
python scripts/research/track_y_ca_subset_preflight.py --self-test
```

Expected output:

```text
[OK] track_y_ca_subset_preflight self-test passed
```

Preflight `next_step` values:

| Value | Meaning |
|---|---|
| `fetch_or_supply_adjacency_artifact` | no usable CA graph was found |
| `repair_or_replace_adjacency_artifact` | a candidate graph exists but failed validation |
| `build_subset_fixture` | a graph exists and the fixture builder can run |

Next data command:

```powershell
cargo run -p bisect-cli -- fetch --year 2020 --states CA --type adjacency --release
```

Current fetch attempt:

- command: `cargo run -p bisect-cli -- fetch --year 2020 --states CA --type adjacency --release`
- result: failed
- reason: `gh release download failed: release not found`
- manifest repo: `giodl73-repo/REDIST`
- manifest release: `data-inputs-v1`
- release check command: `gh release view data-inputs-v1 --repo giodl73-repo/REDIST`
- release check result: `release not found`

Resolution options:

- publish or restore the `data-inputs-v1` release on the manifest repo;
- update `data/manifest.json` or `BISECT_MANIFEST` to point at the current
  adjacency release;
- place a local CA adjacency artifact at one of the preflight candidate paths;
- pass an explicit local adjacency override when doing a research-only subset
  run, if the selected runner path supports it.

## Fixture Builder

Once a native CA RADJ file or release `.pkl` adjacency file is available, build
the bounded subset fixture:

```powershell
python scripts/research/track_y_build_subset_fixture.py `
  --adjacency outputs/V3/data/2020/adjacency/ca_adjacency_2020.adj.bin `
  --geoids outputs/V3/data/2020/adjacency/ca_adjacency_2020_geoids.json `
  --scan-seeds 500 `
  --max-vertices 250 `
  --allow-rejected `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-fixture.json
```

The fixture builder emits:

- tract count;
- edge count;
- total population;
- min/max population;
- degree min/median/max;
- bridge edge count;
- triangle count;
- cycle-evidence and bridge-evidence flags;
- `accepted_for_pulse_05`;
- `acceptance_reasons`;
- selected tract indices and optional GEOIDs.
- selected seed index and scan count.

The first accepted subset should have both `has_cycle_evidence: true` and
`has_bridge_evidence: true`.

The first accepted subset should also report:

```json
"accepted_for_pulse_05": true,
"acceptance_reasons": []
```

If it is not accepted, adjust `--scan-seeds`, `--max-vertices`, or the supplied
adjacency artifact before running the mode pair.

`--scan-seeds N` evaluates the first `N` possible seed vertices and prefers
connected neighborhoods with cycle evidence, bridge evidence, population span,
degree span, and higher induced edge density.

The fixture builder accepts:

- RADJ `.adj.bin` files, with header validation;
- adjacency `.pkl` dictionaries using `adjacency`/`adj`, optional
  `vertex_weights`/`populations`, and optional `edge_weights`.

Local builder validation:

```powershell
python scripts/research/track_y_build_subset_fixture.py --self-test
```

Expected output:

```text
[OK] track_y_build_subset_fixture self-test passed
```

Fixture `next_step` values:

| Value | Meaning |
|---|---|
| `adjust_subset_selection` | subset does not yet satisfy Pulse 05 criteria |
| `run_mode_pair` | run paired `geographic` and `cohesion` subset modes |

Once preflight is ready and the fixture is accepted, generate the mode-pair
commands:

```powershell
python scripts/research/track_y_plan_subset_runs.py `
  --districts 2 `
  --seed 42 `
  --allow-rejected `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-run-plan.json
```

Local run-plan helper validation:

```powershell
python scripts/research/track_y_plan_subset_runs.py --self-test
```

Expected output:

```text
[OK] track_y_plan_subset_runs self-test passed
```

## Subset Requirements

The subset should include:

- a dense cycle-rich urban mesh;
- a sparse connector;
- a bridge-like boundary;
- high enough tract population variance to exercise mass clamping;
- enough adjacency complexity to distinguish `geographic` from `cohesion`.

Recommended first candidates:

| Candidate | Why it fits | Risk |
|---|---|---|
| Bay Area peninsula plus South Bay connector | dense mesh, water barriers, corridor structure | geography can dominate if water edges are already prefiltered |
| Los Angeles basin plus foothill connector | dense urban mesh, valley and ridge structure | large subset may become too close to a full production run |
| Sacramento river corridor plus surrounding sparse tracts | corridor/bridge behavior is inspectable | may be too sparse to stress dense mesh behavior |

The first run should prefer the smallest candidate that still contains all
required features.

## Required Source Record

Before running, record:

- source adjacency path;
- source population path;
- subset selection rule;
- tract count;
- edge count;
- total population;
- excluded surrounding geography;
- any TIGER, census, or derived fields present but unused;
- random seed;
- structure mode;
- district count or target split count;
- balance tolerance.

## Mode Pair

Use the command plan generated by `track_y_plan_subset_runs.py`. It emits paired
commands with the same state, year, district count, structure, seed, adjacency
override, and output root, changing only the weight mode:

```powershell
cargo run -p bisect-cli -- state --state CA ... --weights-override geographic
cargo run -p bisect-cli -- state --state CA ... --weights-override cohesion
```

The generated plan is rejected unless preflight is ready and the subset fixture
is accepted.

Current run-plan evidence:

- `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-run-plan.json`
- `accepted_for_pulse_05: false`
- `commands: {}`
- `next_step: fetch_or_supply_adjacency_artifact`

This rejected plan is intentional while the CA adjacency artifact and accepted
subset fixture are missing.

## Expected Artifacts

For each mode:

- manifest;
- assignments;
- population deviation summary;
- compactness proxy where available;
- split counts where available.

For `cohesion`:

- `data/cohesion.json`;
- `cohesion_sidecar_path` in the manifest;
- `forbidden_fields_used: []`;
- `geo_layers_used: []` for the default profile.

## Comparison Table

Fill this table after the subset runs:

| Metric | `geographic` | `cohesion` | Notes |
|---|---:|---:|---|
| tract count | TBD | TBD | same subset required |
| edge count | TBD | TBD | same subset required |
| total population | TBD | TBD | same subset required |
| max population deviation | TBD | TBD | same tolerance required |
| compactness proxy | TBD | TBD | if available |
| split count | TBD | TBD | if available |
| selected cut edges | TBD | TBD | from run artifacts |
| selected cut low-cycle share | n/a | TBD | from `cohesion.json` |
| selected cut avg bridge-likeness | n/a | TBD | from `cohesion.json` |
| mass factor min | n/a | TBD | from `cohesion.json` |
| mass factor median | n/a | TBD | from `cohesion.json` |
| mass factor max | n/a | TBD | from `cohesion.json` |
| forbidden fields used | n/a | TBD | must be empty |

After both subset runs exist, emit the first machine-readable comparison:

```powershell
python scripts/research/track_y_compare_subset_runs.py `
  --geographic-dir outputs/track-y-ca-subset/geographic `
  --cohesion-dir outputs/track-y-ca-subset/cohesion `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-comparison.json
```

Local comparison helper validation:

```powershell
python scripts/research/track_y_compare_subset_runs.py --self-test
```

Expected output:

```text
[OK] track_y_compare_subset_runs self-test passed
```

Comparison `next_step` values:

| Value | Meaning |
|---|---|
| `repair_subset_runs` | paired run artifacts are incomplete or incompatible |
| `close_pulse_05_or_promote` | comparison artifact satisfies Pulse 05 gates |

## Pulse Status

Summarize the helper chain at any point:

```powershell
python scripts/research/track_y_pulse05_status.py `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-status.json
```

Local status helper validation:

```powershell
python scripts/research/track_y_pulse05_status.py --self-test
```

Expected output:

```text
[OK] track_y_pulse05_status self-test passed
```

## Closure Gate

Gate Pulse 05 closure or promotion on accepted comparison evidence. The
preferred refresh command above writes this artifact automatically; run the gate
helper directly when you only want to re-check closure state:

```powershell
python scripts/research/track_y_gate_pulse05.py `
  --allow-blocked `
  --write-json research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-gate.json
```

Local gate validation:

```powershell
python scripts/research/track_y_gate_pulse05.py --self-test
```

Expected output:

```text
[OK] track_y_gate_pulse05 self-test passed
```

Current gate evidence:

- `research/tracks/Y-bio-symmetry-spatial-factorization/Y.1+cohesion-weighted-bisection/ca-subset-gate.json`
- `accepted_for_pulse_05: false`
- `stage: run_plan`
- `next_step: fetch_or_supply_adjacency_artifact`

The comparison artifact should report:

```json
"accepted_for_pulse_05": true,
"acceptance_reasons": []
```

It is rejected if the paired runs cover different unit sets, have different
district counts, lack a `bisect.cohesion.v1` sidecar, omit the manifest
`cohesion_sidecar_path`, or report forbidden fields.

## Claim Boundary

Allowed:

- "On this selected subset, default cohesion changed the edge-cost structure."
- "On this selected subset, selected cuts had the following cycle/bridge
  diagnostics."
- "The sidecar reports no forbidden fields and no physical-geography layers."

Not allowed:

- statewide California claims;
- legal fairness claims;
- claims about communities of interest;
- claims that river, valley, elevation, or watershed layers improved the run
  unless an explicit geography-aware profile was selected and documented.
