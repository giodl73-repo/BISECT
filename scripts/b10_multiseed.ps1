# B.10 Multi-seed sweep for focal states
# Runs focal states with 25 seeds each at alpha=0 and alpha=5
# Also runs alpha sweep {0,1,2,5,10,20} with 10 seeds for Pareto frontier

param(
    [int]$Seeds   = 25,
    [string]$States = "GA,NC,PA,TX,CA,VA,MI"
)

Set-Location C:\src\apportionment
$redist = ".\redist\target\release\redist.exe"
$outDir = "outputs\b10_multiseed"
$csv    = "$outDir\multiseed_results.csv"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

if (-not (Test-Path $csv)) {
    "state,alpha,seed,ec_km,d_seats,gap_pp,county_splits" | Out-File $csv -Encoding utf8
}

$stateMap = @{
    "GA"=@(14,"georgia"); "NC"=@(14,"north_carolina"); "PA"=@(17,"pennsylvania")
    "TX"=@(38,"texas");   "CA"=@(52,"california");     "VA"=@(11,"virginia")
    "MI"=@(13,"michigan")
}

$done = @{}
if (Test-Path $csv) {
    Import-Csv $csv | ForEach-Object { $done["$($_.state)_$($_.alpha)_$($_.seed)"] = 1 }
}

$req = $States -split "," | ForEach-Object { $_.Trim().ToUpper() }

# Part 1: 25 seeds × {alpha=0, alpha=5}
foreach ($code in $req) {
    if (-not $stateMap.ContainsKey($code)) { continue }
    $nD = $stateMap[$code][0]; $stName = $stateMap[$code][1]

    foreach ($alpha in @(0, 5)) {
        for ($s = 1; $s -le $Seeds; $s++) {
            $key = "${code}_${alpha}_${s}"
            if ($done.ContainsKey($key)) { continue }

            $ver = "b10ms_${code}_a${alpha}_s${s}"
            $alphaArg = if ($alpha -gt 0) { "--alpha-county $alpha" } else { "" }
            $mPath = "outputs\$ver\2020\states\$stName\manifest.json"
            $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"
            $aPath = "outputs\$ver\2020\states\$stName\data\final_assignments.json"
            $gPath = "outputs\V3\data\2020\adjacency\$($code.ToLower())_adjacency_2020_geoids.json"

            if (-not (Test-Path $mPath)) {
                $cmd = "$redist state --state $code --year 2020 --version $ver --seed $s --manifest --force $alphaArg"
                $null = Invoke-Expression $cmd 2>&1
                if ($LASTEXITCODE -ne 0) { Write-Output "  FAIL $code a=$alpha s=$s"; continue }
            }
            if (-not (Test-Path $pPath)) {
                $null = & $redist analyze --state $code --year 2020 --version $ver --types proportionality 2>&1
            }

            $ec=0; $d=0; $gap=0; $splits=0
            if (Test-Path $mPath) {
                $m = Get-Content $mPath | ConvertFrom-Json
                $ec = [math]::Round($m.edge_cut/1000, 0)
            }
            if (Test-Path $pPath) {
                $p = Get-Content $pPath | ConvertFrom-Json
                if ($p.available -ne $false) { $d=$p.dem_seats; $gap=[math]::Round($p.proportionality_gap_pp,2) }
            }
            if ((Test-Path $aPath) -and (Test-Path $gPath)) {
                $asgn = Get-Content $aPath | ConvertFrom-Json
                $geoids = Get-Content $gPath | ConvertFrom-Json
                $cd = @{}
                $asgn.PSObject.Properties | ForEach-Object {
                    $g = $geoids.PSObject.Properties[$_.Name]?.Value
                    if ($g -and $g.Length -ge 5) {
                        $c = $g.Substring(0,5)
                        if (-not $cd[$c]) { $cd[$c]=@{} }
                        $cd[$c][$_.Value]=1
                    }
                }
                $splits = ($cd.Values | Where-Object { $_.Count -gt 1 }).Count
            }

            "$code,$alpha,$s,$ec,$d,$gap,$splits" | Out-File $csv -Append -Encoding utf8
        }
        $rows = Import-Csv $csv | Where-Object { $_.state -eq $code -and $_.alpha -eq $alpha }
        if ($rows.Count -gt 0) {
            $avgS = ($rows | Measure-Object county_splits -Average).Average
            $avgD = ($rows | Measure-Object d_seats -Average).Average
            Write-Output ("  $code a=$alpha ({0} seeds): avg_splits={1:F1} avg_D={2:F1}" -f $rows.Count, $avgS, $avgD)
        }
    }
}

# Part 2: alpha sweep {0,0.5,1,2,5,10,20} with 10 seeds — Pareto frontier
Write-Output "`nAlpha sweep for Pareto frontier..."
foreach ($code in @("GA","NC","PA","TX","CA")) {
    if (-not $stateMap.ContainsKey($code)) { continue }
    $nD = $stateMap[$code][0]; $stName = $stateMap[$code][1]

    foreach ($alpha in @(0.5, 1.0, 2.0, 10.0, 20.0)) {  # 0 and 5 already done above
        for ($s = 1; $s -le 10; $s++) {
            $key = "${code}_${alpha}_${s}"
            if ($done.ContainsKey($key)) { continue }

            $ver = "b10ms_${code}_a$($alpha -replace '\.','p')_s${s}"
            $mPath = "outputs\$ver\2020\states\$stName\manifest.json"
            $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"
            $aPath = "outputs\$ver\2020\states\$stName\data\final_assignments.json"
            $gPath = "outputs\V3\data\2020\adjacency\$($code.ToLower())_adjacency_2020_geoids.json"

            if (-not (Test-Path $mPath)) {
                $cmd = "$redist state --state $code --year 2020 --version $ver --seed $s --manifest --force --alpha-county $alpha"
                $null = Invoke-Expression $cmd 2>&1
                if ($LASTEXITCODE -ne 0) { continue }
            }
            if (-not (Test-Path $pPath)) {
                $null = & $redist analyze --state $code --year 2020 --version $ver --types proportionality 2>&1
            }

            $ec=0; $d=0; $gap=0; $splits=0
            if (Test-Path $mPath) { $m=Get-Content $mPath|ConvertFrom-Json; $ec=[math]::Round($m.edge_cut/1000,0) }
            if (Test-Path $pPath) { $p=Get-Content $pPath|ConvertFrom-Json; if($p.available -ne $false){$d=$p.dem_seats;$gap=[math]::Round($p.proportionality_gap_pp,2)} }
            if ((Test-Path $aPath) -and (Test-Path $gPath)) {
                $asgn=Get-Content $aPath|ConvertFrom-Json; $geoids=Get-Content $gPath|ConvertFrom-Json
                $cd=@{}
                $asgn.PSObject.Properties|ForEach-Object{$g=$geoids.PSObject.Properties[$_.Name]?.Value;if($g -and $g.Length-ge 5){$c=$g.Substring(0,5);if(-not $cd[$c]){$cd[$c]=@{}};$cd[$c][$_.Value]=1}}
                $splits=($cd.Values|Where-Object{$_.Count-gt 1}).Count
            }
            "$code,$alpha,$s,$ec,$d,$gap,$splits" | Out-File $csv -Append -Encoding utf8
        }
    }
}

Write-Output "`nDone. Results: $csv"
