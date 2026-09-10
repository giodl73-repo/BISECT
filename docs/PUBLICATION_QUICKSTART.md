# QGIS publication: Rhode Island pilot

The first publication pipeline uses the completed RI 2020 reference baseline
candidate, not the later adjacent-swap diagnostic. It preserves 25,649 Census
blocks, their assignments and total population of 1,097,379. District 1 has
549,341 residents and District 2 has 548,038. Three pages cover the state and
the two individual districts. Source verification status and non-claims remain
visible; this is not an enacted map or a new solver/legal certification.

## Open the local result

Working project: `.work/publication/runs/ri-2020-pilot-01/map-v2/project.qgs`.
The `exports` folder alongside it contains three PDFs and PNG previews.
Open Project > Layouts in QGIS to edit/export the saved pages. Keep the project,
attachment ZIP and data folder together. Unit attributes, full geometry and
dissolved districts are in `data/publication.gpkg`.

Portable ZIPs are in `.work/publication/packages`:

- `bisect-ri-2020-read-pilot-01.zip`: PDFs, PNGs, provenance and credits; any PDF viewer.
- `bisect-ri-2020-map-pilot-01.zip`: editable project, local GeoPackage, export scripts and guide; QGIS Desktop.
- `bisect-ri-2020-rebuild-pilot-01.zip`: map package plus exact inputs and builder source; QGIS/Python.

Read the verification receipts alongside each ZIP and the compact
[pilot evidence](experiments/qgis-publication-ri-2020/README.md). These are local
artifacts, not uploaded GitHub assets. Large inputs/outputs are ignored by Git.
Archive receipts identify the verified vault copies; the configured vault can
have any drive letter. Recipients need none of its paths.

## Tested requirements

- Windows, QGIS Desktop 3.44.12 and its Python/GDAL environment. The package
  records exact runtime versions in `environment.json`.
- Python 3.12 for the staging/packaging tools and Windows QGIS launcher.
- Poppler `pdfinfo` and `pdftotext` on PATH for PDF validation and the rebuild
  package's automated acceptance check. Opening/styling maps does not need them.
- No ArcGIS, additional QGIS plugins, Rust build, RPLAN/RLINE/RCOUNT checkouts,
  or online basemap is required for this publication pipeline.

The launcher discovers a single QGIS installation; if several are installed,
pass `--qgis-root "C:/Program Files/QGIS 3.44.12"` before the script argument.
Other platforms are not yet tested and the launcher is Windows-specific.
Do not install a separate pip GDAL over QGIS's native GIS libraries.

## Reproduce locally

From the repository root, choose unused run and output names. Stage defaults
to a size/space dry run; `--apply` admits exact hashes into the local cache and
copies them into an isolated run. The committed selection pins 12 files,
36,998,318 input bytes. Eight times that size plus 256 MiB is reserved for the
pilot's copies, derivatives and extraction check; this is an estimate, not a
general upper bound for arbitrary future states.

```powershell
py -3.12 scripts/publication/workspace.py stage --selection configs/publication/ri-2020-baseline.json --run-id ri-new
py -3.12 scripts/publication/workspace.py stage --selection configs/publication/ri-2020-baseline.json --run-id ri-new --apply
py -3.12 scripts/publication/run_qgis.py scripts/publication/build_map.py --run .work/publication/runs/ri-new --output .work/publication/runs/ri-new/map
py -3.12 scripts/publication/run_qgis.py scripts/publication/export_map.py --package .work/publication/runs/ri-new/map --output .work/publication/runs/ri-new/map/exports
py -3.12 scripts/publication/run_qgis.py scripts/publication/validate_map.py --package .work/publication/runs/ri-new/map --run .work/publication/runs/ri-new
py -3.12 scripts/publication/package_map.py --source .work/publication/runs/ri-new/map --run .work/publication/runs/ri-new --kind rebuild --archive .work/publication/packages/ri-new-rebuild.zip
py -3.12 scripts/publication/workspace.py archive --package .work/publication/packages/ri-new-rebuild.zip --release-id ri-new
```

Use `--kind read` or `--kind map` with a different ZIP name for the other
editions. The rebuild packager actually rebuilds from its extracted inputs,
validates the new data/PDFs, and compares all rendered pages. ZIP CRC, complete
checksums and dependency containment are also checked. Existing archive/output
names are refused; use a new name after an interrupted packaging attempt.
An interrupted stage can reuse previously verified cache entries with a new
run ID; interrupted temporary copies are not admitted. The source manifest
keeps canonical hashes distinct from the derivative GeoPackage hash, whose
SQLite metadata may differ between builds.

If the vault is absent but all selected cache entries verify, staging succeeds.
If the vault is absent during archival, the package remains locally verified
and its receipt says `verified-local-awaiting-archive`. No cleanup runs
automatically. Reattach the drive and repeat archival to finish.

## Editing and scope

Change styling in the project. Assignment edits create a new candidate and
require regenerated district geometry, totals, text and normal research
verification; QGIS does not synchronize those derivatives automatically.
The current builder intentionally supports this RI 2020 two-district baseline
schema. General state selection, multi-scenario comparison, a shared RPLAN
adapter, per-district native atlas automation and alternate operating systems
remain future work. The three current pages are saved QGIS layouts.

Run `py -3.12 scripts/publication/test_workspace.py` for the custody failure
tests. Storage policy and package responsibilities are in
[PUBLICATION_PACKAGING.md](PUBLICATION_PACKAGING.md). Relevant role review is
included in the pilot evidence; no foundation dependency revisions changed.
