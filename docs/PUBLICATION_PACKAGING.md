# Local publication work and portable releases

Design agreed from the September 10, 2026 user direction: M: is removable
archive storage; active publication work belongs locally inside the BISECT
checkout. The RI 2020 pilot now implements staging, map building, validation,
three package editions, extracted-copy checks and verified archival. See the
[working commands](PUBLICATION_QUICKSTART.md) and
[pilot evidence](experiments/qgis-publication-ri-2020/README.md). Generalized
state selection and other operating systems remain outside the tested scope.

## Storage ownership

```text
bisect/
  scripts/publication/           builders, checks, packaging and pilot layout code
  cartography/templates/        separate reusable templates (future)
  configs/publication/          pinned input selection; formal schemas are future work
  docs/                        tracked instructions and compact release evidence
  .work/publication/            ignored, actual local directory
    cache/                     selected source files, keyed by SHA-256
    runs/<run-id>/              isolated inputs, derivatives, maps and logs
    packages/                  verified archives awaiting archival/distribution

configured-vault/
  source-data/                  existing preserved inputs
  derived-data/                 existing canonical contexts
  artifacts/publication/<release-id>/  completed releases and archive receipts
```

Do not use existing `data/...` or national `runs/...` links as publication
scratch space: they can resolve to the external drive. Keep their current
routing for existing research commands. The local-work exception here applies
to publication; it does not relocate all research data or change historical
manifests. Never require M: or any particular drive letter in a shared package.

## Work sequence and admission rules

1. Select one existing completed plan, its matching context, required geometry
   and source evidence. Estimate selected bytes and local space for inputs,
   derivatives, ZIP creation and one extracted verification copy. Start with
   one small state, not the national vault. Report sizes before copying.
2. Resolve the workspace path and reject external-drive routing or junctions
   that redirect it. Stage an explicit file list from the vault into local
   temporary files, verify against recorded SHA-256, then admit to the cache.
   Never use recursive synchronization of the vault or mutate its originals.
   Reuse only cache entries whose hashes match. An interrupted copy remains
   unadmitted and can be retried without changing verified files.
3. Create a unique run directory. Copy its selected inputs from the cache;
   avoid writable hard links to cache or source files. Record source-relative
   paths, hashes, plan/context identities, code revision and any uncommitted
   builder changes. Preserve historical manifests verbatim. Keep code and
   machine-specific virtual environments local.
4. Generate and verify maps entirely from local inputs. No network or vault
   access is needed after staging. Export is a derivative: assignments,
   population totals, Census vintage and verification status must survive.
5. Package locally, extract into a fresh directory, validate every dependency
   resolves inside that directory, and re-export without access to source
   paths. Compare complete page sets and rendered content. Record tested OS,
   QGIS/Python/native-library versions, exact commands and package checksums.
6. Archive only completed packages to a new release directory on the configured
   vault. Copy to temporary names on the destination, verify the destination
   bytes, then finalize with a receipt. Refuse different existing releases.
   If M: is absent, keep the local verified package with status
   `verified-local-awaiting-archive`; do not fail the mapping work or claim
   archival success. Never remove local work automatically after copying.

Local cache cleanup is separate and explicit: show candidates and byte counts,
retain active-run inputs, and require verified archive evidence before offering
to remove the only local completed release. Archiving on one external drive
alone is not an independent second backup.

## Packages for other people

| Package | Contents | Recipient requirements | Acceptance |
|---|---|---|---|
| Read | PDF atlas, overview PNG, short methods/status note and credits | A PDF viewer | All expected pages, readable labels and correct evidence status |
| Open and edit | QGIS project and attachment companion, GeoPackage, local basemap, credits/licenses, manifest, checksums, guide and verification report | QGIS Desktop; first validation target is the already installed/tested 3.44.12 environment | Unzip and open; no plugin, ArcGIS, M: path, network, or sibling repositories needed |
| Rebuild publication | Map package plus exact redistributable plan/context/geometry inputs, relevant verification evidence, builder source snapshot, environment declaration, commands and logs | Matching tested QGIS Python/GDAL environment; Python for launcher/checks as documented | Recreate GeoPackage and all pages from included inputs on a fresh setup |

Rebuilding publication does not mean rerunning the district solver. Offer a
separate solver recipe using the pinned BISECT Rust toolchain, Cargo lockfile
and required native build tools only when that path is tested. Do not require
Rust or all three sibling repositories merely to open or restyle a map.

The QGIS version above is now tested with the BISECT RI pilot as well as the
earlier mapping migrations. Exact versions are recorded in each package's
environment.json. Use QGIS's matching Python/GDAL stack;
do not suggest that `pip install qgis` installs QGIS Desktop. Publish separate
platform instructions and mark Windows-only validation honestly until other
platforms pass. A container or bundled desktop installer is not part of the
first release.

Map ZIP structure (the pilot uses its source Census geography for local context
and does not need a separate basemap folder):

```text
bisect-<state>-<vintage>-<plan-id>-<release-id>/
  README.md
  project.qgs
  project_attachments.zip       when the project uses attachments
  data/publication.gpkg
  basemap/                     only the local files the project actually uses
  exports/                     atlas PDFs and previews
  provenance/source-manifest.json
  provenance/verification.json
  environment.json
  LICENSES/                    code/data licenses and required attribution
  SHA256SUMS                   checksums for every other packaged file
```

Use a versioned manifest with relative paths, file sizes/hashes, CRS, text
GEOIDs, units/population definitions, source vintage, plan/context identifiers,
release status and expected pages. Exclude personal paths, local configuration,
credentials, unrelated research, build caches and Python environments. Include
only data/source code whose redistribution terms permit it; otherwise publish
an explicit pinned retrieval recipe and describe the resulting offline limits.
Audit archive members for traversal, absolute paths and symlinks. Packages
contain actual files, not junctions to the author's computer.

Keep templates, tools, schemas and small example fixtures in Git. Publish
large ZIPs as separate release assets when distribution is requested; archive
the identical bytes on the vault with their hashes. No external upload is
performed by local packaging or archival.

## Role review and first implementation order

Agent self-review, following the relevant roles' precedence:

- CONTOUR: stage exact versioned inputs; retain identifiers and population
  definitions. Display convenience cannot alter canonical geography.
- LEDGER: version the package contract, preserve canonical hashes separately
  from derivatives, and test paths and actual interchange files after extraction.
- SURVEY: keep the common recipient path to unzip/open/export. State package
  sizes and requirements; a map recipient should not need the whole vault.

The pilot implements local staging and manifests, the one-state map builder,
completeness/data checks, portable ZIP/extraction checks and verified archival.
Tests exercise missing drive, interrupted copy, checksum mismatch,
insufficient space, existing destination and relocated-package cases where
each mechanism is introduced. No solver or foundation changes were needed.
