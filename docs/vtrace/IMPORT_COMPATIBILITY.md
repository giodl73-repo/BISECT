# Import Compatibility Matrix

## Scope

This matrix is the DCR-005 compatibility control for external plan/package
surfaces named by BISECT docs. Public compatibility claims must cite fixture
evidence from `docs/fixtures/import-label/` or a package-family fixture.

Status vocabulary:

- `supported-fixtured`: implemented and covered by a public fixture or package fixture.
- `supported-bounded`: implemented, but public compatibility is limited to the named adapter behavior.
- `partial`: implemented for a narrow path; broader ecosystem compatibility is not claimed.
- `unsupported`: no current supported adapter.
- `unknown`: mentioned ecosystem surface without enough version/schema evidence for a claim.

## Matrix

| Surface | Direction | Status | Version/schema | Required fields or inputs | Geometry assumptions | Failure modes | Fixture evidence |
|---|---|---|---|---|---|---|---|
| CSV assignment table | import via `bisect label-import --format csv` | supported-fixtured | headered CSV, schema informal | `GEOID`, `district` | none | missing fields, empty file, invalid GEOID/FIPS, non-integer district | `docs/fixtures/import-label/positive/vermont_two_tracts.csv`; negative `csv_bad_district.csv`; `cargo test -p bisect-cli public_import_fixture --lib -- --test-threads=1` |
| GeoJSON assignment features | import via `bisect label-import --format geojson` | supported-fixtured | GeoJSON FeatureCollection with BISECT assignment properties | `GEOID`/`geoid`, `district_id` or `district` property | geometry may be null; geometry is not transformed for label import | missing features, missing properties, invalid GEOID/FIPS, invalid district | `docs/fixtures/import-label/positive/vermont_two_tracts.geojson`; negative `geojson_missing_geoid.geojson`; public fixture parser tests |
| RPLAN package JSON | import via `bisect label-import --format rplan` | supported-fixtured | `rplan_version` `0.2` with `0.1` compatibility accepted through `rplan-io` | explicit `unit_ids`, assignment vector, package schema fields | geometry is package metadata; label import consumes assignments | unsupported version, invalid package, inconsistent assignment length | `docs/fixtures/import-label/positive/washington_two_tracts.rplan`; negative `rplan_unsupported_version.rplan`; public fixture parser tests |
| Shapefile/DBF assignment table | import via `bisect label-import --format shapefile` | supported-fixtured | ESRI shapefile read by `shapefile` crate `0.6`; DBF attributes carry assignments | `.shp`, `.shx`, `.dbf`; DBF `GEOID` and district field (`DISTRICT`, `DIST`, `CD`, etc.) | geometry is read but not transformed; assignment semantics come from DBF attributes | missing sidecar, missing fields, invalid record, invalid GEOID/FIPS, invalid district | `docs/fixtures/import-label/positive/vermont_two_tracts.shp` plus sidecars; negative `shapefile_missing_district.shp`; public fixture parser tests |
| DRA-style CSV | import/export where current CLI paths detect DRA-like rows | partial | version unknown / tool-export dependent | DRA export fields as recognized by current adapter | not a geometry transformer | unrecognized export layout or missing assignment fields | no DCR-001 fixture yet; do not claim public compatibility beyond inspected adapter behavior |
| GerryChain JSON | import/export where current adapter supports GerryChain v2.3 JSON | partial | GerryChain v2.3 JSON | assignment mapping expected by adapter | not a geometry transformer | version/schema mismatch | no DCR-001 fixture yet; notebook/research compatibility remains separate |
| PlanScore | interoperability reference only | unknown | version/schema not locked | not specified | not specified | unsupported or version-unknown | no fixture; no public compatibility claim |
| Census/TIGER | source geography/data acquisition through BISECT data path | supported-bounded | year-specific Census/TIGER sources controlled by fetch/data manifests | source manifests, TIGER/PL data | geometry used by data/adjacency pipeline, not label-import fixture | missing data, source hash/custody mismatch, topology errors | data fetch/source validation, not an import-label fixture |
| NIST/CDF election count | package-family import path where implemented by RCOUNT | partial | NIST/CDF status is adapter-specific | election count files and RCOUNT package schema | not a redistricting geometry input | parser/source completeness and lifecycle findings | RCOUNT tests/fixtures; no public jurisdiction replay claim |

## Claim rule

Docs and release notes may say BISECT has fixture-backed public label-import
compatibility for CSV, GeoJSON, RPLAN, and shapefile/DBF assignment tables.
Shapefile claims remain bounded to DBF assignment attributes and do not imply
geometry repair, projection conversion, or arbitrary external-tool export
compatibility.
