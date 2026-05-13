use rcount_core::{synthetic_summary_basic_package, RctxReference};
use rcount_district::{aggregate_package_districts, synthetic_summary_basic_rplan_document};
use rcount_io::{synthetic_summary_basic_manifest, write_package_dir};
use rplan_io::{write_rctx_string, write_rplan_string};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = std::path::PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("district-aggregation-rplan");
    let package_dir = base_dir.join("package");
    let plan_doc = synthetic_summary_basic_rplan_document()?;
    let mut context = rplan_core::RplanContext {
        rctx_version: rplan_core::RCTX_VERSION.to_string(),
        context_hash: String::new(),
        units: plan_doc.plan.units.clone(),
        graph: None,
        populations: None,
        subdivisions: None,
        demographics: None,
        geometry: None,
        source_hashes: rplan_core::SourceHashes::default(),
    };
    context.context_hash = context.compute_context_hash()?;
    let crosswalks = context
        .units
        .unit_ids
        .iter()
        .map(|unit_id| rctx_core::CrosswalkRecord {
            crosswalk_id: "cw-summary-basic-identity".to_string(),
            from_context_hash: context.context_hash.clone(),
            to_context_hash: context.context_hash.clone(),
            from_unit_id: unit_id.clone(),
            to_unit_id: unit_id.clone(),
            weight: rctx_core::RationalWeight { num: 1, den: 1 },
            weight_kind: rctx_core::CrosswalkWeightKind::UnitCount,
            exhaustive: true,
            source_refs: Vec::new(),
        })
        .collect::<Vec<_>>();
    let crosswalk_hash = rctx_core::crosswalk_set_hash(&crosswalks)?;

    let mut package = synthetic_summary_basic_package();
    package.rctx_refs = vec![RctxReference {
        reference_id: "rctx:summary-basic-to-plan".to_string(),
        context_hash: context.context_hash.clone(),
        context_path: Some("context.rctx".to_string()),
        crosswalk_hash: Some(crosswalk_hash),
        crosswalk_path: Some("crosswalks.ndjson".to_string()),
        role: "aggregation-crosswalk".to_string(),
        note: Some(
            "Synthetic identity crosswalk for the V.8 district aggregation fixture.".to_string(),
        ),
    }];
    let manifest = synthetic_summary_basic_manifest(&package)?;
    write_package_dir(&package_dir, &manifest, &package)?;

    std::fs::create_dir_all(&base_dir)?;
    std::fs::write(
        base_dir.join("plan.rplan.json"),
        write_rplan_string(&plan_doc)?,
    )?;
    std::fs::write(base_dir.join("context.rctx"), write_rctx_string(&context)?)?;
    let crosswalk_text = crosswalks
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    std::fs::write(
        base_dir.join("crosswalks.ndjson"),
        format!("{crosswalk_text}\n"),
    )?;

    let transcript = aggregate_package_districts(
        &package,
        &plan_doc.plan,
        Some(&context),
        Some(&base_dir.join("crosswalks.ndjson")),
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
