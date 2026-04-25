use clap::Parser;
use redist_cli::args::{Cli, Commands};
use redist_cli::runner::{StateConfig, StateResult, run_states_parallel, load_all_states, filter_incomplete};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        // ── redist state: single state ────────────────────────────────────────
        Commands::State(args) => {
            let state_code = args.state.to_uppercase();

            // Resolve state name and district count via Python (once, before Rayon)
            let all = load_all_states(&args.year.to_string())
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });

            let (_, state_name, num_districts) = all.iter()
                .find(|(code, _, _)| code == &state_code)
                .cloned()
                .unwrap_or_else(|| {
                    eprintln!("ERROR: unknown state code '{state_code}'");
                    std::process::exit(1);
                });

            let output_dir = args.output_dir
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(format!("outputs/{}", args.version)));

            let cfg = StateConfig {
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
            };

            let results = run_states_parallel(vec![cfg], 1);
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
            // Load all state metadata ONCE before Rayon starts (avoids process explosion)
            let all = load_all_states(&args.year.to_string())
                .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });

            let states_filter: std::collections::HashSet<String> = args.states
                .iter().map(|s| s.to_uppercase()).collect();

            let configs: Vec<StateConfig> = all.into_iter()
                .filter(|(code, _, _)| states_filter.is_empty() || states_filter.contains(code))
                .enumerate()
                .map(|(i, (code, name, districts))| StateConfig {
                    state_code: code,
                    state_name: name,
                    num_districts: districts,
                    year: args.year.to_string(),
                    version: args.version.clone(),
                    output_dir: std::path::PathBuf::from(&args.output_dir),
                    partition_mode: args.partition_mode.to_string(),
                    position: (i + 2) as i32,
                    debug: args.debug,
                    reset: false,
                    reprocess: args.reprocess,
                    ufactor: 5,
                    niter: 100,
                    seed: None,
                })
                .collect();

            let to_run = filter_incomplete(configs);
            eprintln!("[redist states] {} states to process ({} workers)", to_run.len(), args.workers);

            let results = run_states_parallel(to_run, args.workers);
            report_and_exit(results);
        }

        // ── redist run: multi-year orchestrator ───────────────────────────────
        Commands::Run(args) => {
            // Years run SEQUENTIALLY; states within each year run in parallel.
            // Sequential years: each year's files are distinct; parallel years
            // would saturate disk I/O without meaningful speedup.
            let years: Vec<String> = if args.year == "all" {
                vec!["2020".into(), "2010".into(), "2000".into()]
            } else {
                vec![args.year.clone()]
            };

            let mut any_failure = false;
            for year in &years {
                eprintln!("[redist run] year={year} version={} mode={}", args.version, args.partition_mode);

                let all = load_all_states(year)
                    .unwrap_or_else(|e| { eprintln!("ERROR loading {year} config: {e}"); vec![] });

                let states_filter: std::collections::HashSet<String> = args.states
                    .iter().map(|s| s.to_uppercase()).collect();

                let configs: Vec<StateConfig> = all.into_iter()
                    .filter(|(code, _, _)| states_filter.is_empty() || states_filter.contains(code))
                    .enumerate()
                    .map(|(i, (code, name, districts))| StateConfig {
                        state_code: code,
                        state_name: name,
                        num_districts: districts,
                        year: year.clone(),
                        version: args.version.clone(),
                        output_dir: std::path::PathBuf::from(
                            args.output_dir.clone()
                                .unwrap_or_else(|| format!("outputs/{}", args.version))
                        ),
                        partition_mode: args.partition_mode.to_string(),
                        position: (i + 2) as i32,
                        debug: args.debug,
                        reset: args.reset,
                        reprocess: args.reprocess,
                        ufactor: 5,
                        niter: 100,
                        seed: None,
                    })
                    .collect();

                let to_run = filter_incomplete(configs);
                let results = run_states_parallel(to_run, args.workers);
                for r in results.iter().filter(|r| !r.success) {
                    eprintln!("FAILED {year}/{}: {}", r.state_code, r.error.as_deref().unwrap_or(""));
                    any_failure = true;
                }
                let ok = results.iter().filter(|r| r.success).count();
                eprintln!("[redist run] year={year}: {ok}/{} OK", results.len());
            }
            if any_failure { std::process::exit(1); }
            eprintln!("[OK] redist run complete");
        }
    }
}

fn report_and_exit(results: Vec<StateResult>) {
    let failures: Vec<_> = results.iter().filter(|r| !r.success).collect();
    let ok = results.len() - failures.len();
    for f in &failures {
        eprintln!("FAILED {}: {}", f.state_code, f.error.as_deref().unwrap_or(""));
    }
    eprintln!("[OK] {ok}/{} states complete", results.len());
    if !failures.is_empty() { std::process::exit(1); }
}
