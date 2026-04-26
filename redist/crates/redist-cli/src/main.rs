use clap::Parser;
use redist_cli::args::{Cli, Commands};
use redist_cli::runner::{StateConfig, StateResult, run_states_parallel, load_all_states, filter_incomplete};
use redist_cli::fetch::{load_manifest, build_fetch_list, print_check_report, download_items};
use redist_cli::analyze::run_analyze;
use redist_cli::aggregate::run_aggregate;
use redist_cli::map_cmd::run_map;
use redist_cli::validate::run_validate;
use redist_cli::migrate::run_migrate;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        // ── redist state: single state ────────────────────────────────────────
        Commands::State(args) => {
            let state_code = args.state.to_uppercase();
            let all = load_all_states(&args.year.to_string())
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
            let (_, state_name, num_districts) = all.iter()
                .find(|(code, _, _)| code == &state_code)
                .cloned()
                .unwrap_or_else(|| { eprintln!("ERROR: unknown state '{state_code}'"); std::process::exit(1); });

            let output_dir = args.output_dir
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(format!("outputs/{}", args.version)));

            // Warn for non-congressional chambers (state tolerance applies)
            if args.chamber != "congressional" {
                eprintln!(
                    "NOTE: State legislative balance tolerance set to 5.0%. \
                     Verify your state's constitutional standard."
                );
            }

            let results = run_states_parallel(vec![StateConfig {
                state_code: state_code.clone(),
                state_name,
                num_districts,
                year: args.year.to_string(),
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
                })
                .collect();

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
                    })
                    .collect();

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
            run_migrate(&base, &args.state, &args.label)
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
            eprintln!("[OK] migrated {} → plans/{}", args.state, args.label);
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
