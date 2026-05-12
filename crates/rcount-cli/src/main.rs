use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rcount_audit::{verify_package_dir, write_verification_transcript, VerificationStatus};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rcount")]
#[command(about = "RCOUNT election-count package verifier")]
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
    write_transcript: bool,
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
    let transcript = verify_package_dir(&args.package_dir);
    if args.write_transcript {
        write_verification_transcript(&args.package_dir, &transcript).with_context(|| {
            format!(
                "writing transcript under {}",
                args.package_dir.join("transcripts").display()
            )
        })?;
    }

    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&transcript)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&transcript)?,
    };
    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }

    match transcript.status {
        VerificationStatus::Pass => Ok(0),
        VerificationStatus::Fail => Ok(1),
    }
}
