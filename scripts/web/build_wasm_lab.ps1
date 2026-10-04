param([string]$Output='dist/wasm-laboratory',[string]$States='RI,IA,NC',[string]$Year='2020')
$ErrorActionPreference='Stop'
$labRepo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $labRepo
try {
    if((Test-Path -LiteralPath $Output) -and (Get-ChildItem -LiteralPath $Output -Force | Select-Object -First 1)){throw 'Choose a new or empty output directory.'}
    cargo test -p bisect-core --lib character --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Shared economic/housing character formula and zero-policy verification failed.'}
    cargo test -p bisect-runner --lib --no-default-features proportional --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Proportional recursion, seat allocation and input validation failed.'}
    cargo build -p bisect-wasm --release --target wasm32-unknown-unknown --locked --offline
    if($LASTEXITCODE -ne 0){throw 'WASM build failed.'}
    node scripts/web/test_wasm.mjs target/wasm32-unknown-unknown/release/bisect_wasm.wasm
    if($LASTEXITCODE -ne 0){throw 'WASM engine verification failed.'}
    node scripts/web/test_area_wasm.mjs target/wasm32-unknown-unknown/release/bisect_wasm.wasm
    if($LASTEXITCODE -ne 0){throw 'WASM AreaSection verification failed.'}
    cargo build -p bisect-wasm --release --example execute_request --example execute_tool_request --locked --offline
    if($LASTEXITCODE -ne 0){throw 'Native engine reference build failed.'}
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
    node scripts/web/test_character_national.mjs $Output
    if($LASTEXITCODE -ne 0){throw 'National character inputs, saved projects and maps failed verification.'}
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
    & (Join-Path $PSScriptRoot 'build_wasm.ps1') -Output (Join-Path $Output 'toolkit')
    if($LASTEXITCODE -ne 0){throw 'Integrated practitioner workbench build failed.'}
    node scripts/web/test_wasm_lab.mjs $Output $States $Year
    if($LASTEXITCODE -ne 0){throw 'Laboratory integration tests failed.'}
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
} finally {Pop-Location}
