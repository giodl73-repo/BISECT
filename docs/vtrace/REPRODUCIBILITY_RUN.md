# Reproducibility Run Record

## DCR-007 declared scope

Current reproducibility class: `release-subset-candidate-data-dirty`.

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

## Candidate data-dirty release-subset replay evidence

| Field | Value |
|---|---|
| Scope | `official_proposal/2020`, state `VT`, pre-provisioned local `data/2020/`. |
| Source commit | `af11004a78e654f09cc1f01ca186b3f4487e02a3`. |
| Source policy | `--allow-dirty-data`; source tree was clean except modified local `data/manifest.json`, so this is not L2 clean replay evidence. |
| Config SHA-256 | `efc30a5bbd76692f0b12fea5b1a2395dfcef4ee17b840e350f2ce25cda162212`. |
| Data manifest SHA-256 | `9837e5cd0dfd4b9fad6e6802475d2186ed1b9316bd75d5e577bedb0f74c6dc6e`. |
| Algorithm/search | `apportion-regions`, `county`, `alpha_county=2.0`, `convergence`, `convergence_threshold=600`, `balance_tolerance=0.5`, `metis_engine=c-ffi`. |
| Commands | Harness ran `cargo build`, `bisect build`, `bisect label-analyze`, `bisect label-report`, and `bisect label-verify`; all exited 0. |
| Artifact hashes | 9 run, analysis, and report artifacts hashed by the replay harness. |
| Verification result | Config, build-index, and analysis-index SHA chain reported `MATCH`; verdict `VERIFIED`. |
| Replay class | release-subset candidate with dirty-data allowance; not clean L2 release-subset or full-scale reproducibility evidence. |

## Required L2 release-subset/full-scale record

The maintenance harness
`scripts/maintenance/dcr007_release_subset_replay.py` is the controlled way to
capture a candidate release-subset replay record. It writes an ignored JSON
evidence file under `reports/vtrace/` by default, records environment/tool
versions, git status, config and data-manifest hashes, resolved algorithm/search
parameters, resolved METIS engine, planned command lines, command outputs, and
generated artifact hashes. The replay record separates `clean_for_l2_replay`
from `candidate_command_allowed` so data-dirty candidate execution cannot be
mistaken for clean replay evidence.

The strict L2 launcher
`scripts/maintenance/dcr007_clean_replay.py` performs the operator cleanliness
gate before invoking the harness. It refuses any non-empty `git status
--short`, rejects `--allow-dirty-data`, and then delegates to
`dcr007_release_subset_replay.py` only when the checkout is clean.

Typical candidate preflight while local data manifests are dirty:

```bash
python scripts/maintenance/dcr007_release_subset_replay.py --preflight-only --allow-dirty-data
```

Typical clean release-subset replay command:

```bash
python scripts/maintenance/dcr007_clean_replay.py
```

## L2 clean replay packet

This packet is the required script for a future DCR-007 L2 release-subset or
full-scale replay. It is a procedure and review template, not closure evidence
by itself.

Declared replay fields:

| Field | Value |
|---|---|
| Replay class | release-subset or full-scale. |
| Label | `official_proposal` unless a release manager declares a different label. |
| Year(s) | `2020` for the current VT release-subset packet; all selected years for full-scale replay. |
| State scope | `VT` for the current release-subset packet; all selected states for broader replay. |
| Source checkout | Clean `git status --short` with no data, output, doc, source, or config modifications. |
| Data custody | `data/manifest.json` hash, data source provenance, and any externally stored source-data hash manifest. |
| Output custody | Ignored local `reports/vtrace/*.json` replay record plus promoted evidence-package location if reviewed for release. |

Operator tasks:

1. Start from a clean checkout at the declared commit.
2. Provision the declared source data without modifying tracked files.
3. Run `git --no-pager status --short` and confirm there is no output.
4. Run `python scripts/maintenance/dcr007_clean_replay.py`; the strict launcher must reject dirty checkouts before the replay harness starts.
5. Confirm the JSON record reports `clean_for_l2_replay: true`, `candidate_command_allowed: true`, `result: replay_pass`, and all command exit codes are 0.
6. Inspect `label-verify` output and confirm the config, build-index, and analysis-index SHA links are `MATCH` with verdict `VERIFIED`.
7. Preserve the replay JSON and any promoted evidence package under the declared custody rule.

Reviewer checklist:

| Check | Required result |
|---|---|
| Source cleanliness | `clean_for_l2_replay: true`; no dirty tracked or untracked source/data entries. |
| Scope match | Record label, year, states, workers, report formats, and replay class match the declared scope. |
| Toolchain provenance | OS, Python, Rust/Cargo paths and versions, binary SHA-256, and resolved METIS engine are present. |
| Input provenance | Config SHA-256, data-manifest SHA-256, and data custody pointer are present. |
| Algorithm/search metadata | Structure, weights, tolerance, search strategy, convergence threshold, and engine are present. |
| Command replay | Build, analyze, report, and verify commands all exit 0 or each failure has explicit disposition. |
| Artifact custody | Run, analysis, report, and replay JSON hashes are retained or intentionally not promoted with reason. |
| Claim boundary | Public claims name only the replay class, state/year scope, data custody, and reviewed artifact set. |

Promotion rule: DCR-007 may move beyond
`partial_l1_release_subset_candidate_data_dirty` only after MERIDIAN/COVENANT
review accepts a clean replay record and dispositions every divergence as fixed,
accepted limitation, or environment blocker. VAULT review is required before
promoting artifacts as a public release evidence package.

Before DCR-007 can close at L2 release-subset or full-scale level, the generated
record must include:

- Clean checkout commit and working-tree status.
- Rust toolchain versions, target, OS, CPU class, and build features.
- Binary SHA-256 and resolved METIS engine.
- Config path, SHA-256, and algorithm/search parameters.
- Source-data custody and hash pointers.
- Exact command lines for build, analyze, report, and verify.
- Seed/search metadata and convergence settings.
- Artifact paths and SHA-256 values.
- Clean replay comparison result or divergence disposition.
- `clean_for_l2_replay: true`; data-dirty candidate runs are not sufficient.

## Current disposition

DCR-007 is not closed for L2 full-scale or clean release-subset reproducibility.
Public claims may cite only the declared fixture, smoke, and candidate replay
scopes with their stated dirty-data limitation until a selected clean data-backed
replay is executed and reviewed. The current local checkout has a dirty
`data/manifest.json`, so `--allow-dirty-data` evidence is useful as tooling and
candidate replay evidence, but it is not clean replay evidence.
