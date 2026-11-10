param([string]$Base='dist/wasm-cffi-gated',[string]$Output='dist/wasm-workspace-preview',[switch]$BrowserChecks)
$ErrorActionPreference='Stop'
$repository=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$previousWasm=$env:BISECT_TEST_WASM
$previousCffiEvidence=$env:BISECT_CFFI_BROWSER_EVIDENCE
Push-Location $repository
try {
    $basePath=[IO.Path]::GetFullPath((Join-Path $repository $Base))
    $outputPath=[IO.Path]::GetFullPath((Join-Path $repository $Output))
    $distRoot=[IO.Path]::GetFullPath((Join-Path $repository 'dist'))+[IO.Path]::DirectorySeparatorChar
    if(!$outputPath.StartsWith($distRoot,[StringComparison]::OrdinalIgnoreCase)){throw 'Choose a local dist/ output directory.'}
    if(Test-Path -LiteralPath $outputPath){throw 'Choose a new output directory; existing packages are retained.'}
    if(!(Test-Path -LiteralPath (Join-Path $basePath 'catalog.json')) -or !(Test-Path -LiteralPath (Join-Path $basePath 'toolkit/index.html'))){throw 'Build a base laboratory package first.'}
    python scripts/web/wasm_practitioner_inventory.py
    if($LASTEXITCODE -ne 0){throw 'Practitioner action source inventory drifted.'}
    node scripts/web/test_report_print.mjs
    if($LASTEXITCODE -ne 0){throw 'Report print/PDF byte-observation guards failed.'}
    python scripts/web/wasm_parity_ledger.py
    if($LASTEXITCODE -ne 0){throw 'Source/ledger checks failed.'}
    node scripts/web/test_offline_package.mjs $basePath
    if($LASTEXITCODE -ne 0){throw 'Base package identities are invalid.'}
    cargo build -p bisect-wasm --release --example execute_tool_request --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native host reference build failed.'}
    cargo build -p bisect-cli --no-default-features --example native_adjacency_reference --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native filesystem graph reference build failed.'}
    cargo build -p bisect-cli --bin bisect --release --no-default-features --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native data-reset CLI reference build failed.'}
    $env:BISECT_TEST_WASM=Join-Path $basePath 'toolkit/bisect_wasm.wasm'
    cargo build -p bisect-cli --no-default-features --example fetch_plan_reference --example fletch_sources_reference --example fetch_archive_reference --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Fetch/FLETCH source reference build failed.'}
    node scripts/web/test_fetch_source_coverage.mjs
    if($LASTEXITCODE -ne 0){throw 'Fetch/FLETCH source-type reconciliation drifted.'}
    $fetchArchiveCheck=node scripts/web/test_fetch_archive_acquisition.mjs | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Fetch archive/native acquisition checks failed.'}
    $fetchArchiveRoot=$fetchArchiveCheck.root
    node scripts/web/test_native_state_data_reset.mjs
    if($LASTEXITCODE -ne 0){throw 'Native data reset/CLI filesystem checks failed.'}
    node scripts/web/test_native_state_target_run.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State before-Run target/source/CLI checks failed.'}
    $labelRepositoryCheck=node scripts/web/test_label_repository_installation.mjs --json | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Label repository/native CLI installation checks failed.'}
    $labelRepositoryRoot=$labelRepositoryCheck.root
    node scripts/web/test_native_workspace_continuation.mjs
    if($LASTEXITCODE -ne 0){throw 'Verified saved-workspace continuation checks failed.'}
    node scripts/web/test_native_bulk_workspace.mjs
    if($LASTEXITCODE -ne 0){throw 'Native bulk workspace completion/target/CLI checks failed.'}
    cargo build -p bisect-cli --no-default-features --example tiger_reference --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Tracked acquisition fixture writer build failed.'}
    node scripts/web/test_acquisition_recovery.mjs
    if($LASTEXITCODE -ne 0){throw 'State-source acquisition recovery checks failed.'}
    node scripts/web/test_acquisition_recovery_configured.mjs
    if($LASTEXITCODE -ne 0){throw 'Configured source-acquisition recovery checks failed.'}
    node scripts/web/test_native_output_verification.mjs
    if($LASTEXITCODE -ne 0){throw 'Composed native output/source Verify checks failed.'}
    node scripts/web/test_native_workspace_source.mjs
    if($LASTEXITCODE -ne 0){throw 'Workspace source/native comparisons failed.'}
    if(Test-Path -LiteralPath (Join-Path $basePath 'toolkit/c-metis/metis-sidecar.json')){node scripts/web/test_explicit_c_state_batch.mjs (Join-Path $basePath 'toolkit');if($LASTEXITCODE -ne 0){throw 'Explicit C batch/native checks failed.'}}
    Copy-Item -LiteralPath $basePath -Destination $outputPath -Recurse
    foreach($directory in @($outputPath,(Join-Path $outputPath 'toolkit'))){
        Get-ChildItem -LiteralPath web/lab -File | Where-Object {$_.Name -ne 'index.html'} | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $directory $_.Name)}
    }
    Copy-Item -LiteralPath web/lab/workbench.html -Destination (Join-Path $outputPath 'toolkit/index.html')
    python scripts/web/stage_native_proof_preview.py --destination (Join-Path $outputPath 'toolkit')
    if($LASTEXITCODE -ne 0){throw 'Proof dependency identity refresh failed.'}
    python scripts/web/build_offline_manifest.py $outputPath
    if($LASTEXITCODE -ne 0){throw 'Offline identity generation failed.'}
    node scripts/web/test_toolkit_assets.mjs (Join-Path $outputPath 'toolkit')
    if($LASTEXITCODE -ne 0){throw 'Toolkit asset closure failed.'}
    node scripts/web/test_offline_package.mjs $outputPath
    if($LASTEXITCODE -ne 0){throw 'Final offline package checks failed.'}
    if($BrowserChecks){
        node scripts/web/test_fetch_archive_acquisition_browser.mjs $fetchArchiveRoot (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Final Fetch archive acquisition/local/cache browser checks failed.'}
        node scripts/web/test_native_state_data_reset_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Final packaged native data-reset browser checks failed.'}
        node scripts/web/test_native_state_target_run_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Native State before-Run target browser checks failed.'}
        node scripts/web/test_native_label_summary_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Final native label summary browser checks failed.'}
        node scripts/web/test_count_transcript_installation_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Final RCOUNT transcript installation checks failed.'}
        $reportPdfBrowser=node scripts/web/test_report_pdf_browser.mjs (Join-Path $outputPath 'toolkit') --json | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw 'Actual report PDF/browser/native CLI checks failed.'}
        $labelRepositoryBrowser=node scripts/web/test_label_repository_installation_browser.mjs (Join-Path $outputPath 'toolkit') --json | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw 'Label repository installation browser checks failed.'}
        node scripts/web/test_native_workspace_continuation_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Verified workspace continuation browser checks failed.'}
        Copy-Item -LiteralPath target/native-workspace-continuation-browser/evidence.json -Destination target/native-workspace-continuation-browser/one-district-evidence.json
        node scripts/web/test_native_workspace_continuation_browser.mjs (Join-Path $outputPath 'toolkit') --multi-district
        if($LASTEXITCODE -ne 0){throw 'Multi-district workspace continuation browser checks failed.'}
        Copy-Item -LiteralPath target/native-workspace-continuation-browser/evidence.json -Destination target/native-workspace-continuation-browser/multi-district-evidence.json
        node scripts/web/test_native_bulk_workspace_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Native bulk workspace browser checks failed.'}
        node scripts/web/test_acquisition_recovery_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'State-source acquisition recovery browser checks failed.'}
        node scripts/web/test_acquisition_recovery_browser.mjs (Join-Path $outputPath 'toolkit') --configured
        if($LASTEXITCODE -ne 0){throw 'Configured acquisition recovery browser checks failed.'}
        node scripts/web/test_tui_verify_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Legacy TUI Verify packaged browser regression failed.'}
        node scripts/web/test_native_output_verification_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Composed native output Verify browser checks failed.'}
        node scripts/web/test_native_workspace_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Final packaged workspace browser checks failed.'}
        node scripts/web/test_state_source_acquisition_browser.mjs (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Legacy acquisition/streaming browser regression failed.'}
        node scripts/web/test_native_state_recovery_browser.mjs --package (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Final packaged single output recovery failed.'}
        node scripts/web/test_native_state_batch_recovery_browser.mjs --package (Join-Path $outputPath 'toolkit')
        if($LASTEXITCODE -ne 0){throw 'Final packaged batch output recovery failed.'}
        if(Test-Path -LiteralPath (Join-Path $outputPath 'toolkit/c-metis/metis-sidecar.json')){
            $env:BISECT_CFFI_BROWSER_EVIDENCE=Join-Path $repository 'target/native-workspace-source/cffi-browser'
            node scripts/web/test_explicit_c_state_batch_browser.mjs (Join-Path $outputPath 'toolkit')
            if($LASTEXITCODE -ne 0){throw 'Explicit C state-batch browser checks failed.'}
            node scripts/web/test_cffi_engine_browser.mjs (Join-Path $outputPath 'toolkit')
            if($LASTEXITCODE -ne 0){throw 'Optional C workbench regression failed.'}
        }
    }
    python -c "import hashlib,json,sys; from pathlib import Path; p=Path(sys.argv[1]); pin=lambda f:{'path':str(f).replace(chr(92),'/'),'sha256':hashlib.sha256(f.read_bytes()).hexdigest()}; names=['web/lab/report-print.js','scripts/web/test_report_print.mjs','scripts/web/test_report_pdf_browser.mjs','scripts/web/validate_report_print_pdf.py','scripts/web/fixtures/report-pdf-case.mjs','web/lab/label-repository-install.js','web/lab/label-repository-install-ui.js','scripts/web/test_label_repository_installation.mjs','scripts/web/test_label_repository_installation_browser.mjs','scripts/web/fixtures/label-repository-case.mjs','scripts/web/wasm_practitioner_inventory.py','docs/specs/wasm-practitioner-actions.json','scripts/web/test_fetch_archive_acquisition.mjs','scripts/web/test_fetch_archive_acquisition_browser.mjs','web/lab/fetch-archive-package.js','web/lab/fetch-source-archive.js','web/lab/fetch-source-archive-worker.js','web/lab/fetch-acquisition.js','web/lab/fletch-stage-files.js','web/lab/source-relay-policy.js','docs/specs/wasm-fetch-source-coverage.json','crates/bisect-cli/examples/fetch_archive_reference.rs','scripts/web/test_explicit_c_state_batch.mjs','scripts/web/test_explicit_c_state_batch_browser.mjs','web/lab/native-bulk-checkpoint-verification.js','web/lab/native-bulk-workspace-ui.js','scripts/web/test_native_workspace_continuation.mjs','scripts/web/test_native_workspace_continuation_browser.mjs','scripts/web/fixtures/native-workspace-continuation-case.mjs','web/lab/native-state-output-ui.js','web/lab/native-output-verification.js','crates/bisect-wasm/src/verify_replay.rs','crates/bisect-wasm/src/tui_verify.rs','crates/bisect-wasm/src/engine_replay.rs','crates/bisect-cli/src/verify.rs','scripts/web/test_native_output_verification.mjs','scripts/web/test_native_output_verification_browser.mjs','scripts/web/test_tui_verify_browser.mjs','web/lab/state-source-acquisition-recovery.js','web/lab/state-source-acquisition.js','scripts/web/test_acquisition_recovery.mjs','scripts/web/test_acquisition_recovery_configured.mjs','scripts/web/test_acquisition_recovery_browser.mjs','scripts/web/test_state_source_acquisition_browser.mjs','scripts/web/fixtures/acquisition-recovery-case.mjs','crates/bisect-cli/examples/tiger_reference.rs','web/lab/native-bulk-workspace.js','web/lab/native-bulk-workspace-ui.js','web/lab/state-run-batch.js','web/lab/state-run-years.js','scripts/web/test_native_bulk_workspace.mjs','scripts/web/test_native_bulk_workspace_browser.mjs','web/lab/native-state-target-run.js','web/lab/native-state-target-authority.js','web/lab/native-state-before-run-ui.js','web/lab/native-state-workflow.js','web/lab/native-state-job.js','web/lab/native-selected-state-source.js','web/lab/wasm-worker.js','scripts/web/test_native_state_target_run.mjs','scripts/web/test_native_state_target_run_browser.mjs','scripts/web/test_native_state_recovery_browser.mjs','scripts/web/test_native_state_batch_recovery_browser.mjs','web/lab/native-state-install.js','web/lab/native-state-output-ui.js','scripts/web/test_native_state_data_reset.mjs','scripts/web/test_native_state_data_reset_browser.mjs','web/lab/native-workspace-source.js','web/lab/native-workspace-archive-worker.js','web/lab/workbench.js','web/lab/workbench.html','web/lab/project.js','scripts/web/build_workspace_preview.ps1','scripts/web/test_native_workspace_source.mjs','scripts/web/test_native_workspace_browser.mjs','scripts/web/fixtures/native-workspace-case.mjs','scripts/web/test_cffi_engine_browser.mjs']; refs=[pin(Path(n)) for n in names]; browser=sys.argv[2]=='True'; results=[pin(Path('target/native-workspace-continuation/evidence.json')),pin(Path('target/native-workspace-continuation/verification.json')),pin(Path('target/native-workspace-continuation/completed.json')),pin(Path('target/native-workspace-continuation/multi-completed.json')),pin(Path('target/native-workspace-continuation/cases.json')),pin(Path('target/native-workspace-continuation/filesystem-events.json')),pin(Path('target/native-output-verification/native-inspections.json')),pin(Path('target/native-output-verification/native-tool-reference.exe')),pin(Path('target/native-output-verification/evidence.json')),pin(Path('target/native-output-verification/records.json')),pin(Path('target/native-output-verification/native-cases.json')),pin(Path('target/native-output-verification/native-bisect-reference.exe')),pin(Path('target/acquisition-recovery-configured/evidence.json')),pin(Path('target/acquisition-recovery/native-replies.json')),pin(Path('target/acquisition-recovery/native-tool-reference.exe')),pin(Path('target/acquisition-recovery/native-tiger-fixture.exe')),pin(Path('target/acquisition-recovery/evidence.json')),pin(Path('target/acquisition-recovery/partial.json')),pin(Path('target/acquisition-recovery/resumed.json')),pin(Path('target/acquisition-recovery/verified.json')),pin(Path('target/acquisition-recovery/run.json')),pin(Path('target/native-bulk-workspace/evidence.json')),pin(Path('target/native-bulk-workspace/native-record.json')),pin(Path('target/native-bulk-workspace/additional-results.json')),pin(Path('target/native-bulk-workspace/native-bisect-reference.exe')),pin(Path('target/native-state-target-run/evidence.json')),pin(Path('target/native-state-target-run/backend-cases.json')),pin(Path('target/native-state-target-run/failed-after-reset.json')),pin(Path('target/native-state-target-run/records.json')),pin(Path('target/native-state-target-run/native-bisect-reference.exe')),pin(Path('target/native-workspace-source/evidence.json')),pin(Path('target/native-state-data-reset/evidence.json'))]+([pin(Path('target/native-workspace-continuation-browser/evidence.json')),pin(Path('target/native-output-verification-browser/evidence.json')),pin(Path('target/acquisition-recovery-configured-browser/evidence.json')),pin(Path('target/acquisition-recovery-browser/evidence.json')),pin(Path('target/native-bulk-workspace-browser/evidence.json')),pin(Path('target/native-state-recovery-browser/evidence.json')),pin(Path('target/native-state-batch-recovery-browser/evidence.json')),pin(Path('target/native-state-target-run-browser/evidence.json')),pin(Path('target/native-workspace-browser/evidence.json')),pin(Path('target/native-state-data-reset-browser/evidence.json'))] if browser else []); d={'schema':'bisect-workspace-preview-checks-v1','package':str(p),'source_files':refs,'results':results,'offline_manifest':pin(p/'offline-manifest.json'),'offline_version':json.loads((p/'offline-manifest.json').read_text(encoding='utf-8'))['version'],'browser_status':'passed' if browser else 'not-run','explicit_c_batch_evidence':[pin(Path('target/explicit-c-state-batch',n)) for n in ['evidence.json','completed.json','native-request.json','native-reply.json','native-c-reference.exe','source.bisect']]+([pin(Path('target/explicit-c-state-batch-browser/evidence.json')),pin(Path('target/explicit-c-state-batch-browser/completed.bisect'))] if browser else []) if (p/'toolkit/c-metis/metis-sidecar.json').is_file() else [],'optional_c_browser_evidence':pin(Path('target/native-workspace-source/cffi-browser/evidence.json')) if browser and (p/'toolkit/c-metis/metis-sidecar.json').is_file() else None,'full_parity':False,'deployed':False}; d['report_pdf_evidence']={'status':'passed' if browser else 'not-run','root':sys.argv[6] if browser else None,'browser':pin(Path(sys.argv[6])/'evidence.json') if browser else None}; d['label_repository_evidence']={'node_root':sys.argv[4],'node':pin(Path(sys.argv[4])/'evidence.json'),'browser_root':sys.argv[5] if browser else None,'browser':pin(Path(sys.argv[5])/'evidence.json') if browser else None}; d['fetch_archive_evidence']={'root':sys.argv[3],'node':pin(Path(sys.argv[3])/'evidence.json'),'browser':[pin(f) for f in Path(sys.argv[3]).glob('browser-*/evidence.json')]}; Path('target/native-workspace-source/package-evidence.json').write_text(json.dumps(d,indent=2)+'\n',encoding='utf-8')" $outputPath ([string][bool]$BrowserChecks) $fetchArchiveRoot $labelRepositoryRoot $(if($BrowserChecks){$labelRepositoryBrowser.root}else{'not-run'}) $(if($BrowserChecks){$reportPdfBrowser.root}else{'not-run'})
    if($LASTEXITCODE -ne 0){throw 'Could not retain package check evidence.'}
    Write-Host "Local workspace preview: $outputPath. Broader workspace/parity acceptance remains open; nothing deployed."
} finally {$env:BISECT_TEST_WASM=$previousWasm;$env:BISECT_CFFI_BROWSER_EVIDENCE=$previousCffiEvidence;Pop-Location}
