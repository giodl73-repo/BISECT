//! Native report orchestration at the browser request's recorded timestamp.
use std::path::Path;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 { return Err("Expected label, year-or-all, format, output-or-default, timestamp".into()); }
    bisect_cli::analyze_label::run_label_report_at(
        &args[1], if args[2] == "all" { None } else { Some(&args[2]) },
        &args[3], if args[4] == "default" { None } else { Some(Path::new(&args[4])) }, &args[5],
    )
}
