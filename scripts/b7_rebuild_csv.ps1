# B.7 MEC Convergence CSV Rebuilder
# Scans ALL existing b7_{STATE}_s{N} runs (no seed cap) + runs NH from scratch.
# Filters out stale runs where edge_cut == 0 or missing (pre-feature manifests).
# Outputs: outputs/b7_sweep/convergence.csv (full rebuild)

param(
    [switch]$DryRun,
    [string]$Only = ""   # comma-separated state codes to limit rebuild
)

Set-Location C:\src\apportionment
$env:REDIST_LOCATION_POLICY = "C:\src\apportionment\redist\data\location_policy.json"

$outDir = "outputs\b7_sweep"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$csvPath = "$outDir\convergence.csv"

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

$statesToProcess = if ($Only) {
    $req = $Only -split "," | ForEach-Object { $_.Trim().ToUpper() }
    $allStates.GetEnumerator() | Where-Object { $req -contains $_.Key }
} else {
    $allStates.GetEnumerator()
}

Write-Output "B.7 CSV rebuild — processing $($statesToProcess.Count) states"
Write-Output "Output: $csvPath"
if ($DryRun) { Write-Output "(DRY RUN — no redist calls)" }
Write-Output ""

$rows = [System.Collections.Generic.List[string]]::new()
$rows.Add("state,n_districts,dem_vote_pct,seeds_run,mec_km,n_improvements,last_improvement_seed,tail_seeds,mec_d_seats,mec_gap_pp,modal_d_seats,modal_count,min_ec_by_outcome")

foreach ($entry in $statesToProcess) {
    $code   = $entry.Key
    $nD     = $entry.Value[0]
    $stName = $entry.Value[1]

    # Find all existing runs for this state
    $existingRuns = Get-ChildItem "outputs\" -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match "^b7_${code}_s(\d+)$" } |
        Sort-Object { [int]($_.Name -replace "^b7_${code}_s", "") }

    $maxExisting = if ($existingRuns.Count -gt 0) {
        ($existingRuns | ForEach-Object { [int]($_.Name -replace "^b7_${code}_s", "") } | Measure-Object -Maximum).Maximum
    } else { 0 }

    # For NH (0 existing runs), run 200 seeds
    $seedsToEnsure = if ($maxExisting -eq 0) { 200 } else { $maxExisting }

    Write-Output "=== $code ($nD districts) — existing runs: $($existingRuns.Count) (max seed: $maxExisting) ==="

    # Run missing seeds for NH (or any state with 0 runs)
    if ($maxExisting -eq 0 -and -not $DryRun) {
        Write-Output "  No runs found — running $seedsToEnsure seeds..."
        for ($seed = 1; $seed -le $seedsToEnsure; $seed++) {
            $ver = "b7_${code}_s${seed}"
            $aPath = "outputs\$ver\2020\states\$stName\data\final_assignments.json"
            if (-not (Test-Path $aPath)) {
                $null = .\redist\target\release\redist.exe state --state $code --year 2020 --version $ver --seed $seed --manifest 2>&1
            }
            $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"
            if (-not (Test-Path "outputs\$ver\2020\states\$stName\manifest.json")) {
                # run failed
                continue
            }
            if (-not (Test-Path $pPath)) {
                $null = .\redist\target\release\redist.exe analyze --state $code --year 2020 --version $ver --types proportionality 2>&1
            }
            if ($seed % 50 -eq 0) { Write-Output "  ... seed $seed done" }
        }
        # Refresh existing runs list
        $existingRuns = Get-ChildItem "outputs\" -Directory -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -match "^b7_${code}_s(\d+)$" } |
            Sort-Object { [int]($_.Name -replace "^b7_${code}_s", "") }
    }

    # Collect results from all runs
    $results = [System.Collections.Generic.List[PSCustomObject]]::new()
    $skippedStale = 0

    foreach ($runDir in $existingRuns) {
        $seed = [int]($runDir.Name -replace "^b7_${code}_s", "")
        $mPath = "outputs\$($runDir.Name)\2020\states\$stName\manifest.json"
        $pPath = "outputs\$($runDir.Name)\2020\states\$stName\analysis\proportionality.json"

        # Run proportionality analysis if missing
        if ((Test-Path $mPath) -and (-not (Test-Path $pPath)) -and -not $DryRun) {
            $null = .\redist\target\release\redist.exe analyze --state $code --year 2020 `
                --version $runDir.Name --types proportionality 2>&1
        }

        if (-not (Test-Path $mPath)) { continue }

        $m = Get-Content $mPath | ConvertFrom-Json
        $ecVal = [double]$m.edge_cut

        # Skip stale runs with missing/zero edge_cut.
        # Exception: k=1 states legitimately have edge_cut=0 (no bisection needed).
        if ($ecVal -le 0 -and $nD -gt 1) {
            $skippedStale++
            continue
        }

        if (Test-Path $pPath) {
            $p = Get-Content $pPath | ConvertFrom-Json
            if ($p.available -eq $false) {
                $results.Add([PSCustomObject]@{seed=$seed; ec=999999; d=0; gap=0; vote=0})
            } else {
                $results.Add([PSCustomObject]@{
                    seed = $seed
                    ec   = [math]::Round($ecVal / 1000, 0)
                    d    = [int]$p.dem_seats
                    gap  = [math]::Round([double]$p.proportionality_gap_pp, 2)
                    vote = [math]::Round([double]$p.dem_vote_share_statewide * 100, 1)
                })
            }
        }
    }

    if ($results.Count -eq 0) {
        Write-Output "  SKIP: no valid plans (stale skipped: $skippedStale)"
        continue
    }

    if ($skippedStale -gt 0) {
        Write-Output "  Note: $skippedStale stale runs filtered (edge_cut=0)"
    }

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

    # Min EC by outcome (exclude sentinel 999999)
    $ec_by_outcome = $results | Where-Object {$_.ec -lt 999999} | Group-Object d |
        Sort-Object {[int]$_.Name} |
        ForEach-Object { "$($_.Name)D=$([math]::Round(($_.Group|Measure-Object ec -Minimum).Minimum,0))" }

    $converged = if ($tail -ge 200) { "CONVERGED" } elseif ($tail -ge 100) { "likely" } else { "CHECK" }

    Write-Output ("  MEC: {0}km | last impr: seed {1} | tail: {2} seeds [{3}] | {4}D gap={5}pp" -f `
        [math]::Round($run_min,0), $last_impr, $tail, $converged, $mec_d, $mec_gap)
    Write-Output ("  Dem vote: {0}% | valid runs: {1} | modal: {2}D ({3} seeds)" -f `
        $dem_vote, $results.Count, $modal.Name, $modal.Count)

    $row = "{0},{1},{2},{3},{4},{5},{6},{7},{8},{9},{10},{11},{12}" -f `
        $code, $nD, $dem_vote, $results.Count,
        [math]::Round($run_min,0), $n_impr, $last_impr, $tail,
        $mec_d, $mec_gap, $modal.Name, $modal.Count,
        ($ec_by_outcome -join ";")
    $rows.Add($row)
    Write-Output ""
}

if (-not $DryRun) {
    $rows | Set-Content $csvPath -Encoding utf8
    Write-Output "Done. $($rows.Count - 1) states written to: $csvPath"
}
