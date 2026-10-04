//! Generate native reference transcripts for browser parity tests.
use std::{fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(std::env::args().nth(1).expect("output directory required"));
    let cases = [
        ("comparison", rcount_core::synthetic_batch_comparison_algorithm_package()),
        ("kaplan-markov", rcount_core::synthetic_kaplan_markov_macro_package()),
        ("minerva", rcount_core::synthetic_minerva_multi_round_package()),
        ("athena-boundary", rcount_core::synthetic_athena_boundary_package()),
        ("bad-comparison", rcount_core::synthetic_bad_batch_comparison_algorithm_package()),
    ];
    for (name, package) in cases {
        let dir = root.join(name);
        let manifest = rcount_io::synthetic_summary_basic_manifest(&package)?;
        rcount_io::write_package_dir(&dir, &manifest, &package)?;
        let verification = rcount_audit::verify_package_dir(&dir);
        fs::write(dir.join("native-verification.json"), serde_json::to_vec_pretty(&verification)?)?;
        let runs: Vec<_> = package.audit_algorithm_runs.iter().map(rcount_audit::replay_audit_algorithm_statistics).collect();
        fs::write(dir.join("native-replay.json"), serde_json::to_vec_pretty(&runs)?)?;
    }
    println!("Native election references: {}", root.display());
    Ok(())
}
