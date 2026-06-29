# Release Smoke Bundle

## DCR-002 canonical scope

Status: closed at L1 for the declared VT real-state smoke on the local
`official_proposal` release-candidate config.

| Field | Value |
|---|---|
| Label/config | `official_proposal` from `configs/official_proposal.yml` for the recorded local smoke; otherwise the release candidate label named by the release manager. |
| Year | `2020` |
| State/scope | `VT` for the smallest real-state smoke; fixture-only parser smoke uses `docs/fixtures/import-label/`. |
| Data provisioning | Real-state smoke requires pre-provisioned `data/2020/` and adjacency/source cache or a successful `bisect fetch --year 2020`. Fixture smoke requires no census data. |
| Runtime class | Fixture smoke: seconds. VT real-state smoke: minutes on a configured workstation. |
| Expected artifacts | `runs/{label}/2020/...`, `analysis/{label}/2020/...`, `reports/{label}/2020/...`, label verification output. |
| Expected verification | `bisect label-verify {label} --year 2020` exits `0` only when the recorded artifacts and hashes are present and unchanged. |

## Fixture smoke command

```bash
cargo test -p bisect-cli public_import_fixture --lib -- --test-threads=1
```

This command proves the public import fixtures parse to expected assignments. It
does not prove a full state build, analysis, report, or verification chain.

## Real-state smoke command sequence

```bash
bisect fetch --year 2020 --workers 8
bisect build official_proposal --year 2020 --states VT --workers 1 --no-interactive
bisect label-analyze official_proposal --year 2020 --types all
bisect label-report official_proposal --year 2020 --format html
bisect label-verify official_proposal --year 2020
```

The recorded run used pre-provisioned local `data/2020/` instead of running
`bisect fetch`.

Known failure modes:

- Missing or incomplete `data/2020/` source cache.
- Missing `configs/official_proposal.yml` or release-selected config.
- METIS engine build/runtime mismatch.
- Population/contiguity/source validation failure.
- Hash-chain mismatch after artifacts are modified.

## Current disposition

DCR-002 is closed at L1 for the declared VT smoke scope. It is not L2 public
release readiness and does not prove all-state or all-year health.

Execution recorded 2026-06-01:

| Command / Check | Result | Evidence |
|---|---|---|
| `.\target\debug\bisect.exe build official_proposal --year 2020 --states VT --workers 1 --force --no-interactive` | pass | `runs/official_proposal/2020/index.json`; `runs/official_proposal/2020/vermont/final_assignments.json`; `runs/official_proposal/2020/vermont/provenance.json` |
| `.\target\debug\bisect.exe label-analyze official_proposal --year 2020 --types all` | pass | `analysis/official_proposal/2020/index.json`; `analysis/official_proposal/2020/vermont/all.json` |
| `.\target\debug\bisect.exe label-report official_proposal --year 2020 --format html` | pass | `reports/official_proposal/2020/index.json`; `reports/official_proposal/2020/official_proposal_2020_report.html` |
| `.\target\debug\bisect.exe label-verify official_proposal --year 2020` | pass | Config, build-index, and analysis-index SHA chain reported `MATCH`; verdict `VERIFIED`. |

Implementation note: the first smoke attempt exposed a build/analyze artifact
contract mismatch. `crates/bisect-cli/src/build_cmd.rs` now promotes successful
state runner artifacts from the runner's nested legacy output path into
`runs/{label}/{year}/{state}/final_assignments.json`, which is the label-analyze
contract path.
