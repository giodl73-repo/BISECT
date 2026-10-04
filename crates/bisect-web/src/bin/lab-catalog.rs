use anyhow::{ensure, Result};
use bisect_web::{catalog, data::Paths, jobs::Laboratory, model::Experiment};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Build a static BISECT catalog; computation is opt-in")]
struct Args {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[arg(long, default_value = "runs/lab")]
    store: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    matrix: Option<PathBuf>,
    #[arg(long)]
    compute: bool,
    #[arg(long, value_delimiter = ',')]
    run_ids: Vec<String>,
    #[arg(long, default_value_t = 900_000_000)]
    max_bytes: u64,
}
fn main() -> Result<()> {
    let args = Args::parse();
    let root = args.root.canonicalize()?;
    let store = if args.store.is_absolute() {
        args.store
    } else {
        root.join(args.store)
    };
    let binary = if cfg!(windows) {
        "bisect.exe"
    } else {
        "bisect"
    };
    let engine = std::env::current_exe()?.with_file_name(binary);
    let paths = Paths::new(root, store, engine)?;
    ensure!(
        !args.compute || args.matrix.is_some(),
        "--compute requires --matrix."
    );
    ensure!(
        args.compute || args.matrix.is_none(),
        "--matrix requires --compute; use --run-ids for export only."
    );
    // Refuse before launching a costly matrix, not only at export time.
    ensure!(
        !args.output.exists() || std::fs::read_dir(&args.output)?.next().is_none(),
        "Output must be new or empty."
    );
    let lab = Laboratory::new(paths.clone())?;
    let mut ids = args.run_ids;
    if let Some(matrix) = args.matrix {
        let configs: Vec<Experiment> = serde_json::from_slice(&std::fs::read(matrix)?)?;
        ensure!(
            !configs.is_empty() && configs.len() <= 1000,
            "Matrix must contain 1–1000 experiments."
        );
        for config in &configs {
            config.validate(&paths.codes())?;
        }
        for config in configs {
            let job = lab.submit(config)?;
            eprintln!("Computing {}", job.id);
            while ["queued", "running"].contains(&lab.get(&job.id)?.status.as_str()) {
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            ids.push(job.id);
        }
    }
    let jobs = if ids.is_empty() {
        lab.jobs
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>()
    } else {
        ids.iter()
            .map(|id| lab.get(id))
            .collect::<Result<Vec<_>>>()?
    };
    ensure!(
        jobs.iter()
            .all(|j| !["queued", "running"].contains(&j.status.as_str())),
        "Export only finished experiments."
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&catalog::build(
            &paths,
            &jobs,
            &args.output,
            args.max_bytes
        )?)?
    );
    Ok(())
}
