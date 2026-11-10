param([switch]$BothBuilds,[switch]$BrowserChecks,[string]$EngineRecord,[string]$BuildApplicationEngineRecord)

$ErrorActionPreference='Stop'

$repository=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path

$previousWasmPath=$env:BISECT_TEST_WASM

$previousReportProfile=$env:BISECT_REPORT_PROFILE

Push-Location $repository

try {

    node scripts/web/test_state_batch_zip.mjs

    if($LASTEXITCODE -ne 0){throw 'State batch ZIP format/resource guards failed.'}

    # Native outputs and owned Chrome profiles need headroom beyond their final evidence size.

    python -c "import shutil,sys; free=shutil.disk_usage('.').free; required=2*1024**3; print('Legacy gate capacity: '+str(free)+' bytes free; '+str(required)+' required'); sys.exit(0 if free>=required else 1)"

    if($LASTEXITCODE -ne 0){throw 'Legacy gate capacity refusal before builds or receipt replacement: keep at least 2 GiB free.'}

    if($EngineRecord -and $BuildApplicationEngineRecord){throw 'Choose exactly one owned engine record.'}

    if($BuildApplicationEngineRecord){

        $verifiedBuildInput=python scripts/web/validate_label_build_application_engines.py --record $BuildApplicationEngineRecord

        if($LASTEXITCODE -ne 0){throw 'Build application engine evidence failed validation before receipt replacement.'}

    }

    if($EngineRecord){

        $verifiedEngineInput=python scripts/web/validate_label_rename_engine_record.py --record $EngineRecord

        if($LASTEXITCODE -ne 0){throw 'Supplied engine evidence failed validation before receipt replacement.'}

    }

    $gateRoot=Join-Path $repository 'target/cffi-legacy-gates'

    New-Item -ItemType Directory -Path $gateRoot -Force | Out-Null

    $gateVariants=if($BothBuilds){@('default','c-sidecar')}else{@('c-sidecar')}

    $gateRunId=[guid]::NewGuid().ToString()

    # A failed new attempt must never leave an old passing receipt appearing

    # current. Each requested variant starts explicitly not-run.

    foreach($gateVariant in $gateVariants){

        $variantRoot=Join-Path $gateRoot $gateVariant

        New-Item -ItemType Directory -Path $variantRoot -Force | Out-Null

        @{schema='bisect-cffi-legacy-gates-v1';profile=$gateVariant;run_id=$gateRunId;status='not-run'} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $variantRoot 'build.json') -Encoding utf8

        @{status='not-run';root=$null} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $variantRoot 'report-pdf-root.json') -Encoding utf8

    }

    cargo build -p bisect-wasm --release --example execute_tool_request --example execute_request --locked --offline -j 1

    if($LASTEXITCODE -ne 0){throw 'Native host adapters failed to build.'}

    cargo build -p bisect-cli --no-default-features --example config_reference --example config_template_reference --example config_seed_reference --locked --offline -j 1

    if($LASTEXITCODE -ne 0){throw 'Native configuration references failed to build.'}

    cargo build -p bisect-cli --release --no-default-features --example config_seed_reference --example state_signal_preset_reference --locked --offline -j 1

    if($LASTEXITCODE -ne 0){throw 'Portable native kernel references failed to build.'}

    cargo test -p rgraph-core --release --test prepared_connectivity --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Prepared graph connectivity and query-history regression failed.'}

    cargo build -p bisect-cli --bin bisect --example label_summary_application_reference --release --no-default-features --locked --offline -j 1

    if($LASTEXITCODE -ne 0){throw 'Native Verify dry-run reference build failed.'}

    python scripts/web/test_native_state_inert_flags.py
    if($LASTEXITCODE -ne 0){throw 'Native parsed-unused State flag effects drifted.'}

    cargo build -p bisect-cli --no-default-features --example fetch_plan_reference --example fletch_sources_reference --example fetch_archive_reference --locked --offline -j 1

    if($LASTEXITCODE -ne 0){throw 'Native Fetch/FLETCH references failed to build.'}

    $stateOracle=Join-Path $gateRoot 'state-core-reference.exe'

    Copy-Item -LiteralPath target/release/examples/state_signal_preset_reference.exe -Destination $stateOracle

    foreach($gateVariant in $gateVariants){

        $arguments=@('build','-p','bisect-wasm','--release','--target','wasm32-unknown-unknown','--locked','--offline','-j','1')

        if($gateVariant -eq 'c-sidecar'){$arguments+=@('--features','c-metis-sidecar')}

        $nativeSummaryBrowser=$null

        $labelRenameBrowser=$null

        $labelImportBrowser=$null

        $buildApplicationBrowser=$null

        $summaryApplicationBrowser=$null

        $env:BISECT_TEST_WASM=Join-Path $gateRoot "$gateVariant.wasm"

        if($BuildApplicationEngineRecord){

            $engineInput=python scripts/web/validate_label_build_application_engines.py --record $BuildApplicationEngineRecord --profile $gateVariant --destination $env:BISECT_TEST_WASM

            if($LASTEXITCODE -ne 0){throw 'Could not validate/copy the supplied build application engine.'}

        }elseif($EngineRecord){

            $engineInput=python scripts/web/validate_label_rename_engine_record.py --record $EngineRecord --profile $gateVariant --destination $env:BISECT_TEST_WASM

            if($LASTEXITCODE -ne 0){throw 'Could not validate/copy the supplied owned engine.'}

        }else{

            & cargo @arguments

            if($LASTEXITCODE -ne 0){throw "WASM $gateVariant build failed."}

            Copy-Item -LiteralPath target/wasm32-unknown-unknown/release/bisect_wasm.wasm -Destination $env:BISECT_TEST_WASM

            $engineInput=@{mode='fresh-cargo-build';recompiled_in_this_runner=$true} | ConvertTo-Json -Compress

        }

        $engineInput | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/engine-input.json')) -Encoding utf8

        node scripts/web/test_ceremony_wasm.mjs

        if($LASTEXITCODE -ne 0){throw "$gateVariant shared ceremony adapter/timestamp checks failed."}

        $ceremonyCli=node scripts/web/test_ceremony_cli_contract.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual CLI ceremony verification failed."}

        $ceremonyBrowser=$null

        if($BrowserChecks){

            $ceremonyBrowser=node scripts/web/test_ceremony_cli_browser.mjs ('--contract='+$ceremonyCli.root) ('--wasm='+$env:BISECT_TEST_WASM) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant ceremony browser verification/prefix distinction failed."}

        }

        @{status='passed';node=$ceremonyCli;browser=$ceremonyBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/ceremony-contract-roots.json')) -Encoding utf8

        $ceremonySources=node scripts/web/test_ceremony_source_start.mjs ('--contract='+$ceremonyCli.root) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant original-profile ceremony Start/native comparison failed."}

        $ceremonySourcesBrowser=$null

        if($BrowserChecks){

            $ceremonySourcesBrowser=node scripts/web/test_ceremony_source_start_browser.mjs ('--reference='+$ceremonySources.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant original-profile ceremony Start browser failed."}

        }

        @{status='passed';node=$ceremonySources;browser=$ceremonySourcesBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/ceremony-source-start-roots.json')) -Encoding utf8

        $configCli=node scripts/web/test_config_cli_contract.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual Config CLI comparison failed."}

        $configBrowser=$null

        if($BrowserChecks){

            $configBrowser=node scripts/web/test_config_cli_browser.mjs ('--reference='+$configCli.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant Config create/validate browser comparison failed."}

        }

        @{status='passed';node=$configCli;browser=$configBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/config-cli-roots.json')) -Encoding utf8

        $configPublicationWindows=@{status='not-run';reason='Real Windows sharing locks require Windows.'}
        if ($IsWindows) {
            $configPublicationWindows=node scripts/web/test_config_publication_windows.mjs ('--reference='+$configCli.root) ('--profile='+$gateVariant) | ConvertFrom-Json
            if($LASTEXITCODE -ne 0){throw 'Real Windows config publication sharing-lock checks failed.'}
        }

        $configPublication=node scripts/web/test_config_publication.mjs ('--reference='+$configCli.root) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant config publication/native byte checks failed."}

        $configPublicationBrowser=$null

        if($BrowserChecks){

            $configPublicationBrowser=node scripts/web/test_config_publication_browser.mjs ('--reference='+$configCli.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant config publication workbench failed."}

        }

        @{status='passed';node=$configPublication;windows_host=$configPublicationWindows;browser=$configPublicationBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/config-publication-roots.json')) -Encoding utf8

        $policyCli=node scripts/web/test_policy_cli_contract.mjs --native=target/release/bisect.exe | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual Policy CLI comparison failed."}

        $policyBrowser=$null

        if($BrowserChecks){

            $policyBrowser=node scripts/web/test_policy_cli_browser.mjs ('--reference='+$policyCli.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant original Policy file-input browser checks failed."}

        }

        @{status='passed';node=$policyCli;browser=$policyBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/policy-cli-roots.json')) -Encoding utf8

        $policyFallback=node scripts/web/test_policy_fallback_wasm.mjs ('--wasm='+$env:BISECT_TEST_WASM) ('--reference='+$policyCli.root) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant native Policy fallback comparison failed."}

        $policyFallbackBrowser=$null

        if($BrowserChecks){

            $policyFallbackBrowser=node scripts/web/test_policy_fallback_browser.mjs ('--reference='+$policyFallback.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant Policy fallback workbench failed."}

        }

        @{status='passed';node=$policyFallback;browser=$policyFallbackBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/policy-fallback-roots.json')) -Encoding utf8

        $doctorCommand=node scripts/web/test_doctor_command_wasm.mjs --native=target/release/bisect.exe ('--wasm='+$env:BISECT_TEST_WASM) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual Doctor command comparison failed."}

        $doctorCommandBrowser=$null

        if($BrowserChecks){

            $doctorCommandBrowser=node scripts/web/test_doctor_command_browser.mjs ('--reference='+$doctorCommand.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant Doctor command workbench failed."}

        }

        @{status='passed';node=$doctorCommand;browser=$doctorCommandBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/doctor-command-roots.json')) -Encoding utf8

        $manifestWidth=node scripts/web/test_plan_manifest_width.mjs --native=target/release/bisect.exe ('--wasm='+$env:BISECT_TEST_WASM) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual Read-only manifest width comparison failed."}

        $manifestWidthBrowser=$null

        if($BrowserChecks){

            $manifestWidthBrowser=node scripts/web/test_plan_manifest_width_browser.mjs ('--reference='+$manifestWidth.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant Read-only manifest width workbench failed."}

        }

        @{status='passed';node=$manifestWidth;browser=$manifestWidthBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/plan-manifest-width-roots.json')) -Encoding utf8

        $largePlanText=node scripts/web/test_large_rplan_text.mjs --native=target/release/bisect.exe ('--wasm='+$env:BISECT_TEST_WASM) | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw "$gateVariant large RPLAN typed/native boundary comparison failed."}
        $largePlanText | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/large-rplan-text-root.json')) -Encoding utf8
        $manifestReader=node scripts/web/test_manifest_reader_width.mjs --native=target/release/examples/execute_tool_request.exe --cli=target/release/bisect.exe ('--wasm='+$env:BISECT_TEST_WASM) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual Read-only manifest readers comparison failed."}

        $manifestReaderBrowser=$null

        if($BrowserChecks){

            $manifestReaderBrowser=node scripts/web/test_manifest_reader_width_browser.mjs ('--reference='+$manifestReader.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant Read-only manifest readers workbench failed."}

        }

        @{status='passed';node=$manifestReader;browser=$manifestReaderBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/manifest-reader-width-roots.json')) -Encoding utf8

        $manifestCli=node scripts/web/test_manifest_cli_contract.mjs --native=target/release/bisect.exe ('--wasm='+$env:BISECT_TEST_WASM) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual Manifest CLI comparison failed."}

        $manifestCliBrowser=$null

        if($BrowserChecks){

            $manifestCliBrowser=node scripts/web/test_manifest_cli_browser.mjs ('--reference='+$manifestCli.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant Manifest CLI workbench failed."}

        }

        @{status='passed';node=$manifestCli;browser=$manifestCliBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/manifest-cli-roots.json')) -Encoding utf8

        $tutorialCli=node scripts/web/test_tutorial_cli_contract.mjs --native=target/release/bisect.exe ('--wasm='+$env:BISECT_TEST_WASM) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual Tutorial CLI comparison failed."}

        $tutorialCliBrowser=$null

        if($BrowserChecks){

            $tutorialCliBrowser=node scripts/web/test_tutorial_cli_browser.mjs ('--reference='+$tutorialCli.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant Tutorial CLI workbench failed."}

        }

        @{status='passed';node=$tutorialCli;browser=$tutorialCliBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/tutorial-cli-roots.json')) -Encoding utf8

        $doctorWorkspace=node scripts/web/test_doctor_workspace_cli.mjs --native=target/release/bisect.exe ('--wasm='+$env:BISECT_TEST_WASM) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual Native Doctor workspace comparison failed."}

        $doctorWorkspaceBoundaries=node scripts/web/test_doctor_workspace_boundaries.mjs ('--reference='+$doctorWorkspace.root) | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Native Doctor snapshot boundary checks failed.'}

        $doctorWorkspaceBrowser=$null

        if($BrowserChecks){

            $doctorWorkspaceBrowser=node scripts/web/test_doctor_workspace_browser.mjs ('--reference='+$doctorWorkspace.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant Native Doctor workspace workbench failed."}

        }

        @{status='passed';node=$doctorWorkspace;boundaries=$doctorWorkspaceBoundaries;browser=$doctorWorkspaceBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/doctor-workspace-roots.json')) -Encoding utf8

        $formatCli=node scripts/web/test_native_format_cli.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual native format CLI comparison failed."}

        $formatBrowser=$null

        if($BrowserChecks){

            $formatBrowser=node scripts/web/test_native_format_cli_browser.mjs ('--reference='+$formatCli.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant native format UI/source-byte comparison failed."}

        }

        @{status='passed';node=$formatCli;browser=$formatBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/native-format-cli-roots.json')) -Encoding utf8

        $fletchCli=node scripts/web/test_fletch_cli_contract.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant actual CLI FLETCH report/gate comparison failed."}

        $fletchBrowser=$null

        if($BrowserChecks){

            $fletchBrowser=node scripts/web/test_fletch_cli_browser.mjs ('--reference='+$fletchCli.root) --toolkit=web/lab | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant FLETCH source/cache browser comparison failed."}

        }

        @{status='passed';node=$fletchCli;browser=$fletchBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/fletch-cli-roots.json')) -Encoding utf8

        foreach($test in @('test_config_inspection_wasm.mjs','test_config_validation_wasm.mjs','test_config_template_wasm.mjs','test_config_run_wasm.mjs','test_config_no_metis_oracle.mjs','test_config_one_district_oracle.mjs')){

            & node (Join-Path scripts/web $test)

            if($LASTEXITCODE -ne 0){throw "$gateVariant $test failed."}

        }

        node scripts/web/test_native_state_run.mjs --native-oracle $stateOracle

        if($LASTEXITCODE -ne 0){throw "$gateVariant native State preparation/execution failed."}

        node scripts/web/test_state_identity_inspection.mjs

        if($LASTEXITCODE -ne 0){throw "$gateVariant native State identity inspection/guard failed."}

        $stateNameWorkflow=node scripts/web/test_state_name_workflow.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant selected native State name workflow failed."}

        @{status='passed';node=$stateNameWorkflow} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/state-name-workflow-root.json')) -Encoding utf8

        $reportArguments=@('scripts/web/check_native_report_regression.mjs',('--profile='+$gateVariant))

        if($BrowserChecks){$reportArguments+='--browser'}

        $reportSources=& node @reportArguments | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant report-source native/API/browser integration failed."}

        @{status='passed';run=$reportSources} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/report-source-roots.json')) -Encoding utf8

        cargo test -p bisect-report --release --no-default-features --lib label_summary --locked --offline -j 1

        if($LASTEXITCODE -ne 0){throw 'Shared native label summary/security tests failed.'}

        cargo test -p bisect-report --release --no-default-features --test label_summary_security --locked --offline -j 1

        if($LASTEXITCODE -ne 0){throw 'Native summary metadata escaping tests failed.'}

        node scripts/web/test_report_print.mjs

        if($LASTEXITCODE -ne 0){throw "$gateVariant report print byte/settings guards failed."}

        if($BrowserChecks){

            $nativeSummaryBrowser=node scripts/web/test_native_label_summary_browser.mjs | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw 'Native label summary browser checks failed.'}

            $buildApplicationBrowser=node scripts/web/test_label_build_application_browser.mjs | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw 'Reviewed label build actual-worker browser failed.'}

            $summaryApplicationBrowser=node scripts/web/test_native_summary_application_browser.mjs | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw 'Ordered summary application browser failed.'}

            $labelImportBrowser=node scripts/web/test_label_import_application_browser.mjs | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw 'Reviewed import browser checks failed.'}

            $labelRenameBrowser=node scripts/web/test_label_rename_application_browser.mjs | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw 'Reviewed label rename browser checks failed.'}

            node scripts/web/test_count_transcript_installation_browser.mjs

            if($LASTEXITCODE -ne 0){throw 'RCOUNT transcript browser installation checks failed.'}

            $reportPdfCheck=node scripts/web/test_report_pdf_browser.mjs --json | ConvertFrom-Json

            if($LASTEXITCODE -ne 0){throw "$gateVariant actual report PDF/browser checks failed."}

            @{status='passed';root=$reportPdfCheck.root;wasm_sha256=$reportPdfCheck.wasm_sha256} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/report-pdf-root.json')) -Encoding utf8

        }

        node scripts/web/test_native_output_verification.mjs

        if($LASTEXITCODE -ne 0){throw "$gateVariant composed native output Verify failed."}

        cargo build --release -p rcount-cli --locked --offline -j 1

        if($LASTEXITCODE -ne 0){throw 'Native RCOUNT transcript reference failed to build.'}

        $nativeSummaryDirect=node scripts/web/test_native_label_summary_wasm.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Native label summary files/CLI checks failed.'}

        $nativeSummaryBatch=node scripts/web/test_native_label_summary_batch.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Native label summary ordering/output checks failed.'}

        @{status='passed';direct=$nativeSummaryDirect;batch=$nativeSummaryBatch;browser=$nativeSummaryBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/native-summary-roots.json')) -Encoding utf8

        cargo test --release -p bisect-cli --no-default-features --lib analyze_label::tests::test_report_clock_ --locked --offline -j 1 -- --test-threads=1

        if($LASTEXITCODE -ne 0){throw 'Native report clock ordering/provenance regressions failed.'}

        cargo test --release -p bisect-report --no-default-features --lib label_rename --locked --offline -j 1

        if($LASTEXITCODE -ne 0){throw 'Shared native rename safety tests failed.'}

        $nativeAliasCheck=python scripts/web/test_native_case_alias_cli.py | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Native case-only rename source preservation failed.'}

        $buildApplicationCheck=node scripts/web/test_label_build_application.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Reviewed label build/native CLI application failed.'}

        $buildNativeFailures=node scripts/web/test_label_build_native_failures.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Native label build failure prefixes failed.'}

        node scripts/web/test_label_build_application_ui.mjs

        if($LASTEXITCODE -ne 0){throw 'Reviewed label build DOM controller failed.'}

        @{status='passed';node=$buildApplicationCheck;native_failures=$buildNativeFailures;browser=$buildApplicationBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/label-build-application-roots.json')) -Encoding utf8

        $summaryApplicationCheck=node scripts/web/test_native_summary_application.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Ordered summary application/native reference failed.'}

        @{status='passed';node=$summaryApplicationCheck;browser=$summaryApplicationBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/summary-application-roots.json')) -Encoding utf8

        $labelImportCheck=node scripts/web/test_label_import_application.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Reviewed import/native CLI checks failed.'}

        @{status='passed';node=$labelImportCheck;browser=$labelImportBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/label-import-roots.json')) -Encoding utf8

        $labelRenameCheck=node scripts/web/test_label_rename_application.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw 'Reviewed label rename/native CLI checks failed.'}

        @{status='passed';node=$labelRenameCheck;native_case_alias_cli=$nativeAliasCheck;browser=$labelRenameBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/label-rename-roots.json')) -Encoding utf8

        node scripts/web/test_count_transcript_installation.mjs

        if($LASTEXITCODE -ne 0){throw 'RCOUNT transcript/native CLI installation checks failed.'}

        $labelRepositoryCheck=node scripts/web/test_label_repository_installation.mjs --json | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant label repository/native CLI checks failed."}

        @{root=$labelRepositoryCheck.root;wasm_sha256=$labelRepositoryCheck.wasm_sha256} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/label-repository-root.json')) -Encoding utf8

        node scripts/web/test_native_workspace_continuation.mjs

        if($LASTEXITCODE -ne 0){throw "$gateVariant saved native workspace continuation failed."}

        node scripts/web/test_fetch_source_coverage.mjs

        if($LASTEXITCODE -ne 0){throw "$gateVariant Fetch source coverage failed."}

        $archiveCheck=node scripts/web/test_fetch_archive_acquisition.mjs | ConvertFrom-Json

        if($LASTEXITCODE -ne 0){throw "$gateVariant Fetch archive/native checks failed."}

        @{root=$archiveCheck.root;wasm_sha256=$archiveCheck.wasm_sha256} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $gateRoot ($gateVariant+'/fetch-archive-root.json')) -Encoding utf8

        python -c "import hashlib,json,shutil,sys; from pathlib import Path; p=Path(sys.argv[1]); profile=sys.argv[2]; roots=['config-validation-wasm','config-template-wasm','config-run-wasm','config-no-metis-oracle','config-one-district-oracle','native-state-run','native-output-verification']; d=p/profile; d.mkdir(exist_ok=True); c=d/'continuation'; c.mkdir(exist_ok=True); [shutil.copy2(Path('target/native-workspace-continuation',n),c/n) for n in ['evidence.json','source.bisect','multi-source.bisect','multi-district-source.bisect','multi-district-completed.json','completed.json','multi-completed.json','verification.json','filesystem-events.json','cases.json']]; files=[]; [files.extend(Path('target',name).glob('*.json')) for name in roots]; [shutil.copy2(f,d/(f.parent.name+'-'+f.name)) for f in files if f.name in ['evidence.json','records.json','capability-comparisons.json','c-capability-preparation.json','native-cases.json','native-inspections.json']]; w=p/(profile+'.wasm'); names=['check_native_report_regression.mjs','test_config_publication_windows.mjs','fixtures/hold-read-share.ps1','prepare_population_source_native_reference.py','fixtures/native-report-reference.mjs','fixtures/observed-project-open.mjs','test_native_report_source_snapshot.mjs','test_native_report_export_binding.mjs','test_native_report_state_workflow.mjs','test_native_report_source_ui_browser.mjs','test_native_report_folder_browser.mjs','stage_state_identity_preview.py','test_state_batch_zip.mjs','test_label_build_application.mjs','test_label_build_application_ui.mjs','test_label_build_application_browser.mjs','test_label_build_native_failures.mjs','validate_label_build_application_engines.py','fixtures/label-build-application-case.mjs','test_label_import_application.mjs','test_label_import_application_browser.mjs','validate_label_rename_engine_record.py','test_native_case_alias_cli.py','test_label_rename_application.mjs','test_label_rename_application_browser.mjs','fixtures/native-output-directory.mjs','test_native_label_summary_wasm.mjs','test_native_label_summary_batch.mjs','test_native_label_summary_browser.mjs','test_report_print.mjs','test_report_pdf_browser.mjs','validate_report_print_pdf.py','fixtures/report-pdf-case.mjs','test_label_repository_installation.mjs','fixtures/label-repository-case.mjs','check_cffi_legacy_gates.ps1','fixtures/config-capability-comparison.mjs','test_config_inspection_wasm.mjs','test_config_validation_wasm.mjs','test_config_template_wasm.mjs','test_config_run_wasm.mjs','test_config_no_metis_oracle.mjs','test_config_one_district_oracle.mjs','test_native_state_run.mjs','test_native_output_verification.mjs','test_native_workspace_continuation.mjs','fixtures/native-workspace-continuation-case.mjs','test_fetch_source_coverage.mjs','test_fetch_archive_acquisition.mjs']; pins=[{'path':'scripts/web/'+n,'sha256':hashlib.sha256(Path('scripts/web',n).read_bytes()).hexdigest()} for n in names]; pins += [{'path':n,'sha256':hashlib.sha256(Path(n).read_bytes()).hexdigest()} for n in ['web/lab/label-build-application.js','web/lab/label-build-application-ui.js','web/lab/build-repository.js','web/lab/label-build-files.js','web/lab/label-build-archive.js','crates/bisect-wasm/src/label_build.rs','crates/bisect-wasm/src/config.rs','crates/bisect-wasm/src/engine_build.rs','crates/bisect-wasm/src/toolkit.rs','crates/bisect-cli/src/runner/mod.rs','crates/bisect-runner/src/balance_retry.rs','web/lab/native-summary-application.js','web/lab/native-summary-application-ui.js','scripts/web/test_native_summary_application.mjs','scripts/web/test_native_summary_application_browser.mjs','crates/bisect-cli/examples/label_summary_application_reference.rs','crates/bisect-cli/src/analyze_label.rs','web/lab/label-import-application.js','web/lab/label-import-application-ui.js','web/lab/label-import-files.js','web/lab/project.js','crates/bisect-cli/src/import_label.rs','crates/bisect-wasm/src/label_import.rs','crates/bisect-report/src/label_rename.rs','crates/bisect-cli/src/label_cmd.rs','crates/bisect-wasm/src/label_rename.rs','web/lab/label-rename-application.js','web/lab/label-rename-application-ui.js','web/lab/label-rename-files.js','web/lab/native-label-summary.js','web/lab/label-report-files.js','web/lab/count-results.js','web/lab/wasm-worker.js','crates/bisect-report/src/label_summary.rs','crates/bisect-report/tests/label_summary_security.rs','crates/bisect-wasm/src/native_label_summary.rs','web/lab/report-print.js','web/lab/commission-preview.js','web/lab/generated-report-files.js','web/lab/commission-preview.html','web/lab/workbench.js','web/lab/workbench.html','web/lab/label-repository-install.js','web/lab/native-bulk-checkpoint-verification.js','web/lab/native-bulk-workspace.js','web/lab/state-run-batch.js','web/lab/state-batch-zip.js','web/lab/workbench-archive.js','web/lab/state-run-years.js','web/lab/fetch-acquisition.js','web/lab/fetch-archive-package.js','web/lab/fetch-source-archive.js','web/lab/native-workspace-source.js','web/lab/fletch-stage-files.js','web/lab/fletch-cache-files.js','web/lab/fletch-source-files.js','scripts/web/test_fletch_cli_contract.mjs','scripts/web/test_fletch_cli_browser.mjs','scripts/web/test_native_format_cli.mjs','scripts/web/test_native_format_cli_browser.mjs','scripts/web/test_config_cli_contract.mjs','scripts/web/test_config_cli_browser.mjs','web/lab/doctor-command.js','web/lab/doctor-command-ui.js','web/lab/plan-doctor-files.js','scripts/web/test_doctor_workspace_boundaries.mjs','scripts/web/test_doctor_workspace_cli.mjs','scripts/web/test_doctor_workspace_browser.mjs','web/lab/doctor-workspace-files.js','web/lab/doctor-workspace-ui.js','crates/bisect-wasm/src/doctor_workspace.rs','scripts/web/test_tutorial_cli_contract.mjs','scripts/web/test_tutorial_cli_browser.mjs','scripts/web/test_manifest_cli_contract.mjs','scripts/web/test_manifest_cli_browser.mjs','scripts/web/test_plan_manifest_width.mjs','scripts/web/test_plan_manifest_width_browser.mjs','scripts/web/test_manifest_reader_width.mjs','scripts/web/test_manifest_reader_width_browser.mjs','scripts/web/test_doctor_command_wasm.mjs','scripts/web/test_doctor_command_browser.mjs','web/lab/policy-fallback.js','web/lab/policy-fallback-ui.js','scripts/web/test_policy_fallback_wasm.mjs','scripts/web/test_policy_fallback_browser.mjs','scripts/web/test_policy_fallback_inputs.mjs','crates/bisect-wasm/src/location_policy.rs','web/lab/policy-files.js','scripts/web/test_policy_cli_contract.mjs','scripts/web/test_policy_cli_browser.mjs','web/lab/config-publication.js','web/lab/config-publication-ui.js','scripts/web/test_config_publication.mjs','scripts/web/test_config_publication_browser.mjs','web/lab/config-files.js','web/lab/source-relay-policy.js','docs/specs/wasm-fetch-source-coverage.json','crates/bisect-cli/examples/fetch_archive_reference.rs','scripts/web/check_cffi_legacy_gates.ps1','scripts/web/test_ceremony_cli_contract.mjs','scripts/web/test_ceremony_cli_browser.mjs','scripts/web/test_ceremony_source_start.mjs','scripts/web/test_ceremony_source_start_browser.mjs','web/lab/ceremony-start-sources.js','web/lab/ceremony-start-sources-ui.js','scripts/web/test_ceremony_wasm.mjs','crates/bisect-wasm/src/ceremony.rs','crates/bisect-wasm/src/toolkit.rs','web/lab/package-files.js','scripts/web/test_state_name_workflow.mjs','crates/bisect-wasm/src/plan_audit_text.rs']]; (d/'build.json').write_text(json.dumps({'schema':'bisect-cffi-legacy-gates-v1','status':'passed','run_id':sys.argv[3],'wasm_sha256':hashlib.sha256(w.read_bytes()).hexdigest(),'native_oracle_sha256':hashlib.sha256((p/'state-core-reference.exe').read_bytes()).hexdigest(),'source_files':pins,'profile':profile,'label_build_application_evidence':json.loads((d/'label-build-application-roots.json').read_text(encoding='utf-8-sig')),'native_summary_application_evidence':json.loads((d/'summary-application-roots.json').read_text(encoding='utf-8-sig')),'label_import_evidence':json.loads((d/'label-import-roots.json').read_text(encoding='utf-8-sig')),'engine_build':json.loads((d/'engine-input.json').read_text(encoding='utf-8-sig')),'label_rename_evidence':json.loads((d/'label-rename-roots.json').read_text(encoding='utf-8-sig')),'native_summary_evidence':json.loads((d/'native-summary-roots.json').read_text(encoding='utf-8-sig')),'report_pdf_evidence':json.loads((d/'report-pdf-root.json').read_text(encoding='utf-8-sig')),'doctor_workspace_evidence':json.loads((d/'doctor-workspace-roots.json').read_text(encoding='utf-8-sig')),'tutorial_cli_evidence':json.loads((d/'tutorial-cli-roots.json').read_text(encoding='utf-8-sig')),'manifest_cli_evidence':json.loads((d/'manifest-cli-roots.json').read_text(encoding='utf-8-sig')),'manifest_width_evidence':json.loads((d/'plan-manifest-width-roots.json').read_text(encoding='utf-8-sig')),'manifest_reader_width_evidence':json.loads((d/'manifest-reader-width-roots.json').read_text(encoding='utf-8-sig')),'large_rplan_text_evidence':json.loads((d/'large-rplan-text-root.json').read_text(encoding='utf-8-sig')),'doctor_command_evidence':json.loads((d/'doctor-command-roots.json').read_text(encoding='utf-8-sig')),'policy_fallback_evidence':json.loads((d/'policy-fallback-roots.json').read_text(encoding='utf-8-sig')),'policy_cli_evidence':json.loads((d/'policy-cli-roots.json').read_text(encoding='utf-8-sig')),'config_publication_evidence':json.loads((d/'config-publication-roots.json').read_text(encoding='utf-8-sig')),'config_cli_evidence':json.loads((d/'config-cli-roots.json').read_text(encoding='utf-8-sig')),'native_format_cli_evidence':json.loads((d/'native-format-cli-roots.json').read_text(encoding='utf-8-sig')),'fletch_cli_evidence':json.loads((d/'fletch-cli-roots.json').read_text(encoding='utf-8-sig')),'ceremony_source_start_evidence':json.loads((d/'ceremony-source-start-roots.json').read_text(encoding='utf-8-sig')),'ceremony_contract_evidence':json.loads((d/'ceremony-contract-roots.json').read_text(encoding='utf-8-sig')),'state_name_workflow_evidence':json.loads((d/'state-name-workflow-root.json').read_text(encoding='utf-8-sig')),'report_source_workflow_evidence':json.loads((d/'report-source-roots.json').read_text(encoding='utf-8-sig')),'label_repository_evidence':json.loads((d/'label-repository-root.json').read_text(encoding='utf-8-sig')),'fetch_archive_evidence':json.loads((d/'fetch-archive-root.json').read_text(encoding='utf-8-sig')),'tests':'all seven legacy gates, actual CLI ceremony outcomes and prefix distinction, shared ceremony/timestamp checks, composed output Verify, native workspace continuation, Fetch source coverage, six archive/native cases and reviewed label repository/native CLI checks passed'},indent=2)+'\n',encoding='utf-8')" $gateRoot $gateVariant $gateRunId

        if($LASTEXITCODE -ne 0){throw 'Could not retain gate evidence.'}

    }

    Write-Host 'Legacy gates passed for the requested actual build profiles. C execution/runtime gates remain separate.'

} finally {$env:BISECT_TEST_WASM=$previousWasmPath;$env:BISECT_REPORT_PROFILE=$previousReportProfile;Pop-Location}

