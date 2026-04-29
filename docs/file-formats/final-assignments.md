# `final_assignments.json` Format

**Status:** Canonical (2026-04-29)
**Producer:** `redist-cli::output::write_state_outputs`
**Consumer:** dashboard generators, `redist analyze`, `redist export`, post-processing scripts, court submissions

The canonical per-state output of a `redist state` run. Maps each tract (or block group / block, depending on `--resolution`) to a district number.

## Path

```
{output_base}/{version}/states/{state_name}/data/final_assignments.json
```

Where:
- `{output_base}` is `outputs/` by default (or whatever `--output-dir` was set to)
- `{version}` is `--version` (e.g., `v1`, `V3`)
- `{state_name}` is the lowercase underscore form (e.g., `vermont`, `new_york`)

## Encoding

- UTF-8 JSON
- Pretty-printed (2-space indent), so the file is human-readable
- No trailing whitespace
- LF line endings (written by Rust; do not normalize)

## Schema

A single JSON object whose keys are tract indices (as strings, since JSON object keys must be strings) and whose values are district numbers (1-indexed).

```json
{
  "0": 1,
  "1": 1,
  "2": 2,
  "3": 2,
  "4": 1
}
```

| Field | Type | Description |
|---|---|---|
| key | string | Tract index in adjacency vertex order (`0`..`n_tracts-1`). Use `index_to_geoid` from the adjacency file to map back to TIGER GEOIDs. |
| value | int | District number, 1-indexed. Range is `1..=n_districts`. |

The number of distinct values must equal `n_districts` (no empty districts), and the keys must cover `0..n_tracts` exactly (no missing or extra tracts).

## Atomic write protocol

The producer writes to `.final_assignments.tmp.json` first, then renames to `final_assignments.json` after `vra_analysis.json` (when applicable) is also written. This guarantees the **BOUNDARY invariant**: both `final_assignments.json` and `vra_analysis.json` exist together, or neither does.

If a process crashes between the temp-write and the rename, the next invocation detects `.final_assignments.tmp.json` and returns a `CorruptState` error rather than silently using stale outputs. Recovery: `clean_corrupt_state(data_dir)` (programmatic) or `--reset` (CLI) wipes the partial state.

## Example: Vermont (1 district, 193 tracts)

```json
{
  "0": 1,
  "1": 1,
  "2": 1,
  ...
  "192": 1
}
```

All 193 tracts map to district 1.

## Example: Alabama (7 districts)

```json
{
  "0": 4,
  "1": 4,
  "2": 5,
  ...
  "1436": 7
}
```

1437 tracts split across 7 districts.

## Companion files

In the same `data/` directory, you typically also find:

| File | Producer | Purpose |
|---|---|---|
| `vra_analysis.json` | runner (when `--partition-mode metis-vra`) | Per-district minority percentages, MM count |
| `district_summary.csv` | runner | Per-district population, voting-age population, population balance pct |
| `manifest.json` | runner (when `--manifest`) | Provenance: input adjacency hash, run parameters, binary version |
| `provenance.json` | runner (always) | Sidecar with `redist_version`, `redist_build_commit`, `redist_build_date`, `rustc_version`. Written atomically with `final_assignments.json`. Verifiable via `redist doctor --verify-manifest`. |

## Mapping back to GEOIDs

The keys are adjacency vertex indices, not GEOIDs. To convert:

```python
import json

with open("final_assignments.json") as f:
    assignments = json.load(f)

# Load index_to_geoid from the adjacency file (or wherever your tooling exposes it).
# In Python via redist_py:
import redist_py
graph = redist_py.deserialize_adjacency(open("path/to/state.adj.bin", "rb").read())
geoid_assignments = {
    graph.index_to_geoid[int(idx)]: district
    for idx, district in assignments.items()
}
```

In Rust, the adjacency `AdjacencyGraph::index_to_geoid` field gives the same mapping.

## Compatibility

| Tool | Compatible? |
|---|---|
| GerryChain | No direct consumer. Convert via `redist export --format gerrychain` for a CSV with `geoid,district` columns. |
| Districtr | Use `redist export --format csv` to produce the GEOID-keyed CSV Districtr expects. |
| pandas / polars | `pd.read_json("final_assignments.json", typ="series").to_frame("district")` |
| Excel | Open via `Data → From Text/JSON`. |

## Why this format

- **Compact**: just the assignment table; metadata lives in `manifest.json`.
- **Tooling-friendly**: stable JSON shape, no schema language needed.
- **Deterministic**: serialized in tract-index order, pretty-printed.
- **Atomic**: paired with `vra_analysis.json` via the rename-from-tmp protocol.
