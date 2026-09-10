# Data recovery coverage and priorities

This matrix separates integrity of existing files from the ability to recover
without this particular removable drive. A listed script is not proof that its
full recovery path has been validated. Project-vault paths are relative.

| Track | Existing-vault custody | Without this drive | Acceptance / next action |
| --- | --- | --- | --- |
| 2020 certified block contexts | 50 canonical context hashes plus raw block/PL catalog | `data_vault.py fetch --group sources --year 2020 --state STATE --apply`, then `build-2020` | RI exact rebuild tested; national fresh-build validation remains open. |
| 2000 certified block contexts | 50 canonical context hashes | Historical source is in Git; `scripts/research/build_national_2000.py` and vault recovery recipes need a portable wrapper | Priority 1: bind raw input hashes/URLs and environment, then RI-sized fresh recovery before national claims. |
| 2010 certified block contexts | 50 canonical context hashes | `scripts/data/geography/build_2010_rctx_streaming.ps1` is Windows-oriented, not the new portable CLI | Priority 1: same raw-source and isolated-build acceptance gate as 2000. |
| Enacted CD118 maps | 50 pinned 2020 TIGER archives | `data_vault.py fetch --group enacted --year 2020 --apply` | RI public download tested; all 50 catalogued. Historical enacted-map vintages are separate. |
| National replay packages | `runs/nrs-v0.3-replay/external-replay-2026-09-09-r6` | Historical replay instructions and pinned source checkpoints | Preserve frozen manifests; national recreation requires its historical toolchain. |
| Large governed ensemble evidence | `artifacts/git-evidence`; pinned Git fallback | `python scripts/evidence_vault.py --vault PATH --apply` | Both large traces recovered from Git into an empty vault and checked byte-for-byte. |
| CPS turnout | `source-data/external/cps` | `scripts/fetch_cps_turnout.py` exists | Priority 2: inventory exact inputs, provenance and licensing; recovery not certified by the new catalog. |
| VEST/election data | `source-data/external/vest`, `vest_california`, `raw` | `scripts/fetch_vest.py`, `scripts/data/elections/download_vest.py`, DOI list | Priority 2: dataset-specific access/licensing and checksum coverage before a universal download claim. |
| Voteview | `source-data/external/voteview` | `scripts/fetch_voteview.py` exists | Priority 2: catalog input snapshots and replay consumers. |
| Tract relationships | `source-data/external/tract_relationships` | Existing track-specific source recipes, not catalogued here | Priority 2: inventory vintages/joins before cross-cycle regeneration claims. |
| New code and papers | Git remote plus explicitly versioned offline source bundles | Clone the published revision; bundles are snapshots | Publish reviewed code; never treat a removable drive as the only backup. |

Scope acceptance: the current portable CLI is a 2020 build foundation plus
three-cycle existing-context verification. The 2000/2010 and other-track rows
remain explicit project work, not silently completed by this matrix. No Census,
VEST or other external access terms are overridden by these recovery recipes.
