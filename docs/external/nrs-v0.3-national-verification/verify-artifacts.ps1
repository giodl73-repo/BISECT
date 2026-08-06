param(
    [string]$RecordPath = "runs/nrs-v0.3/external-artifact-record.json",
    [string]$PythonCommand = "python",
    [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "../../..")).Path
Push-Location $repoRoot
try {
    $dirty = (& git status --porcelain) -join "`n"
    if (-not $AllowDirty -and $dirty) {
        throw "Independent verification requires a clean working tree: $dirty"
    }
    $commit = (& git rev-parse HEAD).Trim()
    $pythonVersion = (& $PythonCommand --version 2>&1) -join " "

    function Test-TransportHash([string]$Path, [string]$Expected) {
        $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -eq $Expected) { return $true }
        $bytes = [IO.File]::ReadAllBytes($Path)
        try {
            $utf8 = [Text.UTF8Encoding]::new($false, $true)
            $content = $utf8.GetString($bytes)
        }
        catch { return $false }
        $normalized = $content.Replace("`r`n", "`n").Replace("`r", "`n")
        foreach ($newline in "`n", "`r`n") {
            $transported = $utf8.GetBytes($normalized.Replace("`n", $newline))
            $hasher = [Security.Cryptography.SHA256]::Create()
            try {
                $hash = [BitConverter]::ToString($hasher.ComputeHash($transported)).Replace("-", "").ToLowerInvariant()
            }
            finally { $hasher.Dispose() }
            if ($hash -eq $Expected) { return $true }
        }
        return $false
    }

    function Assert-ManifestFiles([string]$PackageDir) {
        $manifestPath = Join-Path $PackageDir "manifest.json"
        if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
            throw "Missing manifest: $manifestPath"
        }
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        foreach ($file in $manifest.files) {
            $path = Join-Path $PackageDir $file.path
            if (-not (Test-TransportHash $path $file.sha256)) {
                throw "Hash mismatch: $path"
            }
        }
    }

    foreach ($year in 2000, 2010, 2020) {
        Assert-ManifestFiles "docs/experiments/nrs-v0.3-national-$year"
    }
    Assert-ManifestFiles "docs/experiments/nrs-cross-decade-2000-2020/comparison"
    Assert-ManifestFiles $PSScriptRoot

    & $PythonCommand scripts/research/verify_nrs_cross_census.py
    if ($LASTEXITCODE -ne 0) { throw "Independent cross-census verifier failed" }
    $matrix = Get-Content -LiteralPath `
        "docs/experiments/nrs-cross-decade-2000-2020/comparison/stability-matrix.json" `
        -Raw | ConvertFrom-Json
    $comparisonManifest = Get-Content -LiteralPath `
        "docs/experiments/nrs-cross-decade-2000-2020/comparison/manifest.json" `
        -Raw | ConvertFrom-Json
    $governedMatrixHash = ($comparisonManifest.files | Where-Object path -eq "stability-matrix.json").sha256
    if ($matrix.all_cycle_common_node_signatures -ne 120 -or
        $matrix.all_cycle_exact_topology_states -ne 18) {
        throw "Cross-census headline counts drifted"
    }

    $record = [ordered]@{
        schema_version = "nrs-v0.3-external-artifact-verification-v1"
        status = "pass"
        verified_at_utc = [DateTime]::UtcNow.ToString("o")
        git_commit = $commit
        working_tree_clean = -not [bool]$dirty
        machine_name = [Environment]::MachineName
        operating_system = [System.Runtime.InteropServices.RuntimeInformation]::OSDescription
        process_architecture = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString()
        powershell_version = $PSVersionTable.PSVersion.ToString()
        python_command = $PythonCommand
        python_version = $pythonVersion
        census_years = @(2000, 2010, 2020)
        verified_states_per_cycle = 50
        verified_districts_per_cycle = 435
        verified_nodes_per_cycle = 385
        all_cycle_common_node_signatures = 120
        all_cycle_exact_topology_states = 18
        matrix_sha256 = $governedMatrixHash
        matrix_transport_sha256 = (Get-FileHash -LiteralPath `
            "docs/experiments/nrs-cross-decade-2000-2020/comparison/stability-matrix.json" `
            -Algorithm SHA256).Hash.ToLowerInvariant()
        claim_boundary = "Artifact verification only; no assignment regeneration, legal, fairness, VRA, adoption, or exact boundary/canonical claim."
    }
    $resolvedRecord = Join-Path $repoRoot $RecordPath
    $recordDir = Split-Path -Parent $resolvedRecord
    New-Item -ItemType Directory -Path $recordDir -Force | Out-Null
    $record | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $resolvedRecord -Encoding utf8
    Write-Host "NRS v0.3 second-laptop artifact verification: PASS"
    Write-Host "Record: $resolvedRecord"
}
finally {
    Pop-Location
}
