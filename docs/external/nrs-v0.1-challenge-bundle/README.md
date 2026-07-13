# NRS v0.1 Challenge Bundle

**Status:** Internal external-review candidate  
**Purpose:** Permit a non-author to reproduce, challenge, or block the current
national-standard evidence without private project knowledge

## What To Challenge

1. Can the Rhode Island reference assignment be reproduced from the declared
   source, config, toolchain, and data?
2. Do the G.1--G.3 trace hashes and regenerated analysis match?
3. Are the public claims no stronger than the evidence?
4. Does the model statute preserve legal/community judgment?
5. Is the Exact Canonical Benchmark a coherent and verifiable North Star?

## Reference Replay

Source identity:

- upstream base commit:
  `d61a7136d60c27ecdd451067a1c08a063581820f`;
- runtime overlay:
  `docs/fixtures/nrs-reference-v0.1/runtime-overlay.patch`;
- config:
  `docs/fixtures/nrs-reference-v0.1/config.yml`; and
- expected hashes:
  `docs/fixtures/nrs-reference-v0.1/reference_manifest.json`.

From a clone at the base commit:

```powershell
git apply <bundle-repo>\docs\fixtures\nrs-reference-v0.1\runtime-overlay.patch
Copy-Item <bundle-repo>\docs\fixtures\nrs-reference-v0.1\config.yml `
  configs\nrs_reference_v0_1.yml

cargo build --release --locked -p bisect-cli --bin bisect
target\release\bisect.exe fetch --year 2020 --states RI `
  --type tiger redistricting adjacency --release --verify-downloads

target\release\bisect.exe build nrs_reference_v0_1 `
  --year 2020 --states RI --workers 1 --force --no-interactive

target\release\bisect.exe label-analyze nrs_reference_v0_1 `
  --year 2020 --types compactness,contiguity,splits,summary

target\release\bisect.exe label-report nrs_reference_v0_1 `
  --year 2020 --format html

target\release\bisect.exe label-verify nrs_reference_v0_1 --year 2020
```

The public `data-inputs-v1` prerelease contains the RI adjacency pickle,
binary graph, and GEOID index. Fetching only the pickle is insufficient for the
analysis stage.

Or run `replicate-reference.ps1`.

## Ensemble Verification

Run these commands from the challenge-bundle repository checkout, not from the
minimal base-plus-overlay reference worktree:

```powershell
Push-Location <bundle-repo>
python scripts\research\verify_real_ensemble_package.py `
  docs\examples\g-ensemble-evidence-packages\G.1-G.3+real-2020

cargo test -p bisect-ensemble real_g1_g3_package_validates
Pop-Location
```

The package must report Wisconsin's contiguity failure, incomplete 2016
coverage, low ESS in IA/NC, and cross-tool differences.

## Expected Result

A successful review may claim only:

- the reference assignment matched;
- the archived evidence package verified;
- the selected workflow was understandable; and
- the claim boundaries were visible.

It may not claim legal validity, fairness, VRA compliance, block-level
conformance, external peer review, or exact optimality.

## Recording A Result

Use `replication-record-template.md`. A failed or blocked replication is a valid
result and must not be silently repaired by the project author.
