use bisect_ilp::{
    compile_certified_split_cutset_boundary_branch_with_fixes,
    compile_certified_split_reduced_cutset_boundary_branch, CertifiedSplitDiscovery,
    CertifiedSplitInstance, ConnectivityCut,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::path::Path;

#[derive(Serialize)]
struct CutsetModelManifest {
    schema_version: String,
    instance_hash: String,
    discovery_id: String,
    exact_right_population: i64,
    cut_count: usize,
    opb_sha256: String,
    request_sha256: String,
    cuts_sha256: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args_os().collect::<Vec<_>>();
    if args.len() != 7 {
        return Err(
            "usage: certified_cutset_model <instance.json> <discovery.json> <right-population> <cuts.json> <fixed-labels.json> <out-dir>"
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
    let cuts: Vec<ConnectivityCut> = serde_json::from_str(&std::fs::read_to_string(&args[4])?)?;
    let fixed_assignments: Vec<Option<u8>> =
        serde_json::from_str(&std::fs::read_to_string(&args[5])?)?;
    let out_dir = Path::new(&args[6]);
    std::fs::create_dir_all(out_dir)?;

    let artifact = if fixed_assignments.iter().any(Option::is_some) {
        compile_certified_split_reduced_cutset_boundary_branch(
            &instance,
            &discovery,
            exact_right_population,
            &cuts,
            &fixed_assignments,
        )?
    } else {
        compile_certified_split_cutset_boundary_branch_with_fixes(
            &instance,
            &discovery,
            exact_right_population,
            &cuts,
            &fixed_assignments,
        )?
    };
    let opb_path = out_dir.join("boundary.opb");
    let request_path = out_dir.join("request.json");
    let cuts_path = out_dir.join("cuts.json");
    std::fs::write(&opb_path, artifact.opb)?;
    std::fs::write(
        &request_path,
        serde_json::to_string_pretty(&artifact.request)?,
    )?;
    std::fs::write(&cuts_path, serde_json::to_string_pretty(&cuts)?)?;
    let manifest = CutsetModelManifest {
        schema_version: "certified-cutset-model-package-v1".to_string(),
        instance_hash: discovery.instance_hash,
        discovery_id: discovery.discovery_id,
        exact_right_population,
        cut_count: cuts.len(),
        opb_sha256: sha256_file(&opb_path)?,
        request_sha256: sha256_file(&request_path)?,
        cuts_sha256: sha256_file(&cuts_path)?,
    };
    std::fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    println!(
        "Certified cutset model: {} variables, {} constraints, {} cuts",
        artifact.request.variable_count,
        artifact.request.constraint_count,
        cuts.len()
    );
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}
