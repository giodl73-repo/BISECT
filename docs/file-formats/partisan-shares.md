# Partisan Shares TSV Format

**Status:** v1 (2026-04-29)
**Consumed by:** `redist state --partition-mode partisan-weighted --partisan-shares <PATH>`
**Producer:** any tool that joins precinct election results to TIGER tracts (see "Producing the file" below).

## Encoding

- UTF-8 text
- Unix or Windows line endings (both accepted)
- Tab-separated (`\t`) columns
- One record per line
- Lines starting with `#` are comments (ignored)
- Blank lines are ignored

## Schema

| Column | Type | Required | Description |
|---|---|---|---|
| `geoid` | string | yes | 11-character TIGER FIPS code with leading zeros (e.g., `01001020100` for AL Census Tract 201, Autauga County). The loader pads short GEOIDs to 11 characters with leading zeros — but producers should write the canonical form. |
| `dem_share` | float | yes | Democratic vote share in `[0.0, 1.0]`. The loader rejects values outside this range. |

The first non-blank, non-comment line that does **not** parse as a float in column 2 is treated as a header and skipped. So both these are valid:

```tsv
geoid	dem_share
01001020100	0.42
01001020200	0.71
```

```tsv
01001020100	0.42
01001020200	0.71
```

## Example

```tsv
# State: Louisiana
# Year: 2020
# Source: 2016+2020 presidential averaged, area-weighted to tracts
# Producer: scripts/data/political/build_dem_shares.py @abc1234
geoid	dem_share
22001950100	0.318
22001950200	0.412
22001950300	0.279
22003020100	0.557
...
```

## Validation rules

The loader (`redist-cli/src/partisan_shares.rs::load_partisan_shares_map`) enforces:
1. Each line splits into ≥ 2 tab-separated fields
2. Column 2 parses as `f64`
3. The parsed value is in `[0.0, 1.0]`
4. GEOIDs shorter than 11 characters are left-padded with `0`

A tract not present in the TSV gets `dem_share = 0.5` (swing) when aligned to the adjacency vertex order. Swing tracts do not trigger the partisan boost (per the algorithm in `redist-core::partisan_weights`).

## Where it goes

By convention:

```
outputs/data/{year}/partisan/{state_code_lower}/dem_shares.tsv
```

Example: `outputs/data/2020/partisan/la/dem_shares.tsv`. The path is not enforced — `--partisan-shares` accepts any path.

## Producing the file

This format is intentionally simple so any tool can produce it. Common pipelines:

1. **Tract-level results already aggregated.** If your data is already tract-level (e.g., from a state SOS export, or pre-aggregated MGGG OpenPrecincts), drop two columns out of the file and you're done.
2. **Precinct-level results + precinct shapefile + tract shapefile.** Spatial overlay each tract with each precinct, area-weight the precinct's `dem_share` (or `dem_votes / total_votes`) by the overlap, sum across precincts that touch each tract. Output the resulting per-tract share.
3. **Tract demographics + statewide D-share + uniform turnout assumption.** Use voting-age population × turnout × statewide D-share. This is a poor approximation (ignores spatial variation) but trivial to produce. The legacy `scripts/data/load_tract_votes.py` shows this pattern.

We do not ship a producer in `redist` itself — input format is decoupled from how you got the data.

## Compatibility

| Tool | Compatible? |
|---|---|
| Excel | Yes — opens as TSV. **Warning:** Excel may strip leading zeros from GEOIDs unless the column is formatted as Text. Re-save with leading zeros preserved or our loader pads them anyway. |
| GerryChain | No direct consumer. Convert to a column on a GeoDataFrame as needed. |
| Districtr | No direct consumer. |
| pandas / polars | `pd.read_csv(path, sep="\t", dtype={"geoid": str})` |

## Provenance

Per the architecture spec (`docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md`), share files used to produce a court-submitted plan should record:
- Source URL or citation for the underlying election data
- Aggregation method
- Producer tool + version (git commit)
- Date generated

Comment lines at the top of the TSV are the canonical place to record this.
