use rcount_core::synthetic_summary_basic_package;
use rcount_district::{aggregate_package_districts, synthetic_summary_basic_rplan_document};
use rcount_io::{synthetic_summary_basic_manifest, write_package_dir};
use rplan_io::write_rplan_string;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = std::path::PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("district-aggregation-rplan");
    let package_dir = base_dir.join("package");
    let package = synthetic_summary_basic_package();
    let manifest = synthetic_summary_basic_manifest(&package)?;
    write_package_dir(&package_dir, &manifest, &package)?;

    let plan_doc = synthetic_summary_basic_rplan_document()?;
    std::fs::create_dir_all(&base_dir)?;
    std::fs::write(
        base_dir.join("plan.rplan.json"),
        write_rplan_string(&plan_doc)?,
    )?;

    let transcript = aggregate_package_districts(
        &package,
        &plan_doc.plan,
        None,
        "syn-2024-mayor",
        rcount_core::CountStatus::Canvassed,
    )?;
    std::fs::write(
        base_dir.join("district-aggregation-transcript.json"),
        serde_json::to_vec_pretty(&transcript)?,
    )?;
    println!("wrote {}", base_dir.display());
    Ok(())
}
