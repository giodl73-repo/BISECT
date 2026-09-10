# Portable data vault and fresh-machine setup

The vault is a **project data directory**, not a drive letter and not a copy
of the repository. Code, papers, compact evidence and manifests belong in Git.
Large source data, derived contexts, build workspaces and new run outputs belong
in the vault. No source data or Git bundle is added to the repository by this setup.

The [validation record](experiments/portable-vault-2026-09-10/README.md) documents
the tested scope, exact hashes and remaining work.
The subsequent [hardening release record](experiments/portable-vault-2026-09-10/release.md)
documents the isolated environment, tested lean clone and new offline source bundle.

## Existing vault: connect another computer

Clone the repository, install Python 3.11 or newer, then point the tool at the
directory containing `source-data`, `derived-data` and `runs`:

```powershell
python scripts/data_vault.py configure --vault M:\DATA_VAULT\projects\apportionment
python scripts/data_vault.py status --group contexts
python scripts/data_vault.py status --group contexts --hash
python scripts/data_vault.py link
python scripts/data_vault.py link --apply
python scripts/data_vault.py ri-replay --run-name my-first-ri-replay
```

On Linux/macOS the same commands accept paths such as
`--vault /media/mydrive/DATA_VAULT/projects/apportionment`. This is implemented
with POSIX directory symlinks; the actual end-to-end validation so far was on
Windows, using junctions. The optional `.bisect-vault.json` is ignored by Git
and belongs only to that clone. An explicit `--vault` overrides the
`BISECT_DATA_VAULT` environment variable, which overrides that local file.
The drive can be called something other than M. If it moves, use `--vault` or
the environment variable; existing local configuration is never overwritten
silently. Remove/edit only that small ignored config if you want to save a new default.
`link --apply --repair` can retarget links recorded in this clone's ignored
`.bisect-vault-links.json`; it never replaces real local directories or unknown
links. Older links without ownership records require explicit inspection.

`status` without `--hash` checks existence and byte size, not integrity.
`--hash` reads all selected bytes and checks SHA-256. Missing/mismatched files
produce a nonzero exit status. The catalog contains 350 entries: 150 canonical
contexts across 2000/2010/2020, 50 2020 block archives, 100 2020 PL geography/
population members, and 50 enacted CD118 map archives. This is the certified
redistricting data subset, **not every dataset used by every research track**.

`link` defaults to a dry run. Applying it creates only absent repository-facing
directories; existing local folders and mismatched links are reported and left
untouched. Inspect any `existing-local-path-retained` result: it is not evidence
that the existing local files match the vault. In particular, legacy local PL
folders may use a different layout. New clones get the canonical vault layout.
Unresolved mappings now produce exit code 1 (including in dry runs). A successful
link check verifies routing, not file contents: run `status --hash` as well.

## No vault: download and rebuild a 2020 State

Choose a new, dedicated project data directory anywhere with sufficient space.
You can pass it directly without creating a saved configuration:

```powershell
python scripts/data_vault.py fetch --vault D:\BISECT-data --year 2020 --state RI --group sources
python scripts/data_vault.py fetch --vault D:\BISECT-data --year 2020 --state RI --group sources --apply
python -m venv D:\BISECT-tools\builder-venv
D:\BISECT-tools\builder-venv\Scripts\python.exe -m pip install -r configs/data-vault/requirements-build-2020.txt
D:\BISECT-tools\builder-venv\Scripts\python.exe scripts/data_vault.py build-2020 --vault D:\BISECT-data --state RI --run-name ri-build-1 --apply
python scripts/data_vault.py status --vault D:\BISECT-data --year 2020 --state RI --hash
python scripts/data_vault.py ri-replay --vault D:\BISECT-data --run-name ri-replay-1
```

Use another State abbreviation for the same 2020 builder; only RI has been
end-to-end tested through this new fresh-download workflow. Omitting `--state`
downloads selected source groups nationally and can consume many gigabytes.
Derived contexts are not misrepresented as publicly hosted downloads: `fetch`
rejects missing context entries and directs you to rebuilding. Use `--group
sources` for raw data. The ordinary project requirements are separate from the
pinned geospatial dependencies for this historical builder.
On POSIX use `builder-venv/bin/python`. Keep virtual environments machine-local:
recreate them on each computer, rather than expecting an environment on M to
survive a changed drive letter or operating system. The Windows/Python 3.14
resolved lock is `configs/data-vault/requirements-build-2020-win-py314.lock.txt`;
other platforms must resolve compatible wheels and pass the canonical hash gate.

