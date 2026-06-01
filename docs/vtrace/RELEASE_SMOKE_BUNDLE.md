# Release Smoke Bundle

## DCR-002 canonical scope

Status: defined as an L1 smoke contract; data-backed execution is blocked until
the selected census/source cache is available in the execution environment.

| Field | Value |
|---|---|
| Label/config | `official_2020` from `configs/official_2020.yml` when present; otherwise the release candidate label named by the release manager. |
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
bisect build official_2020 --year 2020 --workers 8
bisect label-analyze official_2020 --year 2020 --types all
bisect label-report official_2020 --year 2020 --format html json
bisect label-verify official_2020 --year 2020
```

Known failure modes:

- Missing or incomplete `data/2020/` source cache.
- Missing `configs/official_2020.yml` or release-selected config.
- METIS engine build/runtime mismatch.
- Population/contiguity/source validation failure.
- Hash-chain mismatch after artifacts are modified.

## Current disposition

DCR-002 is partially satisfied at L1 by a predeclared smoke scope and executable
fixture smoke. It is not closed as release-ready until the real-state sequence is
run and its command output, artifact paths, and verification result are recorded.

Environment check recorded 2026-06-01: `configs/official_2020.yml`,
`configs/*.yml`, and `data/2020/` were not present in the working checkout, so
the real-state smoke sequence is blocked in this environment rather than failed.
