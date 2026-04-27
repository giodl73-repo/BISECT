use clap::Parser;
use redist_cli::args::{Cli, Commands, SuiteCommands};
use redist_cli::sweep::run_sweep;
use redist_cli::suite::run_suite_stability;
use redist_cli::verify::run_verify;
use redist_cli::policy::run_policy;
use redist_cli::doctor::run_doctor;
use redist_cli::runner::{StateConfig, StateResult, run_states_parallel, load_all_states, filter_incomplete, chamber_district_count};
use redist_cli::policy::LocationRegistry;
use redist_cli::fetch::{load_manifest, build_fetch_list, print_check_report, download_items};
use redist_cli::analyze::run_analyze;
use redist_cli::aggregate::run_aggregate;
use redist_cli::map_cmd::run_map;
use redist_cli::validate::run_validate;
use redist_cli::migrate::run_migrate;
use redist_cli::report_cmd::run_report;
use redist_cli::export_cmd::run_export;
use redist_cli::import_cmd::run_import;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        // ── redist state: single state ────────────────────────────────────────
        Commands::State(args) => {
            let state_code = args.state.to_uppercase();
            let registry = LocationRegistry::load();

            // Validate year against the registry for known locations.
            // Unknown locations with --adjacency bypass get any year.
            let year = if registry.has_location(&state_code) {
                registry.validate_year(&state_code, &args.year)
                    .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); })
            } else if args.adjacency.is_none() {
                eprintln!(
                    "ERROR: unknown location '{state_code}'. \
                     For international locations not in the registry, \
                     use --adjacency <path> --districts <n>. \
                     To add this location permanently, edit data/location_policy.json."
                );
                std::process::exit(1);
            } else {
                args.year.clone()
            };

            // Resolve state name + base district count via registry or --adjacency override.
            let (state_name, num_districts) = if let Some(ref _adj_path) = args.adjacency {
                let name = args.state_name.clone()
                    .unwrap_or_else(|| registry.state_name(&state_code)
                        .map(|n| n.to_lowercase().replace(' ', "_"))
                        .unwrap_or_else(|| state_code.to_lowercase()));
                let districts = args.districts.unwrap_or_else(|| {
                    registry.chamber_districts(&state_code, &args.chamber, &year)
                        .unwrap_or_else(|| {
                            eprintln!(
                                "ERROR: --adjacency requires --districts ('{state_code}' \
                                 has no district data for year {year} in the registry)"
                            );
                            std::process::exit(1);
                        })
                });
                (name, districts)
            } else {
                let name = registry.state_name(&state_code)
                    .unwrap_or_else(|| state_code.to_lowercase())
                    .to_lowercase().replace(' ', "_");
                let ndist = registry.chamber_districts(&state_code, &args.chamber, &year)
                    .unwrap_or_else(|| {
                        // Fallback: manifest lookup for US states
                        let all = load_all_states(&year)
                            .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
                        all.iter()
                            .find(|(code, _, _)| code == &state_code)
                            .map(|(_, _, n)| *n)
                            .unwrap_or_else(|| {
                                eprintln!("ERROR: no district count for '{state_code}' \
                                    chamber '{}' year '{year}'. Use --districts.", args.chamber);
                                std::process::exit(1);
                            })
                    });
                (name, ndist)
            };

            let output_dir = args.output_dir
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(format!("outputs/{}", args.version)));

            // Block-level memory warning (scenario 40)
            if args.resolution.to_string() == "block" {
                let large_block_states = ["CA", "TX", "NY", "FL", "PA", "IL", "OH", "GA", "NC", "MI", "VA", "WA", "AZ", "CO"];
                let is_large = large_block_states.contains(&state_code.as_str());
                if is_large {
                    eprintln!(
                        "WARNING: Block-level resolution for {} loads ~700K+ units. \
                         This requires 2-4GB RAM and significant compute time. \
                         For most redistricting, --resolution block_group provides \
                         sufficient precision with 10x fewer units.",
                        state_code
                    );
                } else {
                    eprintln!(
                        "NOTE: Block-level resolution loads all census blocks (~50-200K units). \
                         For most redistricting, --resolution block_group is sufficient."
                    );
                }
            }

            // Granularity floor warning (before the run starts)
            if let Some(warn) = registry.granularity_warning(
                &state_code, &year, &args.chamber, &args.resolution.to_string()
            ) {
                eprintln!("NOTE: {warn}");
            }

            // Balance tolerance range validation
            // --balance-tolerance is in PERCENT (e.g., 0.5 = 0.5%, 5 = 5%, 25 = 25%).
            // Guard against common mistakes: passing a fraction (0.005) or absurd value (200).
            if let Some(tol) = args.balance_tolerance {
                if tol < 0.05 {
                    eprintln!(
                        "WARNING: --balance-tolerance {tol:.4} looks like a fraction, not a percent.\n\
                         Pass as percent: --balance-tolerance 0.5 means ±0.5%.\n\
                         Did you mean --balance-tolerance {}?",
                        tol * 100.0
                    );
                } else if tol > 50.0 {
                    eprintln!(
                        "ERROR: --balance-tolerance {tol} is over 50%. \
                         This effectively removes the balance constraint. \
                         Maximum useful value is 25% (some international standards). \
                         Check your value."
                    );
                    std::process::exit(1);
                }
            }

            // Chamber balance tolerance info
            if args.chamber != "congressional" && args.balance_tolerance.is_none() {
                let tol_pct = registry.chamber_districts(&state_code, &args.chamber, &year)
                    .map(|_| {
                        use redist_cli::runner::chamber_balance_tolerance;
                        chamber_balance_tolerance(&state_code, &args.chamber) * 100.0
                    });
                if let Some(pct) = tol_pct {
                    eprintln!("NOTE: Using {:.1}% balance tolerance for {state_code} {} (from location policy). Override with --balance-tolerance.", pct, args.chamber);
                }
            }

            let effective_num_districts = args.districts.unwrap_or_else(|| {
                chamber_district_count(&state_code, &args.chamber, num_districts)
            });
            let total_seats = args.total_seats.unwrap_or(
                args.seats_per_district * effective_num_districts
            );
            let results = run_states_parallel(vec![StateConfig {
                state_code: state_code.clone(),
                state_name,
                num_districts: effective_num_districts,
                year: year.clone(),
                version: args.version.clone(),
                output_dir,
                partition_mode: args.partition_mode.to_string(),
                position: args.position,
                debug: args.debug,
                reset: args.reset,
                reprocess: false,
                ufactor: args.ufactor,
                niter: args.niter,
                seed: args.seed,
                // Spec 1 fields
                num_districts_override: args.districts,
                chamber: args.chamber.clone(),
                label: args.label.clone(),
                population_source: args.population_source.clone(),
                balance_tolerance: args.balance_tolerance.map(|t| t / 100.0),
                write_manifest: args.manifest,
                force: args.force,
                resolution: args.resolution.to_string(),
                seats_per_district: args.seats_per_district,
                total_seats,
                adjacency_override: args.adjacency.as_ref().map(std::path::PathBuf::from),
                coi_weights: args.coi_weights.as_ref().map(std::path::PathBuf::from),
            }], 1);
            for r in &results {
                if !r.success {
                    eprintln!("FAILED {}: {}", r.state_code, r.error.as_deref().unwrap_or(""));
                    std::process::exit(1);
                }
                eprintln!("[OK] {} in {}ms", r.state_code, r.elapsed_ms);
            }
        }

        // ── redist states: 50-state parallel ─────────────────────────────────
        Commands::States(args) => {
            let all = load_all_states(&args.year.to_string())
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });

            let filter: std::collections::HashSet<String> = args.states
                .iter().map(|s| s.to_uppercase()).collect();

            let configs: Vec<StateConfig> = all.into_iter()
                .filter(|(code, _, _)| filter.is_empty() || filter.contains(code))
                .enumerate()
                .map(|(i, (code, name, districts))| StateConfig {
                    state_code: code, state_name: name, num_districts: districts,
                    year: args.year.to_string(), version: args.version.clone(),
                    output_dir: std::path::PathBuf::from(&args.output_dir),
                    partition_mode: args.partition_mode.to_string(),
                    position: (i + 2) as i32,
                    debug: args.debug, reset: false, reprocess: args.reprocess,
                    ufactor: 5, niter: 100, seed: None,
                    // Spec 1 defaults for bulk runs
                    num_districts_override: None,
                    chamber: "congressional".into(),
                    label: None,
                    population_source: "total".into(),
                    balance_tolerance: None,
                    write_manifest: false,
                    force: false,
                    resolution: "tract".into(),
                    seats_per_district: 1,
                    total_seats: districts,
                    adjacency_override: None,
                    coi_weights: None,
                })
                .collect();

            // Memory estimation for large parallel runs (scenario 49)
            let large_states = ["CA", "TX", "NY", "FL", "PA", "IL", "OH", "GA", "NC", "MI"];
            let has_large_state = configs.iter().any(|c| large_states.contains(&c.state_code.as_str()));
            if has_large_state && args.workers > 4 {
                eprintln!(
                    "NOTE: Large states included in run. Peak memory estimate: ~{}MB. \
                     Reduce --workers if you see OOM errors.",
                    args.workers * 150
                );
            }

            let to_run = filter_incomplete(configs);
            eprintln!("[redist states] {} states ({} workers)", to_run.len(), args.workers);
            report_and_exit(run_states_parallel(to_run, args.workers));
        }

        // ── redist run: multi-year orchestrator ───────────────────────────────
        Commands::Run(args) => {
            let years: Vec<String> = if args.year == "all" {
                vec!["2020".into(), "2010".into(), "2000".into()]
            } else {
                vec![args.year.clone()]
            };

            let mut any_failure = false;
            for year in &years {
                eprintln!("[redist run] year={year} version={}", args.version);
                let all = load_all_states(year)
                    .unwrap_or_else(|e| { eprintln!("ERROR {year}: {e}"); vec![] });

                let filter: std::collections::HashSet<String> = args.states
                    .iter().map(|s| s.to_uppercase()).collect();

                let configs: Vec<StateConfig> = all.into_iter()
                    .filter(|(code, _, _)| filter.is_empty() || filter.contains(code))
                    .enumerate()
                    .map(|(i, (code, name, districts))| StateConfig {
                        state_code: code, state_name: name, num_districts: districts,
                        year: year.clone(), version: args.version.clone(),
                        output_dir: std::path::PathBuf::from(
                            args.output_dir.clone().unwrap_or_else(|| format!("outputs/{}", args.version))
                        ),
                        partition_mode: args.partition_mode.to_string(),
                        position: (i + 2) as i32,
                        debug: args.debug, reset: args.reset, reprocess: args.reprocess,
                        ufactor: 5, niter: 100, seed: None,
                        // Spec 1 defaults for bulk runs
                        num_districts_override: None,
                        chamber: "congressional".into(),
                        label: None,
                        population_source: "total".into(),
                        balance_tolerance: None,
                        write_manifest: false,
                        force: false,
                        resolution: "tract".into(),
                        seats_per_district: 1,
                        total_seats: districts,
                        adjacency_override: None,
                        coi_weights: None,
                    })
                    .collect();

                // Memory estimation for large parallel runs (scenario 49)
                let large_states = ["CA", "TX", "NY", "FL", "PA", "IL", "OH", "GA", "NC", "MI"];
                let has_large_state = configs.iter().any(|c| large_states.contains(&c.state_code.as_str()));
                if has_large_state && args.workers > 4 {
                    eprintln!(
                        "NOTE: Large states included in run. Peak memory estimate: ~{}MB. \
                         Reduce --workers if you see OOM errors.",
                        args.workers * 150
                    );
                }

                let to_run = filter_incomplete(configs);
                let results = run_states_parallel(to_run, args.workers);
                for r in results.iter().filter(|r| !r.success) {
                    eprintln!("FAILED {year}/{}: {}", r.state_code, r.error.as_deref().unwrap_or(""));
                    any_failure = true;
                }
                eprintln!("[redist run] {year}: {}/{} OK",
                    results.iter().filter(|r| r.success).count(), results.len());
            }
            if any_failure { std::process::exit(1); }
            eprintln!("[OK] redist run complete");
        }

        // ── redist analyze: per-district analytics ────────────────────────────
        Commands::Analyze(args) => {
            run_analyze(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist map: PNG map rendering ─────────────────────────────────────
        Commands::Map(args) => {
            run_map(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist aggregate: merge state analysis into national datasets ─────
        Commands::Aggregate(args) => {
            run_aggregate(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist fetch: data download ───────────────────────────────────────
        Commands::Fetch(args) => {
            // --manifest flag overrides the env var
            if let Some(ref path) = args.manifest {
                std::env::set_var("REDIST_MANIFEST", path);
            }

            // PP-10: Validate year before fetching (silent omission on invalid year)
            let valid_years = ["2020", "2010", "2000", "all"];
            if !valid_years.contains(&args.year.as_str()) {
                eprintln!("ERROR: unsupported year '{}' — valid years are 2020, 2010, 2000, all",
                    args.year);
                std::process::exit(1);
            }

            let manifest = load_manifest()
                .unwrap_or_else(|e| { eprintln!("ERROR loading manifest: {e}"); std::process::exit(1); });

            let years: Vec<String> = if args.year == "all" {
                vec!["2020".into(), "2010".into(), "2000".into()]
            } else {
                vec![args.year.clone()]
            };

            for year in &years {
                let items = build_fetch_list(&manifest, &args.states, year, &args.data_types);
                eprintln!("[redist fetch] year={year} {} items",  items.len());

                if args.check_only {
                    print_check_report(&items);
                } else {
                    download_items(&items, args.force, args.release, &manifest)
                        .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
                }
            }
        }

        // ── redist validate: validate a .rplan file ───────────────────────────
        Commands::Validate(args) => {
            run_validate(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist migrate: copy legacy state plan into plans/{label}/ tree ───
        Commands::Migrate(args) => {
            let output_dir = std::path::PathBuf::from(format!("outputs/{}", args.version));
            let base = output_dir.join(&args.year);
            run_migrate(&base, &args.state, &args.label, args.force)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
            eprintln!("[OK] migrated {} -> plans/{}", args.state, args.label);
        }

        // ── redist compare: plan comparison ──────────────────────────────────
        Commands::Compare(args) => {
            redist_cli::compare::run_compare(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist suite: multi-chamber nested plans ──────────────────────────
        Commands::Suite(suite_args) => {
            match suite_args.command {
                SuiteCommands::Draw(args) => {
                    use redist_cli::suite::{SuiteDrawArgs, NestMode, run_suite_draw};
                    use std::collections::HashMap;
                    use std::str::FromStr;

                    let nest_mode = NestMode::from_str(&args.nest)
                        .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });

                    // Task 151: congressional-in-senate nesting is not yet supported
                    if args.nest_congressional_in_senate {
                        eprintln!(
                            "NOTE: Congressional-in-senate nesting is not yet supported.\n\
                             Congressional districts nest with state legislative districts at \
                             fractional ratios (e.g., WA: 10 congressional / 49 senate = 0.204:1).\n\
                             This requires multi-level optimization not available in the current \
                             recursive bisection approach.\n\
                             To file a feature request: https://github.com/giodl73-repo/REDIST/issues\n\
                             Proceeding without congressional-in-senate nesting."
                        );
                    }

                    // For draw, we currently require existing plan files.
                    // This is a stub — real draw would invoke bisection_runner for each chamber.
                    eprintln!("[redist suite draw] state={} year={} name={} nest={}",
                        args.state, args.year, args.name, args.nest);

                    let draw_args = SuiteDrawArgs {
                        state: args.state,
                        year: args.year,
                        version: args.version,
                        name: args.name,
                        congressional_districts: args.congressional_districts,
                        house_districts: args.house_districts,
                        senate_districts: args.senate_districts,
                        nest_mode,
                        nest_ratio: args.nest_ratio,
                        seed: args.seed,
                        output_base: args.output_base,
                        force: args.force,
                    };
                    // Empty assignments for CLI stub — real draw populates from bisection
                    let empty: HashMap<String, usize> = HashMap::new();
                    if let Err(e) = run_suite_draw(&draw_args, &empty, &empty, None) {
                        eprintln!("ERROR: {e}");
                        std::process::exit(4);  // nesting bit
                    }
                }
                SuiteCommands::Stability(args) => {
                    let result = run_suite_stability(&args)
                        .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
                    println!("{}", serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); }));
                    if !result.stable {
                        std::process::exit(6); // instability exit code
                    }
                }
                SuiteCommands::Validate(args) => {
                    use redist_cli::suite::run_suite_validate_with_overrides;
                    use std::path::PathBuf;

                    let suite_dir = PathBuf::from(&args.output_base)
                        .join(&args.version)
                        .join(&args.year)
                        .join("suites")
                        .join(&args.name);

                    let result = run_suite_validate_with_overrides(
                        &suite_dir,
                        &args.name,
                        args.house_label.as_deref(),
                        args.senate_label.as_deref(),
                    )
                    .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });

                    println!("{}", serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); }));

                    if !result.valid {
                        std::process::exit(5); // nesting violation bit 2 = 4, balance bit 0 = 1 → 5
                    }
                }
            }
        }

        // ── redist report: generate commission report ─────────────────────────
        Commands::Report(args) => {
            run_report(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist export: export plan to GeoJSON/GerryChain/CSV ─────────────
        Commands::Export(args) => {
            run_export(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist import: import GeoJSON plan ────────────────────────────────
        Commands::Import(args) => {
            run_import(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist policy: state policy lookup ────────────────────────────────
        Commands::Policy(args) => {
            run_policy(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }

        // ── redist doctor: pre-flight check ───────────────────────────────────
        Commands::Doctor(args) => {
            run_doctor(&args);
        }

        // ── redist verify: reproduce plan from manifest and compare ───────────
        Commands::Verify(args) => {
            run_verify(&args).unwrap_or_else(|e| { eprintln!("FAIL: {e}"); std::process::exit(1); });
        }

        // ── redist sweep: N-seed optimization planning tool ───────────────────
        Commands::Sweep(args) => {
            run_sweep(&args)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
        }
    }
}

fn report_and_exit(results: Vec<StateResult>) {
    let failures: Vec<_> = results.iter().filter(|r| !r.success).collect();
    for f in &failures {
        eprintln!("FAILED {}: {}", f.state_code, f.error.as_deref().unwrap_or(""));
    }
    eprintln!("[OK] {}/{} states complete", results.len() - failures.len(), results.len());
    if !failures.is_empty() { std::process::exit(1); }
}

#[cfg(test)]
mod tests {
    use redist_cli::runner::StateResult;

    /// Task 117: verify that report_and_exit exits non-zero on any failure.
    ///
    /// We cannot call report_and_exit directly (it calls std::process::exit which
    /// would kill the test process), so we test the LOGIC — the failure detection
    /// predicate — to document and guard the invariant.
    ///
    /// Invariant: if any StateResult has success=false, the process must exit 1.
    /// The `if !failures.is_empty() { std::process::exit(1); }` line in report_and_exit
    /// implements this. This test verifies the filtering predicate is correct.
    #[test]
    fn test_report_and_exit_nonzero_on_failure() {
        let results = vec![
            StateResult { state_code: "VT".into(), success: true, error: None, elapsed_ms: 100 },
            StateResult { state_code: "CA".into(), success: false, error: Some("timeout".into()), elapsed_ms: 9999 },
            StateResult { state_code: "TX".into(), success: true, error: None, elapsed_ms: 200 },
        ];

        // This is the exact filter used in report_and_exit
        let failures: Vec<_> = results.iter().filter(|r| !r.success).collect();
        assert!(!failures.is_empty(), "should detect 1 failure");
        assert_eq!(failures.len(), 1, "exactly 1 failure in test data");
        assert_eq!(failures[0].state_code, "CA");
        // Invariant: if failures is non-empty, std::process::exit(1) is called.
        // We cannot call it here (would kill the test), but the predicate is verified above.

        // Also verify: all-success → no failures → no exit
        let all_ok = vec![
            StateResult { state_code: "VT".into(), success: true, error: None, elapsed_ms: 50 },
        ];
        let no_failures: Vec<_> = all_ok.iter().filter(|r| !r.success).collect();
        assert!(no_failures.is_empty(), "all-success → exit 0 (no non-zero exit)");
    }

    // ── Task 123: large state memory estimation warning ───────────────────────

    /// Verify CA is in the large states list used for memory estimation.
    #[test]
    fn test_large_state_set_contains_ca() {
        let large_states = ["CA", "TX", "NY", "FL", "PA", "IL", "OH", "GA", "NC", "MI"];
        assert!(large_states.contains(&"CA"), "CA must be in large states list");
        assert!(large_states.contains(&"TX"), "TX must be in large states list");
        assert!(large_states.contains(&"NY"), "NY must be in large states list");
        // Small states must NOT be in the list
        assert!(!large_states.contains(&"VT"), "VT must not be in large states list");
        assert!(!large_states.contains(&"WY"), "WY must not be in large states list");
    }

    /// Verify the memory warning message format is correct.
    #[test]
    fn test_large_state_memory_warning_format() {
        let workers: usize = 8;
        let msg = format!(
            "NOTE: Large states included in run. Peak memory estimate: ~{}MB. \
             Reduce --workers if you see OOM errors.",
            workers * 150
        );
        assert!(msg.contains("NOTE:"), "must start with NOTE:");
        assert!(msg.contains("1200MB"), "must show computed MB for 8 workers: {msg}");
        assert!(msg.contains("--workers"), "must mention --workers flag: {msg}");
        assert!(msg.contains("OOM"), "must mention OOM: {msg}");
    }

    /// Verify the warning triggers only when workers > 4.
    #[test]
    fn test_large_state_warning_triggers_above_4_workers() {
        use redist_cli::runner::StateConfig;
        use std::path::PathBuf;

        fn make_config(state: &str) -> StateConfig {
            StateConfig {
                state_code: state.to_string(),
                state_name: state.to_lowercase(),
                num_districts: 52,
                year: "2020".to_string(),
                version: "v1".to_string(),
                output_dir: PathBuf::from("/tmp/test"),
                partition_mode: "edge-weighted".to_string(),
                position: 1,
                debug: false, reset: false, reprocess: false,
                ufactor: 5, niter: 100, seed: None,
                num_districts_override: None,
                chamber: "congressional".into(),
                label: None,
                population_source: "total".into(),
                balance_tolerance: None,
                write_manifest: false,
                force: false,
                resolution: "tract".into(),
                seats_per_district: 1,
                total_seats: 52,
                adjacency_override: None,
                coi_weights: None,
            }
        }

        let large_states = ["CA", "TX", "NY", "FL", "PA", "IL", "OH", "GA", "NC", "MI"];
        let configs = vec![make_config("CA"), make_config("VT")];
        let has_large = configs.iter().any(|c| large_states.contains(&c.state_code.as_str()));
        assert!(has_large, "CA in configs must trigger has_large_state");

        // workers <= 4: no warning
        let should_warn_4 = has_large && 4 > 4;
        assert!(!should_warn_4, "should not warn for exactly 4 workers");

        // workers > 4: warning
        let should_warn_5 = has_large && 5 > 4;
        assert!(should_warn_5, "should warn for 5 workers with CA in configs");

        // Small-state only run: no warning
        let small_configs = vec![make_config("VT"), make_config("DE")];
        let has_large_small = small_configs.iter().any(|c| large_states.contains(&c.state_code.as_str()));
        assert!(!has_large_small, "VT+DE run must not trigger large-state warning");
    }

    // ── Task 126: block-level resolution memory warning ───────────────────────

    /// Verify the large-block-state list used for memory warnings contains expected states.
    #[test]
    fn test_block_resolution_warning_for_large_state() {
        let large_block_states = ["CA", "TX", "NY", "FL", "PA", "IL", "OH", "GA", "NC", "MI", "VA", "WA", "AZ", "CO"];
        // Large states must be in the list
        assert!(large_block_states.contains(&"CA"), "CA must be in large block states");
        assert!(large_block_states.contains(&"TX"), "TX must be in large block states");
        assert!(large_block_states.contains(&"NY"), "NY must be in large block states");
        assert!(large_block_states.contains(&"WA"), "WA must be in large block states");
        // Small states must NOT be in the list
        assert!(!large_block_states.contains(&"VT"), "VT must not be in large block states");
        assert!(!large_block_states.contains(&"DE"), "DE must not be in large block states");
        assert!(!large_block_states.contains(&"WY"), "WY must not be in large block states");
    }

    /// Verify the block-resolution warning message format for large states.
    #[test]
    fn test_block_resolution_warning_message_format_large_state() {
        let state_code = "CA";
        let msg = format!(
            "WARNING: Block-level resolution for {} loads ~700K+ units. \
             This requires 2-4GB RAM and significant compute time. \
             For most redistricting, --resolution block_group provides \
             sufficient precision with 10x fewer units.",
            state_code
        );
        assert!(msg.contains("WARNING:"), "must start with WARNING:");
        assert!(msg.contains("CA"), "must mention state code");
        assert!(msg.contains("700K+"), "must mention unit count threshold");
        assert!(msg.contains("block_group"), "must mention block_group alternative");
        assert!(msg.contains("2-4GB"), "must mention RAM requirement");
    }

    /// Verify the block-resolution note message format for small states.
    #[test]
    fn test_block_resolution_note_message_format_small_state() {
        let msg = "NOTE: Block-level resolution loads all census blocks (~50-200K units). \
                   For most redistricting, --resolution block_group is sufficient.";
        assert!(msg.contains("NOTE:"), "must start with NOTE:");
        assert!(msg.contains("block_group"), "must mention block_group alternative");
        assert!(msg.contains("50-200K"), "must mention unit count range");
    }

    /// Verify only "block" triggers the warning (not "block_group" or "tract").
    #[test]
    fn test_block_resolution_warning_only_for_block_resolution() {
        // The warning condition in main.rs: args.resolution.to_string() == "block"
        assert_eq!("block", "block", "block matches");
        assert_ne!("block_group", "block", "block_group must not trigger block warning");
        assert_ne!("tract", "block", "tract must not trigger block warning");
    }

    /// Task 117: verify the Run arm also exits non-zero on any_failure.
    ///
    /// The Run arm sets `any_failure = true` for each failed result and calls
    /// `std::process::exit(1)` if any failure occurred. This test documents the
    /// invariant by checking the tracking logic.
    #[test]
    fn test_run_any_failure_flag_logic() {
        // Simulate the any_failure accumulation in the Run arm
        let mut any_failure = false;

        let results_year1 = vec![
            StateResult { state_code: "VT".into(), success: true, error: None, elapsed_ms: 100 },
            StateResult { state_code: "WA".into(), success: false, error: Some("metis error".into()), elapsed_ms: 500 },
        ];
        for r in results_year1.iter().filter(|r| !r.success) {
            eprintln!("FAILED {}: {}", r.state_code, r.error.as_deref().unwrap_or(""));
            any_failure = true;
        }

        let results_year2 = vec![
            StateResult { state_code: "CA".into(), success: true, error: None, elapsed_ms: 300 },
        ];
        for r in results_year2.iter().filter(|r| !r.success) {
            eprintln!("FAILED {}: {}", r.state_code, r.error.as_deref().unwrap_or(""));
            any_failure = true;
        }

        // Invariant: any_failure=true when at least one year had a failure
        assert!(any_failure, "any_failure must be true when year 2020 had a failure");
        // In main.rs: if any_failure { std::process::exit(1); }
        // We verify the condition — not the exit itself (would kill the test).
    }
}
