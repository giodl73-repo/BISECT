use bisect_ilp::{verify_certified_bisection_tree_bounded, CertifiedBisectionTree};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::path::Path;

#[derive(Deserialize)]
struct PackageManifest {
    tree_id: String,
    districts: usize,
    files: BTreeMap<String, String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() == 3 && args[1] == "verify" {
        verify_tree(Path::new(&args[2]))?;
    } else if args.len() == 3 && args[1] == "verify-package" {
        verify_package(Path::new(&args[2]))?;
    } else {
        return Err(
            "usage: certified_recursive verify <tree.json> | verify-package <package-dir>".into(),
        );
    }
    Ok(())
}

fn verify_tree(path: &Path) -> Result<CertifiedBisectionTree, Box<dyn Error>> {
    let tree: CertifiedBisectionTree = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    verify_certified_bisection_tree_bounded(&tree)?;
    println!("Certified recursive tree verification: PASS");
    Ok(tree)
}

fn verify_package(package_dir: &Path) -> Result<(), Box<dyn Error>> {
    let tree = verify_tree(&package_dir.join("certified-bisection-tree.json"))?;
    let plan =
        rplan_io::read_rplan_str(&std::fs::read_to_string(package_dir.join("exact.rplan"))?)?;
    let context =
        rplan_io::read_rctx_str(&std::fs::read_to_string(package_dir.join("exact.rctx"))?)?;
    let audit: rplan_audit::AuditCertificate = serde_json::from_str(&std::fs::read_to_string(
        package_dir.join("audit-certificate.json"),
    )?)?;
    rplan_audit::verify_audit_certificate(&audit, Some(&plan.plan), Some(&context))?;
    if plan.plan.k != tree.k || plan.plan.units.unit_ids != context.units.unit_ids {
        return Err("plan shape or unit order differs from the certified tree package".into());
    }
    let unit_index = context
        .units
        .unit_ids
        .iter()
        .enumerate()
        .map(|(index, unit_id)| (unit_id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut expected_assignment = vec![u32::MAX; context.units.unit_ids.len()];
    for leaf in &tree.leaves {
        let district = u32::try_from(leaf.district_index)?;
        for unit_id in &leaf.unit_ids {
            let index = *unit_index
                .get(unit_id.as_str())
                .ok_or("tree leaf references a unit outside the package context")?;
            if expected_assignment[index] != u32::MAX {
                return Err("tree assigns a package unit more than once".into());
            }
            expected_assignment[index] = district;
        }
    }
    if expected_assignment.contains(&u32::MAX) || plan.plan.assignment != expected_assignment {
        return Err("RPLAN assignment does not match certified tree leaves".into());
    }

    let manifest: PackageManifest = serde_json::from_str(&std::fs::read_to_string(
        package_dir.join("certified-tree-package-manifest.json"),
    )?)?;
    if manifest.tree_id != tree.tree_id || manifest.districts != tree.k {
        return Err("package manifest identity differs from the certified tree".into());
    }
    let expected_files = BTreeSet::from([
        "audit-certificate.json".to_string(),
        "certified-bisection-tree.json".to_string(),
        "exact.rctx".to_string(),
        "exact.rplan".to_string(),
    ]);
    if manifest.files.keys().cloned().collect::<BTreeSet<_>>() != expected_files {
        return Err("package manifest file inventory mismatch".into());
    }
    for (name, expected_hash) in manifest.files {
        let actual_hash = sha256_file(&package_dir.join(name))?;
        if actual_hash != expected_hash {
            return Err(format!(
                "package file hash mismatch: expected {expected_hash}, found {actual_hash}"
            )
            .into());
        }
    }
    println!("Certified recursive package verification: PASS");
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}
