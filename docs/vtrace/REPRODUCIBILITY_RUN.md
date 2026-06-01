# Reproducibility Run Record

## DCR-007 declared scope

Current reproducibility class: `release-subset-smoke`.

Full-scale and L2 release-subset reproducibility remain blocked until a clean
checkout/environment replay and artifact storage target are selected.

Environment update recorded 2026-06-01: local `configs/official_proposal.yml`
and `data/2020/` became available, enabling the VT real-state smoke recorded
below. This does not constitute a clean-environment replay or full-scale run.

## Smoke-only evidence

| Field | Value |
|---|---|
| Scope | Public label-import fixtures under `docs/fixtures/import-label/`. |
| Command | `cargo test -p bisect-cli public_import_fixture --lib -- --test-threads=1` |
| Build features | Cargo defaults for `bisect-cli`. |
| Source-data hashes | Not applicable; fixture inputs are committed text files. |
| Output comparison | Parser assignments compared against committed expected-assignment JSON. |
| Replay class | smoke-only; not a real-state or full-scale redistricting replay. |

## Release-subset smoke evidence

| Field | Value |
|---|---|
| Scope | `official_proposal/2020`, state `VT`, pre-provisioned local `data/2020/`. |
| Build command | `.\target\debug\bisect.exe build official_proposal --year 2020 --states VT --workers 1 --force --no-interactive` |
| Analyze command | `.\target\debug\bisect.exe label-analyze official_proposal --year 2020 --types all` |
| Report command | `.\target\debug\bisect.exe label-report official_proposal --year 2020 --format html` |
| Verify command | `.\target\debug\bisect.exe label-verify official_proposal --year 2020` |
| Build artifact paths | `runs/official_proposal/2020/index.json`; `runs/official_proposal/2020/vermont/final_assignments.json`; `runs/official_proposal/2020/vermont/provenance.json` |
| Analysis/report paths | `analysis/official_proposal/2020/index.json`; `analysis/official_proposal/2020/vermont/all.json`; `reports/official_proposal/2020/index.json`; `reports/official_proposal/2020/official_proposal_2020_report.html` |
| Verification result | Config, build-index, and analysis-index SHA chain reported `MATCH`; verdict `VERIFIED`. |
| Replay class | release-subset-smoke; single-state smoke only, not a clean replay or full-scale reproducibility run. |

## Required L2 release-subset/full-scale record

The maintenance harness
`scripts/maintenance/dcr007_release_subset_replay.py` is the controlled way to
capture a candidate release-subset replay record. It writes an ignored JSON
evidence file under `reports/vtrace/` by default, records environment, git
status, config and data-manifest hashes, planned command lines, command outputs,
and generated artifact hashes, and refuses clean replay unless the source status
policy is satisfied.

Typical preflight while local data manifests are dirty:

```bash
python scripts/maintenance/dcr007_release_subset_replay.py --preflight-only --allow-dirty-data
```

Typical clean release-subset replay command:

```bash
python scripts/maintenance/dcr007_release_subset_replay.py
```

Before DCR-007 can close at L2 release-subset or full-scale level, the generated
record must include:

- Clean checkout commit and working-tree status.
- Rust toolchain, target, OS, CPU class, and build features.
- Binary SHA-256 and METIS engine.
- Config path and SHA-256.
- Source-data custody and hash pointers.
- Exact command lines for build, analyze, report, and verify.
- Seed/search metadata and convergence settings.
- Artifact paths and SHA-256 values.
- Clean replay comparison result or divergence disposition.

## Current disposition

DCR-007 is not closed for L2 full-scale or clean release-subset reproducibility.
Public claims may cite only the declared fixture and VT release-subset smoke
scopes until a selected clean data-backed replay is executed and reviewed. The
current local checkout has a dirty `data/manifest.json`, so preflight may use
`--allow-dirty-data` only as tooling evidence; it is not clean replay evidence.
