use bisect_ilp::{
    compile_certified_split_compact_boundary_branch, compile_certified_split_compact_proof_request,
    CertifiedDecisionStage, CertifiedSplitDiscovery, CertifiedSplitInstance,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

#[derive(Serialize)]
struct ModelArtifact {
    path: String,
    bytes: u64,
    sha256: String,
    request_path: String,
    request_bytes: u64,
    request_sha256: String,
    variable_count: usize,
    constraint_count: usize,
    status: String,
}

#[derive(Serialize)]
struct ModelPackageManifest {
    schema_version: String,
    package_id: String,
    status: String,
    instance_hash: String,
    discovery_id: String,
    compiler_path: String,
    compiler_sha256: String,
    artifacts: BTreeMap<String, ModelArtifact>,
    claim_boundary: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 {
        return Err(
            "usage: certified_model_package <instance.json> <discovery.json> <out-dir>".into(),
        );
    }
    let instance: CertifiedSplitInstance =
        serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
    let discovery: CertifiedSplitDiscovery =
        serde_json::from_str(&std::fs::read_to_string(&args[2])?)?;
    let out_dir = Path::new(&args[3]);
    std::fs::create_dir_all(out_dir)?;
    let mut artifacts = BTreeMap::new();
    for stage in [
        CertifiedDecisionStage::PopulationLowerBound,
        CertifiedDecisionStage::BoundaryLowerBound,
        CertifiedDecisionStage::CanonicalTieBreak,
    ] {
        let stem = match stage {
            CertifiedDecisionStage::PopulationLowerBound => "01-population",
            CertifiedDecisionStage::BoundaryLowerBound => "02-boundary",
            CertifiedDecisionStage::CanonicalTieBreak => "03-canonical",
        };
        let artifact = compile_certified_split_compact_proof_request(&instance, &discovery, stage)?;
        write_artifact(out_dir, stem, artifact, &mut artifacts)?;
    }
    if discovery.objective.primary.max_population_deviation_scaled > 0 {
        let total_population = instance.populations.iter().sum::<i64>();
        let numerator = instance.k_right as i64 * total_population;
        let lower = numerator.div_euclid(instance.k_parent as i64);
        let upper = (numerator + instance.k_parent as i64 - 1).div_euclid(instance.k_parent as i64);
        for (suffix, population) in [("low", lower), ("high", upper)] {
            let artifact =
                compile_certified_split_compact_boundary_branch(&instance, &discovery, population)?;
            let stem = format!("02-{suffix}-boundary-right-{population}");
            write_artifact(out_dir, &stem, artifact, &mut artifacts)?;
        }
    }

    fn write_artifact(
        out_dir: &Path,
        stem: &str,
        artifact: bisect_ilp::CertifiedOpbArtifact,
        artifacts: &mut BTreeMap<String, ModelArtifact>,
    ) -> Result<(), Box<dyn Error>> {
        let opb_name = format!("{stem}.opb");
        let request_name = format!("{stem}.request.json");
        let opb_path = out_dir.join(&opb_name);
        let request_path = out_dir.join(&request_name);
        std::fs::write(&opb_path, artifact.opb)?;
        std::fs::write(
            &request_path,
            serde_json::to_string_pretty(&artifact.request)?,
        )?;
        artifacts.insert(
            stem.to_string(),
            ModelArtifact {
                path: opb_name,
                bytes: opb_path.metadata()?.len(),
                sha256: sha256_file(&opb_path)?,
                request_path: request_name,
                request_bytes: request_path.metadata()?.len(),
                request_sha256: sha256_file(&request_path)?,
                variable_count: artifact.request.variable_count,
                constraint_count: artifact.request.constraint_count,
                status: serde_json::to_value(artifact.request.status)?
                    .as_str()
                    .ok_or("decision status is not a string")?
                    .to_string(),
            },
        );
        Ok(())
    }
    let compiler_path = Path::new("crates/bisect-ilp/src/proof_backend.rs");
    let manifest = ModelPackageManifest {
        schema_version: "certified-split-model-package-v1".to_string(),
        package_id: "ri-2020-root-compact-proof-models".to_string(),
        status: "proof-required-unclassified".to_string(),
        instance_hash: discovery.instance_hash,
        discovery_id: discovery.discovery_id,
        compiler_path: compiler_path.to_string_lossy().replace('\\', "/"),
        compiler_sha256: sha256_file(compiler_path)?,
        artifacts,
        claim_boundary: "Hash-bound compact proof models for an unproved Rhode Island incumbent; no SAT/UNSAT result or certificate is claimed.".to_string(),
    };
    std::fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    println!("Certified compact model package: PASS");
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}
