# BISECT Pitfalls Collection

Structural vulnerabilities in the redistricting system — not bug reports, but the patterns that make whole classes of bugs possible. Each pitfall describes a general failure mode, a structural solution that makes it impossible (not just unlikely), and a test that proves the solution holds.

Owned by: **TRENCH**. Grows every session. Never shrinks.

## Rule

A pitfall is solved when:
1. A structural solution exists that makes the failure mode impossible (not just mitigated)
2. A test exists that would fail if the structural solution were removed

A bug that led to a pitfall discovery is noted in the pitfall, but the pitfall itself is stated at the pattern level.

## Domains

| Domain | Prefix | File | Count |
|--------|--------|------|-------|
| Algorithm | AP | [pitfalls-algorithm.md](pitfalls-algorithm.md) | 12 |
| Pipeline | PP | [pitfalls-pipeline.md](pitfalls-pipeline.md) | 131 |
| Constitutional | CP | [pitfalls-constitutional.md](pitfalls-constitutional.md) | 2 |
| Data | DP | [pitfalls-data.md](pitfalls-data.md) | 4 |
| Research | RP | [pitfalls-research.md](pitfalls-research.md) | 2 |
| **Total** | | | **151** |

## Status

| ID | Pitfall | Status | Test |
|----|---------|--------|------|
| AP-12 | Candidate checks repeatedly rebuild immutable graph topology | **MITIGATED** for scalar repair | Prepared connectivity regression and full-reply parity; fresh package acceptance pending |
| AP-01 | Multi-objective optimization trades constitutional constraints | **SOLVED** | test_vra_pipeline_balance.py |
| AP-02 | Flag-as-mode conflation creates invariant drift | **SOLVED** | test_vra_edge_weighting.py::TestVRACodePath |
| AP-03 | Weighting saturation — signal becomes noise at high density | **SOLVED** | test_vra_edge_weighting.py::TestAdaptiveBoostScaling |
| AP-04 | Indexing assumptions outlast their context | **SOLVED** | test_vra_edge_weighting.py::TestVRACodePath |
| AP-05 | Degenerate input to algorithm designed for N≥2 | **SOLVED** | test_vra_pipeline_balance.py::TestVRACodePathIntegrity |
| PP-128 | Ordinary recorded replay loses original native context identity | **OPEN** | Private physical binding and complete source/browser witnesses pass; maintained adoption/full acceptance pending |
| PP-127 | Counterfactual test confuses controlled provenance with behavior | **MITIGATED** | Exact unsigned seed, verified binding and declared-cwd-only comparison; full builders pending |
| PP-126 | Oracle adapter creates malformed alias input | **MITIGATED** | Direct raw native reader plus corrected alias controls; adopted-source gates pass; full acceptance pending |
| PP-125 | Fine export loses native context identity/classification | **MITIGATED** | Exact original RCTX/plan body and rehashed semantic controls; adopted-source gates pass; full acceptance pending |
| PP-124 | Decimal boundaries overlap in exact binary coordinates | **MITIGATED** | Retained strict refusal, independent exact-fraction audit, separate positive |
| PP-123 | Label path composition applies version twice | **MITIGATED** | Maintained actual-native canonical map controls; full build pending |
| PP-122 | Final maps consume seed geography instead of fine units | **MITIGATED** | Native/WASM/browser fine map controls; complete fine export remains open |
| PP-121 | Source-equality claim outlives deliberate type evolution | **MITIGATED** | Original/current projection and thirteen mutation controls |
| PP-120 | Inventory regeneration drops historical metadata | **SOLVED** | Main actual-scanner preservation tests and inverse catalog audit |
| PP-119 | Available geometry bypasses native method consumption preconditions | **OPEN** | Actual native constraint matrix, executed red, worker and browser controls |
| PP-117 | Raw source identity is omitted when geometry decoding/alignment fails | **OPEN** | Native cwd replay, TIGER source/context and WASM controls |
| PP-118 | Aliased consumed TIGER roles contradict the same source path | **OPEN** | Genuine reader red, native no-output refusal, WASM and browser controls |
| PP-116 | Readonly recipe counts inherit executable pointer width or JavaScript rounding | **OPEN** | Native64 oracle, previous-image refusals, invalid counts, native NWay and Chrome Save/Open |
| PP-01 | Subprocess flag inheritance gap | **SOLVED** | test_pipeline_flag_propagation.py |
| PP-02 | Force-rerun requires two independent mechanisms to both fire | **SOLVED** | test_pipeline_flag_propagation.py::TestReprocessPropagation |
| PP-03 | Conditional step with inversion gap | **SOLVED** | test_pipeline_flag_propagation.py::TestSkipFlags |
| CP-01 | Policy optimization competes with constitutional constraints | **SOLVED** | test_vra_pipeline_balance.py |
| CP-02 | Algorithm operating outside its valid input domain | **SOLVED** | test_vra_pipeline_balance.py::TestVRACodePathIntegrity |
| DP-01 | Population metric ambiguity across legal and algorithmic contexts | **OPEN** | — |
| DP-02 | Module context loss across subprocess boundary | **SOLVED** | (2010/2000 pipeline runs) |
| DP-03 | Identifier representation ambiguity at system boundaries | **SOLVED** | TestRustCLIAcceptance::test_vt_rust_final_assignments_exists |
| RP-01 | Threshold sensitivity presented as a point result | **OPEN** | — |
| RP-02 | Claim-to-data drift across pipeline evolution | **MITIGATED** | test_vra_compliance.py (partial) |
| PP-04 | Cross-language subprocess environment locality | **SOLVED** | TestRustCLIAcceptance — BISECT_PYTHON=sys.executable |
| PP-05 | Dual-purpose path parameter creates read/write coupling | **SOLVED** | TestRustCLIAcceptance — tmp dir + V3 adjacency |
| PP-06 | Parallel task failure invisibility via silent filter | **SOLVED** | TestRustCLIAcceptance::test_al_rust_mm_count |
| PP-07 | External tool parameter unit mismatch at boundary | **SOLVED** | TestRustCLIAcceptance::test_al_rust_population_balance |
| AP-06 | Implicit partition equality assumption in recursive bisection | **SOLVED** | TestRustCLIAcceptance::test_al_rust_population_balance |
| PP-08 | URL filename extraction without query parameter stripping | **SOLVED** | fetch.rs url.split('?').next() |
| PP-09 | In-memory download buffer OOM for variable-size files | **SOLVED** | fetch.rs::download_and_extract_zip (streaming) |
| PP-10 | Silent state omission on invalid year parameter | **SOLVED** | runner.rs::load_all_states year allowlist |
| PP-11 | Compile-time asset path resolution mismatch in workspace builds | **SOLVED** | include_bytes! next to source file |
| PP-12 | WKB decode API surface mismatch across geospatial crate ecosystem | **SOLVED** | bisect-map::dissolve::tests |
| DP-04 | Stale test assertion after format version upgrade | **SOLVED** | bisect-data::serialize::tests::test_magic_header |
| PP-13 | Integer parameter truncation at subprocess boundary | **SOLVED** | bisection_runner::tests::test_ufactor_wasnt_silently_truncated_regression |
| AP-07 | Per-depth tolerance in recursive bisection causes compounding balance error | **SOLVED** | bisection_runner::tests::test_per_node_ufactor_formula |
| AP-08 | Granularity floor constraint in geographic unit-based partitioning | **MITIGATED** | validate_state_legislative.py |
| AP-09 | Single-seed heuristic incumbent blindness | **MITIGATED** | test_certified_metis_ensemble.py |
| AP-10 | Connectivity witness variables dominate exact search | **MITIGATED** | test_certified_cutset.py; proof_backend cutset tests |
| AP-11 | Solver substitution masks a decomposition bottleneck | **MITIGATED** | solver-frontier.json |
| PP-14 | Metadata Resolution Split — dual sources for plan parameters | **SOLVED** | bisect-cli::integration_pipeline_tests::test_plan_context_wa_house_98_not_congressional_10 |
| PP-15 | Entry-point switching without PATH pre-flight check | **SOLVED** | setup_env.bat preflight (Plan 01 Task 3.2) |
| PP-16 | Incremental deletion commits create brittle rollback dependencies | **SOLVED** | Plan 02 Rollback section (procedural artifact) |
| PP-17 | Sensitive-asset commit prevention via manual reminder, not structural | **SOLVED** | pre-commit hook rejecting *.pdf staging (Plan 03 Task 1.4) |
| PP-18 | Pulse completion without trace or risk closure | **MITIGATED** | Future VTRACE-governed pulse L1 review |
| PP-19 | Interrupted admission leaves a canonical-path partial file | **SOLVED** | test_data_vault_failures.py interruption/race tests |
| PP-20 | Successful exit with an unsatisfied setup or selection contract | **SOLVED** | test_data_vault_failures.py CLI and real relocation tests |
| PP-21 | Output identity mistaken for complete execution custody | **SOLVED** | receipt completeness test and isolated RI rebuild |
| PP-22 | Output-gated owner validation can pass without executing | **SOLVED** | ferris-validation-domains.mjs independent web/docs oracle |
| PP-23 | Repository-local change parser drifts from planner semantics | **SOLVED** | Ferris shadow run 33349539121 plus exact Windows replay |
| PP-24 | Engine execution completion mistaken for candidate acceptance | **SOLVED** for State/States result tables | test_state_candidate_results.mjs; retained-native acceptance |
| PP-25 | Declaration coverage mistaken for executable parity | **OPEN** | Closure guards implemented; inventory/evidence/release integration incomplete |
| PP-26 | Source/seed identity mistaken for runtime equivalence | **MITIGATED** | Named C profile, measured CRT/native/kernel/worker regressions; broader acceptance open |
| PP-33 | Acquisition completion promoted to downstream data readiness | **MITIGATED** for Fetch ZIP packages | Native extracted-byte/marker comparisons, visible target presence and retained-data/browser checks; native consumers remain separately bounded |

