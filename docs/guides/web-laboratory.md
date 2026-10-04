# BISECT web laboratory

The laboratory runs the existing Rust engine from a local browser. Use state
mode to inspect one region, or national mode to run the same configuration
across a selected cohort or all available states. Maps, per-state failures,
configuration records and comparison metrics remain available in run history.

For free static hosting of saved experiments, see the
[precomputed catalog guide](precomputed-laboratory.md).

## Start the laboratory

From the repository root:

```powershell
cargo build -p bisect-cli -p bisect-web --locked
cargo run -p bisect-web --locked -- --port 4317
```

Open `http://127.0.0.1:4317`. On Windows,
`scripts/web/start_lab.ps1` performs the build and starts the service. The
existing CLI's native dependency build requirements still apply. The server
selects the native `c-ffi` METIS backend. Keep the CLI's default features enabled:
AreaSection requires its dual constraints, and recursive connectivity behavior
differs from the portable fallback.
The server uses a sibling `bisect` binary, then `target/debug/bisect`, unless `--engine`
specifies another binary. Assets are embedded in the Rust executable; rebuild
the service and reload the browser after changing the UI.

The server binds only to loopback. It is a local research tool, not a remotely
hosted multi-user service. It checks Host, Origin and a session token for
mutations. It invokes the engine with a validated argument vector, never a
shell command supplied by the browser.

## Choose an experiment

Select the census year, chamber, and state or cohort. The catalog shows input
availability and chamber allocations. State mode also allows a custom district
count. The current adapter uses Census tracts and total population. It does not
silently substitute block groups, blocks, VAP, or another population source.

The three dimensions correspond to the engine compositor:

| Dimension | Available choices |
|---|---|
| Structure | Standard bisection, direct n-way, GeoSection, AreaSection, prime-factor, CompactBisect, BFS growth, centroidal Voronoi, moving-knife, spectral-style smoothing |
| Boundary weights | Physical graph boundary length, uniform edge count, boundary length with county preference |
| Search | Single seed, multiple seeds, seed percentile, local bisection ensemble, short-burst variants, flip, forest ReCom, merge-split, parallel tempering |

Standard bisection supports Single, seed percentile, local ensembles and the full-plan chains. GeoSection, AreaSection and CompactBisect support Single or a fixed multiple-seed budget. Direct n-way, prime-factor and constructive methods require Single. The CLI currently ignores Multi for standard bisection and does not dispatch a true convergence stopping rule; those choices are excluded. Historical unsupported requests are visibly flagged. The browser
and API reject incompatible selections rather than silently ignoring the search
setting. Demographic, partisan, economic and other auxiliary signals require
additional dataset adapters and are not exposed by this version. Staged engine
methods and unrestricted optimality certification are also not presented as
available run choices.

Population tolerance is entered as a percent: `0.5` means half of one percent.
This is the heuristic engine's tolerance, not the strict population optimum
defined in the [best immediate cut argument](../concepts/best-immediate-cut.md).
Search budgets have different meanings: short bursts have 20 steps and round the total
step budget up to a whole burst; parallel tempering uses the displayed tempering
step budget with four replicas. The exact flags are retained in the run record.

## Compare state scales

The Across scales preset selects RI, IA and NC when their inputs and chamber
counts are available. State groups are defined by the selected chamber's
district count: Small is 1–3, Medium is 4–12, and Large is 13 or more. These are
selection aids, not a claim that district count fully describes computational
scale. Results also report the actual number of tract units.

National mode supports arbitrary selected cohorts and All ready. States execute
sequentially with up to four Rayon threads inside an engine process. A failure
does not discard other states' successful results. The batch ends completed,
partial or failed as appropriate; cancellation preserves finished results and
marks unfinished states cancelled.

The national geographic view shows the selected states, with Alaska and Hawaii
insets. It does not fill unselected states with fabricated results. The separate
state grid compares statuses and metrics; it is explicitly not a geographic
map. Click a state to inspect its district map and population table.

## Inputs and graph preparation

The service reads `data/manifest.json`, or the file named by `BISECT_MANIFEST`.
Relative data paths are resolved against `--root`. Existing native tract graphs
in the manifest's V3/V4 adjacency stores are used when available.

When a graph is absent but local TIGER and PL 94-171 inputs are present, the
service prepares an experimental tract graph with existing Rust data routines:

- Read sorted tract geometry and aggregate block population to tract GEOIDs.
- Require the join to retain the full state's reported block population.
- Project geometry to EPSG:5070 and retain shared boundaries of at least 10 m.
- Connect island components using the existing county-aware nearest-component
  routine, assigning bridges the median retained graph edge weight.
- Save population weights, land areas, GEOID joins, centroids and a preparation
  profile beside the native adjacency file.

This `lab-tract-v1` preparation profile is separate from the frozen certified NRS
instances. Graph and GEOID-join SHA-256 digests, the engine's provenance record,
configuration and actual command arguments are retained with each completed
state. Generated graphs are cached; changing source files does not regenerate
an existing cache automatically. Use a new `--store` to prepare a new instance.

The map uses simplified TIGER geometry for display. Metrics use original graph
populations and adjacency. Every assigned unit must join to a map feature or
the state result fails rather than displaying an incomplete map as complete.

## Inspect and compare results

Results include district population, unit count, maximum deviation, graph
connected components, cut edge count, graph boundary cost, weighted boundary
cost and split county count. Connectivity includes the declared island bridges;
graph boundary cost also includes synthetic bridge weights. It is not an exact
physical perimeter measurement. County cost raises intra-county cut cost and
does not directly minimize the number of split counties.

The map supports zoom, pan, district selection, unit inspection and colors by
district, population deviation or county. GeoJSON and the run record can be
downloaded. The record contains the weighted boundary score even though the
main comparison uses the common unweighted graph boundary length.

Compare runs from the same engine build for the same state, census year, chamber, district count, graph
digest and GEOID join. Different weighting and structure choices remain
different experimental rules; a lower displayed metric does not establish a
universally better plan. Total elapsed time includes initial preparation and
output work. New records separately retain preparation and engine elapsed
times, so warm caches are not mistaken for faster partition algorithms.

Run completion means execution and result collection completed. Population and
connectivity checks remain visible independently. All plans in this laboratory
are labeled heuristic, with optimality unproved. Sampling completion does not
establish mixing or calibrated percentile inference.

## Storage and operational limits

The default store is `runs/lab`, which is ignored by Git. Each experiment has
its own directory containing a persistent `experiment.json`, engine outputs,
state maps and GEOID assignments. Earlier research outputs are not overwritten.
On service restart, unfinished runs are marked interrupted rather than resumed
or labeled successful.

Cancellation interrupts active engine execution. Native graph preparation
finishes before cancellation takes effect. The per-state time limit applies to
the engine process; preparation and result collection are outside that limit.
Logs in the record are bounded to recent messages. National runs can require
substantial preparation time and storage, especially on the first pass.

## Validation

Run the focused checks with:

```powershell
cargo test -p bisect-web -p bisect-data --locked
node --check web/lab/lab.js
```

Tests cover engine argument parsing for exposed combinations, configuration
validation, population and connectivity metrics, interrupted-run recovery,
local request guards and legacy tract field names. Live checks additionally
exercise real Census preparation, state runs, a cohort spanning multiple state
sizes, browser maps and saved-run comparison. A complete 50-state run is a
separate experiment, not implied by those focused checks.