Public sources are the Census [TIGER 2020 block archive](https://www2.census.gov/geo/tiger/TIGER2020/TABBLOCK20/),
[2020 PL 94-171 State files](https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/),
and [CD118 maps](https://www2.census.gov/geo/tiger/TIGER2020/CD/CD118/).
Downloads must match pinned hashes. PL ZIPs are validated through the exact
geography and population members used by the builder, not an unverified ZIP
checksum. Archive paths cannot write outside the target directory. Existing
different data is refused, not overwritten. Temporary downloaded ZIPs are
removed after admission; the admitted canonical inputs remain.

The builder reconstructs six historical source files from pinned Git commits
and explicit LF/CRLF rules into a **new vault workspace**. It does not edit the
clone or frozen evidence. It only admits a rebuilt context if its size and
SHA-256 match the recorded canonical context. Different geometry-library
behavior can cause a mismatch; such a candidate and its log remain in the
workspace for diagnosis, never as a falsely verified canonical input. The
successful RI run used GEOS 3.13.1 and PROJ 9.5.1. Downloads default to an 8-GiB
per-response ceiling and 900-second deadline; `fetch --max-bytes N --timeout S`
explicitly raises them. Streaming sockets time out after at most 30 seconds;
deadline observation can lag by one socket read. Free-space preflight reserves
the archive ceiling plus staging/admission space. Builds have a 3,600-second
wall-time cap (`build-2020 --timeout S`) and estimated disk-space preflight, but
no hard memory cap. Test small States before large jobs.

The versioned `admission.json` binds the catalog, selected raw inputs, wrapper,
historical builder sources, current State helper, command, resolved State,
installed Python packages/native libraries, thread settings and output/log/
manifest hashes. Existing verified contexts are not retroactively assigned new
execution receipts. Fresh builds create their own evidence.

## What replay means

`ri-replay` reads the actual context from the configured vault and re-executes
the unchanged adjacent-swap engine from its recorded six starting assignments.
It compares result hashes, metrics and stopping statuses with committed evidence.
Its new receipt lives under `VAULT/runs/portable-replay/RUN_NAME/analysis.json`.
It does not rewrite absolute paths or source hashes inside old manifests and
does not claim byte identity of historical machine-specific manifests.

For exact historical national reproduction, use checkpoint `81f396f5` and the
[national replay instructions](external/nrs-v0.3-national-verification/README.md).
Current main has intentionally changed some source hashes; see the
[integration record](specs/2026-09-10-main-integration.md). Installing Rust/METIS
and the relevant build dependencies remains necessary for national CLI runs.

## Historical source custody on the removable drive

The formerly detached replay source `c52b767…` is now preserved, with main
`d3dd4c6e…` and their full histories, in:

```text
VAULT/artifacts/source-custody/replay-c52b767-and-main-d3dd4c6e.bundle
```

Its portable [receipt](../configs/data-vault/source-custody-2026-09-10.json)
records SHA-256, size and refs. `git bundle verify PATH` verifies Git integrity;
compare SHA-256 with the receipt for artifact identity. On another computer:

```text
git clone --branch main PATH-TO-BUNDLE bisect-offline
git -C bisect-offline switch --detach c52b767db3522c0e4439cd3b9f01e9f556e80e8b
```

That archival bundle predates this portable setup code; use current Git for
the setup tool. A bundle is a source snapshot, not an automatic backup of future
commits. It does not include the bulk data. The vault is still a single physical
copy unless you maintain another backup separately.

The newer `VAULT/artifacts/source-custody/portable-vault-5e3c668d.bundle`
**does** include the hardened setup code. Clone it with
`--branch fix/portable-vault-role-review-2026-09-10`; its
[separate receipt](../configs/data-vault/source-custody-portable-2026-09-10.json)
identifies the verified snapshot. That offline clone passed the lean RI replay.

## Repository size and boundaries

Prefer `git clone --filter=blob:none URL` to avoid downloading every historical
blob upfront. It retains commit history needed for the pinned builder; a shallow
clone may need its history fetched before building. Partial clone does not make
the full current working tree small: at this checkpoint tracked files total
about 715 MB before portability hardening. Removing two roughly 42.6-MB ensemble
JSON files from current tracking reduces it to about 630 MB. Both remain on M
and in pinned Git history, recoverable through `scripts/evidence_vault.py`.
No shared history is rewritten and no frozen manifest is changed.

Use the [lean-checkout recipe](LEAN_CHECKOUT.md) for copy/paste commands,
historical evidence hydration and the 650-MB indexed-tree budget. Add other
evidence directories when running their verifiers; there is no promise that
every workspace test works with an intentionally incomplete checkout.

New heavyweight outputs should go to `VAULT/runs`, not under tracked `docs`.
Publish compact metrics, hashes, provenance and recovery recipes in Git.
For the QGIS publication workflow, active work uses the ignored local
`.work/publication` directory and completed packages are archived afterward;
see [local publication work and portable releases](PUBLICATION_PACKAGING.md).
Existing repository-to-vault links are not local publication scratch space.
The first two large historical blobs now have manifest/hydration support;
further archival work can extend that same catalog.

2000/2010 contexts are portable and hash-verified in the existing vault. Their
raw-source download/rebuild automation is **not yet included** in this new CLI.
Older recovery scripts on the vault are Windows-oriented; do not call the whole
three-cycle fresh-download path cross-platform ready. Other tracks' election,
survey and administrative datasets also need catalog expansion.
The [recovery matrix](DATA_RECOVERY_MATRIX.md) lists these tracks, existing
recipes and explicit acceptance gaps. It does not relabel missing recovery as done.
