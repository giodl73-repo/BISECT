use bisect_ilp::{certified_split_discovery, CertifiedSplitInstance};
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Deserialize)]
struct SearchResult {
    final_connected_assignment: Option<Vec<u8>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args_os().collect::<Vec<_>>();
    if args.len() != 4 {
        return Err(
            "usage: certified_discovery_from_assignment <instance.json> <search.json> <out.json>"
                .into(),
        );
    }
    let instance: CertifiedSplitInstance =
        serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
    let search: SearchResult = serde_json::from_str(&std::fs::read_to_string(&args[2])?)?;
    let assignment = search
        .final_connected_assignment
        .ok_or("search result has no connected assignment")?;
    let discovery = certified_split_discovery(
        &instance,
        "SciPy HiGHS",
        Some("scipy.optimize.milp".to_string()),
        "elite-metis-consensus-fixed-core-milp; heuristic-branch-only",
        assignment,
    )?;
    std::fs::write(
        Path::new(&args[3]),
        serde_json::to_string_pretty(&discovery)?,
    )?;
    println!(
        "Certified assignment validation: cut {}",
        discovery.objective.primary.weighted_boundary_cut
    );
    Ok(())
}
