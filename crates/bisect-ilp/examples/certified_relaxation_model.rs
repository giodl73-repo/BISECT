use bisect_ilp::{
    compile_certified_split_boundary_relaxation,
    compile_certified_split_boundary_relaxation_outside_core, CertifiedSplitDiscovery,
    CertifiedSplitInstance,
};
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args_os().collect::<Vec<_>>();
    if args.len() != 5 && args.len() != 6 {
        return Err(
            "usage: certified_relaxation_model <instance.json> <discovery.json> <right-population> [fixed-labels.json] <out-dir>"
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
    let out_dir = Path::new(args.last().expect("output directory"));
    std::fs::create_dir_all(out_dir)?;
    let artifact = if args.len() == 6 {
        let fixed_assignments: Vec<Option<u8>> =
            serde_json::from_str(&std::fs::read_to_string(&args[4])?)?;
        compile_certified_split_boundary_relaxation_outside_core(
            &instance,
            &discovery,
            exact_right_population,
            &fixed_assignments,
        )?
    } else {
        compile_certified_split_boundary_relaxation(&instance, &discovery, exact_right_population)?
    };
    std::fs::write(out_dir.join("boundary.opb"), artifact.opb)?;
    std::fs::write(
        out_dir.join("request.json"),
        serde_json::to_string_pretty(&artifact.request)?,
    )?;
    println!(
        "Certified boundary relaxation: {} variables, {} constraints",
        artifact.request.variable_count, artifact.request.constraint_count
    );
    Ok(())
}
