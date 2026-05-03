# B.11 PFR multi-seed sensitivity sweep
# Runs PFR for N seeds per state, collects D/R outcome distribution.
# Usage: pwsh -File scripts\b11_pfr_multiseed.ps1 [-N 20] [-States "NC,GA,..."]

param(
    [int]$N = 20,
    [string]$States = ""
)

Set-Location C:\src\apportionment
$env:REDIST_LOCATION_POLICY = "C:\src\apportionment\redist\data\location_policy.json"

$outDir = "outputs\b11_multiseed"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$csvPath = "$outDir\seed_sensitivity.csv"

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

$statesToRun = if ($States) {
    $req = $States -split "," | ForEach-Object { $_.Trim().ToUpper() }
    $allStates.GetEnumerator() | Where-Object { $req -contains $_.Key }
} else {
    $allStates.GetEnumerator()
}

# CSV header
if (-not (Test-Path $csvPath)) {
    "state,n_districts,seed,dem_seats,rep_seats,proportionality_gap_pp,ec_km,available" | Set-Content $csvPath -Encoding utf8
}

# Already-run keys
$done = @()
if (Test-Path $csvPath) {
    $done = Import-Csv $csvPath | ForEach-Object { "$($_.state)_$($_.seed)" }
}

Write-Output "B.11 PFR seed sensitivity: $N seeds x $($statesToRun.Count) states"
Write-Output "Output: $csvPath"
Write-Output ""

foreach ($entry in $statesToRun) {
    $code   = $entry.Key
    $nD     = $entry.Value[0]
    $stName = $entry.Value[1]

    $seeds_success = 0; $seeds_fail = 0

    for ($seed = 1; $seed -le $N; $seed++) {
        $key = "${code}_${seed}"
        if ($done -contains $key) { $seeds_success++; continue }

        $ver   = "b11_ms_${code}_s${seed}"
        $mPath = "outputs\$ver\2020\states\$stName\manifest.json"
        $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"

        if (-not (Test-Path $mPath)) {
            $null = .\redist\target\release\redist.exe state --state $code --year 2020 `
                --version $ver --partition-mode prime-factor --seed $seed --manifest 2>&1
            if ($LASTEXITCODE -ne 0) { $seeds_fail++; continue }
        }

        if ((Test-Path $mPath) -and (-not (Test-Path $pPath))) {
            $null = .\redist\target\release\redist.exe analyze --state $code --year 2020 `
                --version $ver --types proportionality 2>&1
        }

        if (-not (Test-Path $mPath)) { $seeds_fail++; continue }

        $m = Get-Content $mPath | ConvertFrom-Json
        $ecKm = [math]::Round([double]$m.edge_cut / 1000, 0)

        $demSeats = 0; $repSeats = 0; $gap = 0; $available = $false
        if (Test-Path $pPath) {
            $p = Get-Content $pPath | ConvertFrom-Json
            $available = $p.available -ne $false
            if ($available) {
                $demSeats = [int]$p.dem_seats
                $repSeats = $nD - $demSeats
                $gap  = [math]::Round([double]$p.proportionality_gap_pp, 2)
            }
        }

        "$code,$nD,$seed,$demSeats,$repSeats,$gap,$ecKm,$available" | Add-Content $csvPath -Encoding utf8
        $seeds_success++
    }

    # Summary for this state
    $rows = Import-Csv $csvPath | Where-Object { $_.state -eq $code -and $_.available -eq "True" }
    if ($rows.Count -gt 0) {
        $outcomes = $rows | Group-Object dem_seats | Sort-Object {[int]$_.Name}
        $outcomeStr = ($outcomes | ForEach-Object { "$($_.Name)D:$($_.Count)" }) -join " "
        Write-Output ("  $code ($nD D): {0}/{1} seeds succeeded | outcomes: {2}" -f $seeds_success, $N, $outcomeStr)
    } else {
        Write-Output "  ${code}: $seeds_success succeeded, $seeds_fail failed (no election data)"
    }
}

Write-Output ""
Write-Output "Done. Results: $csvPath"
