param([string]$Output = 'dist/wasm-preview')
$ErrorActionPreference = 'Stop'
$wasmRepo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $wasmRepo
try {
    if ((Test-Path -LiteralPath $Output) -and (Get-ChildItem -LiteralPath $Output -Force | Select-Object -First 1)) { throw 'Choose a new or empty output directory.' }
    cargo build -p bisect-wasm --release --target wasm32-unknown-unknown --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'WASM build failed.' }
    node scripts/web/test_wasm.mjs target/wasm32-unknown-unknown/release/bisect_wasm.wasm
    if ($LASTEXITCODE -ne 0) { throw 'WASM verification failed.' }
    node scripts/web/test_project.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Project save/load verification failed.' }
    cargo run -p rcount-audit --example wasm_fixtures --locked --offline -- target/rcount-wasm-fixtures
    if ($LASTEXITCODE -ne 0) { throw 'Native election reference generation failed.' }
    node scripts/web/test_count_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Election package verification failed.' }
    node scripts/web/test_package_files.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Package file roundtrip failed.' }
    cargo build -p bisect-wasm --release --example execute_tool_request --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native election import reference build failed.' }
    node scripts/web/test_multiscale_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Portable multiscale native/WASM boundary verification failed.' }
    node scripts/web/test_vra_audit_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'VRA audit verification failed.' }
    node scripts/web/test_plan_export.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Engine plan export verification failed.' }
    node scripts/web/test_election_imports.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Election import parity and project/archive roundtrip failed.' }
    node scripts/web/test_ri_rla_import.mjs
    if ($LASTEXITCODE -ne 0) { throw 'RI RLA import parity and boundary verification failed.' }
    node scripts/web/test_district_aggregation.mjs
    if ($LASTEXITCODE -ne 0) { throw 'District aggregation parity and exact-count verification failed.' }
    cargo build -p rhist-cli --release --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native history verifier reference build failed.' }
    node scripts/web/test_history_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'History verification parity and project roundtrip failed.' }
    New-Item -ItemType Directory -Path "$Output/fixtures" -Force | Out-Null
    Copy-Item -LiteralPath 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm' -Destination "$Output/bisect_wasm.wasm"
    foreach ($file in @('count-results.js','result-preview.js','package-files.js','project.js','project-worker.js','json-worker.js','workbench.css','workbench.js','wasm-engine.js','partisan-input.js','partisan-tsv-worker.js','character-input.js','character-csv-worker.js','election-input.js','election-csv-worker.js','wasm-worker.js')) { Copy-Item -LiteralPath "web/lab/$file" -Destination "$Output/$file" }
    Copy-Item -LiteralPath 'web/lab/workbench.html' -Destination "$Output/index.html"
    Copy-Item -LiteralPath 'crates/rplan-audit/fixtures/grid3x3-valid.rplan' -Destination "$Output/fixtures/plan.rplan"
    Copy-Item -LiteralPath 'crates/rplan-audit/fixtures/grid3x3.rctx' -Destination "$Output/fixtures/context.rctx"
    Copy-Item -LiteralPath 'crates/rplan-audit/fixtures/grid3x3-valid-certificate.json' -Destination "$Output/fixtures/certificate.json"
    $exampleCertificate = Get-Content 'crates/rplan-audit/fixtures/grid3x3-valid-certificate.json' -Raw | ConvertFrom-Json
    $exampleProfile = $exampleCertificate.legal_profile
    $exampleProfile | Add-Member -NotePropertyName contiguity_required -NotePropertyValue $true
    $exampleProfile | Add-Member -NotePropertyName nesting_rule -NotePropertyValue @{type='not-evaluated'}
    $exampleProfile.PSObject.Properties.Remove('legal_disclaimer')
    [IO.File]::WriteAllText((Join-Path (Resolve-Path $Output).Path 'fixtures/profile.json'), ($exampleProfile | ConvertTo-Json -Depth 30))
    $countExampleRoot = (Resolve-Path 'target/rcount-wasm-fixtures/minerva').Path
    $countFiles = @{}
    foreach ($countFile in (Get-ChildItem -LiteralPath $countExampleRoot -File -Recurse)) {
        # Windows PowerShell 5.1 lacks Path.GetRelativePath. Enumerated files
        # must stay beneath this resolved fixture root before stripping it.
        $countPrefix = $countExampleRoot.TrimEnd('\','/') + [IO.Path]::DirectorySeparatorChar
        if (-not $countFile.FullName.StartsWith($countPrefix, [StringComparison]::OrdinalIgnoreCase)) { throw 'Election fixture escaped its package root.' }
        $countPath = $countFile.FullName.Substring($countPrefix.Length).Replace('\','/')
        $countFiles[$countPath] = [Convert]::ToBase64String([IO.File]::ReadAllBytes($countFile.FullName))
    }
    [IO.File]::WriteAllText((Join-Path (Resolve-Path $Output).Path 'fixtures/count-package.json'), ($countFiles | ConvertTo-Json -Depth 5))
    Write-Output "Practitioner WASM preview: $Output/index.html"
} finally { Pop-Location }
