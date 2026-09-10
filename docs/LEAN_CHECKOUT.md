# Lean coding / Rhode Island checkout

The indexed Git tree has a **650,000,000-byte budget** for this release.
`python scripts/evidence_vault.py --check-size` checks staged/indexed contents,
not the size of a hydrated local working directory or historical Git objects.
The two largest governed ensemble traces have moved to recoverable external
storage; historical commits are unchanged. Hydration is optional for RI replay.

From a new destination, using Git's partial clone and cone-mode sparse checkout:

```text
git clone --filter=blob:none --no-checkout https://github.com/giodl73-repo/BISECT.git bisect-lean
cd bisect-lean
git sparse-checkout init --cone
git sparse-checkout set crates configs scripts tests archive/legacy-python docs/experiments/ri-boundary-budget-2020 docs/experiments/ri-adjacent-swaps-2020
git checkout main
python scripts/data_vault.py configure --vault YOUR-PROJECT-VAULT
python scripts/data_vault.py ri-replay --run-name lean-check
```

Use the published portability revision or its branch until it has merged into
main. Cone mode includes root files. Do not use a shallow clone for historical
builders; they need their pinned commits. Git may download missing historical
blobs on demand. An offline clone needs a complete source bundle instead.

To run the older block-ensemble verifiers, first include their evidence directory
and restore its two large files:

```text
git sparse-checkout add docs/experiments/nrs-v0.3-block-ensemble-gate docs/specs
python scripts/evidence_vault.py --vault YOUR-PROJECT-VAULT
python scripts/evidence_vault.py --vault YOUR-PROJECT-VAULT --apply
```

Hydration uses verified vault copies when present, otherwise pinned historical
Git bytes. Both routes enforce the same SHA-256 and size. A differing existing
local file is refused, never overwritten. Restored files are ignored by Git;
the frozen package manifest and its verifiers remain unchanged. Hydrating adds
about 85 MB locally but does not add those bytes back to the current Git tree.
Sparse checkout intentionally does not support every full-workspace test.
