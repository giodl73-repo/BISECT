# B.7 MEC Convergence Sweep — all 50 states x N seeds
# Outputs: outputs/b7_sweep/convergence.csv (appended per state)
# Usage: pwsh -File scripts/b7_sweep.ps1 [-N 200] [-States "PA,NC,..."]

param(
    [int]$N = 200,
    [string]$States = ""
)

Set-Location C:\src\apportionment

$outDir = "outputs\b7_sweep"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$csvPath = "$outDir\convergence.csv"

# Write CSV header if new file
if (-not (Test-Path $csvPath)) {
    "state,n_districts,dem_vote_pct,seeds_run,mec_km,n_improvements,last_improvement_seed,tail_seeds,mec_d_seats,mec_gap_pp,modal_d_seats,modal_count,min_ec_by_outcome" | Out-File $csvPath -Encoding utf8
}

# All 50 states: code -> (districts, state_name)
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

# Filter to requested states
$statesToRun = if ($States) {
    $req = $States -split "," | ForEach-Object { $_.Trim().ToUpper() }
    $allStates.GetEnumerator() | Where-Object { $req -contains $_.Key }
} else {
    # Skip already-complete states (check CSV)
    $done = @()
    if (Test-Path $csvPath) {
        $done = Import-Csv $csvPath | Select-Object -ExpandProperty state
    }
    $allStates.GetEnumerator() | Where-Object { $done -notcontains $_.Key }
}

Write-Output "B.7 sweep: $N seeds x $($statesToRun.Count) states"
Write-Output "Results: $csvPath"
Write-Output ""

foreach ($entry in $statesToRun) {
    $code    = $entry.Key
    $nD      = $entry.Value[0]
    $stName  = $entry.Value[1]

    Write-Output "=== $code ($nD districts) ==="

    $results = @()
    $ok = 0; $fail = 0

    for ($seed = 1; $seed -le $N; $seed++) {
        $ver = "b7_${code}_s${seed}"

        # Check if already run
        $aPath = "outputs\$ver\2020\states\$stName\data\final_assignments.json"
        $mPath = "outputs\$ver\2020\states\$stName\manifest.json"

        if (-not (Test-Path $aPath)) {
            $null = .\redist\target\release\redist.exe state --state $code --year 2020 --version $ver --seed $seed --manifest 2>&1
            if ($LASTEXITCODE -ne 0) { $fail++; continue }
        }

        $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"
        if (-not (Test-Path $pPath)) {
            $null = .\redist\target\release\redist.exe analyze --state $code --year 2020 --version $ver --types proportionality 2>&1
        }

        if ((Test-Path $mPath) -and (Test-Path $pPath)) {
            $m = Get-Content $mPath | ConvertFrom-Json
            $p = Get-Content $pPath | ConvertFrom-Json
            if ($p.available -eq $false) { $ok++; $results += [PSCustomObject]@{seed=$seed;ec=999999;d=0;gap=0;vote=0}; continue }
            $results += [PSCustomObject]@{
                seed = $seed
                ec   = $m.edge_cut / 1000
                d    = $p.dem_seats
                gap  = [math]::Round($p.proportionality_gap_pp, 2)
                vote = [math]::Round($p.dem_vote_share_statewide * 100, 1)
            }
            $ok++
        }
    }

    if ($results.Count -eq 0) { Write-Output "  SKIP: no valid plans"; continue }

    # Convergence analysis
    $run_min = [double]::MaxValue; $last_impr = 0; $n_impr = 0
    $mec_d = 0; $mec_gap = 0; $dem_vote = 0
    foreach ($r in ($results | Sort-Object seed)) {
        if ($r.ec -lt $run_min) {
            $run_min = $r.ec; $n_impr++; $last_impr = $r.seed
            $mec_d = $r.d; $mec_gap = $r.gap; $dem_vote = $r.vote
        }
    }

    $tail = $results.Count - $last_impr

    # Modal outcome
    $modal = $results | Group-Object d | Sort-Object Count -Descending | Select-Object -First 1

    # Min EC by outcome
    $ec_by_outcome = $results | Where-Object {$_.ec -lt 999999} | Group-Object d | Sort-Object {[int]$_.Name} |
        ForEach-Object { "$($_.Name)D=$([math]::Round(($_.Group|Measure-Object ec -Minimum).Minimum,0))" }

    Write-Output ("  MEC: {0}km | last impr: seed {1} | tail: {2} seeds | {3}D/{4}R gap={5}pp" -f `
        [math]::Round($run_min,0), $last_impr, $tail, $mec_d, ($nD-$mec_d), $mec_gap)
    Write-Output ("  Dem vote: {0}% | modal: {1}D ({2} seeds) | failed: {3}" -f `
        $dem_vote, $modal.Name, $modal.Count, $fail)

    # Append to CSV
    $row = "{0},{1},{2},{3},{4},{5},{6},{7},{8},{9},{10},{11},{12}" -f `
        $code, $nD, $dem_vote, $results.Count,
        [math]::Round($run_min,0), $n_impr, $last_impr, $tail,
        $mec_d, $mec_gap, $modal.Name, $modal.Count,
        ($ec_by_outcome -join ";")
    $row | Out-File $csvPath -Append -Encoding utf8
    Write-Output ""
}

Write-Output "Done. Results in: $csvPath"
