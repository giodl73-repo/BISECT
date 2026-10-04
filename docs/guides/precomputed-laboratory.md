# Publish and explore precomputed laboratory results

The public laboratory reads saved results from static files. It needs no Rust
process, account, database or paid compute service for visitors. The local
laboratory generates experiments; the catalog builder publishes their public
fields and shares state geometry across every plan. Visitors load exact saved
settings rather than requesting a live engine run.

## Build a catalog

Build the native engine and catalog tool from the repository root:

```powershell
cargo build -p bisect-cli -p bisect-web --bin bisect --bin lab-catalog --locked
```

Compute the small matrix and export into a new directory:

```powershell
target/debug/lab-catalog --matrix web/lab/matrices/across-scales-2020.json --compute --output dist/precomputed-lab
```

The pilot contains six complete configurations: standard bisection and direct
n-way, each with geographic, uniform and county weights. It uses 2020
congressional allocations for RI, IA and NC, one seed, county strength 2,
0.5 percent population tolerance and 100 refinement iterations. Eighteen state
results are requested; failures remain visible rather than being removed.

`web/lab/matrices/national-2020.json` expands to 66 configurations over all 50
states, or 3,300 requested state results. It includes each supported structure,
weight and search combination with one explicit parameter preset: seed 42,
eight seed candidates or tempering steps, 200 chain steps, minimum percentile,
county strength 2, area swing 1.1 and a 300-second engine timeout. These budgets
are presets, not all possible values, calibrated samples or an optimality claim.
Graph preparation is outside the engine timeout. The full matrix can take many
hours and uses local storage; it is not run by the publication workflow.

To export previously finished runs without computing, pass comma-separated IDs:

```powershell
target/debug/lab-catalog --run-ids RUN_ID_1,RUN_ID_2 --output dist/catalog-export
```

Omitting IDs exports compatible finished runs in the store. Do not export while
experiments are active. Unsupported historical search requests are excluded.
Output must be new or empty; the manifest is written only after validation and
the 900 MB size check. A failed export can leave assets without a manifest;
choose a fresh destination for the next attempt. Use `--store` for another
local run store and `--max-bytes` for a stricter local size budget.

## Catalog format and reproducibility

`catalog.json` has schema version 1. It contains public experiment records,
coverage, the input catalog and geometry references keyed by state and census
year. `assets/SHA256.json` stores each unique geometry or GEOID assignment map.
Geometry is a GeoJSON FeatureCollection using longitude and latitude; assigned
tract GEOIDs remain eleven-digit strings. Display coordinates are simplified
and are not suitable for recomputing original graph boundary metrics.

The viewer checks SHA-256 of downloaded asset bytes, geometry identity, unique
GEOIDs and assignment coverage before displaying a saved map. These checks are
tamper evidence relative to the manifest, not a signature or legal certificate.
Graph and GEOID digests identify derived inputs; they do not prove custody of
the original Census downloads. New computations record the executable digest;
older records expose a null digest. Public exports omit local paths, process
arguments, raw logs and session tokens. Failed states receive a generic reason;
their detailed diagnostic logs remain in the local store.

Effective matching keeps year, chamber, district count, structure, weights,
seed, active budgets, percentile, tolerance and refinement iterations. Inactive
budgets are ignored. Short bursts round to whole 20-step bursts. County strength
follows the engine's minimum of 1; area swing matters only for AreaSection.
Timeout changes do not select a different saved plan. Missing configurations
remain missing and never substitute a nearby seed or weight strength.

Saved experiment selection restores settings. State and national maps, district
tables, comparisons and downloads work locally in the browser. National metrics
summarize only completed states and disclose their coverage. Failures and
uncomputed combinations are distinct from complete results. Population and
connectivity checks remain experimental measurements, not findings of legal
compliance, fairness or optimality.

## Preview with a static server

```powershell
python -m http.server 4319 --directory dist
```

Open `http://127.0.0.1:4319/precomputed-lab/`. This nested path tests the same
relative asset behavior used by GitHub project Pages. HTTPS or localhost is
required for the browser's Web Crypto integrity checks. Opening index.html
directly as a file is not supported.

## Prepare GitHub Pages

Validate and package the completed artifact:

```powershell
node scripts/web/verify_catalog.mjs dist/precomputed-lab
scripts/web/package_catalog.ps1
```

The package script creates `dist/lab-catalog.tar.gz` and a SHA-256 sidecar. Upload
that archive as a GitHub Release asset after reviewing its contents. Set the
repository's Pages source to GitHub Actions. Manually run Publish precomputed
laboratory with the release tag, archive filename and expected archive hash,
plus the branch/commit and directory containing the existing static site.
The default source is `gh-pages` at its root; for a site sourced from `main/docs`,
use `main` and `docs`. Confirm the current source in repository Pages settings.
The workflow requires that source to contain `index.html`, preserves its files,
and replaces only its `laboratory/` directory with this catalog. The viewer is
published at the project site's `/laboratory/` URL. If the source ref is absent,
publication fails before deployment; no existing site is replaced.
The workflow verifies the archive, rejects links and unsafe extraction paths,
checks assets and coverage, and deploys the static Pages artifact. Browser code
contains no GitHub token. Computation, release upload and public deployment are
separate steps; preparing a local artifact does not publish it.

GitHub Pages supports public repositories on GitHub Free, limits a published
site to 1 GB and has a soft monthly bandwidth limit of 100 GB. The builder's
900 MB cap leaves space below the site limit. The workflow checks the combined
existing site and laboratory against that same cap. Review the current
[GitHub Pages limits](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits)
before publication; a public research catalog is the intended use here.

## Verify changes

```powershell
cargo test -p bisect-web --locked
node --test web/lab/static.test.mjs
node scripts/web/verify_catalog.mjs dist/precomputed-lab
```

Tests use small fixtures without Census downloads. They cover shared geometry,
public field sanitization, exact settings, missing results, unsafe references,
unknown schema versions, corrupted assets, output limits and map joins. The
browser acceptance check uses real RI, IA and NC data beneath a nested static
URL. Full national generation and GitHub deployment require their own completed
runs; they are not established by the pilot checks.
