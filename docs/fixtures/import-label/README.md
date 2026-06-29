# BISECT Label Import Fixtures

These fixtures are the public golden set for `bisect label-import` compatibility
claims. They are intentionally tiny, synthetic tract-assignment examples. They
test adapter semantics, not geographic validity or legal plan quality.

## Positive fixtures

| Format | Input | Expected assignments | Notes |
|---|---|---|---|
| CSV | `positive/vermont_two_tracts.csv` | `expected/vermont_two_tracts.assignments.json` | Requires `GEOID` and `district` fields. |
| GeoJSON | `positive/vermont_two_tracts.geojson` | `expected/vermont_two_tracts.assignments.json` | Reads `GEOID` and `district` from feature properties; geometry is not transformed for label import. |
| RPLAN | `positive/washington_two_tracts.rplan` | `expected/washington_two_tracts.assignments.json` | RPLAN assignment values are zero-based and are imported as one-based BISECT district IDs. |
| Shapefile/DBF | `positive/vermont_two_tracts.shp` plus `.shx` and `.dbf` sidecars | `expected/vermont_two_tracts.assignments.json` | Requires `.shp`, `.shx`, and `.dbf` sidecars with DBF `GEOID` and district fields. Geometry transformation is out of scope for label import. |

## Negative fixtures

| Format | Input | Expected failure |
|---|---|---|
| CSV | `negative/csv_bad_district.csv` | `[INPUT]` error for non-integer district value. |
| GeoJSON | `negative/geojson_missing_geoid.geojson` | `[INPUT]` error for missing GEOID/district assignment properties. |
| RPLAN | `negative/rplan_unsupported_version.rplan` | `[INPUT]` error for unsupported RPLAN version. |
| Shapefile/DBF | `negative/shapefile_missing_district.shp` plus `.shx` and `.dbf` sidecars | `[INPUT]` error naming accepted district fields. |

## Smoke commands

Run the fixture-backed parser tests:

```bash
cargo test -p bisect-cli public_import_fixture --lib -- --test-threads=1
```

Run the full label-import parser/end-to-end set:

```bash
cargo test -p bisect-cli import_label --lib -- --test-threads=1
```

Compatibility claims are limited to the fields and schemas named here and in
`docs/vtrace/IMPORT_COMPATIBILITY.md`.
