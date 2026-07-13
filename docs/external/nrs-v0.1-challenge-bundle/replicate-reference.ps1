param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot,
    [string]$BundleRepo = (Resolve-Path "$PSScriptRoot\..\..\.."),
    [switch]$SkipFetch
)

$ErrorActionPreference = "Stop"
$expectedBase = "d61a7136d60c27ecdd451067a1c08a063581820f"
$repo = (Resolve-Path $RepoRoot).Path
$bundle = (Resolve-Path $BundleRepo).Path

$head = (git -C $repo rev-parse HEAD).Trim()
if ($head -ne $expectedBase) {
    throw "expected base commit $expectedBase, found $head"
}
if (git -C $repo status --porcelain) {
    throw "replication checkout must be clean before applying the overlay"
}

$overlay = Join-Path $bundle "docs\fixtures\nrs-reference-v0.1\runtime-overlay.patch"
$config = Join-Path $bundle "docs\fixtures\nrs-reference-v0.1\config.yml"
$manifestPath = Join-Path $bundle "docs\fixtures\nrs-reference-v0.1\reference_manifest.json"
$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json

git -C $repo apply --check $overlay
git -C $repo apply $overlay
Copy-Item $config (Join-Path $repo "configs\nrs_reference_v0_1.yml")

Push-Location $repo
try {
    cargo build --release --locked -p bisect-cli --bin bisect
    if (-not $SkipFetch) {
        .\target\release\bisect.exe fetch --year 2020 --states RI `
          --type tiger redistricting adjacency --release --verify-downloads
    }
    .\target\release\bisect.exe build nrs_reference_v0_1 `
      --year 2020 --states RI --workers 1 --force --no-interactive
    .\target\release\bisect.exe label-analyze nrs_reference_v0_1 `
      --year 2020 --types compactness,contiguity,splits,summary
    .\target\release\bisect.exe label-report nrs_reference_v0_1 `
      --year 2020 --format html
    $assignment = "runs\nrs_reference_v0_1\2020\rhode_island\final_assignments.json"
    $raw = (Get-FileHash $assignment -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($raw -ne $manifest.expected_output.run_1_raw_assignment_sha256) {
        throw "assignment hash mismatch: expected $($manifest.expected_output.run_1_raw_assignment_sha256), found $raw"
    }
    .\target\release\bisect.exe label-verify nrs_reference_v0_1 --year 2020
    "REFERENCE REPLICATION PASS: $raw"
}
finally {
    Pop-Location
}
