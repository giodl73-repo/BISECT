[CmdletBinding()]
param(
    [string[]]$States,
    [string]$BisectOps = "target/release/bisect-ops.exe",
    [switch]$KeepExtracted
)

$ErrorActionPreference = "Stop"
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\..\.."))
$inventoryPath = Join-Path $repoRoot "docs/experiments/nationwide-2010/inventory.json"
$inventory = Get-Content -LiteralPath $inventoryPath -Raw | ConvertFrom-Json
$binary = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $BisectOps))
if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
    throw "bisect-ops release binary not found: $binary"
}

$rows = @{}
foreach ($row in $inventory.states) {
    $rows[$row.state] = $row
}
if (-not $States -or $States.Count -eq 0) {
    $States = @($inventory.batch_order)
}
$States = @($States | ForEach-Object { $_.ToUpperInvariant() })

$archiveRoot = Join-Path $repoRoot "data/2010/tiger/archives"
$blockRoot = Join-Path $repoRoot "data/2010/tiger/blocks"
$contextRoot = Join-Path $repoRoot "data/2010/certified"
$reportRoot = Join-Path $repoRoot "docs/experiments/nationwide-2010/rctx"
foreach ($directory in @($archiveRoot, $blockRoot, $contextRoot, $reportRoot)) {
    New-Item -ItemType Directory -Path $directory -Force | Out-Null
}

foreach ($state in $States) {
    if (-not $rows.ContainsKey($state)) {
        throw "state is not in the governed 2010 inventory: $state"
    }
    $row = $rows[$state]
    $lower = $state.ToLowerInvariant()
    $baseName = "tl_2010_$($row.fips)_tabblock10"
    $archive = Join-Path $archiveRoot "$baseName.zip"
    $partial = "$archive.partial"
    $extractDir = Join-Path $blockRoot $baseName
    $shape = Join-Path $extractDir "$baseName.shp"

    Write-Host "$($state): acquiring governed TIGER archive"
    if (-not (Test-Path -LiteralPath $archive -PathType Leaf)) {
        if (Test-Path -LiteralPath $partial) {
            Remove-Item -LiteralPath $partial -Force
        }
        Invoke-WebRequest -Uri $row.tiger_source_url -OutFile $partial
        Move-Item -LiteralPath $partial -Destination $archive
    }

    if (Test-Path -LiteralPath $extractDir) {
        $resolvedExtract = [System.IO.Path]::GetFullPath($extractDir)
        if (-not $resolvedExtract.StartsWith(
            [System.IO.Path]::GetFullPath($blockRoot) + [System.IO.Path]::DirectorySeparatorChar,
            [System.StringComparison]::OrdinalIgnoreCase
        )) {
            throw "unsafe extraction target: $resolvedExtract"
        }
        Remove-Item -LiteralPath $resolvedExtract -Recurse -Force
    }
    New-Item -ItemType Directory -Path $extractDir -Force | Out-Null
    Expand-Archive -LiteralPath $archive -DestinationPath $extractDir -Force
    if (-not (Test-Path -LiteralPath $shape -PathType Leaf)) {
        throw "$state archive did not contain $baseName.shp"
    }

    Write-Host "$($state): building hash-bound 2010 block context"
    $plDir = Join-Path $repoRoot "data/2010/redistricting/$($lower)2010.pl"
    $buildArgs = @(
        "build-state-rctx",
        "--year", "2010",
        "--state-code", $state,
        "--state-fips", $row.fips,
        "--state-name", $row.name,
        "--shapefile", $shape,
        "--tiger-archive", $archive,
        "--pl-geo", (Join-Path $plDir "$($lower)geo2010.pl"),
        "--pl-population", (Join-Path $plDir "$($lower)000012010.pl"),
        "--rctx", (Join-Path $contextRoot "$($lower)_blocks_2010.rctx"),
        "--report", (Join-Path $reportRoot "$lower.json"),
        "--manifest", (Join-Path $reportRoot "$lower-manifest.json")
    )
    & $binary @buildArgs
    if ($LASTEXITCODE -ne 0) {
        throw "$state RCTX build failed with exit code $LASTEXITCODE"
    }

    if (-not $KeepExtracted) {
        Remove-Item -LiteralPath $extractDir -Recurse -Force
    }
}

Push-Location $repoRoot
try {
    python scripts/research/inventory_national_2010.py
    if ($LASTEXITCODE -ne 0) {
        throw "2010 inventory refresh failed"
    }
    & $binary verify-national-rctx --year 2010
    if ($LASTEXITCODE -ne 0) {
        throw "2010 partial RCTX verification failed"
    }
}
finally {
    Pop-Location
}
