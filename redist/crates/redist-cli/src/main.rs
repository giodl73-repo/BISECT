use clap::Parser;
use redist_cli::args::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run(args) => {
            eprintln!("[redist run] year={} version={} workers={} mode={}",
                args.year, args.version, args.workers, args.partition_mode);
            eprintln!("Phase 3c-d (Rayon runner) not yet implemented");
            std::process::exit(1);
        }
        Commands::State(args) => {
            eprintln!("[redist state] state={} year={} mode={}",
                args.state, args.year, args.partition_mode);
            eprintln!("Phase 3c-d (state runner) not yet implemented");
            std::process::exit(1);
        }
        Commands::States(args) => {
            eprintln!("[redist states] year={} version={} workers={}",
                args.year, args.version, args.workers);
            eprintln!("Phase 3c-d (parallel runner) not yet implemented");
            std::process::exit(1);
        }
    }
}
