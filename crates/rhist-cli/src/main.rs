use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rhist_io::{read_package_dir, verification_transcript, VerificationStatus};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rhist")]
#[command(about = "RHIST unit-history package verifier")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Verify(VerifyArgs),
}

#[derive(Debug, Parser)]
struct VerifyArgs {
    package_dir: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    PrettyJson,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    match Cli::parse().command {
        Commands::Verify(args) => run_verify(args),
    }
}

fn run_verify(args: VerifyArgs) -> Result<i32> {
    let transcript = verification_transcript(read_package_dir(&args.package_dir))?;
    let code = if transcript.status == VerificationStatus::Pass {
        0
    } else {
        1
    };

    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&transcript)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&transcript)?,
    };
    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }
    Ok(code)
}