| PP-34 | Duplicate evidence copies exhaust local storage before acceptance | **MITIGATED** | snapshot_label_repository_installation.py; explicit low-budget refusal and blob verification |
| PP-35 | Native summary text interpolated as executable HTML | **SOLVED** | Shared escaping/metadata regressions; native/default/C files and packaged browser/download checks |
| PP-36 | Identity-bearing fixtures outlive their tested engine | **MITIGATED** | Paired TUI/C fixture producers and exact packaged-browser identity comparisons |

| PP-37 | Prior filesystem receipt appears current after Open | **MITIGATED** | test_label_rename_application_browser.mjs; failing/passing actual Chrome history regression |

| PP-38 | Different label strings alias the same filesystem directory | **SOLVED** | Shared case-only guard; 31 native tests and four actual CLI preservation cases |
| PP-39 | Clock normalization conceals downstream provenance changes | **MITIGATED** | 42 native tests, exact fixed-clock comparison and 27 package gates; broader comparators open |
| PP-40 | Frontend diagnostics drift inside persisted native indices | **MITIGATED** | Actual CLI partial-index witness; shared typed/ASCII formatter under validation |
| PP-41 | Completed worker output lost before final reply | **MITIGATED** | test_label_build_application_browser.mjs; actual default/C worker/OPFS, package pending |
| PP-42 | Evidence writes outgrow validation storage | **MITIGATED** | test_full_chamber_prerequisites.mjs; typed missing/capacity refusal before execution |
| PP-43 | Aggregate limits strand complete workloads and prevent saving | **MITIGATED** | test_national_matrix_browser.mjs; full-roster retention/archive/export checks |
| PP-44 | Evidence producer captures its own changing log | **MITIGATED** | snapshot_national_browser_delivery.py; actual copy-integrity refusal and explicit log exclusions |
| PP-45 | Error display discards recoverable operation outcome | **MITIGATED** | Both-profile installation outcome browser downloads; exact checkpoint and inert Open |
| PP-46 | Persisted intent mistaken for completed filesystem effects | **MITIGATED** | Both-profile reload-mid-write, passive history and atomic stale-tab refusal |
| PP-47 | Display geometry replaces physical source measurements | **MITIGATED** | Native guard refusal; both-profile grouped source import/preparation |
| PP-48 | Source credential outlives/follows request authority | **MITIGATED** | Both-shell redirect/export/retirement gates; live browser authentication still open |

