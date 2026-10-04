use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Local state and national BISECT experiment laboratory")]
struct Args {
    #[arg(long, default_value_t = 4317)]
    port: u16,
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[arg(long, default_value = "runs/lab")]
    store: PathBuf,
    /// Path to the Rust bisect binary. Defaults to a sibling binary, then target/debug.
    #[arg(long)]
    engine: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let root = args.root.canonicalize()?;
    let store = if args.store.is_absolute() {
        args.store
    } else {
        root.join(args.store)
    };
    let engine = args.engine.unwrap_or_else(|| {
        let name = if cfg!(windows) {
            "bisect.exe"
        } else {
            "bisect"
        };
        let sibling = std::env::current_exe()
            .unwrap_or_default()
            .with_file_name(name);
        if sibling.is_file() {
            sibling
        } else {
            root.join("target/debug").join(name)
        }
    });
    let engine = if engine.is_absolute() {
        engine
    } else {
        root.join(engine)
    };
    let paths = bisect_web::data::Paths::new(root, store, engine)?;
    bisect_web::server::serve(bisect_web::jobs::Laboratory::new(paths)?, args.port)
}
