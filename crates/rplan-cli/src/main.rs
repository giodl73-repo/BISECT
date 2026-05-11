use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rplan_audit::{
    audit_plan, AuditConstraint, AuditResult, Chamber, LegalProfile, RuntimeProvenance,
};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rplan")]
#[command(about = "RPLAN interchange and audit tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Audit(AuditArgs),
}

#[derive(Debug, Parser)]
struct AuditArgs {
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    context: Option<PathBuf>,
    #[arg(long)]
    legal_profile: Option<PathBuf>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "plan-shape,population,contiguity"
    )]
    constraints: Vec<ConstraintArg>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
    #[arg(long)]
    allow_warnings: bool,
    #[arg(long)]
    fixed_generated_at: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ConstraintArg {
    PlanShape,
    Population,
    Contiguity,
    Splits,
    Vra,
    Geometry,
}

impl From<ConstraintArg> for AuditConstraint {
    fn from(value: ConstraintArg) -> Self {
        match value {
            ConstraintArg::PlanShape => AuditConstraint::PlanShape,
            ConstraintArg::Population => AuditConstraint::Population,
            ConstraintArg::Contiguity => AuditConstraint::Contiguity,
            ConstraintArg::Splits => AuditConstraint::Splits,
            ConstraintArg::Vra => AuditConstraint::Vra,
            ConstraintArg::Geometry => AuditConstraint::Geometry,
        }
    }
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
        Commands::Audit(args) => run_audit(args),
    }
}

fn run_audit(args: AuditArgs) -> Result<i32> {
    let plan_text = std::fs::read_to_string(&args.plan)
        .with_context(|| format!("reading plan {}", args.plan.display()))?;
    let document = rplan_io::read_rplan_str(&plan_text)
        .with_context(|| format!("parsing plan {}", args.plan.display()))?;

    let context = if let Some(path) = &args.context {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading context {}", path.display()))?;
        Some(
            rplan_io::read_rctx_str(&text)
                .with_context(|| format!("parsing context {}", path.display()))?,
        )
    } else {
        None
    };

    let plan_chamber = parse_chamber(&document.metadata.chamber)?;
    let profile = if let Some(path) = &args.legal_profile {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading legal profile {}", path.display()))?;
        let profile = serde_json::from_str::<LegalProfile>(&text)
            .with_context(|| format!("parsing legal profile {}", path.display()))?;
        validate_profile_applicability(&document, &plan_chamber, &profile)?;
        profile
    } else {
        if !matches!(plan_chamber, Chamber::Congressional) {
            anyhow::bail!(
                "--legal-profile is required for non-congressional chamber '{}'",
                document.metadata.chamber
            );
        }
        let year = document
            .plan
            .units
            .year
            .or_else(|| {
                document
                    .metadata
                    .created_at
                    .get(0..4)
                    .and_then(|year| year.parse().ok())
            })
            .unwrap_or(2020);
        LegalProfile::us_congressional_project_v1(year)
    };

    let constraints: Vec<AuditConstraint> = args
        .constraints
        .iter()
        .copied()
        .map(AuditConstraint::from)
        .collect();
    let runtime = RuntimeProvenance {
        binary_name: "rplan".to_string(),
        binary_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: option_env!("GIT_COMMIT").map(str::to_string),
        build_profile: None,
        solver: None,
    };
    let generated_at = args
        .fixed_generated_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    let certificate = audit_plan(
        &document.plan,
        context.as_ref(),
        &profile,
        runtime,
        &constraints,
        &generated_at,
    )?;
    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&certificate)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&certificate)?,
    };

    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }

    match certificate.result {
        AuditResult::Pass => Ok(0),
        AuditResult::PassWithWarnings if args.allow_warnings => Ok(0),
        AuditResult::PassWithWarnings => {
            eprintln!("audit passed with warnings");
            Ok(1)
        }
        AuditResult::Fail => {
            eprintln!("audit failed");
            Ok(1)
        }
    }
}

fn validate_profile_applicability(
    document: &rplan_io::RplanDocument,
    plan_chamber: &Chamber,
    profile: &LegalProfile,
) -> Result<()> {
    if &profile.chamber != plan_chamber {
        anyhow::bail!(
            "legal profile chamber {:?} does not match plan chamber {:?}",
            profile.chamber,
            plan_chamber
        );
    }
    if profile.jurisdiction != "US" && profile.jurisdiction != document.metadata.jurisdiction {
        anyhow::bail!(
            "legal profile jurisdiction '{}' does not match plan jurisdiction '{}'",
            profile.jurisdiction,
            document.metadata.jurisdiction
        );
    }
    if let Some(plan_year) = document.plan.units.year {
        if profile.year != plan_year {
            anyhow::bail!(
                "legal profile year {} does not match plan year {}",
                profile.year,
                plan_year
            );
        }
    }
    Ok(())
}

fn parse_chamber(value: &str) -> Result<Chamber> {
    match value {
        "congressional" => Ok(Chamber::Congressional),
        "state-house" => Ok(Chamber::StateHouse),
        "state-senate" => Ok(Chamber::StateSenate),
        "local" => Ok(Chamber::Local),
        "custom" => Ok(Chamber::Custom("custom".to_string())),
        other => anyhow::bail!("unknown chamber '{other}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_non_congressional_chambers() {
        assert_eq!(parse_chamber("state-house").unwrap(), Chamber::StateHouse);
        assert_eq!(parse_chamber("state-senate").unwrap(), Chamber::StateSenate);
        assert!(parse_chamber("weird").is_err());
    }
}
