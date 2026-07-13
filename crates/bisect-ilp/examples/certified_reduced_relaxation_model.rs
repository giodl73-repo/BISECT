use bisect_ilp::{
    compile_certified_split_reduced_boundary_relaxation, CertifiedSplitDiscovery,
    CertifiedSplitInstance,
};
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args_os().collect::<Vec<_>>();
    if args.len() != 6 {
        return Err(
            "usage: certified_reduced_relaxation_model <instance.json> <discovery.json> <right-population> <fixed-labels.json> <out-dir>"
                .into(),
        );
    }
    let instance: CertifiedSplitInstance =
        serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
    let discovery: CertifiedSplitDiscovery =
        serde_json::from_str(&std::fs::read_to_string(&args[2])?)?;
    let exact_right_population = args[3]
        .to_string_lossy()
        .parse::<i64>()
        .map_err(|error| format!("invalid right population: {error}"))?;
    let fixed_assignments: Vec<Option<u8>> =
        serde_json::from_str(&std::fs::read_to_string(&args[4])?)?;
    let out_dir = Path::new(&args[5]);
    std::fs::create_dir_all(out_dir)?;
    let artifact = compile_certified_split_reduced_boundary_relaxation(
        &instance,
        &discovery,
        exact_right_population,
        &fixed_assignments,
    )?;
    std::fs::write(out_dir.join("boundary.opb"), artifact.opb)?;
    std::fs::write(
        out_dir.join("request.json"),
        serde_json::to_string_pretty(&artifact.request)?,
    )?;
    println!(
        "Certified reduced relaxation: {} variables, {} constraints",
        artifact.request.variable_count, artifact.request.constraint_count
    );
    Ok(())
}
