# B.11 Prime-Factor Redistricting sweep — all 50 states, 2020 seat counts
# Runs redist state --partition-mode prime-factor for each state.
# Outputs: outputs/b11_sweep/pfr_results.csv
#
# Usage: pwsh -File scripts\b11_pfr_sweep.ps1 [-States "CA,TX,..."] [-Force]

param(
    [string]$States  = "",
    [switch]$Force                  # re-run even if output exists
)

Set-Location C:\src\apportionment
$env:REDIST_LOCATION_POLICY = "C:\src\apportionment\redist\data\location_policy.json"

$outDir = "outputs\b11_sweep"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$csvPath = "$outDir\pfr_results.csv"

$allStates = [ordered]@{
    "VT"=@(1,"vermont");"AK"=@(1,"alaska");"WY"=@(1,"wyoming");"DE"=@(1,"delaware")
    "ND"=@(1,"north_dakota");"SD"=@(1,"south_dakota");"MT"=@(2,"montana")
    "RI"=@(2,"rhode_island");"NH"=@(2,"new_hampshire");"ME"=@(2,"maine")
    "HI"=@(2,"hawaii");"ID"=@(2,"idaho");"WV"=@(2,"west_virginia")
    "NE"=@(3,"nebraska");"NM"=@(3,"new_mexico");"KS"=@(4,"kansas")
    "UT"=@(4,"utah");"NV"=@(4,"nevada");"AR"=@(4,"arkansas")
    "IA"=@(4,"iowa");"MS"=@(4,"mississippi");"CT"=@(5,"connecticut")
    "OK"=@(5,"oklahoma");"OR"=@(6,"oregon");"KY"=@(6,"kentucky")
    "LA"=@(6,"louisiana");"SC"=@(7,"south_carolina");"AL"=@(7,"alabama")
    "CO"=@(8,"colorado");"WI"=@(8,"wisconsin");"TN"=@(9,"tennessee")
    "AZ"=@(9,"arizona");"MA"=@(9,"massachusetts");"MO"=@(8,"missouri")
    "MD"=@(8,"maryland");"MN"=@(8,"minnesota");"IN"=@(9,"indiana")
    "WA"=@(10,"washington");"VA"=@(11,"virginia");"NJ"=@(12,"new_jersey")
    "MI"=@(13,"michigan");"GA"=@(14,"georgia");"NC"=@(14,"north_carolina")
    "OH"=@(15,"ohio");"PA"=@(17,"pennsylvania");"IL"=@(17,"illinois")
    "NY"=@(26,"new_york");"FL"=@(28,"florida");"TX"=@(38,"texas")
    "CA"=@(52,"california")
}

# Prime factorization for display (largest prime first)
function Get-PrimeFactor([int]$n) {
    if ($n -le 1) { return @() }
    $factors = [System.Collections.Generic.List[int]]::new()
    $d = 2
    $remaining = $n
    while ($d * $d -le $remaining) {
        while ($remaining % $d -eq 0) { $factors.Add($d); $remaining /= $d }
        $d++
    }
    if ($remaining -gt 1) { $factors.Add($remaining) }
    # Return largest-prime-first (reverse of smallest-first)
    $arr = $factors.ToArray()
    [Array]::Reverse($arr)
    return $arr
}

$statesToRun = if ($States) {
    $req = $States -split "," | ForEach-Object { $_.Trim().ToUpper() }
    $allStates.GetEnumerator() | Where-Object { $req -contains $_.Key }
} else {
    $done = @()
    if ((Test-Path $csvPath) -and -not $Force) {
        $done = Import-Csv $csvPath | Select-Object -ExpandProperty state
    }
    $allStates.GetEnumerator() | Where-Object { $done -notcontains $_.Key }
}

# Write CSV header if new file
if (-not (Test-Path $csvPath) -or $Force) {
    "state,n_districts,factor_sequence,split_tree,dem_vote_pct,dem_seats,rep_seats,proportionality_gap_pp,ec_km,available" | Set-Content $csvPath -Encoding utf8
}

Write-Output "B.11 PFR sweep: $($statesToRun.Count) states"
Write-Output "Output: $csvPath"
Write-Output ""

foreach ($entry in $statesToRun) {
    $code   = $entry.Key
    $nD     = $entry.Value[0]
    $stName = $entry.Value[1]
    $ver    = "b11_pfr_${code}"

    $factors = Get-PrimeFactor $nD
    $factorStr = $factors -join "x"
    Write-Output "=== $code ($nD districts = $factorStr) ==="

    $mPath = "outputs\$ver\2020\states\$stName\manifest.json"
    $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"

    # Run if not already done
    if (-not (Test-Path $mPath) -or $Force) {
        $out = .\redist\target\release\redist.exe state --state $code --year 2020 `
            --version $ver --partition-mode prime-factor --manifest 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Output "  FAILED: $($out | Select-Object -Last 3 | Out-String)"
            continue
        }
    }

    if ((Test-Path $mPath) -and (-not (Test-Path $pPath))) {
        $null = .\redist\target\release\redist.exe analyze --state $code --year 2020 `
            --version $ver --types proportionality 2>&1
    }

    if (-not (Test-Path $mPath)) { Write-Output "  SKIP: no manifest"; continue }

    $m = Get-Content $mPath | ConvertFrom-Json
    $ecKm = [math]::Round([double]$m.edge_cut / 1000, 0)

    $demSeats = 0; $repSeats = 0; $gap = 0; $vote = 0; $available = $false
    if (Test-Path $pPath) {
        $p = Get-Content $pPath | ConvertFrom-Json
        $available = $p.available -ne $false
        if ($available) {
            $demSeats = [int]$p.dem_seats
            $repSeats = $nD - $demSeats
            $gap  = [math]::Round([double]$p.proportionality_gap_pp, 2)
            $vote = [math]::Round([double]$p.dem_vote_share_statewide * 100, 1)
        }
    }

    # Build split tree description: e.g. "17→(8,9)→..." for PA
    # Just show the top-level split for now
    $splitTree = if ($factors.Count -gt 0) { $factorStr } else { "1" }

    Write-Output ("  EC={0}km | {1}D/{2}R | gap={3}pp | vote={4}%" -f $ecKm, $demSeats, $repSeats, $gap, $vote)

    $row = "{0},{1},{2},{3},{4},{5},{6},{7},{8},{9}" -f `
        $code, $nD, $factorStr, $splitTree,
        $vote, $demSeats, $repSeats, $gap, $ecKm, $available
    $row | Add-Content $csvPath -Encoding utf8
    Write-Output ""
}

Write-Output "Done. Results: $csvPath"

# Summary
if (Test-Path $csvPath) {
    $csv = Import-Csv $csvPath
    $eligible = $csv | Where-Object { $_.available -eq "True" }
    $totalD = ($eligible | Measure-Object dem_seats -Sum).Sum
    $totalR = ($eligible | Measure-Object rep_seats -Sum).Sum
    Write-Output ""
    Write-Output ("National PFR totals ({0} states with election data): {1}D/{2}R" -f $eligible.Count, $totalD, $totalR)
}
