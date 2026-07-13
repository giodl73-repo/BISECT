use bisect_ilp::{
    certified_split_discovery, compile_certified_split_compact_proof_requests,
    compile_certified_split_proof_requests, verify_certified_bisection_tree_bounded,
    CertifiedBisectionTree, CertifiedSplitResult,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

#[derive(Serialize)]
struct ProofPrototypeManifest {
    schema_version: String,
    package_id: String,
    source_tree_id: String,
    source_tree_sha256: String,
    compiler_path: String,
    compiler_sha256: String,
    files: BTreeMap<String, String>,
    proof_generator_status: String,
    required_toolchain: Vec<String>,
    claim_boundary: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err("usage: certified_proof_path8 <tree.json> <out-dir>".into());
    }
    let tree_path = Path::new(&args[1]);
    let out_dir = Path::new(&args[2]);
    let tree: CertifiedBisectionTree = serde_json::from_str(&std::fs::read_to_string(tree_path)?)?;
    verify_certified_bisection_tree_bounded(&tree)?;
    let root = tree
        .nodes
        .first()
        .ok_or("certified tree has no root split")?;
    let CertifiedSplitResult::Optimal { assignment, .. } = &root.certificate.result else {
        return Err("certified tree root is infeasible".into());
    };
    std::fs::create_dir_all(out_dir)?;
    write_discovery_package(
        out_dir,
        "optimal",
        &root.instance,
        assignment.clone(),
        "bounded-oracle-optimum",
        false,
    )?;
    write_discovery_package(
        out_dir,
        "compact-optimal",
        &root.instance,
        assignment.clone(),
        "bounded-oracle-compact-connectivity",
        true,
    )?;
    if root.instance.unit_ids.len() != 8 || root.instance.k_parent != 4 {
        return Err("the committed prototype requires the path8-k4 fixture".into());
    }
    write_discovery_package(
        out_dir,
        "suboptimal",
        &root.instance,
        vec![0, 0, 0, 1, 1, 1, 1, 1],
        "deliberate-counterexample-fixture",
        false,
    )?;
    write_discovery_package(
        out_dir,
        "compact-suboptimal",
        &root.instance,
        vec![0, 0, 0, 1, 1, 1, 1, 1],
        "deliberate-compact-counterexample-fixture",
        true,
    )?;

    let mut files = BTreeMap::new();
    for path in sorted_files(out_dir)? {
        let relative = path
            .strip_prefix(out_dir)?
            .to_string_lossy()
            .replace('\\', "/");
        files.insert(relative, sha256_file(&path)?);
    }
    let manifest = ProofPrototypeManifest {
        schema_version: "certified-proof-backend-prototype-package-v1".to_string(),
        package_id: "certified-proof-path8-root".to_string(),
        source_tree_id: tree.tree_id,
        source_tree_sha256: sha256_file(tree_path)?,
        compiler_path: "crates/bisect-ilp/src/proof_backend.rs".to_string(),
        compiler_sha256: sha256_file(Path::new(
            "crates/bisect-ilp/src/proof_backend.rs",
        ))?,
        files,
        proof_generator_status: "smoke-verified-proof-not-bundled".to_string(),
        required_toolchain: vec!["roundingsat".to_string(), "veripb".to_string()],
        claim_boundary: "Deterministic OPB decision compiler and bounded SAT/UNSAT classification; external proof compatibility is verified in the separate proof-toolchain-smoke package, but these requests do not bundle proofs.".to_string(),
    };
    std::fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    println!("Certified proof backend prototype: PASS");
    Ok(())
}

fn write_discovery_package(
    out_dir: &Path,
    name: &str,
    instance: &bisect_ilp::CertifiedSplitInstance,
    assignment: Vec<u8>,
    method: &str,
    compact_connectivity: bool,
) -> Result<(), Box<dyn Error>> {
    let directory = out_dir.join(name);
    std::fs::create_dir_all(&directory)?;
    let discovery = certified_split_discovery(instance, "prototype", None, method, assignment)?;
    std::fs::write(
        directory.join("discovery.json"),
        serde_json::to_string_pretty(&discovery)?,
    )?;
    let artifacts = if compact_connectivity {
        compile_certified_split_compact_proof_requests(instance, &discovery)?
    } else {
        compile_certified_split_proof_requests(instance, &discovery)?
    };
    for artifact in artifacts {
        let stem = match artifact.request.stage {
            bisect_ilp::CertifiedDecisionStage::PopulationLowerBound => "01-population",
            bisect_ilp::CertifiedDecisionStage::BoundaryLowerBound => "02-boundary",
            bisect_ilp::CertifiedDecisionStage::CanonicalTieBreak => "03-canonical",
        };
        std::fs::write(directory.join(format!("{stem}.opb")), artifact.opb)?;
        std::fs::write(
            directory.join(format!("{stem}.request.json")),
            serde_json::to_string_pretty(&artifact.request)?,
        )?;
    }
    Ok(())
}

fn sorted_files(root: &Path) -> Result<Vec<std::path::PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    for directory in [
        "optimal",
        "compact-optimal",
        "suboptimal",
        "compact-suboptimal",
    ] {
        for entry in std::fs::read_dir(root.join(directory))? {
            let path = entry?.path();
            if path.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}
