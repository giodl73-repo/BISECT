param([string]$Base='dist/wasm-lab-build',[string]$Output='dist/wasm-cffi-preview',[switch]$BrowserChecks)

$ErrorActionPreference='Stop'

$repository=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path

Push-Location $repository

try {
    python scripts/web/test_build_gate_dependencies.py
    if($LASTEXITCODE -ne 0){throw 'Build dependency guard mutation checks failed.'}
    python scripts/web/check_build_gate_dependencies.py
    if($LASTEXITCODE -ne 0){throw 'Build fixture or native executable producer order is invalid.'}
    python scripts/web/test_native_state_extraction.py
    if($LASTEXITCODE -ne 0){throw 'Native State reviewed extraction check failed.'}
    python scripts/web/test_native_state_extraction_transitions.py
    if($LASTEXITCODE -ne 0){throw 'Native State reviewed source transition mutation checks failed.'}
    python scripts/web/proof_dependency_preflight.py
    if($LASTEXITCODE -ne 0){throw 'Pinned proof prerequisites are missing, stale or altered; prepare the local solver/checker builds first.'}
    python -B scripts/web/test_proof_dependency_preflight.py
    if($LASTEXITCODE -ne 0){throw 'Proof prerequisite refusal and Git read-only regression failed.'}
    node scripts/web/test_native_command_comparison.mjs
    if($LASTEXITCODE -ne 0){throw 'Native/WASM command comparison corruption guards failed.'}


    $basePath=[IO.Path]::GetFullPath((Join-Path $repository $Base))

    $outputPath=[IO.Path]::GetFullPath((Join-Path $repository $Output))

    $distRoot=[IO.Path]::GetFullPath((Join-Path $repository 'dist'))+[IO.Path]::DirectorySeparatorChar

    if(!$outputPath.StartsWith($distRoot,[StringComparison]::OrdinalIgnoreCase)){throw 'Choose a local dist/ output directory.'}

    if(!(Test-Path -LiteralPath (Join-Path $basePath 'catalog.json')) -or !(Test-Path -LiteralPath (Join-Path $basePath 'toolkit/index.html'))){throw 'Build a base laboratory package first; this experimental extension does not replace its full build gates.'}

    if(Test-Path -LiteralPath $outputPath){throw 'Choose a new output directory; existing packages are retained.'}
    if($BrowserChecks -and !(Test-Path -LiteralPath (Join-Path $basePath 'toolkit/fixtures/count-package.json') -PathType Leaf)){throw 'The base toolkit is missing the generated Minerva count example; build the complete standard laboratory first.'}

    python scripts/web/test_engine_option_inventory_history.py
    if($LASTEXITCODE -ne 0){throw 'Engine declaration history preservation failed.'}
    python scripts/web/test_parity_evidence_archive.py
    if($LASTEXITCODE -ne 0){throw 'Child evidence archive resource/path guards failed.'}
    python scripts/web/parity_evidence_archive.py ensure --root . --lock docs/specs/wasm-parity-child-evidence.lock.json
    if($LASTEXITCODE -ne 0){throw 'Restore the declared local child-evidence capsule before ledger validation.'}
    python scripts/web/wasm_parity_ledger.py

    if($LASTEXITCODE -ne 0){throw 'Source/ledger checks failed.'}

    node scripts/web/test_native_asset_build.mjs
    if($LASTEXITCODE -ne 0){throw 'Native embedded asset rebuild controls failed.'}
    cargo test -p bisect-web --lib --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native web/catalog asset and budget checks failed.'}
    $nativeAssetSuffix = if ([IO.Path]::DirectorySeparatorChar -eq '\') {'.exe'} else {''}
    if($BrowserChecks){
        cargo build -p bisect-web --bin bisect-web --bin lab-catalog --locked --offline -j 1
        if($LASTEXITCODE -ne 0){throw 'Native web/catalog browser fixture build failed.'}
        $nativeAssetWeb = Join-Path $repository ('target/debug/bisect-web'+$nativeAssetSuffix)
        $nativeAssetCatalog = Join-Path $repository ('target/debug/lab-catalog'+$nativeAssetSuffix)
        node scripts/web/test_native_web_assets_browser.mjs $nativeAssetWeb $nativeAssetCatalog (Join-Path $repository 'web/lab')
        if($LASTEXITCODE -ne 0){throw 'Native web/catalog module and saved-view browser checks failed.'}
    }


    # This builds the actual default and optional variants and leaves the latter

    # as Cargo's final output. Its portable native oracle is intentionally built

    # before the separate native C oracle below.

    & (Join-Path $PSScriptRoot 'check_cffi_legacy_gates.ps1') -BothBuilds -BrowserChecks:$BrowserChecks

    python scripts/web/build_metis_sidecar.py

    if($LASTEXITCODE -ne 0){throw 'Explicit C sidecar build failed; install the pinned SDK explicitly first.'}

    cargo test -p bisect-metis-sidecar --locked --offline

    if($LASTEXITCODE -ne 0){throw 'Rust bridge validation failed.'}

    # These CLI lifecycle/unused-field checks run the default Rust WASM path.
    # Preserve its matching native backend before the C build replaces Cargo's
    # shared binary. Exact assignment comparisons must not cross backends.
    cargo build -p bisect-cli --release --bin bisect --no-default-features --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Portable native CLI reference build failed.'}
    $coreCliRoot=Join-Path $repository ('target/cffi-core-cli-reference/'+[guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $coreCliRoot -ErrorAction Stop | Out-Null
    $coreCliPath=Join-Path $coreCliRoot 'bisect.exe'
    Copy-Item -LiteralPath target/release/bisect.exe -Destination $coreCliPath
    $coreCliHash=(Get-FileHash -LiteralPath $coreCliPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if($coreCliHash -ne (Get-FileHash -LiteralPath target/release/bisect.exe -Algorithm SHA256).Hash.ToLowerInvariant()){throw 'Portable native CLI snapshot changed.'}
    @{schema_version='bisect-cffi-core-cli-reference-v1';path=$coreCliPath;sha256=$coreCliHash;native_backend='metis-core';cargo_features='no-default-features';builder_sha256=(Get-FileHash -LiteralPath (Join-Path $repository 'scripts/web/build_cffi_preview.ps1') -Algorithm SHA256).Hash.ToLowerInvariant()} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $coreCliRoot 'reference.json') -Encoding utf8

    cargo build -p bisect-cli --release --bin bisect --example state_signal_preset_reference --no-default-features --features c-ffi-engine --locked --offline -j 1

    if($LASTEXITCODE -ne 0){throw 'Native C reference build failed.'}

    node scripts/web/test_cffi_engine.mjs

    if($LASTEXITCODE -ne 0){throw 'Native C/Rust engine comparisons failed.'}
    python scripts/web/test_native_cli_workspace.py --native-executable $coreCliPath
    if($LASTEXITCODE -ne 0){throw 'Actual native CLI/WASM workspace lifecycle regression failed.'}
    python scripts/web/test_native_cli_unused.py --native-executable $coreCliPath
    if($LASTEXITCODE -ne 0){throw 'Actual native CLI/WASM unused-field and intent regression failed.'}
    python scripts/web/test_native_cli_verify_replay.py --native-executable $coreCliPath
    if($LASTEXITCODE -ne 0){throw 'Actual CLI Verify replay output-root regression failed; preserve its raw State/Verify receipts.'}
    node scripts/web/test_native_aggregate_selection.mjs (Join-Path $repository 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $repository ('target/release/bisect'+$nativeAssetSuffix))
    if($LASTEXITCODE -ne 0){throw 'Native Aggregate effective-source selection regression failed.'}
    $nativeAggregateSuiteArguments=@((Join-Path $repository 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),(Join-Path $repository ('target/release/bisect'+$nativeAssetSuffix)))
    if($BrowserChecks){$nativeAggregateSuiteArguments+='--browser'}
    node scripts/web/test_native_aggregate_effects_suite.mjs @nativeAggregateSuiteArguments
    if($LASTEXITCODE -ne 0){throw 'Native Aggregate command/recovery/backup suite failed; preserve its owned evidence.'}
    $census2010Check=node scripts/web/test_census_2010_endpoints.mjs (Join-Path $repository 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $repository ('target/release/examples/execute_tool_request'+$nativeAssetSuffix)) $coreCliPath | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Native 2010 Census endpoint regression failed; preserve its raw native/WASM/CLI evidence.'}
    if($census2010Check.status -ne 'passed'){throw 'Native 2010 Census endpoint receipt did not pass.'}
    if($BrowserChecks){
        node scripts/web/test_census_2010_browser.mjs $census2010Check.census_2010_endpoint_evidence_root
        if($LASTEXITCODE -ne 0){throw 'Native 2010 Census acquisition/browser boundary regression failed.'}
    }
    $ilpNativeSummaryCheck=node scripts/web/test_ilp_native_summary_paths.mjs (Join-Path $repository 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $repository ('target/release/examples/execute_tool_request'+$nativeAssetSuffix)) $coreCliPath | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Native ILP summary path binding regression failed; preserve its actual CLI/host/WASM receipts.'}
    if($ilpNativeSummaryCheck.status -ne 'passed'){throw 'Native ILP summary path binding receipt did not pass.'}
    if($BrowserChecks){
        node scripts/web/test_ilp_native_summary_paths_browser.mjs $ilpNativeSummaryCheck.ilp_native_summary_evidence_root
        if($LASTEXITCODE -ne 0){throw 'Native ILP source-byte/browser summary regression failed.'}
    }

    if($BrowserChecks){
        node scripts/web/test_workbench_project_open_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Workbench inert Open observation regression failed.'}
        node scripts/web/test_native_aggregate_order_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native Aggregate ordered-type browser regression failed.'}
    }

    Copy-Item -LiteralPath $basePath -Destination $outputPath -Recurse

    foreach($directory in @($outputPath,(Join-Path $outputPath 'toolkit'))){

        Get-ChildItem -LiteralPath web/lab -File | Where-Object {$_.Name -ne 'index.html'} | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $directory $_.Name)}

        Copy-Item -LiteralPath target/wasm32-unknown-unknown/release/bisect_wasm.wasm -Destination (Join-Path $directory 'bisect_wasm.wasm')

    }

    $legacyReceipt=Get-Content -LiteralPath target/cffi-legacy-gates/c-sidecar/build.json -Raw | ConvertFrom-Json

    $packagedHash=(Get-FileHash -LiteralPath (Join-Path $outputPath 'toolkit/bisect_wasm.wasm') -Algorithm SHA256).Hash.ToLowerInvariant()

    if($legacyReceipt.status -ne 'passed' -or $legacyReceipt.wasm_sha256 -ne $packagedHash){throw 'Packaged engine differs from the completed C-feature legacy gates.'}

    Copy-Item -LiteralPath web/lab/workbench.html -Destination (Join-Path $outputPath 'toolkit/index.html')

    python -c "import hashlib,json,sys; from pathlib import Path; p=Path(sys.argv[1]); f=p/'catalog.json'; d=json.loads(f.read_text(encoding='utf-8')); d['wasm_sha256']=hashlib.sha256((p/'bisect_wasm.wasm').read_bytes()).hexdigest(); f.write_text(json.dumps(d,indent=2)+'\n',encoding='utf-8')" $outputPath

    if($LASTEXITCODE -ne 0){throw 'Catalog identity refresh failed.'}

    python scripts/web/stage_metis_sidecar.py --destination (Join-Path $outputPath 'toolkit')

    if($LASTEXITCODE -ne 0){throw 'C sidecar staging failed.'}

    python scripts/web/stage_native_proof_preview.py --destination (Join-Path $outputPath 'toolkit')

    if($LASTEXITCODE -ne 0){throw 'Proof dependency identity refresh failed.'}

    python scripts/web/build_offline_manifest.py $outputPath

    if($LASTEXITCODE -ne 0){throw 'Offline identity generation failed.'}

    node scripts/web/test_cffi_engine.mjs --toolkit (Join-Path $outputPath 'toolkit')

    if($LASTEXITCODE -ne 0){throw 'Final packaged native C/Rust engine comparisons failed.'}

    node scripts/web/test_explicit_c_state_batch.mjs (Join-Path $outputPath 'toolkit')

    if($LASTEXITCODE -ne 0){throw 'Explicit C state-batch/native checks failed.'}

    node scripts/web/verify_proof_package.mjs (Join-Path $outputPath 'toolkit')
    if($LASTEXITCODE -ne 0){throw 'Final C proof dependency identity check failed.'}
    node scripts/web/test_proof_package.mjs (Join-Path $outputPath 'toolkit')
    if($LASTEXITCODE -ne 0){throw 'Final C proof dependency refusal checks failed.'}
    node scripts/web/test_toolkit_assets.mjs (Join-Path $outputPath 'toolkit')

    if($LASTEXITCODE -ne 0){throw 'Toolkit asset closure failed.'}

    node scripts/web/test_offline_package.mjs $outputPath

    if($LASTEXITCODE -ne 0){throw 'Offline package checks failed.'}

    cargo build -p rplan-io --example rewrite_context --release --target-dir target/explicit-county-context-host --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native context fixture writer build failed.'}
    $municipal=node scripts/web/test_explicit_municipal_splits.mjs --wasm (Join-Path $outputPath 'toolkit/bisect_wasm.wasm') --cli target/release/bisect.exe --fixture-wasm (Join-Path $basePath 'bisect_wasm.wasm') --rewriter target/explicit-county-context-host/release/examples/rewrite_context.exe | ConvertFrom-Json
    if($LASTEXITCODE -ne 0 -or $municipal.cases -ne 9){throw 'Explicit municipal split native/WASM gate failed.'}
    if($BrowserChecks){
        $municipalBrowser=node scripts/web/test_explicit_municipal_splits_browser.mjs --package (Join-Path $outputPath 'toolkit') --runtime-receipt $municipal.receipt | ConvertFrom-Json
        if($LASTEXITCODE -ne 0 -or $municipalBrowser.cases -ne 3){throw 'Explicit municipal split packaged browser gate failed.'}
    }

    $ceremonyPackage=node scripts/web/test_ceremony_cli_contract.mjs ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) --native=target/release/bisect.exe | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual CLI ceremony contract failed.'}

    $ceremonyPackage | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/ceremony-contract-root.json -Encoding utf8

    $ceremonySourcesPackage=node scripts/web/test_ceremony_source_start.mjs ('--contract='+$ceremonyPackage.root) ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package original-profile ceremony/native comparison failed.'}

    $ceremonySourcesPackage | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/ceremony-source-start-root.json -Encoding utf8

    $configPackage=node scripts/web/test_config_cli_contract.mjs ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual Config CLI comparison failed.'}

    $configBrowser=$null

    if($BrowserChecks){

        $configBrowser=node scripts/web/test_config_cli_browser.mjs ('--reference='+$configPackage.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package Config create/validate browser failed.'}

    }

    @{status='passed';node=$configPackage;browser=$configBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/config-cli-root.json -Encoding utf8

    $configPublicationWindows=@{status='not-run';reason='Real Windows sharing locks require Windows.'}
    if ($IsWindows) {
        $configPublicationWindows=node scripts/web/test_config_publication_windows.mjs ('--reference='+$configPackage.root) ('--profile='+'final-package') | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw 'Real Windows config publication sharing-lock checks failed.'}
    }

    $configPublication=node scripts/web/test_config_publication.mjs ('--reference='+$configPackage.root) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package config publication/native byte checks failed.'}

    $configPublicationBrowser=$null

    if($BrowserChecks){

        $configPublicationBrowser=node scripts/web/test_config_publication_browser.mjs ('--reference='+$configPackage.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package config publication workbench failed.'}

    }

    @{status='passed';node=$configPublication;windows_host=$configPublicationWindows;browser=$configPublicationBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/config-publication-root.json -Encoding utf8

    $policyPackage=node scripts/web/test_policy_cli_contract.mjs --native=target/release/bisect.exe ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual Policy CLI comparison failed.'}

    $policyBrowser=$null

    if($BrowserChecks){

        $policyBrowser=node scripts/web/test_policy_cli_browser.mjs ('--reference='+$policyPackage.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package original Policy browser checks failed.'}

    }

    @{status='passed';node=$policyPackage;browser=$policyBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/policy-cli-root.json -Encoding utf8

    $policyFallback=node scripts/web/test_policy_fallback_wasm.mjs ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) ('--reference='+$policyPackage.root) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package native Policy fallback comparison failed.'}

    $policyFallbackBrowser=$null

    if($BrowserChecks){

        $policyFallbackBrowser=node scripts/web/test_policy_fallback_browser.mjs ('--reference='+$policyFallback.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package Policy fallback workbench failed.'}

    }

    @{status='passed';node=$policyFallback;browser=$policyFallbackBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/policy-fallback-root.json -Encoding utf8

    $doctorCommand=node scripts/web/test_doctor_command_wasm.mjs --native=target/release/bisect.exe ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual Doctor command comparison failed.'}

    $doctorCommandBrowser=$null

    if($BrowserChecks){

        $doctorCommandBrowser=node scripts/web/test_doctor_command_browser.mjs ('--reference='+$doctorCommand.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package Doctor command workbench failed.'}

    }

    @{status='passed';node=$doctorCommand;browser=$doctorCommandBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/doctor-command-root.json -Encoding utf8

    $manifestWidth=node scripts/web/test_plan_manifest_width.mjs --native=target/release/bisect.exe ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual Read-only manifest width comparison failed.'}

    $manifestWidthBrowser=$null

    if($BrowserChecks){

        $manifestWidthBrowser=node scripts/web/test_plan_manifest_width_browser.mjs ('--reference='+$manifestWidth.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package Read-only manifest width workbench failed.'}

    }

    @{status='passed';node=$manifestWidth;browser=$manifestWidthBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/plan-manifest-width-root.json -Encoding utf8

    $largePlanText=node scripts/web/test_large_rplan_text.mjs --native=target/release/bisect.exe ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Final package large RPLAN typed/native boundary comparison failed.'}
    $largePlanText | ConvertTo-Json -Depth 8 | Set-Content target/cffi-integration/large-rplan-text-root.json -Encoding utf8
    $largeAuditCatalog=Get-Content -LiteralPath (Join-Path $outputPath 'catalog.json') -Raw | ConvertFrom-Json
    if($largeAuditCatalog.graphs.PSObject.Properties.Name -contains 'CA:2020'){
        $largeAudit=node scripts/web/test_laboratory_practitioner_archive.mjs $outputPath | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw 'Final package full-state practitioner archive/native comparison failed.'}
        $largeAuditBrowser=$null
        if($BrowserChecks){
            $largeAuditBrowser=node scripts/web/test_laboratory_practitioner_archive_browser.mjs ('--reference='+$largeAudit.root) ('--laboratory='+$outputPath) ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json
            if($LASTEXITCODE -ne 0){throw 'Final package full-state practitioner archive browser check failed.'}
        }
        @{status='passed';node=$largeAudit;browser=$largeAuditBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content target/cffi-integration/large-practitioner-archive-root.json -Encoding utf8
    }else{
        @{status='not-run';reason='CA prepared graph/geometry not included in this package scope.'} | ConvertTo-Json | Set-Content target/cffi-integration/large-practitioner-archive-root.json -Encoding utf8
    }
    $manifestReader=node scripts/web/test_manifest_reader_width.mjs --native=target/release/examples/execute_tool_request.exe --cli=target/release/bisect.exe ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual Read-only manifest readers comparison failed.'}

    $manifestReaderBrowser=$null

    if($BrowserChecks){

        $manifestReaderBrowser=node scripts/web/test_manifest_reader_width_browser.mjs ('--reference='+$manifestReader.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package Read-only manifest readers workbench failed.'}

    }

    @{status='passed';node=$manifestReader;browser=$manifestReaderBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/manifest-reader-width-root.json -Encoding utf8

    $manifestCli=node scripts/web/test_manifest_cli_contract.mjs --native=target/release/bisect.exe ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual Manifest CLI comparison failed.'}

    $manifestCliBrowser=$null

    if($BrowserChecks){

        $manifestCliBrowser=node scripts/web/test_manifest_cli_browser.mjs ('--reference='+$manifestCli.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package Manifest CLI workbench failed.'}

    }

    @{status='passed';node=$manifestCli;browser=$manifestCliBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/manifest-cli-root.json -Encoding utf8

    $tutorialCli=node scripts/web/test_tutorial_cli_contract.mjs --native=target/release/bisect.exe ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual Tutorial CLI comparison failed.'}

    $tutorialCliBrowser=$null

    if($BrowserChecks){

        $tutorialCliBrowser=node scripts/web/test_tutorial_cli_browser.mjs ('--reference='+$tutorialCli.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package Tutorial CLI workbench failed.'}

    }

    @{status='passed';node=$tutorialCli;browser=$tutorialCliBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/tutorial-cli-root.json -Encoding utf8

    $doctorWorkspace=node scripts/web/test_doctor_workspace_cli.mjs --native=target/release/bisect.exe ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual Native Doctor workspace comparison failed.'}

    $doctorWorkspaceBoundaries=node scripts/web/test_doctor_workspace_boundaries.mjs ('--reference='+$doctorWorkspace.root) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Native Doctor snapshot boundary checks failed.'}

    $doctorWorkspaceBrowser=$null

    if($BrowserChecks){

        $doctorWorkspaceBrowser=node scripts/web/test_doctor_workspace_browser.mjs ('--reference='+$doctorWorkspace.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package Native Doctor workspace workbench failed.'}

    }

    @{status='passed';node=$doctorWorkspace;boundaries=$doctorWorkspaceBoundaries;browser=$doctorWorkspaceBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/doctor-workspace-root.json -Encoding utf8

    $formatPackage=node scripts/web/test_native_format_cli.mjs ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual native format CLI comparison failed.'}

    $formatBrowser=$null

    if($BrowserChecks){

        $formatBrowser=node scripts/web/test_native_format_cli_browser.mjs ('--reference='+$formatPackage.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package native format UI/source-byte comparison failed.'}

    }

    @{status='passed';node=$formatPackage;browser=$formatBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/native-format-cli-root.json -Encoding utf8

    $fletchPackage=node scripts/web/test_fletch_cli_contract.mjs ('--wasm='+(Join-Path $outputPath 'toolkit/bisect_wasm.wasm')) | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Final package actual CLI FLETCH comparison failed.'}

    $fletchBrowser=$null

    if($BrowserChecks){

        $fletchBrowser=node scripts/web/test_fletch_cli_browser.mjs ('--reference='+$fletchPackage.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package FLETCH source/cache browser failed.'}

    }

    @{status='passed';node=$fletchPackage;browser=$fletchBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/fletch-cli-root.json -Encoding utf8

    $faultArguments=@('scripts/web/test_cffi_fault_recovery.mjs',('--toolkit='+(Join-Path $outputPath 'toolkit')),'--receipt=target/cffi-integration/fault-recovery-root.json')

    if($BrowserChecks){$faultArguments+='--browser'}

    node @faultArguments

    if($LASTEXITCODE -ne 0){throw 'Controlled C trap/allocation recovery checks failed.'}

    $reportCArguments=@('scripts/web/check_cffi_report_workflow.mjs',('--toolkit='+(Join-Path $outputPath 'toolkit')),('--native='+(Join-Path $repository 'target/release/bisect.exe')),('--native-reference='+(Join-Path $repository 'target/release/examples/state_signal_preset_reference.exe')))

    if($BrowserChecks){$reportCArguments+='--browser'}

    $reportCCheck=& node @reportCArguments | ConvertFrom-Json

    if($LASTEXITCODE -ne 0){throw 'Original State report-source C workflow failed.'}

    $reportCCheck | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath target/cffi-integration/report-source-root.json -Encoding utf8

    if($BrowserChecks){

        node scripts/web/test_ceremony_cli_browser.mjs ('--contract='+$ceremonyPackage.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package

        if($LASTEXITCODE -ne 0){throw 'Final package ceremony verification/prefix UI failed.'}

        $ceremonySourcesBrowser=node scripts/web/test_ceremony_source_start_browser.mjs ('--reference='+$ceremonySourcesPackage.root) ('--toolkit='+(Join-Path $outputPath 'toolkit')) --require-package | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Final package original-profile ceremony Start UI failed.'}

        $ceremonySourcesBrowser | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath target/cffi-integration/ceremony-source-start-browser-root.json -Encoding utf8

        node scripts/web/test_explicit_c_state_batch_browser.mjs (Join-Path $outputPath 'toolkit')

        if($LASTEXITCODE -ne 0){throw 'Explicit C state-batch browser checks failed.'}

        node scripts/web/test_cffi_engine_browser.mjs (Join-Path $outputPath 'toolkit')

        if($LASTEXITCODE -ne 0){throw 'C workbench browser checks failed.'}

        node scripts/web/test_cffi_worker_recovery_browser.mjs (Join-Path $outputPath 'toolkit') --require-package --receipt=target/cffi-integration/worker-recovery-root.json

        if($LASTEXITCODE -ne 0){throw 'C workbench explicit retry/recovery checks failed.'}

    }

    # Prepared-state checks exercise portable Rust paths in this exact package.
    cargo build -p bisect-wasm --release --example execute_tool_request --example execute_tool_sequence --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Prepared-state portable native oracle build failed.' }
    # Frozen native assignment references and source-refusal checks on this exact package.
    $replayCargoMetadata = cargo metadata --format-version 1 --no-deps --locked --offline | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Could not locate the native replay reference executable.' }
    $replayHostName = if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) { 'execute_tool_request.exe' } else { 'execute_tool_request' }
    $replayHost = Join-Path $replayCargoMetadata.target_directory ('release/examples/' + $replayHostName)
    $replayOutput = Join-Path 'target/native-recorded-replay-package' ([Guid]::NewGuid().ToString('N'))
    $replayArguments = @('--package', (Join-Path $outputPath 'toolkit'), '--host', $replayHost, '--output', $replayOutput)
    $replayArguments += '--c-sidecar'
    if ($BrowserChecks) { $replayArguments += '--browser' }
    python scripts/web/check_native_recorded_replay_package.py @replayArguments
    if ($LASTEXITCODE -ne 0) { throw 'Packaged native recorded replay regression failed.' }
    # Exact native map PNGs and explicit fine-unit browser workflow on this package.
    $fineHostName = if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) { 'execute_tool_sequence.exe' } else { 'execute_tool_sequence' }
    $fineCliName = if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) { 'bisect.exe' } else { 'bisect' }
    $fineHost = Join-Path $replayCargoMetadata.target_directory ('release/examples/' + $fineHostName)
    $fineCli = Join-Path $replayCargoMetadata.target_directory ('release/' + $fineCliName)
    $fineOutput = Join-Path 'target/fine-unit-map-package' ([Guid]::NewGuid().ToString('N'))
    $fineArguments = @('--package', (Join-Path $outputPath 'toolkit'), '--host-sequence', $fineHost, '--native-cli', $fineCli, '--output', $fineOutput)
    $fineArguments += '--c-sidecar'
    if ($BrowserChecks) { $fineArguments += '--browser' }
    python scripts/web/check_fine_unit_map_package.py @fineArguments
    if ($LASTEXITCODE -ne 0) { throw 'Packaged fine-unit native map regression failed.' }
    cargo test -p bisect-wasm --release --lib native_fine_context --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Original fine context preservation regressions failed.' }
    cargo build -p bisect-cli --release --example native_map_reference --example fine_context_map_reference --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native fine export map reference build failed.' }
    $replayExportSuffix = if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) { '.exe' } else { '' }
    $replayExportMap = Join-Path $replayCargoMetadata.target_directory ('release/examples/fine_context_map_reference' + $replayExportSuffix)
    $replayExportAdapter = Join-Path $replayCargoMetadata.target_directory ('release/examples/native_map_reference' + $replayExportSuffix)
    $replayExportOutput = Join-Path 'target/native-replay-export-package' ([Guid]::NewGuid().ToString('N'))
    $replayExportArgs = @('--package', (Join-Path $outputPath 'toolkit'), '--host-sequence', $fineHost, '--native-map-reference', $replayExportMap, '--native-map-adapter', $replayExportAdapter, '--output', $replayExportOutput)
    $replayExportArgs += '--c-sidecar'
    if ($BrowserChecks) { $replayExportArgs += '--browser' }
    python scripts/web/check_native_replay_export_package.py @replayExportArgs
    if ($LASTEXITCODE -ne 0) { throw 'Original native fine export/handoff package regression failed.' }

    cargo test -p bisect-wasm --release --lib native_ordinary_context --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Original ordinary context preservation regressions failed.' }
    $ordinaryContextOutput = Join-Path 'target/native-ordinary-context-package' ([Guid]::NewGuid().ToString('N'))
    $ordinaryContextArgs = @('--package', (Join-Path $outputPath 'toolkit'), '--host', $replayHost, '--native-map-reference', $replayExportMap, '--output', $ordinaryContextOutput)
    $ordinaryContextArgs += '--c-sidecar'
    if ($BrowserChecks) { $ordinaryContextArgs += '--browser' }
    python scripts/web/check_native_ordinary_context_package.py @ordinaryContextArgs
    if ($LASTEXITCODE -ne 0) { throw 'Original native ordinary context/handoff package regression failed.' }

    cargo test -p bisect-wasm --release --lib native_imported_context --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native unit/source identity regressions failed.' }
    $nativeIdentityOutput = Join-Path 'target/native-unit-identity-package' ([Guid]::NewGuid().ToString('N'))
    $nativeIdentityArgs = @('--package', (Join-Path $outputPath 'toolkit'), '--host', $replayHost, '--host-sequence', $fineHost, '--native-map-reference', $replayExportMap, '--output', $nativeIdentityOutput)
    $nativeIdentityArgs += '--c-sidecar'
    if ($BrowserChecks) { $nativeIdentityArgs += '--browser' }
    python scripts/web/check_native_unit_identity_package.py @nativeIdentityArgs
    if ($LASTEXITCODE -ne 0) { throw 'Native original unit identity/source-index package regression failed.' }

    $freshStateSite = (Join-Path $outputPath 'toolkit')
    if ($BrowserChecks) {
        cargo build --release -p rhist-cli -p rcount-cli --locked --offline -j 1
        if ($LASTEXITCODE -ne 0) { throw 'Native history/count verification references failed to build.' }
        node scripts/web/test_package_verification_lifecycle_browser.mjs --site $freshStateSite --output target/cffi-integration/package-verification-lifecycle-browser --native-rhist (Join-Path 'target/release' ('rhist'+$nativeAssetSuffix)) --native-rcount (Join-Path 'target/release' ('rcount'+$nativeAssetSuffix))
        if ($LASTEXITCODE -ne 0) { throw 'C package history/count verification lifecycle browser checks failed.' }
    }
    $freshStateWasm = Join-Path $freshStateSite 'bisect_wasm.wasm'
    $freshStateOutput = Join-Path 'target/fresh-native-state-package' ([Guid]::NewGuid().ToString('N'))
    node scripts/web/test_fresh_native_state.mjs --wasm $freshStateWasm --host $replayHost --output $freshStateOutput
    if ($LASTEXITCODE -ne 0) { throw 'Fresh selected-source native State runtime regression failed.' }
    node scripts/web/test_fresh_native_state_project.mjs --wasm $freshStateWasm --output $freshStateOutput
    if ($LASTEXITCODE -ne 0) { throw 'Fresh State project/archive persistence regression failed.' }
    node scripts/web/test_fresh_native_state_retained_view.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Bounded fresh saved-run viewer regression failed.' }
    $freshMultiscaleArgs = @('--wasm', $freshStateWasm, '--host', $replayHost, '--output', $freshStateOutput)
    $freshMultiscaleArgs += @('--c-sidecar', (Join-Path $freshStateSite 'c-metis'))
    node scripts/web/test_fresh_native_multiscale.mjs @freshMultiscaleArgs
    if ($LASTEXITCODE -ne 0) { throw 'Fresh native multiscale raw-source/native-context regression failed.' }
    $freshBackendArgs = @('--wasm', $freshStateWasm, '--native-cli', $coreCliPath, '--output', $freshStateOutput)
    $freshBackendArgs += @('--c-sidecar', (Join-Path $freshStateSite 'c-metis'))
    node scripts/web/test_fresh_native_backend_profile.mjs @freshBackendArgs
    if ($LASTEXITCODE -ne 0) { throw 'Fresh native backend profile/dispatch regression failed.' }


    if ($BrowserChecks) {
        node scripts/web/test_native_assignment_compare_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath
        if ($LASTEXITCODE -ne 0) { throw 'Native assignment Compare CLI/browser byte regression failed.' }
        node scripts/web/test_native_assignment_compare_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath --block-scale-fixture
        if ($LASTEXITCODE -ne 0) { throw 'Native assignment Compare scale and browser size-bound regression failed.' }
        node scripts/web/test_native_comparison_narrative_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath
        if ($LASTEXITCODE -ne 0) { throw 'Native comparison narrative CLI/browser byte regression failed.' }
        node scripts/web/test_civic_desktop_folder_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath
        if ($LASTEXITCODE -ne 0) { throw 'Civic desktop folder browser gate failed.' }
        node scripts/web/test_native_civic_conflicts_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath
        if ($LASTEXITCODE -ne 0) { throw 'Native civic Conflicts browser gate failed.' }
        node scripts/web/test_native_candidate_package_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath
        if ($LASTEXITCODE -ne 0) { throw 'Native candidate package browser gate failed.' }
        node scripts/web/test_native_candidate_workspace_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath
        if ($LASTEXITCODE -ne 0) { throw 'Native candidate workspace browser gate failed.' }
        node scripts/web/test_large_workbench_archive_preference.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Large workbench archive preference gate failed.' }
        node scripts/web/test_fresh_native_absence_sources.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Explicit native absence source boundaries failed.' }
        node scripts/web/test_fresh_native_absence_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath
        if ($LASTEXITCODE -ne 0) { throw 'Explicit native absence CLI/browser/persistence regression failed.' }
        node scripts/web/test_fresh_native_auxiliary_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $coreCliPath
        if ($LASTEXITCODE -ne 0) { throw 'Fresh selected native auxiliary CLI/browser/persistence regression failed.' }
        node scripts/web/test_fresh_native_backend_profile_browser.mjs --site $freshStateSite --output $freshStateOutput --c-profile
        if ($LASTEXITCODE -ne 0) { throw 'Fresh native backend profile/browser regression failed.' }
        node scripts/web/test_fresh_native_state_browser.mjs --site $freshStateSite --output $freshStateOutput
        if ($LASTEXITCODE -ne 0) { throw 'Fresh native State browser review/save/open/map regression failed.' }
        node scripts/web/test_fresh_native_state_project_browser.mjs --site $freshStateSite --output $freshStateOutput
        if ($LASTEXITCODE -ne 0) { throw 'Fresh State complete-project/archive/autosave browser regression failed.' }
        $freshMultiscaleBrowserArgs = @('--site', $freshStateSite, '--output', $freshStateOutput)
        $freshMultiscaleBrowserArgs += '--c-profile'
        node scripts/web/test_fresh_native_multiscale_browser.mjs @freshMultiscaleBrowserArgs
        if ($LASTEXITCODE -ne 0) { throw 'Fresh native multiscale project/map/browser regression failed.' }
        node scripts/web/test_fresh_native_followups_browser.mjs --site $freshStateSite --output $freshStateOutput --host $replayHost --c-profile
        if ($LASTEXITCODE -ne 0) { throw 'Fresh native audit/analysis/improvement handoff browser regression failed.' }


    }





    $preparedStateOutput = Join-Path 'target/prepared-state-package' ([Guid]::NewGuid().ToString('N'))
    $preparedStateArguments = @('--package', (Join-Path $outputPath 'toolkit'), '--output', $preparedStateOutput)
    if ($BrowserChecks) { $preparedStateArguments += '--browser' }
    python scripts/web/check_prepared_state_package.py @preparedStateArguments
    if ($LASTEXITCODE -ne 0) { throw 'Packaged prepared-state workflow checks failed.' }
    # Keep acceptance inputs outside the immutable/offline package; adding a

    # receipt to that package after its manifest would invalidate its closure.

    python -c "import hashlib,json,sys; from pathlib import Path; root=Path(sys.argv[1]); evidence=Path('target/cffi-integration'); pin=lambda p:{'path':str(p).replace(chr(92),'/'),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}; names=['check_cffi_report_workflow.mjs','prepare_population_source_native_reference.py','build_cffi_preview.ps1','check_cffi_legacy_gates.ps1','test_cffi_engine.mjs','test_cffi_engine_browser.mjs','test_cffi_fault_recovery.mjs','fixtures/cffi-fault-cases.mjs','test_cffi_worker_recovery_browser.mjs','test_ceremony_cli_contract.mjs','test_ceremony_cli_browser.mjs','test_ceremony_source_start.mjs','test_ceremony_source_start_browser.mjs','test_fletch_cli_contract.mjs','test_fletch_cli_browser.mjs','test_native_format_cli.mjs','test_native_format_cli_browser.mjs','test_config_cli_contract.mjs','test_config_cli_browser.mjs','test_doctor_workspace_boundaries.mjs','test_doctor_workspace_cli.mjs','test_doctor_workspace_browser.mjs','test_tutorial_cli_contract.mjs','test_tutorial_cli_browser.mjs','test_manifest_cli_contract.mjs','test_manifest_cli_browser.mjs','test_plan_manifest_width.mjs','test_plan_manifest_width_browser.mjs','test_manifest_reader_width.mjs','test_manifest_reader_width_browser.mjs','test_large_rplan_text.mjs','test_laboratory_practitioner_archive.mjs','test_laboratory_practitioner_archive_browser.mjs','test_doctor_command_wasm.mjs','test_doctor_command_browser.mjs','test_policy_fallback_wasm.mjs','test_policy_fallback_browser.mjs','test_policy_fallback_inputs.mjs','test_policy_cli_contract.mjs','test_policy_cli_browser.mjs','test_config_publication.mjs','test_config_publication_browser.mjs','test_config_publication_windows.mjs','fixtures/hold-read-share.ps1']; refs=[pin(Path('scripts/web',n)) for n in names]; results=[pin(evidence/'engine-evidence.json'),pin(evidence/'report-source-root.json'),pin(evidence/'fault-recovery-root.json'),pin(evidence/'ceremony-contract-root.json'),pin(evidence/'ceremony-source-start-root.json'),pin(evidence/'fletch-cli-root.json'),pin(evidence/'native-format-cli-root.json'),pin(evidence/'config-cli-root.json'),pin(evidence/'config-publication-root.json'),pin(evidence/'policy-cli-root.json'),pin(evidence/'policy-fallback-root.json'),pin(evidence/'doctor-command-root.json')]+[pin(Path('target/cffi-legacy-gates',v,'build.json')) for v in ['default','c-sidecar']]; browser=sys.argv[2]=='True'; results+=([pin(evidence/'browser/evidence.json'),pin(evidence/'worker-recovery-root.json'),pin(evidence/'ceremony-source-start-browser-root.json')] if browser else []); record={'schema':'bisect-cffi-preview-checks-v1','package':str(root),'offline_manifest':pin(root/'offline-manifest.json'),'offline_version':json.loads((root/'offline-manifest.json').read_text(encoding='utf-8'))['version'],'source_files':refs,'results':results,'browser_status':'passed' if browser else 'not-run','full_parity':False,'deployed':False}; (evidence/'package-evidence.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')" $outputPath ([string][bool]$BrowserChecks)

    if($LASTEXITCODE -ne 0){throw 'Could not retain final package check evidence.'}

    Write-Host "Experimental C-capable preview: $outputPath. Broader parity/release acceptance remains open; nothing deployed."

} finally {Pop-Location}

