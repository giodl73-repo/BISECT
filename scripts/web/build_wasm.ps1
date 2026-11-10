param([ValidateSet('county','block-group','block')][string[]]$OriginalNativeCensusChecks = @(),[string]$Output = 'dist/wasm-preview',[switch]$BrowserChecks,[switch]$LocalCensusChecks,[switch]$LargeStateChecks,[switch]$AllOriginalTractChecks,[switch]$OriginalNationalMapChecks,[switch]$OriginalAlaskaChecks,[ValidatePattern('^[A-Z]{2}$')][string[]]$OriginalAlgorithmStateChecks = @(),[ValidatePattern('^[A-Z]{2}$')][string[]]$OriginalGeometryStateChecks = @(),[ValidatePattern('^[A-Z]{2}$')][string[]]$OriginalStateChecks = @())
$ErrorActionPreference = 'Stop'
$wasmRepo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $wasmRepo
$wasmPreviousTestWasm=$env:BISECT_TEST_WASM
$env:BISECT_TEST_WASM=Join-Path $wasmRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'
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
    node scripts/web/test_native_command_comparison.mjs
    if($LASTEXITCODE -ne 0){throw 'Native/WASM command comparison corruption guards failed.'}
    python scripts/web/test_proof_dependency_preflight.py
    if($LASTEXITCODE -ne 0){throw 'Proof prerequisite refusal regression failed.'}

    python scripts/web/test_engine_option_inventory_history.py
    if($LASTEXITCODE -ne 0){throw 'Engine declaration history preservation failed.'}
    python scripts/web/wasm_practitioner_inventory.py
    if($LASTEXITCODE -ne 0){throw 'Practitioner action source inventory drifted.'}
    node scripts/web/test_lab_project_limits.mjs
    if($LASTEXITCODE -ne 0){throw 'Laboratory project resource boundaries failed.'}
    cargo test -p rgraph-core --release --test prepared_connectivity --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Prepared graph connectivity and query-history regression failed.'}
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
        $nativeAssetWeb = Join-Path $wasmRepo ('target/debug/bisect-web'+$nativeAssetSuffix)
        $nativeAssetCatalog = Join-Path $wasmRepo ('target/debug/lab-catalog'+$nativeAssetSuffix)
        node scripts/web/test_native_web_assets_browser.mjs $nativeAssetWeb $nativeAssetCatalog (Join-Path $wasmRepo 'web/lab')
        if($LASTEXITCODE -ne 0){throw 'Native web/catalog module and saved-view browser checks failed.'}
    }

    python scripts/web/test_wasm_native_consumption.py
    if($LASTEXITCODE -ne 0){throw 'Native consumption source guards failed.'}
    python scripts/web/native_state_boundary.py --check
    if($LASTEXITCODE -ne 0){throw 'Prepared State browser boundary drifted from the compiler.'}
    foreach ($originalState in $OriginalStateChecks) {
        foreach ($originalKind in @('tiger','pl94171')) {
            $originalArchive = Join-Path $wasmRepo "target/original-state-acquisition/$originalState/$originalKind.zip"
            if (!(Test-Path -LiteralPath $originalArchive) -or !(Test-Path -LiteralPath "$originalArchive.receipt.json")) { throw "Acquire original state prerequisites explicitly first: node scripts/web/acquire_original_state_sources.mjs $originalState" }
        }
    }
    if ((Test-Path -LiteralPath $Output) -and (Get-ChildItem -LiteralPath $Output -Force | Select-Object -First 1)) { throw 'Choose a new or empty output directory.' }
    cargo build -p bisect-wasm --release --target wasm32-unknown-unknown --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'WASM build failed.' }
    cargo test -p bisect-core --lib state_catalog::tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native state catalog validation failed.' }
    cargo build -p bisect-cli --no-default-features --example state_selection_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native state selection reference failed.' }
    cargo build -p bisect-cli --no-default-features --example state_completion_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native state completion filesystem oracle build failed.' }
    node scripts/web/test_state_batch_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native state batch selection WASM parity failed.' }
    cargo test -p bisect-data --no-default-features --lib serialize::tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native binary adjacency decoder validation failed.' }
    cargo test -p bisect-runner --no-default-features --lib adjacency_loader::tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native adjacency selected-byte loader validation failed.' }
    cargo build -p bisect-cli --no-default-features --example native_adjacency_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native binary adjacency reference build failed.' }
    node scripts/web/test_native_adjacency_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native binary adjacency WASM verification failed.' }
    cargo build -p bisect-wasm --release --example execute_tool_request --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native toolkit engine reference build failed.' }
    cargo build -p bisect-cli --example run_config_reference --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Frozen native Run configuration reference build failed.' }
    python scripts/web/test_native_run_extraction.py
    if ($LASTEXITCODE -ne 0) { throw 'Actual native Run extraction baseline changed.' }
    node scripts/web/test_native_run_command.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Actual native Run parser/compositor/execution parity failed.' }
    python scripts/web/test_native_run_command_zip.py --skip-browser
    if ($LASTEXITCODE -ne 0) { throw 'Actual native Run exact source/export integrity failed.' }
    node scripts/web/test_native_run_queue.mjs
    if($LASTEXITCODE -ne 0){throw 'Native Run original-source queue parity failed.'}
    python scripts/web/test_native_run_queue_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Native Run multi-year archive integrity failed.'}
    node scripts/web/test_state_completion_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native state completion filtering/source parity failed.' }
    cargo build -p bisect-cli --bin bisect --example label_summary_application_reference --release --no-default-features --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native data-reset CLI reference build failed.'}
    python scripts/web/test_native_cli_workspace.py
    if($LASTEXITCODE -ne 0){throw 'Actual native CLI/WASM workspace lifecycle regression failed.'}
    python scripts/web/test_native_cli_unused.py
    if($LASTEXITCODE -ne 0){throw 'Actual native CLI/WASM unused-field and intent regression failed.'}
    python scripts/web/test_native_cli_verify_replay.py --native-executable (Join-Path $wasmRepo ('target/release/bisect'+$nativeAssetSuffix))
    if($LASTEXITCODE -ne 0){throw 'Actual CLI Verify replay output-root regression failed; preserve its raw State/Verify receipts.'}
    node scripts/web/test_native_aggregate_selection.mjs (Join-Path $wasmRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $wasmRepo ('target/release/bisect'+$nativeAssetSuffix))
    if($LASTEXITCODE -ne 0){throw 'Native Aggregate effective-source selection regression failed.'}
    $nativeAggregateSuiteArguments=@((Join-Path $wasmRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),(Join-Path $wasmRepo ('target/release/bisect'+$nativeAssetSuffix)))
    if($BrowserChecks){$nativeAggregateSuiteArguments+='--browser'}
    node scripts/web/test_native_aggregate_effects_suite.mjs @nativeAggregateSuiteArguments
    if($LASTEXITCODE -ne 0){throw 'Native Aggregate command/recovery/backup suite failed; preserve its owned evidence.'}
    $census2010Check=node scripts/web/test_census_2010_endpoints.mjs (Join-Path $wasmRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $wasmRepo ('target/release/examples/execute_tool_request'+$nativeAssetSuffix)) (Join-Path $wasmRepo ('target/release/bisect'+$nativeAssetSuffix)) | ConvertFrom-Json
    if($LASTEXITCODE -ne 0){throw 'Native 2010 Census endpoint regression failed; preserve its raw native/WASM/CLI evidence.'}
    if($census2010Check.status -ne 'passed'){throw 'Native 2010 Census endpoint receipt did not pass.'}
    if($BrowserChecks){
        node scripts/web/test_census_2010_browser.mjs $census2010Check.census_2010_endpoint_evidence_root
        if($LASTEXITCODE -ne 0){throw 'Native 2010 Census acquisition/browser boundary regression failed.'}
    }
    $ilpNativeSummaryCheck=node scripts/web/test_ilp_native_summary_paths.mjs (Join-Path $wasmRepo 'target/wasm32-unknown-unknown/release/bisect_wasm.wasm') (Join-Path $wasmRepo ('target/release/examples/execute_tool_request'+$nativeAssetSuffix)) (Join-Path $wasmRepo ('target/release/bisect'+$nativeAssetSuffix)) | ConvertFrom-Json
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
    node scripts/web/test_native_engine_input_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native binary engine preparation verification failed.' }
    node scripts/web/test_native_geometry_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Supplemental native geometry validation failed.' }
    node scripts/web/test_wasm.mjs target/wasm32-unknown-unknown/release/bisect_wasm.wasm
    if ($LASTEXITCODE -ne 0) { throw 'WASM verification failed.' }
    cargo build -p bisect-cli --example build_index_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native label build index reference build failed.' }
    node scripts/web/test_build_index_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Label build index parity and dry-run preservation failed.' }
    cargo run -p rcount-audit --example wasm_fixtures --locked --offline -- target/rcount-wasm-fixtures
    if ($LASTEXITCODE -ne 0) { throw 'Native election reference generation failed.' }
    node scripts/web/test_count_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Election package verification failed.' }
    node scripts/web/test_package_files.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Package file roundtrip failed.' }
    cargo build -p bisect-wasm --release --example execute_tool_request --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native election import reference build failed.' }
    cargo build -p bisect-cli --example policy_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native policy reference build failed.' }
    node scripts/web/test_policy_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native policy lookup and browser project parity failed.' }
    cargo build -p bisect-cli --example doctor_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native doctor reference build failed.' }
    node scripts/web/test_doctor_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Location preflight and native registry parity failed.' }
    cargo build -p bisect-cli --example manifest_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native manifest reference build failed.' }
    cargo test -p bisect-report --no-default-features binary_provenance --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native binary hash advisory rules failed.' }
    node scripts/web/test_runtime_artifact_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Loaded WASM fingerprint and worker identity verification failed.' }
    node scripts/web/test_manifest_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native manifest compatibility parity failed.' }
    cargo build -p bisect-cli --example tutorial_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native tutorial reference build failed.' }
    node scripts/web/test_tutorial_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native tutorial checksum and streamed hash verification failed.' }
    cargo build -p bisect-cli --example plan_doctor_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native plan Doctor reference build failed.' }
    node scripts/web/test_plan_doctor_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Plan Doctor native/WASM artifact inventory parity failed.' }
    cargo test -p bisect-tui --lib --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI regression failed.' }
    cargo test -p bisect-report tui_library --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Shared TUI library rules failed.' }
    cargo build -p bisect-tui --example plan_library_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI library reference build failed.' }
    node scripts/web/test_tui_library_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI library WASM parity failed.' }
    cargo test -p bisect-report tui_session --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Shared native TUI session rules failed.' }
    cargo build -p bisect-tui --example session_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native session reference build failed.' }
    node scripts/web/test_tui_session_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI session WASM parity failed.' }
    cargo test -p bisect-report tui_compare --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Shared native TUI Compare rules failed.' }
    cargo build -p bisect-tui --example compare_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI Compare reference build failed.' }
    node scripts/web/test_tui_compare_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI Compare WASM parity failed.' }
    node scripts/web/test_tui_library_compare.mjs
    if($LASTEXITCODE -ne 0){throw 'TUI library assignment-to-Compare handoff failed.'}
    cargo test -p bisect-report tui_doctor --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Shared native TUI Doctor rules failed.' }
    cargo build -p bisect-tui --example tui_doctor_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI Doctor reference build failed.' }
    node scripts/web/test_tui_doctor_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI Doctor WASM parity failed.' }
    cargo test -p bisect-report tui_verify --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI Verify parser rules failed.' }
    cargo build -p bisect-cli --example format_validation_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native format validation reference build failed.' }
    node scripts/web/test_format_validation_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native RPLAN format and strict warning policy parity failed.' }
    cargo build -p bisect-cli --example suite_assembly_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native suite assembly reference build failed.' }
    cargo build -p bisect-cli --example civic_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Civic native reference build failed.' }
    node scripts/web/test_civic_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Civic native/WASM verification failed.' }
    cargo build -p bisect-cli --example label_inventory_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native label inventory reference build failed.' }
    cargo test -p bisect-cli --no-default-features --lib import_label::tests::test_label_compare --locked --offline -j 1 -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Native label comparison regressions failed.' }
    cargo build -p bisect-cli --example label_import_reference --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native label import filesystem reference build failed.' }
    cargo test -p bisect-cli --no-default-features --lib import_label --locked --offline -j 1 -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Native label import regressions failed.' }
    cargo build -p bisect-cli --example plan_hub_reference --example plan_hub_child --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native plan hub reference build failed.' }
    cargo test -p bisect-cli --no-default-features --lib plan_cmd --locked --offline -j 1 -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Native plan command regressions failed.' }
    cargo check -p bisect-tui --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI label selection check failed.' }
    node scripts/web/test_plan_hub_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native plan hub WASM/dispatch verification failed.' }
    node scripts/web/test_label_import_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "Native label import verification failed." }
    cargo test -p bisect-report --no-default-features --features label-import context_tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw "Imported context alignment tests failed." }
    node scripts/web/test_label_import_plan_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "Imported plan WASM verification failed." }
    python scripts/web/test_label_import_plan_zip.py
    if ($LASTEXITCODE -ne 0) { throw "Imported-plan ZIP verification failed." }
    python scripts/web/test_label_import_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Native label import ZIP verification failed.' }
    node scripts/web/test_label_comparison_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native label comparison readiness parity failed.' }
    node scripts/web/test_label_inventory_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native label list/show and inert project parity failed.' }
    cargo build -p bisect-cli --example label_rename_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native label rename reference build failed.' }
    node scripts/web/test_label_rename_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native label rename package parity failed.' }
    python scripts/web/test_label_rename_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Native label rename ZIP parity failed.' }
    cargo build -p bisect-cli --example label_chain_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native label SHA-chain reference build failed.' }
    node scripts/web/test_label_chain_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native label SHA-chain parity failed.' }
    cargo build -p bisect-cli --example migration_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native migration reference build failed.' }
    node scripts/web/test_migration_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Legacy migration native parity failed.' }
    python scripts/web/test_migration_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Legacy migration ZIP parity failed.' }
    cargo build -p bisect-cli --example candidate_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native candidate-race reference build failed.' }
    node scripts/web/test_candidate_ingest_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Candidate-race package native parity failed.' }
    python scripts/web/test_candidate_ingest_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Candidate-race package ZIP readback failed.' }
    node scripts/web/test_civic_ingest_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native civic ingestion package parity failed.' }
    python scripts/web/test_civic_ingest_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Native civic package ZIP readback failed.' }
    node scripts/web/test_civic_inventory_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Civic List/Show native parity failed.' }
    python scripts/web/test_civic_inventory_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Civic List/Show ZIP verification failed.' }
    python scripts/web/test_civic_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Civic ZIP verification failed.' }
    node scripts/web/test_suite_assembly_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native suite assembly and portable readback parity failed.' }
    python scripts/web/test_suite_assembly_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Suite ZIP byte integrity and native readback failed.' }
    cargo build -p bisect-cli --example county_weights_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native county-weighter reference build failed.' }
    cargo build -p bisect-wasm --release --example execute_request --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native engine reference build failed.' }
    node scripts/web/test_county_weights_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native county weighting semantics verification failed.' }
    cargo build -p bisect-cli --example minority_weights_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native minority-weighter reference build failed.' }
    cargo build -p bisect-cli --example config_seed_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native optional-seed dispatch reference build failed.' }
    node scripts/web/test_minority_weights_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native minority weights and project evidence parity failed.' }
    cargo test -p bisect-cli algo_config --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native algorithm configuration regression failed.' }
    cargo test -p bisect-cli yaml_build_tolerance --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native YAML build population tolerance regression failed.' }
    node scripts/web/test_config_inspection_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Configuration schema native/WASM inspection parity failed.' }
    cargo build -p bisect-cli --example config_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native configuration conversion reference build failed.' }
    node scripts/web/test_config_validation_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Configuration compositor and native summary parity failed.' }
    cargo build -p bisect-cli --example config_template_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native config template reference build failed.' }
    node scripts/web/test_config_template_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native YAML template parity failed.' }
    cargo build -p bisect-cli --example config_area_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native YAML AreaSection reference build failed.' }
    node scripts/web/test_config_area_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native YAML AreaSection execution parity failed.' }
    cargo build -p bisect-cli --release --example config_seed_reference --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native release optional-seed dispatch reference build failed.' }
    node scripts/web/test_optional_seed_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native optional-seed and exact-u64 execution parity failed.' }
    node scripts/web/test_config_run_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Configured YAML execution and native dispatch parity failed.' }
    cargo test -p bisect-apportion contiguity_tests --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Pure Rust recursive partition connectivity contract failed.' }
    node scripts/web/test_config_run_ui.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Configured YAML UI source, map and project roundtrip failed.' }
    node scripts/web/test_config_export_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Configured native export and text audit parity failed.' }
    node scripts/web/test_config_build_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native configured balance retries and accepted seed export failed.' }
    cargo test -p bisect-report tui_run --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Shared native TUI Run form rules failed.' }
    cargo test -p bisect-runner run_form_tests --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native Run form compositor override preservation failed.' }
    cargo build -p bisect-cli --example tui_run_reference --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI Run YAML reference build failed.' }
    node scripts/web/test_tui_run_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI Run form/engine/export parity failed.' }
    node scripts/web/test_state_workflow_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Optional accepted-plan native/WASM workflow parity failed.' }
    cargo test -p bisect-runner --lib coi_weights::tests --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Shared native COI parser/adjustment validation failed.' }
    cargo test -p bisect-cli test_coi_weights --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native COI selected-file regressions failed.' }
    cargo build -p bisect-cli --no-default-features --example coi_weights_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native COI file reference build failed.' }
    node scripts/web/test_coi_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'COI native file/kernel/assignment/export/replay parity failed.' }
    if ($BrowserChecks) {
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
        node scripts/web/test_tui_run_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native TUI Run browser workflow failed.' }
        node scripts/web/test_state_workflow_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Optional accepted-plan browser workflow failed.' }
        node scripts/web/test_coi_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'COI selected-source browser workflow failed.' }
    }
    node scripts/web/test_precinct_engine_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Explicit precinct/imported county identity native/WASM verification failed.' }
    node scripts/web/test_unit_kind_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Prepared Census unit-kind engine/export interoperability failed.' }
    cargo build -p bisect-cli --no-default-features --bin bisect --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native certified context CLI build failed.' }
    node scripts/web/test_certified_context_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Certified context native CLI/WASM package parity failed.' }
    node scripts/web/test_canonical_context_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Canonical context native CLI/WASM package parity failed.' }
    cargo test -p rgraph-core --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Portable discovery articulation graph tests failed.' }
    cargo test -p bisect-cli --no-default-features --lib exact_cmd::tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Shared native discovery and Exact command tests failed.' }
    node scripts/web/test_certified_discovery_context_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Discovery context native CLI/WASM package parity failed.' }
    cargo build -p bisect-ilp --no-default-features --example certified_model_package --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native compact proof model compiler reference build failed.' }
    cargo test -p bisect-ilp --no-default-features proof_backend::tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native compact proof backend tests failed.' }
    node scripts/web/test_proof_models_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Compact proof model native/WASM files and identity parity failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_certified_context_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Certified context browser verification failed.' }
        node scripts/web/test_canonical_context_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Canonical context browser workflow failed.' }
        node scripts/web/test_certified_discovery_context_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Discovery context browser workflow failed.' }
        node scripts/web/test_proof_models_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Compact proof model browser workflow failed.' }
        node scripts/web/test_precinct_engine_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Explicit precinct/imported Run/map browser verification failed.' }
        node scripts/web/test_unit_kind_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Prepared Census unit-kind browser workflow failed.' }
    }
    node scripts/web/test_label_build_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Multi-year label build engine and artifact parity failed.' }
    node scripts/web/test_label_build_progress.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Label build progress callback and worker relay failed.' }
    node scripts/web/test_label_build_files.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Label build checkpoint retention and project parity failed.' }
    node scripts/web/test_label_build_archive.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Build archive codec and worker validation failed.' }
    node scripts/web/test_label_build_start.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Build source worker preflight and engine handoff failed.' }
    cargo run -p bisect-report --example report_package_oracle --locked --offline -- target/commission-report-wasm target/label-build-wasm/browser-archive.bisect-build
    if ($LASTEXITCODE -ne 0) { throw 'Native commission report package oracle failed.' }
    node scripts/web/test_commission_report_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Commission report WASM byte parity failed.' }
    cargo build -p bisect-cli --bin bisect --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native commission audit CLI build failed.' }
    node scripts/web/test_commission_fallback_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Commission fallback CLI parity failed.' }
    cargo build -p bisect-wasm --release --example plan_contiguity_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native contiguity reference build failed.' }
    node scripts/web/test_plan_contiguity_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native contiguity report parity failed.' }
    node scripts/web/test_analysis_collection.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Retained analysis/report handoff failed.' }
    node scripts/web/test_plan_report_manifest.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Generated native manifest/report workflow failed.' }
    cargo build -p bisect-wasm --release --example plan_splits_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native splits reference build failed.' }
    node scripts/web/test_smc_real_state_wasm.mjs $Output
    if ($LASTEXITCODE -ne 0) { throw 'Original real-state SMC fixture and native parity gate failed.' }
    node scripts/web/test_plan_splits_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native splits parity failed.' }
    cargo test -p bisect-analysis summary::tests --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native measured population summary validation failed.' }
    node scripts/web/test_population_summary_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Matching-context population summary and report continuity failed.' }
    node scripts/web/test_analysis_package_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Combined native analysis package verification failed.' }
    python scripts/web/test_analysis_package_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Independent analysis ZIP verification failed.' }
    node scripts/web/test_external_label_analysis_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "External label analysis verification failed." }
    python scripts/web/test_external_label_analysis_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Native label import WASM/file package parity failed.' }
    node scripts/web/test_label_analysis_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Label analysis orchestration verification failed.' }
    python scripts/web/test_label_analysis_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Independent label analysis ZIP verification failed.' }
    node scripts/web/test_label_report_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Label report orchestration verification failed.' }
    python scripts/web/test_label_report_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Independent label report ZIP verification failed.' }
    node scripts/web/test_label_stage_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native label registry stages verification failed.' }
    python scripts/web/test_label_stage_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Independent stage registry ZIP verification failed.' }
    node scripts/web/test_label_analysis_years_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Multi-year label analysis native/WASM parity failed.' }
    python scripts/web/test_label_analysis_years_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Multi-year label analysis ZIP verification failed.' }
    node scripts/web/test_label_report_years_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Multi-year reports native/WASM parity failed.' }
    python scripts/web/test_label_report_years_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Multi-year reports ZIP verification failed.' }
    cargo test -p bisect-wasm --release --lib label_years_resume::tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Checkpoint JSON numeric equivalence guard failed.' }
    node scripts/web/test_label_years_resume_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Verified native year resume parity/security failed.' }
    python scripts/web/test_label_years_resume_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Resumed native year ZIP verification failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_plan_hub_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Plan hub browser verification failed.' }
        node scripts/web/test_label_import_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw "Label import browser verification failed." }
        node scripts/web/test_label_import_plan_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw "Imported plan browser verification failed." }
        python scripts/web/test_label_import_plan_zip.py --browser
        if ($LASTEXITCODE -ne 0) { throw "Imported-plan browser ZIP verification failed." }
        node scripts/web/test_external_label_analysis_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw "External label browser verification failed." }
        python scripts/web/test_external_label_analysis_zip.py --browser
        if ($LASTEXITCODE -ne 0) { throw 'Actual label import browser checks failed.' }
        python scripts/web/test_label_import_zip.py --browser
        if ($LASTEXITCODE -ne 0) { throw 'Actual browser label import ZIP checks failed.' }
        node scripts/web/test_label_comparison_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Actual label comparison browser checks failed.' }
        node scripts/web/test_label_pipeline_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Actual browser label Build/analysis/report pipeline failed.' }
        python scripts/web/test_label_pipeline_browser_zip.py
        if ($LASTEXITCODE -ne 0) { throw 'Actual browser label pipeline ZIP byte/chain verification failed.' }
        python scripts/web/test_label_analysis_years_zip.py --browser
        if ($LASTEXITCODE -ne 0) { throw 'Actual browser multi-year analysis ZIP verification failed.' }
        python scripts/web/test_label_report_years_zip.py --browser
        if ($LASTEXITCODE -ne 0) { throw 'Actual browser multi-year reports ZIP verification failed.' }
    }
    node scripts/web/test_label_reconcile_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Registry snapshot reconciliation verification failed.' }
    node scripts/web/test_label_package_merge_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "Label package merge WASM gate failed." }
    cargo build -p bisect-cli --bin bisect --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw "Native ILP audit CLI build failed." }
    node scripts/web/test_ilp_audit_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "ILP audit native/WASM gate failed." }
    node scripts/web/test_package_verification_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "RPLAN package native/WASM verification gate failed." }
    node scripts/web/test_manifest_certificate_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native manifest certificate verification parity failed.' }
    cargo test -p bisect-wasm --lib assignment_discovery --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Selected original assignment discovery kernel tests failed.' }
    node scripts/web/test_assignment_discovery_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Original assignment discovery WASM verification failed.' }
    node scripts/web/test_assignment_verification_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native assignment verification parity failed.' }
    cargo test -p bisect-cli --no-default-features --lib sweep --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native Sweep regressions failed.' }
    cargo test -p bisect-core --lib acs_housing_import --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'ACS housing shared parser regressions failed.' }
    cargo build -p bisect-cli --no-default-features --example acs_housing_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native ACS housing reference build failed.' }
    cargo test -p bisect-core --lib lodes_import --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Shared LODES aggregation regressions failed.' }
    cargo test -p bisect-cli --no-default-features --lib lodes --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native LODES regressions failed.' }
    cargo build -p bisect-cli --no-default-features --example lodes_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native LODES reference build failed.' }
    cargo test -p bisect-cli --no-default-features --lib fetch::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native Fetch planning regressions failed.' }
    cargo build -p bisect-cli --no-default-features --example fetch_plan_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native Fetch planner reference build failed.' }
    cargo build -p bisect-cli --no-default-features --example fetch_data_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native Fetch data reference build failed.' }
    cargo build -p bisect-cli --no-default-features --example fletch_cache_reference -j 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'FLETCH cache native reference build failed.' }
    cargo build -p bisect-cli --no-default-features --example fletch_sources_reference -j 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'FLETCH source native reference build failed.' }
    cargo build -p bisect-cli --no-default-features --example fletch_verify_reference -j 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'FLETCH cache verification native reference build failed.' }
    node scripts/web/test_fletch_verify_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'FLETCH cache-file verification failed.' }
    cargo build -p bisect-cli --no-default-features --example fletch_merge_reference -j 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native FLETCH manifest merge reference build failed.' }
    node scripts/web/test_fletch_merge_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'FLETCH manifest merge native/WASM parity failed.' }
    cargo build -p bisect-cli --no-default-features --example fletch_prune_reference -j 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native cache prune reference build failed.' }
    node scripts/web/test_fletch_prune_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native/WASM cache prune parity failed.' }
    cargo build -p bisect-cli --no-default-features --example fletch_request_reference -j 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native cache request reference build failed.' }
    node scripts/web/test_fletch_request_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native/WASM immutable cache request parity failed.' }
    cargo build -p bisect-cli --no-default-features --example fletch_retry_reference -j 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native HTTP retry reference build failed.' }
    node scripts/web/test_fetch_network.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Source retry/timeout native parity failed.' }
    node scripts/web/test_fletch_sources_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'FLETCH source handoff native/WASM validation failed.' }
    node scripts/web/test_fletch_cache_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'FLETCH cache native/WASM validation failed.' }
    node scripts/web/test_fetch_plan_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Fetch planning native/WASM validation failed.' }
    node scripts/web/test_fetch_source_coverage.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Fetch source-type/consumer reconciliation drifted.' }
    cargo build -p bisect-cli --no-default-features --example fetch_archive_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Fetch archive/native reference build failed.' }
    $fetchArchiveCheck=node scripts/web/test_fetch_archive_acquisition.mjs | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Fetch archive acquisition/native checks failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_fetch_archive_acquisition_browser.mjs $fetchArchiveCheck.root
        if ($LASTEXITCODE -ne 0) { throw 'Fetch archive acquisition browser checks failed.' }
    }
    node scripts/web/test_lodes_gzip_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'LODES gzip native/WASM validation failed.' }
    node scripts/web/test_lodes_source_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'LODES source native/WASM/browser helper verification failed.' }
    node scripts/web/test_acs_housing_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'ACS housing native/WASM/browser helper verification failed.' }
    node scripts/web/test_fetch_acquisition.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Browser public source acquisition validation failed.' }
    node scripts/web/test_fetch_data_package_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Derived Fetch data native/WASM validation failed.' }
    python scripts/web/test_fetch_data_package_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Derived Fetch data ZIP validation failed.' }
    cargo build -p bisect-cli --no-default-features --example pl94_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native PL94 reference build failed.' }
    cargo test -p bisect-data pl94::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native PL94 regressions failed.' }
    node scripts/web/test_pl94_source_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'PL94 native/WASM/source verification failed.' }
    cargo test -p bisect-data tiger::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native TIGER reader regressions failed.' }
    cargo build -p bisect-cli --no-default-features --example tiger_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native TIGER reference build failed.' }
    node scripts/web/test_tiger_source_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'TIGER native/WASM/source verification failed.' }
    cargo test -p bisect-data adjacency::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native tract adjacency regressions failed.' }
    cargo build -p bisect-cli --no-default-features --example tract_prepare_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native tract preparation reference build failed.' }
    python scripts/web/census_endpoints.py --check
    if ($LASTEXITCODE -ne 0) { throw 'Census endpoint manifest has drifted.' }
    node scripts/web/test_census_archive.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Census archive validation failed.' }
    node scripts/web/test_census_zip_acquisition.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Census ZIP acquisition verification failed.' }
    cargo build -p bisect-cli --no-default-features --example fletch_stage_reference -j 1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native FLETCH raw source HTTP reference build failed.' }
    node scripts/web/test_fletch_stage_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Raw FLETCH cache staging native/WASM verification failed.' }
    node scripts/web/test_fletch_history_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "Fetch history WASM validation failed" }
    python scripts/web/test_fletch_stage_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Portable FLETCH cache ZIP verification failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_tui_doctor_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native TUI Doctor browser workflow failed.' }
        node scripts/web/test_tui_compare_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native TUI Compare browser workflow failed.' }
        node scripts/web/test_tui_session_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native TUI session browser workflow failed.' }
        node scripts/web/test_tui_library_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'TUI library browser workflow failed.' }
        node scripts/web/test_tui_library_compare_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'TUI library Compare handoff browser workflow failed.'}
        node scripts/web/test_state_completion_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native state completion browser source/freshness verification failed.' }
        node scripts/web/test_state_batch_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native state batch browser workflow failed.' }
        node scripts/web/test_native_engine_input_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native binary engine browser handoff failed.' }
        node scripts/web/test_native_engine_input_browser.mjs --v2
        if ($LASTEXITCODE -ne 0) { throw 'Native supplemental graph browser validation failed.' }
        node scripts/web/test_native_engine_input_browser.mjs --auxiliary
        if ($LASTEXITCODE -ne 0) { throw 'Native binary graph auxiliary-input/options handoff failed.' }
        node scripts/web/test_native_adjacency_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native binary adjacency browser verification failed.' }
        node scripts/web/test_fletch_cache_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Real-browser cache persistence and workbench verification failed.' }
        python scripts/web/test_fletch_cache_backup_zip.py
        if ($LASTEXITCODE -ne 0) { throw 'Independent browser-cache backup ZIP verification failed.' }
        node scripts/web/test_fletch_cache_backup_zip.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Browser-cache backup ZIP security guards failed.' }
    }
    cargo build -p bisect-cli --no-default-features --example tract_prepare_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native tract preparation reference build failed.' }
    $tractPreparationArguments = @()
    if ($LocalCensusChecks) { $tractPreparationArguments += '--local-ri' }
    node scripts/web/test_tract_preparation_wasm.mjs @tractPreparationArguments
    if ($LASTEXITCODE -ne 0) { throw 'Tract preparation native/WASM/filesystem verification failed.' }
    if ($LocalCensusChecks) {
        node scripts/web/test_native_engine_input_wasm.mjs --local-ri
        if ($LASTEXITCODE -ne 0) { throw 'Original RI native binary graph preparation failed.' }
        if ($BrowserChecks) {
            node scripts/web/test_native_engine_input_browser.mjs --local-ri --auxiliary
            if ($LASTEXITCODE -ne 0) { throw 'Original RI native binary graph auxiliary-input/options browser handoff failed.' }
        }
    }
    cargo test -p bisect-data tract_preparation::census_tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Shared Census resolution population validation failed.' }
    cargo build -p bisect-cli --no-default-features --example census_prepare_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native Census preparation reference build failed.' }
    cargo test -p rplan-io --lib compact_context_preserves_native_fields_hash_and_tamper_rejection --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Lossless compact native context writing failed.' }
    node scripts/web/test_census_preparation_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Four-resolution Census native/WASM preparation failed.' }
    cargo build -p bisect-data --example census_source_geometry --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native exact-source geometry reader build failed.' }
    node scripts/web/test_large_geometry_profile.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Large geometry root profile and source bounds failed.' }
    node scripts/web/test_source_geometry_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Exact-source native/WASM geometry parity failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_census_preparation_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Census source-to-Run browser workflow failed.' }
    }
    node scripts/web/test_prepared_graph_capacity.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Bounded prepared-graph and large-archive parser security failed.' }
    if ($LocalCensusChecks) {
        foreach ($censusKind in @('county','block-group','block')) {
            node scripts/web/test_real_census_preparation_wasm.mjs $censusKind
            if ($LASTEXITCODE -ne 0) { throw "Original RI $censusKind preparation and native Run failed." }
            if ($BrowserChecks) {
                node scripts/web/test_real_census_run_browser.mjs $censusKind
                if ($LASTEXITCODE -ne 0) { throw "Original RI $censusKind large browser archive/Run failed." }
            }
        }
        node scripts/web/test_real_county_archive.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Original national county ZIP preservation failed.' }
        node scripts/web/test_source_geometry_wasm.mjs --real --native-host
        if ($LASTEXITCODE -ne 0) { throw 'Real exact-source geometry workflow failed.' }
        if ($BrowserChecks) {
            node scripts/web/test_source_geometry_browser.mjs
            if ($LASTEXITCODE -ne 0) { throw 'Real exact-source geometry browser workflow failed.' }
        }
        node scripts/web/test_large_source_geometry_wasm.mjs --native-host
        if ($LASTEXITCODE -ne 0) { throw 'Original RI block larger exact-source workflow failed.' }
        node scripts/web/test_large_source_job.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Original block source-job recipe and archive bounds failed.' }
        python scripts/web/test_large_source_geometry_zip.py
        if ($LASTEXITCODE -ne 0) { throw 'Original RI block larger-source ZIP integrity failed.' }
        if ($BrowserChecks) {
            node scripts/web/test_large_source_geometry_browser.mjs
            if ($LASTEXITCODE -ne 0) { throw 'Original RI block larger-source browser workflow failed.' }
        }
        node scripts/web/test_real_state_workflow_wasm.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Real Census accepted-plan follow-up evidence failed.' }
        node scripts/web/test_real_census_context_wasm.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Original RI complete context native/WASM reader parity failed.' }
    }
    node scripts/web/test_state_source_files.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Paired state source validation failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_state_source_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Paired state source browser validation failed.' }
    }
    node scripts/web/test_tiger_gzip_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Compressed TIGER source identity, reader parity and expansion bounds failed.' }
    node scripts/web/test_pl94_large_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Large PL94 decoded stream native parity and source archive checks failed.' }
    node scripts/web/test_workbench_archive.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Workbench archive validation failed.' }
    node scripts/web/test_workbench_shared_archive.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Shared workbench archive exact roundtrip and reference bounds failed.' }
    node --max-old-space-size=4096 scripts/web/test_compressed_workbench_archive.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Compressed workbench exact roundtrip, checksum and expansion guards failed.' }
    $sharedArchiveBrowserArgs = @()
    if ($LocalCensusChecks) {
        node scripts/web/test_workbench_shared_archive_real.mjs --native-host
        if ($LASTEXITCODE -ne 0) { throw 'Original block multi-panel archive and native analysis failed.' }
        $sharedArchiveBrowserArgs += '--real'
    }
    if ($BrowserChecks) {
        node scripts/web/test_workbench_shared_archive_browser.mjs @sharedArchiveBrowserArgs
        if ($LASTEXITCODE -ne 0) { throw 'Shared workbench archive worker/UI roundtrip and inert import failed.' }
    }
    node scripts/web/test_sweep_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Sweep command planning native/WASM/CLI verification failed.' }
    node scripts/web/test_multiscale_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Portable multiscale native/WASM boundary verification failed.' }
    node scripts/web/test_engine_replay_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Engine replay validation failed.' }
    node scripts/web/test_assignment_discovery_ui.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Discovery folder/project/handoff helper verification failed.' }
    node scripts/web/test_manifest_prechecks_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native manifest prechecks parity failed.' }
    node scripts/web/test_verify_assignments_only_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native assignments-only orchestration verification failed.' }
    cargo test -p bisect-wasm --lib replay_bindings --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native replay correspondence context test failed.' }
    node scripts/web/test_replay_multiscale_bindings_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Multiscale replay context binding failed.' }
    node scripts/web/test_replay_aux_bindings_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Replay auxiliary source binding failed.' }
    node scripts/web/test_replay_bindings_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Replay correspondence binding failed.' }
    node scripts/web/test_manifest_replay_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Manifest/replay composition failed.' }
    node scripts/web/test_verify_binary_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Composed replay binary advisory verification failed.' }
    node scripts/web/test_tui_verify_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native TUI staged verification WASM parity failed.' }
    node scripts/web/test_tui_library_verify.mjs
    if($LASTEXITCODE -ne 0){throw 'Selected library verification handoff failed.'}
    if ($BrowserChecks) {
        node scripts/web/test_verify_binary_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Composed replay binary advisory browser verification failed.' }
        node scripts/web/test_tui_verify_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native TUI Verify browser workflow failed.' }
        node scripts/web/test_tui_library_verify_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Selected library verification browser workflow failed.'}
    }
    python scripts/web/test_label_package_merge_zip.py
    if ($LASTEXITCODE -ne 0) { throw "Label package ZIP gate failed." }
    node scripts/web/test_project.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Project save/load verification failed.' }
    node scripts/web/test_state_batch_zip.mjs
    if ($LASTEXITCODE -ne 0) { throw 'State batch ZIP format/resource guards failed.' }
    node scripts/web/test_build_plan_handoff.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Build practitioner handoff failed.' }
    node scripts/web/test_commission_delivery_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Commission native report delivery parity failed.' }
    python scripts/web/test_commission_delivery_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Commission ZIP native file/byte integrity failed.' }
    node scripts/web/test_commission_fallback_files.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Commission fallback source/project verification failed.' }
    node scripts/web/test_commission_audit_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Commission audit-only/companion CLI parity failed.' }
    node scripts/web/test_commission_files.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Commission report source/project verification failed.' }
    python scripts/web/test_label_build_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Label build ZIP integrity and exact files failed.' }
    node scripts/web/test_certified_exact_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Certified exact split/tree parity and proof verification failed.' }
    cargo test -p bisect-column --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native complete-column input contract failed.' }
    cargo test -p bisect-pareto --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Portable Pareto kernel tests failed.' }
    node scripts/web/test_pareto_selection_wasm.mjs $Output
    if ($LASTEXITCODE -ne 0) { throw 'Pareto selected native package parity failed.' }
    node scripts/web/test_pareto_context_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Pareto native/WASM frontier parity failed.' }
    node scripts/web/test_smc_context_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "SMC context WASM parity failed" }
    node scripts/web/test_smc_selection_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "SMC selection WASM parity failed" }
    node scripts/web/test_improvement_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw "Improvement WASM parity failed" }
    node scripts/web/test_plan_import_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native plan import parity failed.' }
    node scripts/web/test_reproducibility_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native reproducibility-package parity failed.' }
    cargo test -p rstat-core mcmc --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native MCMC diagnostic checks failed.' }
    cargo build -p bisect-cli --bin bisect --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native research CLI reference build failed.' }
    node scripts/web/test_research_ensemble_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Research diagnostic native/WASM and CLI parity failed.' }
    cargo test -p bisect-analysis nesting::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native suite nesting checks failed.' }
    cargo test -p bisect-analysis race_of_candidate --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native candidate annotation checks failed.' }
    cargo test -p bisect-analysis bloc_voting --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native bloc-voting kernel/writer/input checks failed.' }
    cargo test -p bisect-cli --test bloc_voting_l1 --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native bloc-voting CLI/seed checks failed.' }
    cargo build -p bisect-wasm --release --example bloc_analysis_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native bloc report reference build failed.' }
    node scripts/web/test_bloc_analysis_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Bloc native/WASM report parity failed.' }
    python scripts/web/test_bloc_evidence_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Bloc evidence ZIP byte verification failed.' }
    cargo test -p bisect-analysis district_geometry --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native geometry report checks failed.' }
    cargo test -p bisect-web --bin wasm-analysis-geometry --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Unsimplified TIGER boundary preparation checks failed.' }
    cargo test -p bisect-analysis compactness --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native compactness kernel checks failed.' }
    cargo test -p bisect-map dissolve --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native shared dissolve checks failed.' }
    cargo build -p bisect-wasm --release --example compactness_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native compactness reference build failed.' }
    cargo test -p bisect-analysis population_geometry --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native population centroid compactness tests failed.' }
    node scripts/web/test_plan_compactness_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Compactness native/WASM and report parity failed.' }
    cargo test -p bisect-map --lib projection::tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native state/national display projection tests failed.' }
    cargo test -p bisect-map --lib renderer::tests --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native state map layout tests failed.' }
    cargo build -p bisect-cli --no-default-features --example native_map_reference --example native_national_map_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native map reference build failed.' }
    node scripts/web/test_multipart_contact_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Original Alaska multipart geometry native/WASM parity failed.' }
    node scripts/web/test_native_map_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native map SVG/PNG parity failed.' }
    node scripts/web/test_native_choropleth_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native choropleth SVG/PNG parity failed.' }
    node scripts/web/test_native_round_map_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native recorded round map parity failed.' }
    cargo test -p bisect-runner round_capture --no-default-features --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native round capture boundaries failed.' }
    cargo build -p bisect-wasm --example execute_request --example execute_tool_request --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Round capture reference build failed.' }
    node scripts/web/test_round_capture_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Round capture native/WASM parity failed.' }
    node scripts/web/test_batch_round_capture_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Batch round capture native/WASM parity failed.' }
    python scripts/web/test_batch_round_capture_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'Batch round capture ZIP integrity failed.' }
    node scripts/web/test_antimeridian_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Antimeridian state/national map parity failed.' }
    cargo test -p bisect-wasm --lib national_map_stream --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native national map session validation failed.' }
    cargo build -p bisect-wasm --example execute_tool_sequence --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native national stream reference build failed.' }
    node scripts/web/test_national_map_package_capacity.mjs
    if ($LASTEXITCODE -ne 0) { throw 'National map package pre-read bounds failed.' }
    node scripts/web/test_national_map_stream_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native national stream WASM parity failed.' }
    node scripts/web/test_native_national_map_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native national inset SVG/PNG parity failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_multipart_contact_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Original Alaska multipart map browser workflow failed.' }
        node scripts/web/test_native_map_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native map browser workflow failed.' }
        node scripts/web/test_native_round_map_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native recorded round browser workflow failed.' }
        node scripts/web/test_round_capture_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Round capture browser workflow failed.' }
        node scripts/web/test_batch_round_capture_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Batch round capture browser workflow failed.' }
        node scripts/web/test_antimeridian_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Antimeridian state/national map browser validation failed.' }
        node scripts/web/test_national_map_stream_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native national stream browser validation failed.' }
        node scripts/web/test_native_national_map_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native national map browser workflow failed.' }
        foreach ($mapType in @('political','demographic','compactness')) {
            node scripts/web/test_native_map_browser.mjs "--map-type=$mapType"
            if ($LASTEXITCODE -ne 0) { throw "Native $mapType browser workflow failed." }
        }
    }
    node scripts/web/test_state_geometry_wasm.mjs $Output
    if ($LASTEXITCODE -ne 0) { throw 'Prepared state geometry rejection and synthetic capacity parity failed.' }
    $fullStateCompactnessInputs = @(
        'target/analysis-geometry-source-v1-RI/unit-geometry.json',
        'target/analysis-geometry-source-v1-RI/provenance.json',
        'target/smc-real-state-wasm/RI-selected.rplan'
    )
    $missingFullStateInputs = @($fullStateCompactnessInputs | Where-Object { -not (Test-Path -LiteralPath $_ -PathType Leaf) })
    if ($missingFullStateInputs.Count -eq 0) {
        node scripts/web/test_full_state_compactness_wasm.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Full-source Rhode Island compactness native/WASM and report parity failed.' }
        $fullIowaInputs = @(
            'target/analysis-geometry-source-v1-IA/unit-geometry.json',
            'target/analysis-geometry-source-v1-IA/provenance.json',
            'target/smc-real-state-wasm/IA-selected.rplan'
        )
        $missingIowaInputs = @($fullIowaInputs | Where-Object { -not (Test-Path -LiteralPath $_ -PathType Leaf) })
        if ($missingIowaInputs.Count -eq 0) {
            node scripts/web/test_full_iowa_geometry_wasm.mjs
            if ($LASTEXITCODE -ne 0) { throw 'Full-source Iowa admission, bounded rejection and recovery verification failed.' }
            node scripts/web/test_full_state_batch_wasm.mjs
            if ($LASTEXITCODE -ne 0) { throw 'Full-source RI/IA batch compactness native/WASM, report bytes and independent area checks failed.' }
        } else {
            Write-Host "Full-source Iowa geometry probe not run; missing inputs: $($missingIowaInputs -join ', ')"
        }
    } else {
        Write-Host "Full-source Rhode Island compactness probe not run; missing inputs: $($missingFullStateInputs -join ', ')"
    }
    cargo test -p bisect-analysis national --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native national CSV checks failed.' }
    cargo test -p bisect-cli aggregate::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native CLI aggregation regression checks failed.' }
    cargo build -p bisect-wasm --release --example national_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native national export reference build failed.' }
    node scripts/web/test_national_analysis_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'National report native/WASM and export byte parity failed.' }
    cargo test -p bisect-analysis demographic::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native demographic checks failed.' }
    cargo test -p bisect-analysis urban::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native urban checks failed.' }
    cargo test -p bisect-analysis summary::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native summary checks failed.' }
    cargo build -p bisect-wasm --release --example plan_statistics_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native statistics reference build failed.' }
    node scripts/web/test_plan_statistics_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Statistics native/WASM and report parity failed.' }
    cargo test -p rstat-core resampling --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Portable bootstrap protocol checks failed.' }
    cargo test -p bisect-cli partisan::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native partisan runner/bootstrap record checks failed.' }
    cargo test -p bisect-analysis partisan::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native partisan metric checks failed.' }
    cargo test -p bisect-analysis political::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native political analysis checks failed.' }
    cargo test -p bisect-analysis proportionality::tests --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native proportionality analysis checks failed.' }
    cargo build -p bisect-wasm --release --example election_analysis_reference --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native election analyzer reference build failed.' }
    node scripts/web/test_election_analysis_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Election analysis native/WASM parity failed.' }
    cargo test -p bisect-analysis suite_stability --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native suite stability checks failed.' }
    node scripts/web/test_suite_stability_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Suite stability native/WASM and CLI parity failed.' }
    node scripts/web/test_suite_nesting_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Suite nesting native/WASM and CLI parity failed.' }
    node scripts/web/test_polygon_import_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'District polygon import parity failed.' }
    node scripts/web/test_interchange_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native interchange export parity failed.' }
    node scripts/web/test_plan_map_wasm.mjs $Output
    if ($LASTEXITCODE -ne 0) { throw 'Plan map and exact validation verification failed.' }
    node scripts/web/test_text_audit_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Exact text audit/certificate native parity and count binding failed.' }
    node scripts/web/test_branch_price_context_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'RCTX column native/WASM context, plan export and integer checks failed.' }
    node scripts/web/test_branch_price_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Branch-price native/WASM parity and partition oracle failed.' }
    node scripts/web/test_ceremony_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Ceremony native/WASM record and proof verification failed.' }
    cargo test -p rplan-audit --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Shared audit/measurement regression tests failed.' }
    cargo test -p bisect-analysis --lib comparison --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native comparison kernel tests failed.' }
    cargo test -p bisect-report comparison --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Native report assignment-change regression tests failed.' }
    cargo run -p bisect-wasm --example comparison_fixtures --locked --offline -- target/comparison-fixtures
    if ($LASTEXITCODE -ne 0) { throw 'Native comparison contexts failed.' }
    cargo test -p bisect-report --lib --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Portable report renderer tests failed.' }
    node scripts/web/test_report_plan_evidence_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Report plan/context measurement parity and exact population verification failed.' }
    node scripts/web/test_report_comparison_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Imported comparison report rendering and byte receipt verification failed.' }
    node scripts/web/test_report_side_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Imported report reader native/WASM verification failed.' }
    node scripts/web/test_plan_comparison.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Plan comparison native/WASM and project verification failed.' }
    node scripts/web/test_lab_multiscale_coi.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Laboratory multiscale COI source/project/native parity failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_lab_multiscale_coi_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Laboratory COI actual worker/source UI verification failed.' }
    }
    node scripts/web/test_coi_multiscale_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Multiscale tract-seed COI and exact replay correspondence parity failed.' }
    node scripts/web/test_multiscale_seed_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Multiscale declared/effective replay seed verification failed.' }
    cargo build -p bisect-cli --no-default-features --example electoral_settings_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native State electoral settings oracle build failed.' }
    node scripts/web/test_electoral_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Electoral settings/configured context native/WASM parity failed.' }
    cargo build -p bisect-cli --no-default-features --example run_target_reference --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native target folder guard oracle build failed.' }
    node scripts/web/test_run_target_wasm.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native Run target/folder/source parity failed.' }
    node scripts/web/test_state_run_batch.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native independent State/browser scheduler parity failed.' }
    python scripts/web/test_state_run_batch_zip.py
    if ($LASTEXITCODE -ne 0) { throw 'State batch native ZIP byte verification failed.' }
    $sourceBatchArgs = @()
    if ($LocalCensusChecks) { $sourceBatchArgs += '--real' }
    node scripts/web/test_state_source_batch.mjs @sourceBatchArgs
    if ($LASTEXITCODE -ne 0) { throw 'Selected Census source/State scheduler verification failed.' }
    python scripts/web/generate_state_source_acquisition_fixtures.py
    if ($LASTEXITCODE -ne 0) { throw 'Synthetic state acquisition fixtures failed.' }
    node scripts/web/test_large_census_archive_profile.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Explicit large-state Census acquisition bounds failed.' }
    $stateSourceAcquisitionArgs = @()
    if ($LocalCensusChecks) { $stateSourceAcquisitionArgs += '--local-ri' }
    node scripts/web/test_state_source_acquisition.mjs @stateSourceAcquisitionArgs
    if ($LASTEXITCODE -ne 0) { throw 'Paired native Fetch/Census source job acquisition failed.' }
    node scripts/web/test_native_state_workflow.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State workflow and streamed original-source handoff failed.'}
    cargo build -p rplan-io --example rewrite_context --release --target-dir target/explicit-county-context-host --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native context fixture writer build failed.'}
    cargo build -p bisect-cli --bin bisect --example label_summary_application_reference --release --no-default-features --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Native CLI split reference build failed.'}
    node scripts/web/test_explicit_county_splits.mjs
    if($LASTEXITCODE -ne 0){throw 'Explicit county identities and native CLI/WASM split analysis failed.'}
    node scripts/web/test_native_state_recovery.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State explicit interruption recovery failed.'}
    node scripts/web/test_native_state_batch_recovery.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State batch interruption recovery failed.'}
    node scripts/web/test_state_source_preflight.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State source acquisition preflight failed.'}
    cargo build -p bisect-cli --example geometric_state_reference --no-default-features --locked --offline -j 1
    if($LASTEXITCODE -ne 0){throw 'Independent native State adjacency kernel oracle build failed.'}
    node scripts/web/test_native_state_adjacency_workflow.mjs
    if($LASTEXITCODE -ne 0){throw 'Native State selected adjacency workflow failed.'}
    python scripts/web/test_native_state_adjacency_workflow_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Native State selected adjacency ZIP integrity failed.'}
    node scripts/web/test_native_selected_state_source.mjs
    if($LASTEXITCODE -ne 0){throw 'Single selected-source State count/execution parity failed.'}
    python scripts/web/test_native_selected_state_source_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Single selected-source State ZIP integrity failed.'}
    node scripts/web/test_native_state_chamber_sources.mjs
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
    python scripts/web/test_native_state_workflow_zip.py --skip-browser
    if($LASTEXITCODE -ne 0){throw 'Native State output workflow ZIP integrity failed.'}
    node scripts/web/test_topology_profile.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Shared-boundary display profile native/WASM parity and source guards failed.' }
    $streamStateArgs = @($stateSourceAcquisitionArgs)
    if ($LargeStateChecks) { $streamStateArgs += '--local-ca' }
    node --max-old-space-size=6144 scripts/web/test_stream_state_source_batch.mjs @streamStateArgs
    if ($LASTEXITCODE -ne 0) { throw 'Serial acquisition/State Run/local output stream failed.' }
    node scripts/web/test_stream_native_output.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Streamed native State output orchestration failed.' }
    node scripts/web/test_native_states_workers.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native States requested-worker scheduling and failure cleanup failed.' }
    node scripts/web/test_native_states_queue.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Native States queued-source policy and saved replay failed.' }
    python scripts/web/test_native_states_queue_zip.py --skip-browser
    if ($LASTEXITCODE -ne 0) { throw 'Native States queued-source ZIP integrity failed.' }
    python scripts/web/test_native_states_option_consumption.py
    if ($LASTEXITCODE -ne 0) { throw 'Native States option-consumption baseline changed; review parity.' }
    python scripts/web/test_stream_native_output_files.py --skip-browser
    if ($LASTEXITCODE -ne 0) { throw 'Streamed native State output file verification failed.' }
    if ($LocalCensusChecks) {
        node scripts/web/test_stream_native_states.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Shared native States source acquisition and streaming failed.' }
        node scripts/web/test_stream_native_run.mjs
        if($LASTEXITCODE -ne 0){throw 'Native Run source streaming parity failed.'}
        node scripts/web/test_stream_native_run_years.mjs
        if($LASTEXITCODE -ne 0){throw 'Native Run multi-year acquisition/streaming failed.'}
        node scripts/web/test_census_2000_acquisition.mjs
        if($LASTEXITCODE -ne 0){throw 'Census 2000 source acquisition and native Run parity failed.'}
        python scripts/web/test_stream_native_run_years_files.py --skip-browser
        if($LASTEXITCODE -ne 0){throw 'Native Run multi-year local output readback failed.'}
        python scripts/web/test_stream_native_states_files.py --skip-browser
        if ($LASTEXITCODE -ne 0) { throw 'Shared native States source/output archive verification failed.' }
    }
    python scripts/web/test_stream_state_source_batch_zip.py @streamStateArgs
    if ($LASTEXITCODE -ne 0) { throw 'Streaming local/native output ZIP byte comparison failed.' }
    if ($LargeStateChecks) {
        python scripts/web/prepare_original_ca_oracle.py
        if ($LASTEXITCODE -ne 0) { throw 'Original CA native source preparation failed.' }
        python scripts/web/test_original_ca_preparation.py
        if ($LASTEXITCODE -ne 0) { throw 'Original CA native/WASM source graph comparison failed.' }
        node --max-old-space-size=6144 scripts/web/test_stream_large_source_project.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Original CA retained source project failed.' }
        cargo build -p bisect-data --release --example census_topology_geometry --locked --offline -j 1
        if ($LASTEXITCODE -ne 0) { throw 'Native shared-boundary geometry reference build failed.' }
        python scripts/web/prepare_topology_geometry_oracle.py
        if ($LASTEXITCODE -ne 0) { throw 'Original CA independent native shared-boundary geometry export failed.' }
        python scripts/web/test_topology_geometry.py
        if ($LASTEXITCODE -ne 0) { throw 'Original CA independent GEOS/source-coordinate validation failed.' }
        node --max-old-space-size=6144 scripts/web/test_stream_state_source_batch.mjs --local-ca --topology
        if ($LASTEXITCODE -ne 0) { throw 'Original CA shared-boundary display Run/map/report failed.' }
        python scripts/web/test_stream_state_source_batch_zip.py --local-ca --topology
        if ($LASTEXITCODE -ne 0) { throw 'Original CA shared-boundary output bytes and unchanged plan verification failed.' }
        node --max-old-space-size=6144 scripts/web/test_stream_large_source_project.mjs --topology
        if ($LASTEXITCODE -ne 0) { throw 'Original CA shared-boundary profile source project roundtrip failed.' }
        python scripts/web/prepare_full_ca_browser.py
        if ($LASTEXITCODE -ne 0) { throw 'Original CA full browser oracle preparation failed.' }
        node --max-old-space-size=4096 scripts/web/test_topology_plan_map.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Original CA interactive topology map model failed.' }
    }
    $geometryStates = @($OriginalGeometryStateChecks)
    if ($OriginalAlaskaChecks) { $geometryStates += 'AK' }
    foreach ($geometryState in @($geometryStates | Select-Object -Unique)) {
        node scripts/web/test_original_alaska_workflow_wasm.mjs "--state=$geometryState"
        if ($LASTEXITCODE -ne 0) { throw "Original $geometryState full workflow native/WASM parity failed; retain original source preparation receipts first." }
        python scripts/web/test_original_state_geometry_outputs.py "--states=$geometryState"
        if ($LASTEXITCODE -ne 0) { throw "Independent original $geometryState fixed-geometry outputs failed." }
        if ($geometryState -eq 'AK') {
            node scripts/web/test_antimeridian_wasm.mjs --original-ak --counterfactual
            if ($LASTEXITCODE -ne 0) { throw 'Original Alaska native national antimeridian map parity failed.' }
        }
        if ($BrowserChecks) {
            if ($geometryState -eq 'AK') {
                node scripts/web/test_antimeridian_browser.mjs --original-ak
                if ($LASTEXITCODE -ne 0) { throw 'Original Alaska national inset browser validation failed.' }
            }
            node scripts/web/test_original_alaska_browser.mjs "--state=$geometryState"
            if ($LASTEXITCODE -ne 0) { throw "Original $geometryState full workflow browser validation failed." }
        }
    }
    foreach ($algorithmState in @($OriginalAlgorithmStateChecks | Select-Object -Unique)) {
        node scripts/web/test_original_alaska_workflow_wasm.mjs "--state=$algorithmState" --structure=standard-bisect
        if ($LASTEXITCODE -ne 0) { throw "Explicit original $algorithmState StandardBisect workflow parity failed." }
        python scripts/web/test_original_state_geometry_outputs.py "--states=$algorithmState" --structure=standard-bisect
        if ($LASTEXITCODE -ne 0) { throw "Independent original $algorithmState StandardBisect population/contiguity verification failed." }
        if ($BrowserChecks) {
            node scripts/web/test_original_alaska_browser.mjs "--state=$algorithmState" --structure=standard-bisect
            if ($LASTEXITCODE -ne 0) { throw "Original $algorithmState StandardBisect browser workflow failed." }
        }
    }
    if (($OriginalAlgorithmStateChecks -contains 'AL') -and ($OriginalAlgorithmStateChecks -contains 'AZ')) {
        node --max-old-space-size=6144 scripts/web/test_national_map_package_capacity.mjs --original-byte-capacity
        if ($LASTEXITCODE -ne 0) { throw 'Original AL/AZ national package byte-capacity verification failed.' }
        if ($BrowserChecks) {
            node --max-old-space-size=6144 scripts/web/test_national_map_package_browser.mjs
            if ($LASTEXITCODE -ne 0) { throw 'Original AL/AZ large national package browser verification failed.' }
        }
    }
    if ($OriginalNationalMapChecks) {
        python scripts/web/test_original_all_state_sources.py
        if ($LASTEXITCODE -ne 0) { throw 'Original all-50 source/derived identity verification failed.' }
        node --max-old-space-size=6144 scripts/web/test_original_national_map_stream.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Original all-50 national map native/WASM verification failed; retain each original generation/source prerequisite first.' }
        python scripts/web/test_original_national_map_inputs.py
        if ($LASTEXITCODE -ne 0) { throw 'Independent original all-50 national plan/population/contiguity verification failed.' }
        if ($BrowserChecks) {
            node --max-old-space-size=6144 scripts/web/test_national_map_package_browser.mjs --original-national
            if ($LASTEXITCODE -ne 0) { throw 'Original all-50 national map browser/backup verification failed.' }
        }
    }
    if ($AllOriginalTractChecks) {
        node --max-old-space-size=6144 scripts/web/test_original_all_states_wasm.mjs
        if ($LASTEXITCODE -ne 0) { throw 'All original 2020 state tract preparation/Run/workflow verification failed.' }
        python scripts/web/test_original_all_states_outputs.py
        if ($LASTEXITCODE -ne 0) { throw 'Independent all-state population/contiguity verification failed.' }
    }
    if ($OriginalStateChecks.Count -gt 0) {
        $originalStateArguments = @($OriginalStateChecks | ForEach-Object { "--original-state=$_" })
        node --max-old-space-size=6144 scripts/web/test_stream_state_source_batch.mjs @originalStateArguments
        if ($LASTEXITCODE -ne 0) { throw 'Original state source-to-Run/workflow parity failed.' }
        python scripts/web/test_original_state_stream_zip.py @OriginalStateChecks
        if ($LASTEXITCODE -ne 0) { throw 'Original state independent identities/populations/ZIP verification failed.' }
        if ($BrowserChecks) {
            python scripts/web/prepare_original_state_browser.py @OriginalStateChecks
            if ($LASTEXITCODE -ne 0) { throw 'Original state browser prerequisites failed.' }
            foreach ($originalState in $OriginalStateChecks) {
                node --max-old-space-size=6144 scripts/web/test_full_ca_browser.mjs "--state=$originalState"
                if ($LASTEXITCODE -ne 0) { throw "Original $originalState full browser preparation/Run/workflow/backup failed." }
                python scripts/web/test_full_ca_browser_zip.py $originalState
                if ($LASTEXITCODE -ne 0) { throw "Original $originalState browser ZIP bytes/CRCs failed." }
            }
        }
    }
    node scripts/web/test_state_run_batch_resume.mjs
    if ($LASTEXITCODE -ne 0) { throw 'State batch Resume native receipt reuse and identity guards failed.' }
    node scripts/web/test_state_run_batch_recovery.mjs @sourceBatchArgs
    if ($LASTEXITCODE -ne 0) { throw 'Saved State batch full native replay and recovery guards failed.' }
    python scripts/web/test_state_run_batch_resume_zip.py --recovery @sourceBatchArgs
    if ($LASTEXITCODE -ne 0) { throw 'Resumed state output ZIP exact native byte verification failed.' }
    python scripts/web/test_state_source_batch_zip.py @sourceBatchArgs
    if ($LASTEXITCODE -ne 0) { throw 'Selected source/prepared/native batch ZIP verification failed.' }
    if ($BrowserChecks) {
        node scripts/web/test_native_run_command_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Actual native Run browser workflow failed.' }
        node scripts/web/test_native_state_workflow_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State output workflow browser failed.'}
        node scripts/web/test_state_source_preflight_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native State queue acquisition preflight browser failed.'}
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
        python scripts/web/test_native_run_command_zip.py
        if ($LASTEXITCODE -ne 0) { throw 'Actual native Run browser source/export integrity failed.' }
        node scripts/web/test_native_run_queue_browser.mjs
        if($LASTEXITCODE -ne 0){throw 'Native Run source queue browser workflow failed.'}
        python scripts/web/test_native_run_queue_zip.py
        if($LASTEXITCODE -ne 0){throw 'Native Run browser multi-year archive integrity failed.'}
        node scripts/web/test_native_states_queue_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native States queue policy browser workflow failed.' }
        python scripts/web/test_native_states_queue_zip.py
        if ($LASTEXITCODE -ne 0) { throw 'Native States queue browser/source/output integrity failed.' }
        $sourceAcquisitionBrowserArgs = @()
        if ($LargeStateChecks) { $sourceAcquisitionBrowserArgs += '--local-ca' }
        node scripts/web/test_stream_native_output_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Streamed native State output browser installation failed.' }
        python scripts/web/test_stream_native_output_files.py
        if ($LASTEXITCODE -ne 0) { throw 'Streamed native State browser/local file comparison failed.' }
        if ($LocalCensusChecks) {
            node scripts/web/test_stream_native_states_browser.mjs
            if ($LASTEXITCODE -ne 0) { throw 'Shared native States streamed browser workflow failed.' }
            node scripts/web/test_stream_native_run_years_browser.mjs
            if($LASTEXITCODE -ne 0){throw 'Native Run multi-year streaming browser workflow failed.'}
            python scripts/web/test_stream_native_run_years_files.py
            if($LASTEXITCODE -ne 0){throw 'Native Run browser multi-year file integrity failed.'}
            python scripts/web/test_stream_native_states_files.py
            if ($LASTEXITCODE -ne 0) { throw 'Shared native States browser installation/readback failed.' }
        }
        node --max-old-space-size=6144 scripts/web/test_state_source_acquisition_browser.mjs @sourceAcquisitionBrowserArgs
        if ($LASTEXITCODE -ne 0) { throw 'Paired acquisition atomic queue and explicit browser Run failed.' }
        if ($LargeStateChecks) {
            node --max-old-space-size=6144 scripts/web/test_topology_geometry_browser.mjs
            if ($LASTEXITCODE -ne 0) { throw 'Original CA shared-boundary geometry browser map and archive failed.' }
            node --max-old-space-size=6144 scripts/web/test_full_ca_browser.mjs
            if ($LASTEXITCODE -ne 0) { throw 'Original CA full browser preparation, Run/workflow and compressed backup failed.' }
            python scripts/web/test_full_ca_browser_zip.py
            if ($LASTEXITCODE -ne 0) { throw 'Original CA browser native ZIP byte/CRC verification failed.' }
        }
        node scripts/web/test_state_source_batch_browser.mjs @sourceBatchArgs
        if ($LASTEXITCODE -ne 0) { throw 'Census source-to-State actual browser verification failed.' }
    }
    if ($BrowserChecks) {
        node scripts/web/test_state_run_batch_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'State worker pool/source/cancellation browser verification failed.' }
        python scripts/web/test_state_run_batch_resume_zip.py --recovery --browser @sourceBatchArgs
        if ($LASTEXITCODE -ne 0) { throw 'Resumed browser ZIP exact native byte verification failed.' }
    }
    if ($BrowserChecks) {
        node scripts/web/test_run_target_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Native Run target actual worker/folder browser verification failed.' }
    }
    if ($BrowserChecks) {
        node scripts/web/test_electoral_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Electoral settings source/receipt browser verification failed.' }
    }
    if ($BrowserChecks) {
        node scripts/web/test_multiscale_seed_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Multiscale replay seed display/source browser verification failed.' }
    }
    if ($BrowserChecks) {
        node scripts/web/test_coi_multiscale_browser.mjs
        if ($LASTEXITCODE -ne 0) { throw 'COI engine/multiscale replay correspondence browser verification failed.' }
    }
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
    & "$PSScriptRoot/copy_toolkit_assets.ps1" -Output $Output
    # The copied worker modules reference these verified solver/checker assets.
    # Reuse the standalone staging recipes, including their identity/license checks.
    python scripts/web/stage_roundingsat_preview.py --destination $Output
    if ($LASTEXITCODE -ne 0) { throw 'Verified solver assets are required; build the RoundingSat WASM prerequisite first.' }
    python scripts/web/stage_proof_checker_preview.py --destination $Output
    if ($LASTEXITCODE -ne 0) { throw 'Verified checker assets are required; build the proof-checker WASM prerequisite first.' }
    python scripts/web/stage_native_proof_preview.py --destination $Output
    if ($LASTEXITCODE -ne 0) { throw 'Native proof dependency identity staging failed.' }
    $municipal=node scripts/web/test_explicit_municipal_splits.mjs --wasm (Join-Path $Output 'bisect_wasm.wasm') --cli target/release/bisect.exe --fixture-wasm (Join-Path $Output 'bisect_wasm.wasm') --rewriter target/explicit-county-context-host/release/examples/rewrite_context.exe | ConvertFrom-Json
    if($LASTEXITCODE -ne 0 -or $municipal.cases -ne 9){throw 'Explicit municipal split native/WASM gate failed.'}
    if($BrowserChecks){
        $municipalBrowser=node scripts/web/test_explicit_municipal_splits_browser.mjs --package $Output --runtime-receipt $municipal.receipt | ConvertFrom-Json
        if($LASTEXITCODE -ne 0 -or $municipalBrowser.cases -ne 3){throw 'Explicit municipal split packaged browser gate failed.'}
    }

    node scripts/web/test_toolkit_assets.mjs $Output
    if ($LASTEXITCODE -ne 0) { throw 'Packaged toolkit module/worker dependencies are incomplete.' }
    node scripts/web/verify_proof_package.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Proof runtime dependency identities are incomplete.'}
    node scripts/web/test_proof_package.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'Proof package refusal regression failed.'}
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
    if ($BrowserChecks) {
        node scripts/web/test_package_verification_lifecycle_browser.mjs --site $Output --output target/package-verification-lifecycle-browser --native-rhist (Join-Path 'target/release' ('rhist'+$nativeAssetSuffix)) --native-rcount (Join-Path 'target/release' ('rcount'+$nativeAssetSuffix))
        if ($LASTEXITCODE -ne 0) { throw 'Packaged history/count verification lifecycle browser checks failed.' }
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
    $replayArguments = @('--package', $Output, '--host', $replayHost, '--output', $replayOutput)
    if ($BrowserChecks) { $replayArguments += '--browser' }
    python scripts/web/check_native_recorded_replay_package.py @replayArguments
    if ($LASTEXITCODE -ne 0) { throw 'Packaged native recorded replay regression failed.' }
    # Exact native map PNGs and explicit fine-unit browser workflow on this package.
    $fineHostName = if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) { 'execute_tool_sequence.exe' } else { 'execute_tool_sequence' }
    $fineCliName = if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) { 'bisect.exe' } else { 'bisect' }
    $fineHost = Join-Path $replayCargoMetadata.target_directory ('release/examples/' + $fineHostName)
    $fineCli = Join-Path $replayCargoMetadata.target_directory ('release/' + $fineCliName)
    $fineOutput = Join-Path 'target/fine-unit-map-package' ([Guid]::NewGuid().ToString('N'))
    $fineArguments = @('--package', $Output, '--host-sequence', $fineHost, '--native-cli', $fineCli, '--output', $fineOutput)
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
    $replayExportArgs = @('--package', $Output, '--host-sequence', $fineHost, '--native-map-reference', $replayExportMap, '--native-map-adapter', $replayExportAdapter, '--output', $replayExportOutput)
    if ($BrowserChecks) { $replayExportArgs += '--browser' }
    python scripts/web/check_native_replay_export_package.py @replayExportArgs
    if ($LASTEXITCODE -ne 0) { throw 'Original native fine export/handoff package regression failed.' }

    cargo test -p bisect-wasm --release --lib native_ordinary_context --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Original ordinary context preservation regressions failed.' }
    $ordinaryContextOutput = Join-Path 'target/native-ordinary-context-package' ([Guid]::NewGuid().ToString('N'))
    $ordinaryContextArgs = @('--package', $Output, '--host', $replayHost, '--native-map-reference', $replayExportMap, '--output', $ordinaryContextOutput)
    if ($BrowserChecks) { $ordinaryContextArgs += '--browser' }
    python scripts/web/check_native_ordinary_context_package.py @ordinaryContextArgs
    if ($LASTEXITCODE -ne 0) { throw 'Original native ordinary context/handoff package regression failed.' }

    cargo test -p bisect-wasm --release --lib native_imported_context --locked --offline -j 1
    if ($LASTEXITCODE -ne 0) { throw 'Native unit/source identity regressions failed.' }
    $nativeIdentityOutput = Join-Path 'target/native-unit-identity-package' ([Guid]::NewGuid().ToString('N'))
    $nativeIdentityArgs = @('--package', $Output, '--host', $replayHost, '--host-sequence', $fineHost, '--native-map-reference', $replayExportMap, '--output', $nativeIdentityOutput)
    if ($BrowserChecks) { $nativeIdentityArgs += '--browser' }
    python scripts/web/check_native_unit_identity_package.py @nativeIdentityArgs
    if ($LASTEXITCODE -ne 0) { throw 'Native original unit identity/source-index package regression failed.' }

    $freshStateSite = $Output
    $freshStateWasm = Join-Path $freshStateSite 'bisect_wasm.wasm'
    $freshStateOutput = Join-Path 'target/fresh-native-state-package' ([Guid]::NewGuid().ToString('N'))
    node scripts/web/test_fresh_native_state.mjs --wasm $freshStateWasm --host $replayHost --output $freshStateOutput
    if ($LASTEXITCODE -ne 0) { throw 'Fresh selected-source native State runtime regression failed.' }
    node scripts/web/test_fresh_native_state_project.mjs --wasm $freshStateWasm --output $freshStateOutput
    if ($LASTEXITCODE -ne 0) { throw 'Fresh State project/archive persistence regression failed.' }
    node scripts/web/test_fresh_native_state_retained_view.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Bounded fresh saved-run viewer regression failed.' }
    $freshMultiscaleArgs = @('--wasm', $freshStateWasm, '--host', $replayHost, '--output', $freshStateOutput)
    node scripts/web/test_fresh_native_multiscale.mjs @freshMultiscaleArgs
    if ($LASTEXITCODE -ne 0) { throw 'Fresh native multiscale raw-source/native-context regression failed.' }
    $freshBackendArgs = @('--wasm', $freshStateWasm, '--native-cli', $fineCli, '--output', $freshStateOutput)
    node scripts/web/test_fresh_native_backend_profile.mjs @freshBackendArgs
    if ($LASTEXITCODE -ne 0) { throw 'Fresh native backend profile/dispatch regression failed.' }


    if ($BrowserChecks) {
        node scripts/web/test_native_assignment_compare_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli
        if ($LASTEXITCODE -ne 0) { throw 'Native assignment Compare CLI/browser byte regression failed.' }
        node scripts/web/test_native_assignment_compare_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli --block-scale-fixture
        if ($LASTEXITCODE -ne 0) { throw 'Native assignment Compare scale and browser size-bound regression failed.' }
        node scripts/web/test_native_comparison_narrative_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli
        if ($LASTEXITCODE -ne 0) { throw 'Native comparison narrative CLI/browser byte regression failed.' }
        node scripts/web/test_civic_desktop_folder_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli
        if ($LASTEXITCODE -ne 0) { throw 'Civic desktop folder browser gate failed.' }
        node scripts/web/test_native_civic_conflicts_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli
        if ($LASTEXITCODE -ne 0) { throw 'Native civic Conflicts browser gate failed.' }
        node scripts/web/test_native_candidate_package_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli
        if ($LASTEXITCODE -ne 0) { throw 'Native candidate package browser gate failed.' }
        node scripts/web/test_native_candidate_workspace_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli
        if ($LASTEXITCODE -ne 0) { throw 'Native candidate workspace browser gate failed.' }
        node scripts/web/test_large_workbench_archive_preference.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Large workbench archive preference gate failed.' }
        node scripts/web/test_fresh_native_absence_sources.mjs
        if ($LASTEXITCODE -ne 0) { throw 'Explicit native absence source boundaries failed.' }
        node scripts/web/test_fresh_native_absence_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli
        if ($LASTEXITCODE -ne 0) { throw 'Explicit native absence CLI/browser/persistence regression failed.' }
        node scripts/web/test_fresh_native_auxiliary_browser.mjs --site $freshStateSite --output $freshStateOutput --cli $fineCli
        if ($LASTEXITCODE -ne 0) { throw 'Fresh selected native auxiliary CLI/browser/persistence regression failed.' }
        node scripts/web/test_fresh_native_backend_profile_browser.mjs --site $freshStateSite --output $freshStateOutput
        if ($LASTEXITCODE -ne 0) { throw 'Fresh native backend profile/browser regression failed.' }
        node scripts/web/test_fresh_native_state_browser.mjs --site $freshStateSite --output $freshStateOutput
        if ($LASTEXITCODE -ne 0) { throw 'Fresh native State browser review/save/open/map regression failed.' }
        node scripts/web/test_fresh_native_state_project_browser.mjs --site $freshStateSite --output $freshStateOutput
        if ($LASTEXITCODE -ne 0) { throw 'Fresh State complete-project/archive/autosave browser regression failed.' }
        $freshMultiscaleBrowserArgs = @('--site', $freshStateSite, '--output', $freshStateOutput)
        node scripts/web/test_fresh_native_multiscale_browser.mjs @freshMultiscaleBrowserArgs
        if ($LASTEXITCODE -ne 0) { throw 'Fresh native multiscale project/map/browser regression failed.' }
        node scripts/web/test_fresh_native_followups_browser.mjs --site $freshStateSite --output $freshStateOutput --host $replayHost
        if ($LASTEXITCODE -ne 0) { throw 'Fresh native audit/analysis/improvement handoff browser regression failed.' }


    }





    $preparedStateOutput = Join-Path 'target/prepared-state-package' ([Guid]::NewGuid().ToString('N'))
    $preparedStateArguments = @('--package', $Output, '--output', $preparedStateOutput)
    if ($BrowserChecks) { $preparedStateArguments += '--browser' }
    python scripts/web/check_prepared_state_package.py @preparedStateArguments
    if ($LASTEXITCODE -ne 0) { throw 'Packaged prepared-state workflow checks failed.' }
    Write-Output "Practitioner WASM preview: $Output/index.html"
} finally {
    if($null -eq $wasmPreviousTestWasm){Remove-Item Env:BISECT_TEST_WASM -ErrorAction SilentlyContinue}else{$env:BISECT_TEST_WASM=$wasmPreviousTestWasm}
    Pop-Location
}
