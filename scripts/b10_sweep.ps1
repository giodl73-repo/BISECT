# B.10 Governmental Edge Sweep — county stickiness
# Runs each state twice: alpha=0 (baseline) and alpha=5 (county sticky)
# Records county_splits, ec_km, d_seats, gap_pp for each.
# Usage: pwsh -File scripts\b10_sweep.ps1 [-States "NC,GA,..."]

param(
    [string]$States = "",
    [float]$Alpha   = 5.0,
    [int]$Seed      = 1
)

Set-Location C:\src\apportionment
$redist = ".\redist\target\release\redist.exe"
$outDir = "outputs\b10_sweep"
$csv    = "$outDir\county_stickiness.csv"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

if (-not (Test-Path $csv)) {
    "state,n_districts,alpha,ec_km,d_seats,gap_pp,dem_vote_pct,county_splits" |
        Out-File $csv -Encoding utf8
}

$allStates = [ordered]@{
    "VT"=@(1,"vermont");      "AK"=@(1,"alaska");       "WY"=@(1,"wyoming")
    "DE"=@(1,"delaware");     "ND"=@(1,"north_dakota");  "SD"=@(1,"south_dakota")
    "MT"=@(2,"montana");      "RI"=@(2,"rhode_island");  "NH"=@(2,"new_hampshire")
    "ME"=@(2,"maine");        "HI"=@(2,"hawaii");        "ID"=@(2,"idaho")
    "WV"=@(2,"west_virginia")
    "NE"=@(3,"nebraska");     "NM"=@(3,"new_mexico")
    "KS"=@(4,"kansas");       "UT"=@(4,"utah");          "NV"=@(4,"nevada")
    "AR"=@(4,"arkansas");     "IA"=@(4,"iowa");          "MS"=@(4,"mississippi")
    "CT"=@(5,"connecticut");  "OK"=@(5,"oklahoma")
    "OR"=@(6,"oregon");       "KY"=@(6,"kentucky");      "LA"=@(6,"louisiana")
    "SC"=@(7,"south_carolina"); "AL"=@(7,"alabama")
    "CO"=@(8,"colorado");     "WI"=@(8,"wisconsin");     "MO"=@(8,"missouri")
    "MD"=@(8,"maryland");     "MN"=@(8,"minnesota")
    "TN"=@(9,"tennessee");    "AZ"=@(9,"arizona");       "MA"=@(9,"massachusetts")
    "IN"=@(9,"indiana")
    "WA"=@(10,"washington");  "VA"=@(11,"virginia");     "NJ"=@(12,"new_jersey")
    "MI"=@(13,"michigan");    "GA"=@(14,"georgia");      "NC"=@(14,"north_carolina")
    "OH"=@(15,"ohio");        "PA"=@(17,"pennsylvania"); "IL"=@(17,"illinois")
    "NY"=@(26,"new_york");    "FL"=@(28,"florida");      "TX"=@(38,"texas")
    "CA"=@(52,"california")
}

$done = @()
if (Test-Path $csv) {
    $done = Import-Csv $csv | ForEach-Object { "$($_.state)_$($_.alpha)" }
}

$statesToRun = if ($States) {
    $req = $States -split "," | ForEach-Object { $_.Trim().ToUpper() }
    $allStates.GetEnumerator() | Where-Object { $req -contains $_.Key }
} else {
    $allStates.GetEnumerator()
}

Write-Output "B.10 county stickiness sweep: alpha=0 and alpha=$Alpha, seed=$Seed"

