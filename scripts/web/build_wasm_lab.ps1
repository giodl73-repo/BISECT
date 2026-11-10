param([string]$Output='dist/wasm-laboratory',[string]$States='RI,IA,NC',[string]$Year='2020',[switch]$BrowserChecks)
$ErrorActionPreference='Stop'
$labRepo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $labRepo
$labPreviousTestWasm=$env:BISECT_TEST_WASM
$env:BISECT_TEST_WASM=Join-Path $labRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'
try {
    python scripts/web/test_build_gate_dependencies.py
    if($LASTEXITCODE -ne 0){throw 'Build dependency guard mutation checks failed.'}
    python scripts/web/check_build_gate_dependencies.py
    if($LASTEXITCODE -ne 0){throw 'Build fixture or native executable producer order is invalid.'}
    python scripts/web/proof_dependency_preflight.py
    if($LASTEXITCODE -ne 0){throw 'Pinned proof prerequisites are missing, stale or altered; prepare the local solver/checker builds first.'}
    node scripts/web/test_native_command_comparison.mjs
    if($LASTEXITCODE -ne 0){throw 'Native/WASM command comparison corruption guards failed.'}

    python scripts/web/wasm_practitioner_inventory.py
    if($LASTEXITCODE -ne 0){throw 'Practitioner action source inventory drifted.'}
    node scripts/web/test_lab_project_limits.mjs
    if($LASTEXITCODE -ne 0){throw 'Laboratory project resource boundaries failed.'}
    cargo test -p rgraph-core --release --test prepared_connectivity --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Prepared graph connectivity and query-history regression failed.'}
    node scripts/web/test_state_batch_zip.mjs
    if($LASTEXITCODE -ne 0){throw 'State batch ZIP format/resource guards failed.'}
    cargo test -p bisect-report --release --no-default-features --lib label_summary --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Shared native label summary/security tests failed.'}
    cargo test -p bisect-report --release --no-default-features --test label_summary_security --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native summary metadata escaping tests failed.'}
    node scripts/web/test_report_print.mjs
    if($LASTEXITCODE -ne 0){throw 'Report print/PDF byte-observation guards failed.'}
    python scripts/web/test_parity_evidence_archive.py
    if($LASTEXITCODE -ne 0){throw 'Child evidence archive resource/path guards failed.'}
    python scripts/web/parity_evidence_archive.py ensure --root . --lock docs/specs/wasm-parity-child-evidence.lock.json
    if($LASTEXITCODE -ne 0){throw 'Restore the declared local child-evidence capsule before ledger validation.'}
    python scripts/web/wasm_parity_ledger.py
    if($LASTEXITCODE -ne 0){throw 'Structured parity ledger is stale or invalid.'}
    python scripts/web/test_wasm_parity_ledger.py
    if($LASTEXITCODE -ne 0){throw 'Parity closure mutation guards failed.'}

    node scripts/web/test_native_asset_build.mjs
    if($LASTEXITCODE -ne 0){throw 'Native embedded asset rebuild controls failed.'}
    cargo test -p bisect-web --lib --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native web/catalog asset and budget checks failed.'}
    $nativeAssetSuffix = if ([IO.Path]::DirectorySeparatorChar -eq '\') {'.exe'} else {''}
    if($BrowserChecks){
        cargo build -p bisect-web --bin bisect-web --bin lab-catalog --locked --offline -j 1
        if($LASTEXITCODE -ne 0){throw 'Native web/catalog browser fixture build failed.'}
        $nativeAssetWeb = Join-Path $labRepo ('target/debug/bisect-web'+$nativeAssetSuffix)
        $nativeAssetCatalog = Join-Path $labRepo ('target/debug/lab-catalog'+$nativeAssetSuffix)
        node scripts/web/test_native_web_assets_browser.mjs $nativeAssetWeb $nativeAssetCatalog (Join-Path $labRepo 'web/lab')
        if($LASTEXITCODE -ne 0){throw 'Native web/catalog module and saved-view browser checks failed.'}
    }

    python scripts/web/test_wasm_native_consumption.py
    if($LASTEXITCODE -ne 0){throw 'Native consumption source guards failed.'}
    python scripts/web/native_state_boundary.py --check
    if($LASTEXITCODE -ne 0){throw 'Prepared State browser boundary drifted from the compiler.'}
    python scripts/web/test_native_state_extraction.py
    if($LASTEXITCODE -ne 0){throw 'Native State source extraction contract failed.'}
    python scripts/web/test_native_state_extraction_transitions.py
    if($LASTEXITCODE -ne 0){throw 'Native State reviewed source transition mutation checks failed.'}
    python scripts/web/test_native_states_extraction.py
    if($LASTEXITCODE -ne 0){throw 'Native States source extraction contract failed.'}
    if((Test-Path -LiteralPath $Output) -and (Get-ChildItem -LiteralPath $Output -Force | Select-Object -First 1)){throw 'Choose a new or empty output directory.'}
    node scripts/web/test_source_relay.mjs
    if($LASTEXITCODE -ne 0){throw 'Pinned public-source relay boundary verification failed.'}
    cargo test -p bisect-data --lib block_group_reader --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Native block-group TIGER reader verification failed.'}
    cargo test -p bisect-data --lib bridge --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Deterministic native island bridge verification failed.'}
    cargo test -p bisect-data --lib projection --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Portable EPSG:5070 projection accuracy verification failed.'}
    cargo test -p bisect-data --lib adjacency --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Portable native boundary-length and adjacency verification failed.'}
    cargo test -p bisect-core --lib character --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Shared economic/housing character formula and zero-policy verification failed.'}
    cargo test -p bisect-runner --lib --no-default-features compact_geometry --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Native State circular geometry arithmetic verification failed.'}
    cargo test -p bisect-runner --lib --no-default-features proportional --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Proportional recursion, seat allocation and input validation failed.'}
    cargo build -p bisect-wasm --release --target wasm32-unknown-unknown --locked --offline
    if($LASTEXITCODE -ne 0){throw 'WASM build failed.'}
    node scripts/web/test_county_augmentation_contract.mjs
    if($LASTEXITCODE -ne 0){throw 'Native county composition contract failed.'}
    node scripts/web/test_native_state_vra_contract.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State minority summary contract failed.'}
    node scripts/web/test_native_default_progress_contract.mjs
    if($LASTEXITCODE -ne 0){throw 'Default native progress callback contract failed.'}
    node scripts/web/test_native_round_progress_contract.mjs
    if($LASTEXITCODE -ne 0){throw 'Native round progress callback contract failed.'}
    node scripts/web/test_wasm.mjs target/wasm32-unknown-unknown/release/bisect_wasm.wasm
    if($LASTEXITCODE -ne 0){throw 'WASM engine verification failed.'}
    node scripts/web/test_area_wasm.mjs target/wasm32-unknown-unknown/release/bisect_wasm.wasm
    if($LASTEXITCODE -ne 0){throw 'WASM AreaSection verification failed.'}
    cargo build -p bisect-wasm --release --example execute_request --example execute_tool_request --example execute_tool_sequence --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Native host reference build failed.'}
    cargo build -p bisect-cli --no-default-features --example fetch_plan_reference --example fletch_sources_reference --example fetch_archive_reference --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Fetch/FLETCH source reference build failed.'}
    node scripts/web/test_fetch_source_coverage.mjs
    if($LASTEXITCODE -ne 0){throw 'Fetch/FLETCH source-type reconciliation drifted.'}
    $fetchArchiveCheck=node scripts/web/test_fetch_archive_acquisition.mjs | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Fetch archive/native acquisition checks failed.'}
    cargo build -p bisect-cli --release --no-default-features --example state_signal_preset_reference --example state_command_reference --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Native State signal preset reference build failed.'}
    node scripts/web/test_native_state_command.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State command correspondence failed.'}
    node scripts/web/test_native_state_scope.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State scope conversion failed.'}
    node scripts/web/test_native_signal_presets.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State signal preset correspondence failed.'}
    node scripts/web/test_character_csv.mjs
    if($LASTEXITCODE -ne 0){throw 'Economic/housing character native/WASM CSV import validation failed.'}
    node scripts/web/test_cvd_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'CVD native/WASM parameter and portability verification failed.'}
    node scripts/web/test_mka_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'Moving-knife native/WASM parameter and scoring-boundary verification failed.'}
    node scripts/web/test_compact_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'CompactBisect native/WASM slack and seed-policy verification failed.'}
    node scripts/web/test_area_init_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'AreaSection native/WASM initialization verification failed.'}
    node scripts/web/test_burst_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'Short-burst native/WASM controls and portability verification failed.'}
    node scripts/web/test_pt_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'Parallel-tempering native/WASM controls and portability verification failed.'}
    node scripts/web/test_local_ensemble_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'Local ensemble native/WASM portability and target verification failed.'}
    node scripts/web/test_flip_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'Boundary-flip native/WASM portability and population verification failed.'}
    node scripts/web/test_chains_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'Standalone chain native/WASM portability verification failed.'}
    node scripts/web/test_demographic_input.mjs
    if($LASTEXITCODE -ne 0){throw 'Demographic input validation and identity verification failed.'}
    node scripts/web/test_vra_recom_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'VRA ReCom native/WASM preservation and demographic identity verification failed.'}
    node scripts/web/test_vra_section_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'VRASection native/WASM scoring and demographic verification failed.'}
    cargo run -p bisect-web --bin wasm-lab-assets --release --locked --offline -- --output $Output --states $States --year $Year
    if($LASTEXITCODE -ne 0){throw 'Static laboratory export failed.'}
    node scripts/web/test_character_wasm.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Character weighting, native/WASM parity, project and export verification failed.'}
    New-Item -ItemType Directory -Path target/laboratory-practitioner-archive -Force | Out-Null
    if(($States -split ',') -contains 'CA'){
        $largeAudit=node scripts/web/test_laboratory_practitioner_archive.mjs $Output | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw 'Full-state practitioner archive/native export comparison failed.'}
        $largeAuditBrowser=$null
        if($BrowserChecks){
            $largeAuditBrowser=node scripts/web/test_laboratory_practitioner_archive_browser.mjs ('--reference='+$largeAudit.root) ('--laboratory='+$Output) ('--wasm='+(Join-Path $Output 'bisect_wasm.wasm')) --toolkit=web/lab | ConvertFrom-Json
            if($LASTEXITCODE -ne 0){throw 'Full-state practitioner archive laboratory/workbench browser check failed.'}
        }
        @{status='passed';node=$largeAudit;browser=$largeAuditBrowser;browser_status=$(if($BrowserChecks){'passed'}else{'not-run'})} | ConvertTo-Json -Depth 8 | Set-Content target/laboratory-practitioner-archive/standard-root.json -Encoding utf8
    }else{
        @{status='not-run';reason='CA prepared graph/geometry not included in this laboratory scope.'} | ConvertTo-Json | Set-Content target/laboratory-practitioner-archive/standard-root.json -Encoding utf8
    }
    node scripts/web/test_multiscale_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'Portable multiscale fixed/adaptive resolution and input verification failed.'}
    node scripts/web/test_native_state_run.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State flags-to-engine correspondence failed.'}
    node scripts/web/test_native_state_option_contract.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State option execution contract failed.'}
    node scripts/web/test_state_identity_inspection.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State identity inspection/guard failed.'}
    node scripts/web/test_native_state_workflow.mjs --core-only
    if($LASTEXITCODE -ne 0){throw 'Native State output workflow handoff failed.'}
    node scripts/web/test_state_source_preflight.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State source acquisition preflight failed.'}
    python scripts/web/test_native_state_workflow_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Native State workflow ZIP integrity failed.'}
    node scripts/web/test_native_state_graph.mjs --core-only
    if($LASTEXITCODE -ne 0){throw 'Native State graph-to-flags execution correspondence failed.'}
    cargo build -p bisect-cli --example states_config_reference --example geometric_state_reference --example run_config_reference --no-default-features --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Frozen native States configuration oracle build failed.'}
    node scripts/web/test_native_state_adjacency_workflow.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State selected adjacency workflow failed.'}
    python scripts/web/test_native_state_adjacency_workflow_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Native State selected adjacency ZIP integrity failed.'}
    node scripts/web/test_native_selected_state_source.mjs --core-only
    if($LASTEXITCODE -ne 0){throw 'Single selected-source State count/execution parity failed.'}
    python scripts/web/test_native_selected_state_source_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Single selected-source State ZIP integrity failed.'}
    node scripts/web/test_native_state_chamber_sources.mjs --core-only
    if($LASTEXITCODE -ne 0){throw 'House/Senate selected-source State parity failed.'}
    python scripts/web/test_native_state_chamber_sources_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'House/Senate State source ZIP integrity failed.'}
    node scripts/web/test_native_adjacency_state_boundary.mjs --skip-previous
    if($LASTEXITCODE -ne 0){throw 'Selected native adjacency State compiler parity failed.'}
    python scripts/web/test_native_adjacency_state_artifacts.py
    if($LASTEXITCODE -ne 0){throw 'Selected native adjacency State artifact integrity failed.'}
    node scripts/web/test_practitioner_inputs.mjs
    if($LASTEXITCODE -ne 0){throw 'Practitioner source selection hints failed.'}
    node scripts/web/test_native_custom_state_sources.mjs
    if($LASTEXITCODE -ne 0){throw 'Custom selected-source State parity failed.'}
    python scripts/web/test_native_custom_state_sources_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Custom State source ZIP integrity failed.'}
    node scripts/web/test_native_custom_state_workflow.mjs
    if($LASTEXITCODE -ne 0){throw 'Custom State output binding/follow-up parity failed.'}
    python scripts/web/test_native_custom_state_workflow_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Custom State workflow artifact integrity failed.'}
    python scripts/web/test_native_run_extraction.py
    if($LASTEXITCODE -ne 0){throw 'Actual native Run extraction baseline changed.'}
    node scripts/web/test_native_run_command.mjs
    if($LASTEXITCODE -ne 0){throw 'Actual native Run configuration/execution parity failed.'}
    python scripts/web/test_native_run_command_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Actual native Run source/export integrity failed.'}
    node scripts/web/test_native_run_queue.mjs
    if($LASTEXITCODE -ne 0){throw 'Native Run original-source queue parity failed.'}
    python scripts/web/test_native_run_queue_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Native Run multi-year archive integrity failed.'}
    node scripts/web/test_native_states_engine.mjs
    if($LASTEXITCODE -ne 0){throw 'Actual native States bulk engine configuration failed.'}
    node scripts/web/test_bulk_cvd_controls.mjs
    if($LASTEXITCODE -ne 0){throw 'Native bulk CVD iteration/metric controls failed.'}
    python scripts/web/test_bulk_cvd_controls_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Native bulk CVD export archive verification failed.'}
    python scripts/web/test_native_states_engine_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Native States source/export archive verification failed.'}
    cargo build -p bisect-cli --no-default-features --example native_adjacency_reference --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native workspace file-reader reference build failed.'}
    cargo build -p bisect-cli --bin bisect --example label_summary_application_reference --release --no-default-features --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native data-reset CLI reference build failed.'}
    python scripts/web/test_native_cli_workspace.py
    if($LASTEXITCODE -ne 0){throw 'Actual native CLI/WASM workspace lifecycle regression failed.'}
    python scripts/web/test_native_cli_unused.py
    if($LASTEXITCODE -ne 0){throw 'Actual native CLI/WASM unused-field and intent regression failed.'}
    python scripts/web/test_native_cli_verify_replay.py --native-executable (Join-Path $labRepo ('target/release/bisect'+$nativeAssetSuffix))
    if($LASTEXITCODE -ne 0){throw 'Actual CLI Verify replay output-root regression failed; preserve its raw State/Verify receipts.'}
    node scripts/web/test_native_aggregate_selection.mjs (Join-Path $labRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $labRepo ('target/release/bisect'+$nativeAssetSuffix))
    if($LASTEXITCODE -ne 0){throw 'Native Aggregate effective-source selection regression failed.'}
    $nativeAggregateSuiteArguments=@((Join-Path $labRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),(Join-Path $labRepo ('target/release/bisect'+$nativeAssetSuffix)))
    if($BrowserChecks){$nativeAggregateSuiteArguments+='--browser'}
    node scripts/web/test_native_aggregate_effects_suite.mjs @nativeAggregateSuiteArguments
    if($LASTEXITCODE -ne 0){throw 'Native Aggregate command/recovery/backup suite failed; preserve its owned evidence.'}
    $census2010Check=node scripts/web/test_census_2010_endpoints.mjs (Join-Path $labRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $labRepo ('target/release/examples/execute_tool_request'+$nativeAssetSuffix)) (Join-Path $labRepo ('target/release/bisect'+$nativeAssetSuffix)) | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Native 2010 Census endpoint regression failed; preserve its raw native/WASM/CLI evidence.'}
    if($census2010Check.status -ne 'passed'){throw 'Native 2010 Census endpoint receipt did not pass.'}
    if($BrowserChecks){
        node scripts/web/test_census_2010_browser.mjs $census2010Check.census_2010_endpoint_evidence_root
        if($LASTEXITCODE -ne 0){throw 'Native 2010 Census acquisition/browser boundary regression failed.'}
    }
    $ilpNativeSummaryCheck=node scripts/web/test_ilp_native_summary_paths.mjs (Join-Path $labRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $labRepo ('target/release/examples/execute_tool_request'+$nativeAssetSuffix)) (Join-Path $labRepo ('target/release/bisect'+$nativeAssetSuffix)) | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Native ILP summary path binding regression failed; preserve its actual CLI/host/WASM receipts.'}
    if($ilpNativeSummaryCheck.status -ne 'passed'){throw 'Native ILP summary path binding receipt did not pass.'}
    if($BrowserChecks){
        node scripts/web/test_ilp_native_summary_paths_browser.mjs $ilpNativeSummaryCheck.ilp_native_summary_evidence_root
        if($LASTEXITCODE -ne 0){throw 'Native ILP source-byte/browser summary regression failed.'}
    }

    python scripts/web/test_native_state_inert_flags.py
    if($LASTEXITCODE -ne 0){throw 'Native parsed-unused State flag effects drifted.'}
    node scripts/web/test_native_state_data_reset.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State data-reset filesystem correspondence failed.'}
    node scripts/web/test_native_state_target_run.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State before-Run target/source/CLI checks failed.'}
    cargo build --release -p rcount-cli --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native RCOUNT transcript reference failed to build.'}
    node scripts/web/test_native_label_summary_wasm.mjs
    if($LASTEXITCODE -ne 0){throw 'Native label summary files/CLI checks failed.'}
    node scripts/web/test_native_label_summary_batch.mjs
    if($LASTEXITCODE -ne 0){throw 'Native label summary ordering/output checks failed.'}
    cargo test --release -p bisect-cli --no-default-features --lib analyze_label::tests::test_report_clock_ --locked --offline -j 1 -- --test-threads=1
    if($LASTEXITCODE -ne 0){throw 'Native report clock ordering/provenance regressions failed.'}
    cargo test --release -p bisect-report --no-default-features --lib label_rename --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Shared native rename safety/kernel tests failed.'}
    python scripts/web/test_native_case_alias_cli.py
    if($LASTEXITCODE -ne 0){throw 'Native case-only rename source preservation failed.'}
    node scripts/web/test_label_build_application.mjs
    if($LASTEXITCODE -ne 0){throw 'Reviewed label build/native CLI application checks failed.'}
    node scripts/web/test_label_build_application_ui.mjs
    if($LASTEXITCODE -ne 0){throw 'Reviewed label build DOM controller checks failed.'}
    node scripts/web/test_label_build_native_failures.mjs
    if($LASTEXITCODE -ne 0){throw 'Native label build failure-prefix checks failed.'}
    node scripts/web/test_native_summary_application.mjs
    if($LASTEXITCODE -ne 0){throw 'Ordered native summary application/reference checks failed.'}
    node scripts/web/test_label_import_application.mjs
    if($LASTEXITCODE -ne 0){throw 'Reviewed native label import application/CLI checks failed.'}
    node scripts/web/test_label_rename_application.mjs
    if($LASTEXITCODE -ne 0){throw 'Reviewed label rename/native CLI application checks failed.'}
    node scripts/web/test_count_transcript_installation.mjs
    if($LASTEXITCODE -ne 0){throw 'RCOUNT transcript/native CLI installation checks failed.'}
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
    if($LASTEXITCODE -ne 0){throw 'Selected native workspace source/Run/output checks failed.'}
    if($BrowserChecks){
        node scripts/web/test_fetch_archive_acquisition_browser.mjs $fetchArchiveCheck.root
        if($LASTEXITCODE -ne 0){throw 'Fetch archive acquisition browser checks failed.'}
        cargo build -p rplan-io --example rewrite_context --release --target-dir target/explicit-county-context-host --locked --offline -j 1
        if($LASTEXITCODE -ne 0){throw 'Native context fixture writer build failed.'}
        node scripts/web/test_explicit_county_splits_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Explicit custom county analysis browser checks failed.'}
        node scripts/web/test_native_state_data_reset_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State data-reset browser checks failed.'}
        node scripts/web/test_workbench_project_open_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Workbench inert Open observation regression failed.'}
        node scripts/web/test_native_aggregate_order_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native Aggregate ordered-type browser regression failed.'}
        node scripts/web/test_native_state_target_run_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State before-Run target browser checks failed.'}
        node scripts/web/test_native_label_summary_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native label summary browser checks failed.'}
        node scripts/web/test_label_build_application_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Reviewed label build actual-worker browser checks failed.'}
        node scripts/web/test_native_summary_application_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Ordered native summary application browser checks failed.'}
        node scripts/web/test_label_import_application_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Reviewed native label import browser application failed.'}
        node scripts/web/test_label_rename_application_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Reviewed label rename browser application checks failed.'}
        node scripts/web/test_count_transcript_installation_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'RCOUNT transcript browser installation checks failed.'}
        $reportPdfBrowser=node scripts/web/test_report_pdf_browser.mjs --json | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw 'Actual report PDF/browser/native CLI checks failed.'}
        $labelRepositoryBrowser=node scripts/web/test_label_repository_installation_browser.mjs --json | ConvertFrom-Json
        if($LASTEXITCODE -ne 0){throw 'Label repository installation browser checks failed.'}
        node scripts/web/test_native_workspace_continuation_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Verified workspace continuation browser checks failed.'}
        node scripts/web/test_native_workspace_continuation_browser.mjs --multi-district
        if($LASTEXITCODE -ne 0){throw 'Multi-district workspace continuation browser checks failed.'}
        node scripts/web/test_native_bulk_workspace_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native bulk workspace browser checks failed.'}
        node scripts/web/test_acquisition_recovery_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'State-source acquisition recovery browser checks failed.'}
        node scripts/web/test_acquisition_recovery_browser.mjs --configured
        if($LASTEXITCODE -ne 0){throw 'Configured acquisition recovery browser checks failed.'}
        node scripts/web/test_native_output_verification_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Composed native output Verify browser checks failed.'}
        node scripts/web/test_native_workspace_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Selected native workspace browser workflow failed.'}
        node scripts/web/test_native_state_recovery_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State interruption recovery browser checks failed.'}
        node scripts/web/test_native_state_batch_recovery_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State batch recovery browser checks failed.'}
        node scripts/web/test_native_state_option_contract_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State execution boundary notice failed.'}
        node scripts/web/test_native_state_workflow_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State output workflow browser failed.'}
        node scripts/web/test_native_state_adjacency_workflow_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State selected adjacency browser workflow failed.'}
        python scripts/web/test_native_state_adjacency_workflow_zip.py
        if($LASTEXITCODE -ne 0){throw 'Native State adjacency browser ZIP integrity failed.'}
        node scripts/web/test_native_selected_state_source_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Single selected-source State browser workflow failed.'}
        python scripts/web/test_native_selected_state_source_zip.py
        if($LASTEXITCODE -ne 0){throw 'Single selected-source State browser ZIP integrity failed.'}
        node scripts/web/test_native_state_chamber_sources_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'House/Senate selected-source State browser workflow failed.'}
        python scripts/web/test_native_state_chamber_sources_zip.py
        if($LASTEXITCODE -ne 0){throw 'House/Senate State browser ZIP integrity failed.'}
        node scripts/web/test_native_custom_state_sources_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Custom State source browser workflow failed.'}
        python scripts/web/test_native_custom_state_sources_zip.py
        if($LASTEXITCODE -ne 0){throw 'Custom State browser ZIP integrity failed.'}
        node scripts/web/test_native_custom_state_workflow_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Custom State bound browser workflow failed.'}
        python scripts/web/test_native_custom_state_workflow_zip.py
        if($LASTEXITCODE -ne 0){throw 'Custom State browser workflow artifact integrity failed.'}
        node scripts/web/test_practitioner_flow_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Practitioner guide workflow failed.'}
        python scripts/web/test_native_state_workflow_zip.py
        if($LASTEXITCODE -ne 0){throw 'Native State browser workflow ZIP integrity failed.'}
        node scripts/web/test_native_run_command_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Actual native Run browser workflow failed.'}
        python scripts/web/test_native_run_command_zip.py
        if($LASTEXITCODE -ne 0){throw 'Actual native Run browser export verification failed.'}
        node scripts/web/test_native_run_queue_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native Run source queue browser workflow failed.'}
        python scripts/web/test_native_run_queue_zip.py
        if($LASTEXITCODE -ne 0){throw 'Native Run browser multi-year archive integrity failed.'}
        node scripts/web/test_native_states_engine_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native States browser workflow failed.'}
        node scripts/web/test_bulk_cvd_controls_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native bulk CVD browser controls failed.'}
        python scripts/web/test_bulk_cvd_controls_zip.py
        if($LASTEXITCODE -ne 0){throw 'Native bulk CVD browser archive comparison failed.'}
        python scripts/web/test_native_states_engine_zip.py
        if($LASTEXITCODE -ne 0){throw 'Native States browser archive comparison failed.'}
    }
    node scripts/web/test_native_state_batch.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State flags batch orchestration failed.'}
    python scripts/web/test_native_state_batch_zip.py
    if($LASTEXITCODE -ne 0){throw 'Native State flags archive verification failed.'}
    node scripts/web/test_native_state_sources.mjs --core-only
    if($LASTEXITCODE -ne 0){throw 'Native State raw source execution failed.'}
    node scripts/web/test_accepted_state_workflow.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Accepted State workflow parity checks failed.' }
    node scripts/web/test_native_state_followup.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native State automatic follow-up checks failed.' }
    node scripts/web/test_native_state_output.mjs --core-only
    if ($LASTEXITCODE -ne 0) { throw 'Native State native-layout output checks failed.' }
    cargo build -p rplan-io --example rewrite_context --release --target-dir target/explicit-county-context-host --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native context fixture writer build failed.'}
    cargo build -p bisect-cli --bin bisect --example label_summary_application_reference --release --no-default-features --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native CLI split reference build failed.'}
    node scripts/web/test_explicit_county_splits.mjs
    if($LASTEXITCODE -ne 0){throw 'Explicit county identities and native CLI/WASM split analysis failed.'}
    node scripts/web/test_native_state_recovery.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State explicit interruption recovery failed.'}
    node scripts/web/test_native_state_batch_recovery.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native State batch interruption recovery failed.' }
    node scripts/web/test_native_state_batch_output.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native State folder and batch output checks failed.' }
    node scripts/web/test_native_state_files.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State data export correspondence failed.'}
    node scripts/web/test_multiscale_project.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Multiscale state/national Save/Open and fine-map verification failed.'}
    node scripts/web/test_multiscale_build.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Multiscale native balance retries, seed evidence and practitioner export failed.'}
    node scripts/web/test_multiscale_export.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Multiscale practitioner export, audit and certificate verification failed.'}
    node scripts/web/test_multiscale_input_worker.mjs
    if($LASTEXITCODE -ne 0){throw 'Fine input Worker validation and cancellation failed.'}
    node scripts/web/test_character_national.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'National character inputs, saved projects and maps failed verification.'}
    if($BrowserChecks){
        node scripts/web/test_character_national_browser.mjs $Output --require-package
        if($LASTEXITCODE -ne 0){throw 'National character browser Save/Open resource and retention checks failed.'}
    }
    node scripts/web/test_character_csv_worker.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Character CSV worker integrity, validation and cancellation failed.'}
    node scripts/web/test_proportional_wasm.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Proportional native/WASM, election evidence, project and export verification failed.'}
    node scripts/web/test_election_csv.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Election CSV native/WASM import and project roundtrip failed.'}
    node scripts/web/test_election_csv_worker.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Election CSV worker integrity, validation and cancellation failed.'}
    node scripts/web/test_partisan_tsv.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Partisan TSV native/WASM import and project roundtrip failed.'}
    node scripts/web/test_partisan_tsv_worker.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Partisan TSV worker integrity, validation and cancellation failed.'}
    node scripts/web/test_partisan_wasm.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Partisan weighting, input identity and project verification failed.'}
    node scripts/web/test_partisan_search_edges.mjs
    if($LASTEXITCODE -ne 0){throw 'Partisan default controls, percentile endpoints and trivial/zero-proposal boundaries failed.'}
    node scripts/web/test_bfs_wasm.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'BFS native/WASM reproducibility and project verification failed.'}
    node scripts/web/test_u64_seed.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Exact u64 seed transport and project verification failed.'}
    node scripts/web/test_nway_tuning.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'N-way METIS laboratory project verification failed.'}
    node scripts/web/test_ratio_constraint_tuning.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Area/VRA refinement, constraint and project verification failed.'}
    node scripts/web/test_geosection_tuning.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'GeoSection METIS refinement and project verification failed.'}
    node scripts/web/test_recursive_tuning.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Recursive METIS objective/trials and project verification failed.'}
    node scripts/web/test_percentile_tuning.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Percentile METIS selection evidence and project verification failed.'}
    node scripts/web/test_ensemble_tuning.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Local ensemble METIS initialization and project verification failed.'}
    node scripts/web/test_convergence_wasm.mjs $Output $Year
    if($LASTEXITCODE -ne 0){throw 'Convergence refinement, candidate replay and project verification failed.'}
    & (Join-Path $PSScriptRoot 'build_wasm.ps1') -Output (Join-Path $Output 'toolkit') -BrowserChecks:$BrowserChecks
    if($LASTEXITCODE -ne 0){throw 'Integrated practitioner workbench build failed.'}
    if($States -eq 'RI,IA,NC' -and $Year -eq '2020') {
        node scripts/web/test_plan_comparison_real.mjs $Output
        if($LASTEXITCODE -ne 0){throw 'Real-state plan comparison verification failed.'}
    }
    node scripts/web/test_config_catalog.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Published YAML catalog state input verification failed.'}
    node scripts/web/test_label_build_catalog.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Published multi-state build job preparation failed.'}
    node scripts/web/test_catalog_preparation_worker.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Published build preparation worker verification failed.'}
    if(($States -split ',').Count -eq 50 -and $Year -eq '2020') {
        node scripts/web/test_catalog_preparation_worker.mjs $Output --national
        if($LASTEXITCODE -ne 0){throw 'Fifty-state catalog preparation worker verification failed.'}
        node scripts/web/test_label_build_national.mjs $Output
        if($LASTEXITCODE -ne 0){throw 'Fifty-state prepared-source and build capacity verification failed.'}
        python scripts/web/test_label_build_national_zip.py
        if($LASTEXITCODE -ne 0){throw 'Fifty-state archive and assignment integrity failed.'}
        node scripts/web/test_label_build_national_archive.mjs
        if($LASTEXITCODE -ne 0){throw 'Fifty-state complete source/output archive capacity failed.'}
    }
    node scripts/web/test_wasm_lab.mjs $Output $States $Year
    if($LASTEXITCODE -ne 0){throw 'Laboratory integration tests failed.'}
    node scripts/web/test_lab_build.mjs $Output $States $Year
    if($LASTEXITCODE -ne 0){throw 'Laboratory native balance retries and saved-project verification failed.'}
    node scripts/web/test_audit_handoff.mjs $Output $Year
    if($LASTEXITCODE -ne 0){throw 'Laboratory practitioner handoff verification failed.'}
    node scripts/web/test_one_district_wasm.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'One-district engine/project/map verification failed.'}
    node scripts/web/test_lab_project.mjs $Output $States $Year
    if($LASTEXITCODE -ne 0){throw 'Laboratory project verification failed.'}
    node scripts/web/test_demographic_csv.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Demographic CSV native/WASM and count-preserving project verification failed.'}
    node scripts/web/test_demographic_csv_worker.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Demographic CSV browser Worker integrity and import verification failed.'}
    node scripts/web/test_chains_real_wasm.mjs $Output $Year
    if($LASTEXITCODE -ne 0){throw 'Standalone chains real-input native/WASM and project verification failed.'}
    node scripts/web/test_vra_project.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'VRA real-graph demographic project and verification-worker checks failed.'}
    python scripts/web/build_offline_manifest.py $Output
    if($LASTEXITCODE -ne 0){throw 'Offline application manifest generation failed.'}
    node scripts/web/test_offline_package.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Offline application package verification failed.'}
} finally {
    if($null -eq $labPreviousTestWasm){Remove-Item Env:BISECT_TEST_WASM -ErrorAction SilentlyContinue}else{$env:BISECT_TEST_WASM=$labPreviousTestWasm}
    Pop-Location
}
