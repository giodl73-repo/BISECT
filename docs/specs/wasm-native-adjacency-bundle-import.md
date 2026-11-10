# Grouped local native adjacency import

In native adjacency inspection/preparation, choose **Import matching release files**.
The selected Census state/year and tract scope determine the required filename
stem, for example `ri_adjacency_2020`. Select its `.adj.bin`, optional
`_geoids.json`, `_centroids.json` and `_county_ids.json` together. The importer
replaces all native source fields atomically after every selected source validates.
A binary-only replacement clears old companions. No inspection or plan Run starts.

This is an explicit local-file adapter for the filenames matched by the desktop
Fetch release wildcard. It does not invoke `gh`, query GitHub, authenticate a
publisher, download release assets, mark a Fetch item complete or authorize disk
installation. Automated public/private release acquisition remains missing.
Filename matching establishes declared selection scope, not actual graph identity;
Run native inspection/preparation explicitly to validate binary/companion content.

## Display and executable files

Matching `.pkl`, `.request.json` and `_display.geojson` may be included in the
selection but remain unread and unapplied. Python pickle is never decoded or
executed, and a request file never becomes execution intent. Select display
geometry separately when needed. Display coordinates must not overwrite physical
measurements embedded in a native RADJ binary. Supplemental physical geometry
remains a separate explicit input with its existing native guard.

The first native comparison rejected the earlier attempt to bind display geometry
as supplemental geometry: `Supplemental geometry cannot override embedded native
geometry`. The failed input/project and original producer bytes remain retained;
the corrected importer does not read the display file. No Rust or native geometry
semantics were changed to obtain a pass.

## Bounds and asynchronous selection

Up to seven matching files, one required binary, 64 MiB combined read inputs,
64 MiB binary and 16 MiB per JSON companion. Ignored file metadata is bounded;
its body is not read. Unsupported state/year/resolution, mixed names, unsafe or
duplicate filenames, absent binary and size failures refuse before body reads.
JSON companions retain the existing strict parser and native source validation.

Original bytes and names become the existing typed project sources, without
normalization. Open remains inert. Hashes in the helper receipt identify selected
bytes and do not authenticate their source. The receipt is in-process evidence,
not a new persisted project field or installation grant. A failed import preserves
the previous sources. Project changes or Clear retire pending selection; a late
read cannot replace current inputs. Run stays disabled while native sources read,
matching its existing execution guard. Existing individual-file selection remains.

## Current local evidence — 2026-10-24

`target/native-adjacency-bundle/package-1792872648628282300/evidence.json` passes
ten stages across default/C-enabled shells: offline-manifest creation, toolkit
asset closure, offline closure, exact native-host/WASM preparation and actual
browser grouped import on each profile. Synthetic RADJ fixture sources regenerate;
no retained external data or remote requests are required. Both profiles retain
their accepted engine identities and base-package bytes.

The unit gate compares complete native/WASM preparation replies and original
source bytes, exact project roundtrip, nine pre-read refusal classes, changed-size,
stale and cancellation cases. Browser gates prove explicit preparation, no import
execution, unread ignored files, exact Save/Open, failed-import preservation,
binary-only companion clearing, Clear during a paused File read and 390px layout.

The source-coverage reconciliation at
`target/fetch-source-coverage/19ab174eaa807da4-1792872715296/evidence.json` passes
42 native planner and 42 handoff comparisons without acquisition. Only active
source/candidate identities were reconciled; historical execution bundles remain
unchanged. Both the original display-binding refusal and first packaging metadata
failure (wrong Rust source filename) remain retained under
`target/native-adjacency-bundle/source-before-1792872246584534800`.

Candidates: `dist/wasm-native-adjacency-bundle-default-preview-v2` and
`dist/wasm-native-adjacency-bundle-c-sidecar-preview-v2`. First candidate directories
and their successful test logs remain intact; the bad metadata attempt emitted no
passing package record. Current packaging preflights source paths and binds source
identities before/after checks. Native parent, other resolutions, real source/
provider capacity and release acceptance remain open. Nothing is deployed.


## Scoped evidence closure

The [six-role review](../../signals/roles/check/wasm-native-adjacency-bundle-import-roles-check-2026-10-24.md)
records four locally closed P2 conditions and 14 bounded/carried P3 findings.
One coding agent applied installed role lenses, not independent signatories.
The standalone scope snapshot is
`target/native-adjacency-bundle/snapshot-scope/snapshot.json`, SHA-256
`e568c81b19fcc7e1976a4517ebb80cbaba54524b16f4629b83889019c4100a24`.
All 1,795 mappings /741 content identities /516,062,599 logical bytes pass full
readback verification. It has independent blob paths and no inherited-blob lookup
requirement; reused content links only previously digest-verified immutable blobs.
This closure and support registration are post-freeze documentation; the frozen
contract retains its pre-registration bytes. The State child is implemented with
partial evidence; parent/source-acquisition/host and acceptance-v1 remain open.
