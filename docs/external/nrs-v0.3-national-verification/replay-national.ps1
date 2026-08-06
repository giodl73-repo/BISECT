param(
    [Parameter(Mandatory = $true)]
    [string]$DataRoot,
    [string]$OutputRoot = "runs/nrs-v0.3/external-replay",
    [string]$PythonCommand = "python",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "../../..")).Path
$dataRootResolved = (Resolve-Path -LiteralPath $DataRoot).Path
$outputRootResolved = Join-Path $repoRoot $OutputRoot
if (Test-Path -LiteralPath $outputRootResolved) {
    throw "Replay output already exists: $outputRootResolved"
}

Push-Location $repoRoot
try {
    $dirty = (& git status --porcelain) -join "`n"
    if ($dirty) { throw "Full replay requires a clean working tree: $dirty" }
    New-Item -ItemType Directory -Path $outputRootResolved | Out-Null
    Start-Transcript -LiteralPath (Join-Path $outputRootResolved "transcript.txt") | Out-Null
    try {
        if (-not $SkipBuild) {
            & cargo build --release --locked -p bisect-cli --bin bisect -p bisect-ops
            if ($LASTEXITCODE -ne 0) { throw "Locked release build failed" }
        }
        $suffix = if ($IsWindows) { ".exe" } else { "" }
        $bisect = Join-Path $repoRoot "target/release/bisect$suffix"
        $ops = Join-Path $repoRoot "target/release/bisect-ops$suffix"
        foreach ($path in $bisect, $ops) {
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                throw "Missing release executable: $path"
            }
        }

        $cycles = @(
            [ordered]@{ year = 2000; blocks = 8199908; population = 280849847; inventory = "docs/experiments/nationwide-2000/inventory.json"; standard = "configs/nrs_v0_3/standard_profile_2000.json"; legal = "configs/nrs_v0_1/legal_profile_2000.json" },
            [ordered]@{ year = 2010; blocks = 11071790; population = 308143815; inventory = "docs/experiments/nationwide-2010/inventory.json"; standard = "configs/nrs_v0_3/standard_profile_2010.json"; legal = "configs/nrs_v0_1/legal_profile_2010.json" },
            [ordered]@{ year = 2020; blocks = 8126956; population = 330759736; inventory = "docs/experiments/nationwide-2020/inventory.json"; standard = "configs/nrs_v0_3/standard_profile_2020.json"; legal = "configs/nrs_v0_1/legal_profile.json" }
        )
        $summaries = @()
        $snapshots = @()
        foreach ($cycle in $cycles) {
            $year = [int]$cycle.year
            $contextRoot = Join-Path $dataRootResolved "$year/certified"
            $contexts = @(Get-ChildItem -LiteralPath $contextRoot -Filter *.rctx -File)
            if ($contexts.Count -ne 50) {
                throw "Expected 50 certified $year RCTX files; found $($contexts.Count)"
            }
            & $ops verify-national-rctx --year $year `
                --out-dir "docs/experiments/nationwide-$year" `
                --context-root $contextRoot --require-complete
            if ($LASTEXITCODE -ne 0) { throw "$year RCTX verification failed" }

            $runDir = Join-Path $outputRootResolved "national-$year"
            & $ops nrs-batch --year $year --bisect $bisect `
                --inventory $cycle.inventory --standard-profile $cycle.standard `
                --legal-profile $cycle.legal --out-dir $runDir `
                --generated-at "2026-08-06T00:00:00Z"
            if ($LASTEXITCODE -ne 0) { throw "$year national batch failed" }
            & $ops verify-nrs-batch --year $year --inventory $cycle.inventory `
                --standard-profile $cycle.standard --legal-profile $cycle.legal `
                --out-dir $runDir --require-complete
            if ($LASTEXITCODE -ne 0) { throw "$year complete verification failed" }

            $publication = Join-Path $outputRootResolved "publication-$year"
            & $ops summarize-nrs-batch --year $year --inventory $cycle.inventory `
                --standard-profile $cycle.standard --legal-profile $cycle.legal `
                --out-dir $runDir --report-dir $publication
            if ($LASTEXITCODE -ne 0) { throw "$year summary failed" }
            $snapshot = Join-Path $outputRootResolved "node-snapshot-$year.json"
            & $ops snapshot-nrs-batch --year $year --inventory $cycle.inventory `
                --standard-profile $cycle.standard --legal-profile $cycle.legal `
                --out-dir $runDir --snapshot $snapshot
            if ($LASTEXITCODE -ne 0) { throw "$year snapshot failed" }
            $summary = Get-Content -LiteralPath (Join-Path $publication "national-summary.json") -Raw | ConvertFrom-Json
            $blockCount = ($summary.states | Measure-Object -Property unit_count -Sum).Sum
            if ($summary.state_count -ne 50 -or $summary.district_count -ne 435 -or
                $summary.recursive_node_count -ne 385 -or
                $blockCount -ne $cycle.blocks -or
                $summary.population_total -ne $cycle.population -or
                $summary.population_tolerance_failures -ne 0 -or
                $summary.disconnected_districts -ne 0 -or
                $summary.duplicate_units -ne 0 -or
                $summary.omitted_units -ne 0) {
                throw "$year national summary count drift"
            }
            $summary | Add-Member -NotePropertyName verified_block_count -NotePropertyValue $blockCount
            $summaries += $summary
            $snapshots += $snapshot
        }

        $comparison = Join-Path $outputRootResolved "comparison"
        & $ops compare-nrs-snapshots --snapshots $snapshots --out-dir $comparison
        if ($LASTEXITCODE -ne 0) { throw "Three-cycle comparison failed" }
        & $PythonCommand scripts/research/verify_nrs_cross_census.py $comparison
        if ($LASTEXITCODE -ne 0) { throw "Independent comparison verification failed" }

        $matrix = Get-Content -LiteralPath (Join-Path $comparison "stability-matrix.json") -Raw | ConvertFrom-Json
        $record = [ordered]@{
            schema_version = "nrs-v0.3-external-full-replay-v1"
            status = "pass"
            verified_at_utc = [DateTime]::UtcNow.ToString("o")
            git_commit = (& git rev-parse HEAD).Trim()
            machine_name = [Environment]::MachineName
            operating_system = [System.Runtime.InteropServices.RuntimeInformation]::OSDescription
            process_architecture = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString()
            powershell_version = $PSVersionTable.PSVersion.ToString()
            rustc_version = ((& rustc --version) -join " ")
            cargo_version = ((& cargo --version) -join " ")
            python_command = $PythonCommand
            python_version = ((& $PythonCommand --version 2>&1) -join " ")
            bisect_sha256 = (Get-FileHash -LiteralPath $bisect -Algorithm SHA256).Hash.ToLowerInvariant()
            bisect_ops_sha256 = (Get-FileHash -LiteralPath $ops -Algorithm SHA256).Hash.ToLowerInvariant()
            data_root = $dataRootResolved
            output_root = $outputRootResolved
            cycles = @($summaries | ForEach-Object {
                [ordered]@{
                    census_year = $_.census_year
                    states = $_.state_count
                    districts = $_.district_count
                    nodes = $_.recursive_node_count
                    blocks = $_.verified_block_count
                    population = $_.population_total
                    ledger_sha256 = $_.ledger_sha256
                }
            })
            all_cycle_common_node_signatures = $matrix.all_cycle_common_node_signatures
            all_cycle_exact_topology_states = $matrix.all_cycle_exact_topology_states
            matrix_sha256 = (Get-FileHash -LiteralPath (Join-Path $comparison "stability-matrix.json") -Algorithm SHA256).Hash.ToLowerInvariant()
            claim_boundary = "Operational replay; no legal, fairness, VRA, adoption, or exact boundary/canonical claim."
        }
        $record | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath `
            (Join-Path $outputRootResolved "replication-record.json") -Encoding utf8
        Write-Host "NRS v0.3 full national replay: PASS"
    }
    finally {
        Stop-Transcript | Out-Null
    }
}
finally {
    Pop-Location
}
