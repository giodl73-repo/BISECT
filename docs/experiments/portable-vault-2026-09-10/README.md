# Portable vault validation — 2026-09-10

Outcome: existing-vault integrity and a fresh public-download RI 2020 rebuild
passed on Windows, Python 3.14.2. This is not a national fresh-build or
cross-platform validation claim. See the [setup guide](../../PORTABLE_DATA.md).

## Checked in this round

- All 150 canonical contexts (50 each for 2000, 2010, 2020) on the existing
  vault matched the catalog SHA-256 values.
- An initially empty validation vault downloaded RI blocks, PL geography,
  PL population and enacted maps from the catalog's public Census URLs.
  All four admitted inputs matched the pinned hashes.
- The pinned historical builder regenerated the RI context with SHA-256
  `3d574a360458feb915f4febee97afd42d29f8a2fccd22d8e9ee8df04f7984371`.
  The build reported 25,649 units, 66,161 edges and 64 bridges.
- All six adjacent-swap replay arms matched recorded assignment hashes,
  final metrics and stopping statuses. The compact [replay receipt](replay.json)
  records input and engine hashes. It tests semantic replay from recorded
  starting assignments, not rebuilding those starting assignments from scratch.
- Final `status --year 2020 --state RI --hash` in the fresh vault verified
  all five catalog entries, including the rebuilt context.
- 47 tests passed: 12 vault safety/relocation tests and 35 existing county/RI
  diagnostic tests. These unit tests require neither M nor Census network access.
- `git bundle verify` passed for the offline source bundle, which contains
  complete history and two refs. Its [custody receipt](../../../configs/data-vault/source-custody-2026-09-10.json)
  records its identity. It predates the new setup code.
- The machine-local configuration is ignored by Git. Existing local data
  directories were retained rather than replaced.

## Evidence identity and location

The tested base commit was `d3dd4c6ef6a66ffea490bca21a6ad2853ccaf257`.
The new portability implementation was uncommitted during validation; its
driver hash, rather than that base commit alone, identifies tested code.

| Artifact | SHA-256 |
| --- | --- |
| `scripts/data_vault.py` | `2ad2143ef40dff03bfcf9dfbd5cdb4dcc8f93c63c65da7d7fb9421db4cddb999` |
| `configs/data-vault/catalog-v1.json` | `65b66562ea4bb8692276550b6830812511d869d503e7afb126f16c304a9a5110` |
| `configs/data-vault/builder-2020.json` | `84361916c0032b8f1490bbd4b18ce45d4327c6da93ce325a3b02554d5630d6d9` |

Large validation artifacts remain under the project vault at
`portability-validation/fresh-ri-2026-09-10/`. Its build workspace is
`runs/context-builds/from-public-downloads/` and its replay receipt is
`runs/portable-replay/fresh-download-replay/analysis.json`.
No downloaded datasets or rebuilt contexts were added to Git.

## Remaining work

2000/2010 raw download/rebuild automation, other research-track catalogs,
national fresh-build validation and non-Windows end-to-end validation remain
open. The current tracked tree was approximately 715 MB before this change;
moving legacy large evidence into a recoverable manifest/hydration workflow
also remains open. No shared Git history was rewritten. A removable vault
still needs a separate backup to protect against drive loss.