| PP-50 | Source schema conversion changes vote meaning or identity | **MITIGATED** | Independent CSV/native comparisons; both-profile browser source retention |

| PP-51 | Opaque source/marker or cancelled package becomes authority | **MITIGATED** | Native wildcard byte equality; actual private browsers; post-acquisition cancel red/green |

| PP-49 | CORS workaround silently delegates credentials or becomes a proxy | **MITIGATED** | test_adjacency_release_bridge.mjs; test_adjacency_release_bridge_streams.mjs; actual private browser bridge tier |

| PP-52 | Awaited directory outlives current write intent | **MITIGATED** | Final-lookup cancel red/green; native CLI and actual OPFS |
| PP-53 | Passive progress mistaken for durable filesystem custody | **MITIGATED** | test_fetch_install_outcome_browser.mjs |
| PP-54 | Debug transport disconnect mistaken for process exit | **MITIGATED** | test_fetch_install_outcome_crash.mjs |
| PP-55 | Diagnostic declarations mistaken for algorithm effects | **MITIGATED** | test_state_diagnostic_flags.mjs |
| PP-56 | Whole-job timing substituted for partition-attempt timing | **MITIGATED** | test_partition_timing.mjs |
| PP-57 | No overflow mistaken for readable mobile tables | **MITIGATED** | Actual V1/V2 timing screenshots |
| PP-58 | Concurrent evidence jobs overcommit validation-host disk | **OPEN** | National ENOSPC/download failures retained; shared reservation untested |
| PP-59 | Manifest metadata editing rounds an exact unsigned seed | **MITIGATED** | Typed native manifest binding and exact output-text seed assertions |
| PP-60 | Unused selection result hides an output-source consumer | **OPEN** | Native population-source/reference cases; browser integration pending |
| PP-61 | Concurrent tests share a timestamp-named fixture directory | **MITIGATED** | Atomic unique roots; parallel default/C report-source checks |
| PP-62 | Follow-up regenerates a manifest that conflicts with final output | **MITIGATED** | Containing workflow supplies manifest before analysis/report hashing |
| PP-63 | Polling misses a transient successful UI status | **MITIGATED** | Observe exact import transition before Open; independently check retained project data |
| PP-64 | Results publish before fallible cleanup completes | **SOLVED** | Delay C bridge output publication; old-source regression and real WASM traps |
| PP-65 | Loaded-runtime cache retains a permanently retired instance | **SOLVED** | Private retirement check invalidates cache; fresh runtime only on explicit Run |
| PP-66 | Shared adapter comparison misses CLI orchestration refusals | **MITIGATED** | Actual ceremony CLI boundaries; distinct browser prefix inspection |
| PP-67 | Browser metadata bounds narrow a native scalar domain | **MITIGATED** | Exact u64 review duration, actual CLI maximum/lifecycle comparisons |
| PP-68 | Text decoding changes exact native artifact bytes | **MITIGATED** | Ceremony BOM preserved and refused like actual CLI; generic readers separate |

