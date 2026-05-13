use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rcount_audit::{verify_package_dir, write_verification_transcript, VerificationStatus};
use rcount_core::CountStatus;
use rcount_district::aggregate_package_dir_with_plan_path;
use rcount_io::{
    import_statement_csv, synthetic_summary_basic_manifest, write_statement_csv_package_dir,
};
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
    AggregateDistricts(AggregateDistrictsArgs),
    ImportStatementCsv(ImportStatementCsvArgs),
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

#[derive(Debug, Parser)]
struct AggregateDistrictsArgs {
    package_dir: PathBuf,
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    context: Option<PathBuf>,
    #[arg(long, default_value = "syn-2024-mayor")]
    contest_id: String,
    #[arg(long, default_value = "canvassed")]
    status: String,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
struct ImportStatementCsvArgs {
    csv: PathBuf,
    output_dir: PathBuf,
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
        Commands::AggregateDistricts(args) => run_aggregate_districts(args),
        Commands::ImportStatementCsv(args) => run_import_statement_csv(args),
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

fn run_aggregate_districts(args: AggregateDistrictsArgs) -> Result<i32> {
    let status = parse_count_status(&args.status)?;
    let transcript = aggregate_package_dir_with_plan_path(
        &args.package_dir,
        &args.plan,
        args.context.as_deref(),
        &args.contest_id,
        status,
    )?;
    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&transcript)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&transcript)?,
    };
    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }
    Ok(0)
}

fn run_import_statement_csv(args: ImportStatementCsvArgs) -> Result<i32> {
    let package = import_statement_csv(&args.csv)
        .with_context(|| format!("importing statement CSV {}", args.csv.display()))?;
    let manifest = synthetic_summary_basic_manifest(&package)?;
    write_statement_csv_package_dir(&args.output_dir, &args.csv, &manifest, &package)
        .with_context(|| format!("writing RCOUNT package {}", args.output_dir.display()))?;
    Ok(0)
}

fn parse_count_status(value: &str) -> Result<CountStatus> {
    match value {
        "unofficial" => Ok(CountStatus::Unofficial),
        "canvassed" => Ok(CountStatus::Canvassed),
        "recounted" => Ok(CountStatus::Recounted),
        "amended" => Ok(CountStatus::Amended),
        "certified" => Ok(CountStatus::Certified),
        "withdrawn" => Ok(CountStatus::Withdrawn),
        "superseded" => Ok(CountStatus::Superseded),
        other => anyhow::bail!("unsupported count status: {other}"),
    }
}
