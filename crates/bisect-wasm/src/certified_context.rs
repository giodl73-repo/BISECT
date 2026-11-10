//! Native certified-recursive context/package path adapted to in-memory delivery.
use std::collections::BTreeMap;
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {pub districts:usize,pub tolerance:f64,pub exact_fixture_limit:usize,pub generated_at:String}
fn digest(text:&str)->String{format!("{:x}",Sha256::digest(text.as_bytes()))}
pub(crate) fn solve(text:&str,settings:Settings)->Result<Value,String>{inner(text,settings).map_err(|e|e.to_string())}
fn inner(text:&str,settings:Settings)->Result<Value,Box<dyn std::error::Error>>{
 if text.len()>8*1024*1024{return Err("Certified context exceeds 8 MiB".into());}
 if !(2..=24).contains(&settings.districts)||!(2..=24).contains(&settings.exact_fixture_limit)||!settings.tolerance.is_finite()||!(0.0..=100.0).contains(&settings.tolerance)||settings.generated_at.is_empty()||settings.generated_at.len()>200||settings.generated_at.chars().any(char::is_control){return Err("Invalid bounded certified-recursive settings".into());}
 let context=rplan_io::read_rctx_str(text)?;
 if context.units.unit_ids.len()>settings.exact_fixture_limit{return Err("Certified recursive context exceeds the selected exact unit limit".into());}
 let populations=context.populations.as_ref().ok_or("RCTX populations are required for certified splitting")?;
 let graph=context.graph.as_ref().ok_or("RCTX graph is required for certified split solving")?;
 let mut edges=BTreeMap::new();
 for (left,neighbors) in graph.adjacency.iter().enumerate(){for edge in neighbors{
  let right=usize::try_from(edge.to)?;if left>=right{continue;}
  let weight=match edge.weight{None=>1,Some(value) if value.is_finite()&&value>0.0&&(value-value.round()).abs()<1e-9&&value<=u64::MAX as f64=>value as u64,Some(_)=>return Err("Certified split edges require positive integer weights".into())};
  edges.insert((left,right),bisect_ilp::ExactEdge{left,right,weight});
 }}
 let (k_left,k_right)=bisect_ilp::canonical_seat_split(settings.districts)?;
 let root=bisect_ilp::CertifiedSplitInstance{schema_version:bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.into(),model_id:bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.into(),node_path:String::new(),parent_certificate_id:None,unit_universe_hash:bisect_ilp::certified_split_unit_universe_hash(&context.units.unit_ids)?,unit_ids:context.units.unit_ids.clone(),populations:populations.clone(),edges:edges.into_values().collect(),k_parent:settings.districts,k_left,k_right,orientation_rule:bisect_ilp::canonical_orientation_rule(k_left,k_right)};
 crate::certified_exact::bounded(&root)?;
 let tree=bisect_ilp::solve_certified_bisection_tree_bounded(root)?;
 bisect_ilp::verify_certified_bisection_tree_bounded(&tree)?;
 let assignment=recursive_tree_assignment(&context,&tree)?;
 let generated_at=settings.generated_at.clone();
 let context=&context;let tree=&tree;let assignment=&assignment;
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "XX".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: assignment.to_vec(),
            k: settings.districts,
            display_labels: (1..=settings.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "certified-recursive-bisection".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some(
                "Bounded exact certification of the standard-bisect recursive tree".to_string(),
            ),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                ("crate".to_string(), serde_json::json!("bisect-ilp")),
                (
                    "method".to_string(),
                    serde_json::json!("certified-recursive"),
                ),
                (
                    "certified_tree_id".to_string(),
                    serde_json::json!(tree.tree_id),
                ),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "CERTIFIED_RECURSIVE_BISECTION_BOUNDED_V1".to_string(),
        jurisdiction,
        chamber: rplan_audit::Chamber::Congressional,
        year: document.plan.units.year.unwrap_or(2020),
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: settings.tolerance,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::NotEvaluated,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    };
    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-ilp",
        env!("CARGO_PKG_VERSION"),
        "certified-recursive",
        Vec::new(),
        serde_json::json!({
            "certified_tree_id": tree.tree_id,
            "root_unit_universe_hash": tree.root_unit_universe_hash,
            "split_count": tree.nodes.len(),
            "leaf_count": tree.leaves.len(),
        }),
    )?;
    let audit = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect-wasm".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: None,
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "bounded-certified-recursive-enumeration".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some("standard-bisect-tree".to_string()),
                time_limit_secs: None,
                optimality_gap: Some(0.0),
            }),
        },
        &[
            rplan_audit::AuditConstraint::PlanShape,
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        &generated_at,
        Some(lineage),
    )?;
    if matches!(audit.result, rplan_audit::AuditResult::Fail) {
        return Err("certified recursive solution failed RPLAN audit".into());
    }

 let document_json=rplan_io::write_rplan_string(&document)?;
 rplan_io::read_rplan_str(&document_json)?;
 let context_json=rplan_io::write_rctx_string(context)?;
 let tree_json=serde_json::to_string_pretty(tree)?;
 let audit_json=serde_json::to_string_pretty(&audit)?;
 let mut files=BTreeMap::from([("certified-bisection-tree.json".to_string(),tree_json.clone()),("exact.rplan".to_string(),document_json.clone()),("exact.rctx".to_string(),context_json.clone()),("audit-certificate.json".to_string(),audit_json.clone())]);
 let hashes=files.iter().map(|(name,text)|(name.clone(),digest(text))).collect::<BTreeMap<_,_>>();
 let manifest=json!({"schema_version":"certified-recursive-bisection-package-v1","method":"certified-recursive","tree_id":tree.tree_id,"root_unit_universe_hash":tree.root_unit_universe_hash,"districts":tree.k,"split_count":tree.nodes.len(),"leaf_count":tree.leaves.len(),"declared_population_tolerance_percent":settings.tolerance,"files":hashes,"claim_boundary":format!("Bounded exact certification of the sequential standard-bisect tree with at most {} units per split; not global unrestricted-map optimality or block-scale readiness.",settings.exact_fixture_limit)});
 files.insert("certified-tree-package-manifest.json".into(),serde_json::to_string_pretty(&manifest)?);
 let file_sha256=files.iter().map(|(name,text)|(name.clone(),digest(text))).collect::<BTreeMap<_,_>>();
 Ok(json!({"schema_version":"bisect-certified-tree-output-v1","status":"completed","tree_id":tree.tree_id,"districts":tree.k,"tree_json":tree_json,"document_json":document_json,"context_json":context_json,"audit_certificate_json":audit_json,"files":files,"file_sha256":file_sha256,"context_hash":context.context_hash,"context_source_sha256":digest(text),"total_population_text":populations.iter().map(|p|*p as i128).sum::<i128>().to_string(),"scope":"Native bounded certified-recursive solve, complete tree reenumeration, RPLAN/RCTX export and shape/population/contiguity audit under the supplied tolerance. Each local split is exact; no global plan optimum or legal compliance is certified. Context source integrity is checked, not source truth. Runtime is bisect-wasm; no native executable identity or external witness is fabricated. Files retain native schemas and serialization. No geometry is invented; select matching geographic units separately for a map."}))
}
fn recursive_tree_assignment(
    context: &rplan_core::RplanContext,
    tree: &bisect_ilp::CertifiedBisectionTree,
) -> Result<Vec<u32>,Box<dyn std::error::Error>> {
    let index = context
        .units
        .unit_ids
        .iter()
        .enumerate()
        .map(|(index, unit_id)| (unit_id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut assignment = vec![u32::MAX; context.units.unit_ids.len()];
    for leaf in &tree.leaves {
        let district = u32::try_from(leaf.district_index)
            .map_err(|_| "recursive district index does not fit u32")?;
        for unit_id in &leaf.unit_ids {
            let unit_index = *index
                .get(unit_id.as_str())
                .ok_or_else(|| format!("tree leaf references unknown unit {unit_id}"))?;
            if assignment[unit_index] != u32::MAX {
                return Err(format!("tree leaf assigns unit {unit_id} more than once").into());
            }
            assignment[unit_index] = district;
        }
    }
    if assignment.contains(&u32::MAX) {
        return Err("recursive tree does not assign every context unit".into());
    }
    Ok(assignment)
}