| PP-71 | Reentrant refusal executes another operation's cleanup | **MITIGATED** for Config publication | test_config_publication_browser.mjs |
| PP-72 | Host-sized metadata narrows a portable reader's accepted domain | **MITIGATED** for selected read-only manifest consumers | test_manifest_reader_width.mjs, test_manifest_reader_width_browser.mjs |
| PP-73 | Workflow instrumentation follows obsolete operation boundaries | **MITIGATED** for adjacency and legislative source checks | test_native_state_adjacency_workflow.mjs, test_native_state_chamber_sources_zip.py |

| PP-74 | Copied worker modules outpace packaged runtime dependencies | **MITIGATED** for standard proof staging | Literal asset closure and diagnostic 15; dynamic inventory/source enforcement/packaged proof execution pending |

## Adding a Pitfall

Latest finding: PP-64, publishing results before fallible cleanup completes —
SOLVED for the synchronous C bridge by delayed publication and a regression
that fails on the prior implementation. Other operations require their own review.

Latest continuation finding: [PP-32](pitfalls-pipeline.md#pp-32-computational-continuation-repeats-destructive-preparation) — **MITIGATED** for packaged native Run/States continuation; broader lifecycle/release evidence remains required.

Latest finding: [PP-31](pitfalls-pipeline.md#pp-31-aggregate-verification-success-conceals-missing-stages), aggregate verification success conceals missing stages — **MITIGATED** for selected-output Verify with packaged regressions; broader consumers remain open. The numbered pipeline file is authoritative for the intervening PP-27–PP-30 entries.

Every session that discovers a new failure mode ends with a new entry. The pattern:

1. State the failure mode generically — not "the --version flag defaulted to v1" but "subprocess flag inheritance gap"
2. Describe which domain it belongs to and why
3. State the structural solution
4. Link to the test that proves it
5. Add to the status table above

Current finding: [PP-110](pitfalls-pipeline.md#pp-110-configured-engine-does-not-identify-effective-compiled-dispatch) - **OPEN** pending maintained integration/full acceptance. Domain counts use distinct pitfall IDs; historical repeated headings remain in the authoritative files. Earlier latest-finding notes are historical checkpoints.
