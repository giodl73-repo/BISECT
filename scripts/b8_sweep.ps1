# B.8 GeoSection 50-State Sweep
# Runs GeoSection (ratio-optimal bisection) for all 50 states and records:
#   state, n_districts, natural ratio, EC, D/R seats, proportionality gap
# vs B.7 MEC baseline from outputs/b7_sweep/convergence.csv
#
# Usage: pwsh -File scripts/b8_sweep.ps1 [-Seeds 30] [-States "NC,GA,..."]
#
# Each state is ONE run (GeoSection internally tries all ratios × Seeds).
# Resumable: skips states already present in the CSV.

param(
    [int]$Seeds  = 30,
    [string]$States = ""
)

Set-Location C:\src\apportionment

$redist  = ".\redist\target\release\redist.exe"
$outDir  = "outputs\b8_sweep"
$csvPath = "$outDir\geosection.csv"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# CSV header
if (-not (Test-Path $csvPath)) {
    "state,n_districts,nat_ratio_l,nat_ratio_r,ec_km,d_seats,r_seats,gap_pp,dem_vote_pct,b7_d_seats,b7_gap_pp" |
        Out-File $csvPath -Encoding utf8
}

# Load B.7 MEC baseline for comparison
$b7 = @{}
if (Test-Path "outputs\b7_sweep\convergence.csv") {
    Import-Csv "outputs\b7_sweep\convergence.csv" | ForEach-Object {
        $b7[$_.state] = $_
    }
}

# All 50 states
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

# Filter to requested states, skip already done
$done = @()
if (Test-Path $csvPath) {
    $done = Import-Csv $csvPath | Select-Object -ExpandProperty state
}

$statesToRun = if ($States) {
    $req = $States -split "," | ForEach-Object { $_.Trim().ToUpper() }
    $allStates.GetEnumerator() | Where-Object { $req -contains $_.Key }
} else {
    $allStates.GetEnumerator() | Where-Object { $done -notcontains $_.Key }
}

Write-Output "B.8 GeoSection sweep: $Seeds seeds/ratio × $($statesToRun.Count) states"
Write-Output "Results: $csvPath"
Write-Output ""

foreach ($entry in $statesToRun) {
    $code   = $entry.Key
    $nD     = $entry.Value[0]
    $stName = $entry.Value[1]
    $ver    = "b8_${code}"

    Write-Output "=== $code ($nD districts) ==="
    $t0 = Get-Date

    # 1-district states: no bisection
    if ($nD -eq 1) {
        Write-Output "  1 district — no bisection, skipping GeoSection"
        $b7row = $b7[$code]
        $b7d = if ($b7row) { $b7row.mec_d_seats } else { "NA" }
        $b7g = if ($b7row) { $b7row.mec_gap_pp  } else { "NA" }
        "$code,$nD,1,0,0,1,0,0,0,$b7d,$b7g" | Out-File $csvPath -Append -Encoding utf8
        continue
    }

    $mPath = "outputs\$ver\2020\states\$stName\manifest.json"
    $pPath = "outputs\$ver\2020\states\$stName\analysis\proportionality.json"

    # Run GeoSection if not already done
    if (-not (Test-Path $mPath)) {
        $out = & $redist state --state $code --year 2020 --version $ver `
            --partition-mode geosection --geosection-seeds $Seeds --manifest --force 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Output "  FAILED (exit $LASTEXITCODE)"
            continue
        }
        # Parse natural ratio from output
        $ratioLine = $out | Where-Object { $_ -match "natural ratio (\d+):(\d+) at" } | Select-Object -Last 1
    } else {
        $ratioLine = ""
    }

    # Run proportionality if not done
    if (-not (Test-Path $pPath)) {
        $null = & $redist analyze --state $code --year 2020 --version $ver --types proportionality 2>&1
    }

    if (-not (Test-Path $mPath)) { Write-Output "  SKIP: no manifest"; continue }

    # Parse results
    $m = Get-Content $mPath | ConvertFrom-Json
    $ec = [math]::Round($m.edge_cut / 1000, 0)

    $natL = 0; $natR = 0
    if ($ratioLine -match "natural ratio (\d+):(\d+)") {
        $natL = [int]$Matches[1]; $natR = [int]$Matches[2]
    }

    $dSeats = 0; $gapPp = 0; $demVote = 0
    if (Test-Path $pPath) {
        $p = Get-Content $pPath | ConvertFrom-Json
        if ($p.available -ne $false) {
            $dSeats  = $p.dem_seats
            $gapPp   = [math]::Round($p.proportionality_gap_pp, 2)
            $demVote = [math]::Round($p.dem_vote_share_statewide * 100, 1)
        }
    }
    $rSeats = $nD - $dSeats

    $b7row = $b7[$code]
    $b7d = if ($b7row) { $b7row.mec_d_seats } else { "NA" }
    $b7g = if ($b7row) { $b7row.mec_gap_pp  } else { "NA" }

    $elapsed = [math]::Round(((Get-Date) - $t0).TotalSeconds, 0)
    Write-Output ("  natural ratio {0}:{1} | EC: {2}km | {3}D/{4}R | gap={5}pp | Dem={6}% | {7}s" -f `
        $natL, $natR, $ec, $dSeats, $rSeats, $gapPp, $demVote, $elapsed)
    Write-Output ("  B.7 MEC baseline: {0}D gap={1}pp" -f $b7d, $b7g)

    "$code,$nD,$natL,$natR,$ec,$dSeats,$rSeats,$gapPp,$demVote,$b7d,$b7g" |
        Out-File $csvPath -Append -Encoding utf8
    Write-Output ""
}

Write-Output "Done. Results: $csvPath"

# Summary table
if (Test-Path $csvPath) {
    Write-Output ""
    Write-Output "=== Summary ==="
    Import-Csv $csvPath | Sort-Object {[int]$_.n_districts} |
        Format-Table state, n_districts, nat_ratio_l, nat_ratio_r, ec_km,
                     d_seats, gap_pp, b7_d_seats, b7_gap_pp -AutoSize
}
