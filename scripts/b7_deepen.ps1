# B.7 Deepening — run NH (0→200 seeds), TX (201→500), FL (201→500), WI (1201→1400)
# Run with: pwsh -File scripts\b7_deepen.ps1 [-States "NH,TX,FL,WI"]

param(
    [string]$States = "NH,TX,FL,WI"
)

Set-Location C:\src\apportionment
$env:REDIST_LOCATION_POLICY = "C:\src\apportionment\redist\data\location_policy.json"

$stateConfig = @{
    "NH" = @{ name="new_hampshire"; nD=2;  from=1;    to=200  }
    "TX" = @{ name="texas";         nD=38; from=201;  to=500  }
    "FL" = @{ name="florida";       nD=28; from=201;  to=500  }
    "WI" = @{ name="wisconsin";     nD=8;  from=1201; to=1400 }
}

$req = $States -split "," | ForEach-Object { $_.Trim().ToUpper() }

foreach ($code in $req) {
    if (-not $stateConfig.ContainsKey($code)) {
        Write-Output "Unknown state: $code — skip"
        continue
    }
    $cfg    = $stateConfig[$code]
    $stName = $cfg.name
    $from   = $cfg.from
    $to     = $cfg.to
    $nD     = $cfg.nD

    Write-Output "=== $code ($nD districts) seeds $from → $to ==="

    for ($seed = $from; $seed -le $to; $seed++) {
        $ver   = "b7_${code}_s${seed}"
        $aPath = "outputs\$ver\2020\states\$stName\data\final_assignments.json"
        $mPath = "outputs\$ver\2020\states\$stName\manifest.json"
        $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"

        if (-not (Test-Path $aPath)) {
            $null = .\redist\target\release\redist.exe state --state $code --year 2020 `
                --version $ver --seed $seed --manifest 2>&1
            if ($LASTEXITCODE -ne 0) { Write-Output "  seed $seed FAILED"; continue }
        }

        if ((Test-Path $mPath) -and (-not (Test-Path $pPath))) {
            $null = .\redist\target\release\redist.exe analyze --state $code --year 2020 `
                --version $ver --types proportionality 2>&1
        }

        if ($seed % 50 -eq 0) {
            $m = if (Test-Path $mPath) { (Get-Content $mPath | ConvertFrom-Json).edge_cut / 1000 } else { "?" }
            Write-Output ("  seed {0} done — latest EC: {1}km" -f $seed, [math]::Round($m, 0))
        }
    }

    Write-Output "  $code done."
    Write-Output ""
}

Write-Output "All deepening complete. Run scripts\b7_rebuild_csv.ps1 to refresh convergence.csv"
