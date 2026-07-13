use bisect_ilp::{
    build_certified_single_district, certified_split_unit_universe_hash,
    verify_certified_single_district, CertifiedSingleDistrictCertificate,
    CertifiedSingleDistrictInstance, ExactEdge, CERTIFIED_SINGLE_INSTANCE_SCHEMA_VERSION,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

#[derive(Serialize)]
struct PackageManifest {
    schema_version: String,
    package_id: String,
    status: String,
    instance_hash: String,
    certificate_id: String,
    files: BTreeMap<String, String>,
    claim_boundary: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args_os().collect::<Vec<_>>();
    if args.len() < 3 {
        return Err(
            "usage: certified_single_district <build|verify-package> <rctx|package-dir> [out-dir]"
                .into(),
        );
    }
    match args[1].to_string_lossy().as_ref() {
        "build" if args.len() == 4 => build(Path::new(&args[2]), Path::new(&args[3])),
        "verify-package" if args.len() == 3 => verify_package(Path::new(&args[2])),
        _ => Err("invalid certified_single_district arguments".into()),
    }
}

fn build(context_path: &Path, out_dir: &Path) -> Result<(), Box<dyn Error>> {
    let context = rplan_io::read_rctx_str(&std::fs::read_to_string(context_path)?)?;
    let populations = context
        .populations
        .as_ref()
        .ok_or("RCTX populations are required")?
        .clone();
    let graph = context.graph.as_ref().ok_or("RCTX graph is required")?;
    let mut edges = Vec::new();
    for (left, neighbors) in graph.adjacency.iter().enumerate() {
        for edge in neighbors {
            let right = usize::try_from(edge.to)?;
            if left >= right {
                continue;
            }
            let weight = edge.weight.unwrap_or(1.0);
            if !weight.is_finite()
                || weight <= 0.0
                || (weight - weight.round()).abs() > 1e-9
                || weight > u64::MAX as f64
            {
                return Err("RCTX edge weight must be a positive integer".into());
            }
            edges.push(ExactEdge {
                left,
                right,
                weight: weight as u64,
            });
        }
    }
    edges.sort_by_key(|edge| (edge.left, edge.right));
    let instance = CertifiedSingleDistrictInstance {
        schema_version: CERTIFIED_SINGLE_INSTANCE_SCHEMA_VERSION.to_string(),
        unit_universe_hash: certified_split_unit_universe_hash(&context.units.unit_ids)?,
        unit_ids: context.units.unit_ids.clone(),
        populations,
        edges,
    };
    let certificate = build_certified_single_district(&instance)?;
    verify_certified_single_district(&instance, &certificate)?;
    std::fs::create_dir_all(out_dir)?;
    let instance_path = out_dir.join("single-district-instance.json");
    let certificate_path = out_dir.join("single-district-certificate.json");
    std::fs::write(&instance_path, serde_json::to_string_pretty(&instance)?)?;
    std::fs::write(
        &certificate_path,
        serde_json::to_string_pretty(&certificate)?,
    )?;
    let files = BTreeMap::from([
        (
            "single-district-certificate.json".to_string(),
            sha256_file(&certificate_path)?,
        ),
        (
            "single-district-instance.json".to_string(),
            sha256_file(&instance_path)?,
        ),
    ]);
    let manifest = PackageManifest {
        schema_version: "certified-single-district-package-v1".to_string(),
        package_id: certificate.certificate_id.clone(),
        status: "verified".to_string(),
        instance_hash: certificate.instance_hash.clone(),
        certificate_id: certificate.certificate_id,
        files,
        claim_boundary: "Complete connected wall-to-wall one-district assignment; no nontrivial boundary optimization exists.".to_string(),
    };
    std::fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    println!("Certified single-district package: PASS");
    Ok(())
}

fn verify_package(package: &Path) -> Result<(), Box<dyn Error>> {
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(package.join("manifest.json"))?)?;
    for (name, expected) in manifest["files"]
        .as_object()
        .ok_or("manifest files must be an object")?
    {
        if expected.as_str() != Some(&sha256_file(&package.join(name))?) {
            return Err(format!("package hash mismatch: {name}").into());
        }
    }
    let instance: CertifiedSingleDistrictInstance = serde_json::from_str(
        &std::fs::read_to_string(package.join("single-district-instance.json"))?,
    )?;
    let certificate: CertifiedSingleDistrictCertificate = serde_json::from_str(
        &std::fs::read_to_string(package.join("single-district-certificate.json"))?,
    )?;
    verify_certified_single_district(&instance, &certificate)?;
    println!("Certified single-district package verification: PASS");
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}
