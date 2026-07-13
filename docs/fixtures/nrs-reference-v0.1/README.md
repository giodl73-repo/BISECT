# NRS v0.1 Rhode Island Reference Replay

This package records a two-execution Rhode Island 2020 functional replay for
the candidate national-standard benchmark profile.

## Claim Boundary

The package establishes that two clean-source executions produced identical
parsed tract-to-district assignments and independently verified label chains.
It does not establish:

- block-level NRS v0.1 conformance;
- byte-identical executable reproduction across build environments;
- external/non-author replication;
- legal validity, fairness, VRA compliance, or community preservation; or
- custody of the raw Census files outside the recorded local hashes.

## Source Reconstruction

The validation snapshot was constructed from:

```powershell
git checkout d61a7136d60c27ecdd451067a1c08a063581820f
git apply docs\fixtures\nrs-reference-v0.1\runtime-overlay.patch
Copy-Item docs\fixtures\nrs-reference-v0.1\config.yml `
  configs\nrs_reference_v0_1.yml
```

The portable overlay and config hashes are computed after LF normalization, as
required by `docs/file-formats/manifests.md`. The manifest separately records
the exact bytes observed in the Windows validation worktree where they differ.

## Build And Run

Provision the hashed Rhode Island inputs at the paths in the manifest, then:

```powershell
cargo build --release --locked -p bisect-cli --bin bisect

target\release\bisect.exe build nrs_reference_v0_1 `
  --year 2020 --states RI --workers 1 --force --no-interactive

target\release\bisect.exe label-analyze nrs_reference_v0_1 `
  --year 2020 --types compactness,contiguity,splits,summary

target\release\bisect.exe label-report nrs_reference_v0_1 `
  --year 2020 --format html

target\release\bisect.exe label-verify nrs_reference_v0_1 --year 2020
```

## Assignment Verification

The assignment writer now emits lexicographically ordered JSON keys. Verify the
raw file first:

```powershell
(Get-FileHash `
  runs\nrs_reference_v0_1\2020\rhode_island\final_assignments.json `
  -Algorithm SHA256).Hash.ToLowerInvariant()
```

Expected raw-file SHA-256:

```text
930d3b18024d64ed17f640ac37d16a0204fc318c9df5332f074b5cb0491dac71
```

The functional comparator remains the canonical parsed mapping:

```powershell
python -c "import json,hashlib,sys; d=json.load(open(sys.argv[1],encoding='utf-8')); b=json.dumps(d,sort_keys=True,separators=(',',':')).encode(); print(hashlib.sha256(b).hexdigest())" `
  runs\nrs_reference_v0_1\2020\rhode_island\final_assignments.json
```

Expected:

```text
6cd96b33ac8fdae2d8e5e4b7bc9674358311eed62becbe624e6913d1507b4822
```

Both clean-source executions produced the same raw and canonical hashes.
