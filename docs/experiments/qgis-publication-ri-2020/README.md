# RI 2020 QGIS publication pilot — 2026-09-10

Completed locally and archived with destination hash checks. No remote release,
commit, solver run or foundation dependency change is included. Commands and
recipient requirements are in [the quickstart](../../PUBLICATION_QUICKSTART.md).
The machine-readable [release receipt](release.json) binds the exact ZIPs.

## Result and source custody

Selected 12 files totaling 36,998,318 bytes from the configured vault: canonical
RI context, its matching 2020 TIGER block archive, and the completed national
replay's RI baseline package. Every selected file has a pinned SHA-256 in
`configs/publication/ri-2020-baseline.json`; plan artifact hashes and context
source hashes provide additional cross-checks. All 12 original hashes were
rechecked unchanged after publication and archival.

Active work is in the actual local `.work/publication` tree. Staging with an
explicit nonexistent vault path passed using only verified cached files.
No national data synchronization or rewrite of historical manifests occurred.

The three pages are a state overview and two district details. All 25,649
block GEOIDs, original attributes, source feature IDs, assignments and exact
ISO-WKB geometry match their inputs. The GeoPackage retains NAD83 coordinates;
QGIS uses EPSG:26919 for display. The source shapefile contains Polygon and
MultiPolygon records, so the units layer declares GEOMETRY to preserve both
without changing source WKB. District and county outlines are explicit unions.
Independent OGR unions verify exact district coverage against the assignments.

Population is 2020 PL 94-171 total resident population: 1,097,379 statewide;
549,341 and 548,038 in districts 1 and 2. Each is about 0.119% above/below the
equal-population target. No new claim of legal population compliance follows
from this display. The source status is `reference-baseline-candidate`, with
source package `verification_status: pass`. Its six explicit non-claims remain
in provenance and the map qualification preserves their meaning.

## Verification

- Ten custody tests passed, none skipped: bad hashes, existing different files,
  interrupted-copy retry, cache reuse without source, path traversal, Windows
  junction rejection, absent archive drive, insufficient space, duplicate run
  refusal and malicious/duplicate ZIP paths are covered across these tests.
- GeoPackage integrity, unique text GEOIDs, exact input assignments/populations,
  unchanged source fields/geometry, CRS equality, declared geometry type and
  district-union coverage checks passed.
- Exactly three nonempty, single-page PDFs and three PNGs; all expected totals,
  page identities and status qualifications checked. All three map images were
  visually reviewed. The overview uses both district colors; detail pages gray
  the other district. Blue-gray overlays water-only Census blocks without
  removing their underlying geometry/assignments.
- Read, map and rebuild ZIPs passed CRC, complete SHA256SUMS and fresh extraction
  checks. Saved-project export from the extracted map and rebuild editions
  resolves every layer inside that package and reproduces all three PNGs
  byte-for-byte. The rebuild edition additionally reconstructs the GeoPackage
  and projects solely from its included inputs/scripts, rechecks the data and
  PDFs, and reproduces all three PNGs byte-for-byte. Binary GeoPackage hashes
  differ between builds because database metadata can differ; exact source
  geometry/attributes, district coverage, totals and rendered content were
  checked rather than claiming database-file byte identity.

| Edition | Bytes | Requirements |
|---|---:|---|
| Read | 4,522,765 | PDF viewer |
| Map | 22,357,844 | QGIS Desktop; Python only for scripted exports |
| Rebuild | 39,297,885 | Tested QGIS/Python stack; Poppler for PDF checks |

ZIPs remain in `.work/publication/packages`. Identical verified archives are at
`VAULT/artifacts/publication/ri-2020-<read|map|rebuild>-pilot-01/`, with receipts.
No local copy was deleted. The package licenses preserve BISECT's separate MIT
software and CC BY-NC content terms and the source-specific Census terms.

## Review through .roles

Agent self-review through relevant role definitions, not independent approval:

- **CONTOUR:** accept input/derivative separation. Exact vintage, GEOIDs and
  population definition are retained; source members match the context hashes.
- **MERIDIAN:** accept as cartography of the existing candidate. The solver,
  graph, assignments and existing source verification are not re-certified by
  a successful export. Water display does not change graph connectivity.
- **DATUM:** accept the bounded reproduction evidence. This is one state on
  Windows, not a cross-platform or national publication validation.
- **LEDGER:** accept relative-path packages with explicit versions, licenses,
  hashes and extraction checks. Keep canonical and derivative identity distinct.
- **SURVEY:** accept three recipient editions and the short unzip/open path.
  Generalize state selection and layouts after this pilot; a map recipient
  does not need the national vault or a Rust toolchain.

Remaining limits: RI-only builder, three saved layouts rather than a generic
district atlas generator, Windows-only launcher/testing, and no automatic
regeneration after manual assignment edits. No data download/reconstruction
from raw PL files, fresh solver replay, public upload, or external review is
claimed by the rebuild edition.