function Run-State($code, $nD, $stName, $alpha) {
    $ver = "b10_${code}_a${alpha}_s${Seed}"
    $alphaStr = if ($alpha -eq 0) { "" } else { "--alpha-county $alpha" }

    $mPath = "outputs\$ver\2020\states\$stName\manifest.json"
    $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"

    if (-not (Test-Path $mPath)) {
        $cmd = "$redist state --state $code --year 2020 --version $ver --seed $Seed --manifest --force $alphaStr"
        $null = Invoke-Expression $cmd 2>&1
        if ($LASTEXITCODE -ne 0) { return $null }
    }
    if (-not (Test-Path $pPath)) {
        $null = & $redist analyze --state $code --year 2020 --version $ver --types proportionality 2>&1
    }

    if (-not (Test-Path $mPath)) { return $null }
    $m = Get-Content $mPath | ConvertFrom-Json
    $ec = [math]::Round($m.edge_cut / 1000, 0)

    $dSeats = 0; $gapPp = 0; $demVote = 0
    if (Test-Path $pPath) {
        $p = Get-Content $pPath | ConvertFrom-Json
        if ($p.available -ne $false) {
            $dSeats  = $p.dem_seats
            $gapPp   = [math]::Round($p.proportionality_gap_pp, 2)
            $demVote = [math]::Round($p.dem_vote_share_statewide * 100, 1)
        }
    }

    # Count county splits using tract index→GEOID mapping
    $aPath = "outputs\$ver\2020\states\$stName\data\final_assignments.json"
    $gPath = "outputs\V3\data\2020\adjacency\$($code.ToLower())_adjacency_2020_geoids.json"
    $countySplits = 0
    if ((Test-Path $aPath) -and (Test-Path $gPath)) {
        $asgn   = Get-Content $aPath | ConvertFrom-Json   # idx -> district
        $geoids = Get-Content $gPath | ConvertFrom-Json   # idx -> geoid
        # Build county -> set of districts
        $countyDistricts = @{}
        $asgn.PSObject.Properties | ForEach-Object {
            $idx  = $_.Name; $dist = $_.Value
            $geoid = $geoids.PSObject.Properties[$idx]?.Value
            if ($geoid -and $geoid.Length -ge 5) {
                $county = $geoid.Substring(0, 5)
                if (-not $countyDistricts.ContainsKey($county)) { $countyDistricts[$county] = @{} }
                $countyDistricts[$county][$dist] = 1
            }
        }
        $countySplits = ($countyDistricts.Values | Where-Object { $_.Count -gt 1 }).Count
    }

    return [PSCustomObject]@{
        ec=$ec; d=$dSeats; gap=$gapPp; vote=$demVote; splits=$countySplits
    }
}

foreach ($entry in $statesToRun) {
    $code   = $entry.Key
    $nD     = $entry.Value[0]
    $stName = $entry.Value[1]

    foreach ($alpha in @(0, $Alpha)) {
        $key = "${code}_${alpha}"
        if ($done -contains $key) { continue }

        $r = Run-State $code $nD $stName $alpha
        if ($r -eq $null) {
            Write-Output "  $code alpha=$alpha FAILED"
            continue
        }

        "$code,$nD,$alpha,$($r.ec),$($r.d),$($r.gap),$($r.vote),$($r.splits)" |
            Out-File $csv -Append -Encoding utf8
        Write-Output ("  $code alpha=$alpha | EC:{0}km | {1}D | gap={2}pp | splits={3}" -f `
            $r.ec, $r.d, $r.gap, $r.splits)
    }
}

Write-Output "Done. Results: $csv"

# Summary: alpha=0 vs alpha=5 comparison
if (Test-Path $csv) {
    $data = Import-Csv $csv
    $a0 = $data | Where-Object { $_.alpha -eq "0" }
    $a5 = $data | Where-Object { $_.alpha -eq $Alpha.ToString() }
    $avgSplits0 = ($a0 | Measure-Object county_splits -Average).Average
    $avgSplits5 = ($a5 | Measure-Object county_splits -Average).Average
    Write-Output ""
    Write-Output "=== Summary: alpha=0 vs alpha=$Alpha ==="
    Write-Output ("Avg county splits: alpha=0={0:F1}, alpha={1}={2:F1}" -f $avgSplits0, $Alpha, $avgSplits5)
}
