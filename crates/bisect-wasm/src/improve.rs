//! Native one-move improvement and its portable output package.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub tolerance_percent: f64,
    pub label: Option<String>,
    pub generated_at_utc: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (String,String) {
        use rplan_core::*;
        let mut units = PlanUnitIndex {unit_kind:UnitKind::Imported,state:Some("ZZ".into()),year:Some(2020),canonical_order:CanonicalOrder::ExplicitUnitIds,
            unit_ids:(0..6).map(|i|format!("u{i}")).collect(),unit_universe_hash:String::new(),source_id:Some("fixture".into())};
        units.unit_universe_hash=units.compute_unit_universe_hash().unwrap();
        let plan = rplan_io::RplanDocument {rplan_version:rplan_io::RPLAN_V02.into(),plan:DistrictPlan {schema_version:DISTRICT_PLAN_SCHEMA_VERSION.into(),units:units.clone(),assignment:vec![0,0,0,1,1,1],k:2,display_labels:vec!["1".into(),"2".into()],allow_empty_districts:false},
            metadata:rplan_io::RplanMetadataV02 {label:"fixture".into(),jurisdiction:"ZZ".into(),chamber:"congressional".into(),created_at:"2026-10-04T00:00:00Z".into(),description:None},provenance:Default::default(),geometry:None,extensions:Default::default()};
        let adjacency = [vec![1,2],vec![0],vec![0,3,4,5],vec![2,4],vec![2,3,5],vec![2,4]];
        let context = RplanContext {rctx_version:RCTX_VERSION.into(),context_hash:String::new(),units,graph:Some(UnitGraph {edge_semantics:EdgeSemantics::Undirected,
            adjacency:adjacency.iter().map(|neighbors|neighbors.iter().map(|&to|UnitEdge {to,kind:EdgeKind::Boundary,weight:None}).collect()).collect()}),populations:Some(vec![100;6]),subdivisions:None,demographics:None,geometry:None,source_hashes:Default::default()};
        (rplan_io::write_rplan_string(&plan).unwrap(),rplan_io::write_rctx_string(&context).unwrap())
    }
    fn settings() -> Settings {Settings {tolerance_percent:34.0,label:Some("improved-fixture".into()),generated_at_utc:"2026-10-04T00:00:00Z".into()}}
    #[test]
    fn native_package_has_exact_hashes_lineage_and_verified_certificate() {
        let (plan,context)=fixture();let output=run(&plan,&context,settings()).unwrap();
        assert_eq!(output["summary"]["initial_edge_cut"],3);assert_eq!(output["summary"]["final_edge_cut"],1);
        let files=output["files"].as_object().unwrap();assert_eq!(files.len(),5);
        for (name,text) in files {assert_eq!(output["file_sha256"][name],hash(text.as_str().unwrap()));}
        let improved=rplan_io::read_rplan_str(files["improved.rplan"].as_str().unwrap()).unwrap();
        assert_eq!(improved.plan.assignment,vec![0,0,1,1,1,1]);
        let context=rplan_io::read_rctx_str(files["improved.rctx"].as_str().unwrap()).unwrap();
        let cert:rplan_audit::AuditCertificate=serde_json::from_str(files["audit-certificate.json"].as_str().unwrap()).unwrap();
        rplan_audit::verify_audit_certificate(&cert,Some(&improved.plan),Some(&context)).unwrap();
        let lineage=cert.algorithm_lineage.as_ref().unwrap();
        assert_eq!(lineage.parent_plan_hashes,vec![output["parent_plan_hash"].as_str().unwrap()]);
        assert_eq!(lineage.extra["summary_sha256"],output["file_sha256"]["local-search-summary.json"]);
        let manifest:bisect_report::PlanManifest=serde_json::from_str(files["manifest.json"].as_str().unwrap()).unwrap();
        assert_eq!(manifest.audit_certificate_sha256.as_deref(),output["file_sha256"]["audit-certificate.json"].as_str());
        assert_eq!(cert.runtime.binary_name,"bisect-wasm");
    }
    #[test]
    fn rejects_invalid_settings_and_context_identity() {
        let (plan,context)=fixture();
        for tolerance in [f64::NAN,-1.0,f64::INFINITY] {let mut s=settings();s.tolerance_percent=tolerance;assert!(run(&plan,&context,s).is_err());}
        let mut s=settings();s.generated_at_utc.clear();assert!(run(&plan,&context,s).is_err());
        let mut ctx=rplan_io::read_rctx_str(&context).unwrap();ctx.units.unit_ids.swap(0,1);ctx.units.unit_universe_hash=ctx.units.compute_unit_universe_hash().unwrap();
        assert!(run(&plan,&rplan_io::write_rctx_string(&ctx).unwrap(),settings()).is_err());
        let mut raw:Value=serde_json::from_str(&plan).unwrap();raw["plan"]["k"]=json!(u64::MAX);
        assert!(run(&raw.to_string(),&context,settings()).is_err());
    }
}

fn hash(text: &str) -> String { format!("{:x}", Sha256::digest(text.as_bytes())) }

pub(crate) fn run(plan_text: &str, context_text: &str, settings: Settings) -> Result<Value, String> {
    // Preflight allocation before native conversion, including legacy RPLAN.
    let raw: Value = crate::certified_exact::parse(plan_text)?;
    let k = if raw["rplan_version"] == "0.1" { raw["metadata"]["num_districts"].as_u64() }
        else { raw["plan"]["k"].as_u64() };
    if !k.is_some_and(|k| (1..=500).contains(&k)) { return Err("Improvement supports 1 through 500 districts.".into()); }
    if context_text.len() > 8 * 1024 * 1024 { return Err("Context exceeds 8 MiB.".into()); }
    if !settings.tolerance_percent.is_finite() || settings.tolerance_percent < 0.0 {
        return Err("Population tolerance percent must be finite and nonnegative.".into());
    }
    for field in settings.label.iter().chain(std::iter::once(&settings.generated_at_utc)) {
        if field.trim().is_empty() || field.len() > 200 { return Err("Label and recorded timestamp must be nonempty and at most 200 bytes.".into()); }
    }
    let mut document = rplan_io::read_rplan_str(plan_text).map_err(|e|e.to_string())?;
    let context = rplan_io::read_rctx_str(context_text).map_err(|e|e.to_string())?;
    let graph = context.graph.as_ref().ok_or("RCTX graph is required.")?;
    // One-move scans candidates with full connectivity checks; bound work in the worker.
    if document.plan.assignment.len() > 10_000 || graph.adjacency.iter().map(Vec::len).sum::<usize>() > 100_000 {
        return Err("Browser one-move supports at most 10,000 units and 100,000 graph arcs.".into());
    }
    if graph.edge_semantics != rplan_core::EdgeSemantics::Undirected { return Err("Improvement requires an undirected graph.".into()); }
    let measurements = rplan_audit::measurements::measure_plan(&document.plan, Some(&context))?;
    let populations = context.populations.as_ref().ok_or("RCTX populations are required.")?;
    let adjacency: Vec<Vec<usize>> = graph.adjacency.iter().map(|edges| edges.iter().map(|e|e.to as usize).collect()).collect();
    let parent_hash = document.plan.plan_hash().map_err(|e|e.to_string())?;
    let result = bisect_local_search::improve_one_move(&adjacency, populations,
        &document.plan.assignment.iter().map(|&d|d as usize).collect::<Vec<_>>(),
        bisect_local_search::LocalSearchConfig::one_move(document.plan.k, settings.tolerance_percent / 100.0)).map_err(|e|e.to_string())?;
    document.plan.assignment = result.assignment.iter().map(|&d|d as u32).collect();
    document.metadata.label = settings.label.unwrap_or_else(||format!("{}-improved",document.metadata.label));
    document.metadata.created_at = settings.generated_at_utc.clone();
    for (key,value) in [("name",json!("bisect improve")),("crate",json!("bisect-local-search")),("method",json!(result.summary.method))] {
        document.provenance.producer.insert(key.into(),value);
    }
    let summary_json = serde_json::to_string_pretty(&result.summary).map_err(|e|e.to_string())?;
    let mut extra = result.summary.algorithm_lineage_extra();
    extra["summary_path"] = json!("local-search-summary.json");
    extra["summary_sha256"] = json!(hash(&summary_json));
    let lineage = rplan_audit::AlgorithmLineage::new("bisect-local-search",env!("CARGO_PKG_VERSION"),result.summary.method.clone(),vec![parent_hash.clone()],extra).map_err(|e|e.to_string())?;
    let profile = rplan_audit::LegalProfile {
        schema_version:rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.into(),profile_id:"BISECT_LOCAL_SEARCH_IMPROVE_V1".into(),
        jurisdiction:document.metadata.jurisdiction.clone(),
        chamber:if document.metadata.chamber=="congressional" {rplan_audit::Chamber::Congressional} else {rplan_audit::Chamber::Custom(document.metadata.chamber.clone())},
        year:document.plan.units.year.unwrap_or(2020),population_tolerance:rplan_audit::PopulationToleranceRule::Percent {max_deviation_percent:settings.tolerance_percent},
        contiguity_required:true,county_split_rule:rplan_audit::SplitRule::CountOnly,municipal_split_rule:rplan_audit::SplitRule::NotEvaluated,
        nesting_rule:rplan_audit::NestingRule::NotEvaluated,vra_policy:rplan_audit::VraPolicy::NotEvaluated,
    };
    let certificate = crate::toolkit::audit_native(&document.plan,Some(&context),&profile,
        &[rplan_audit::AuditConstraint::Population,rplan_audit::AuditConstraint::Contiguity],&settings.generated_at_utc,Some(lineage.clone()))?;
    if certificate.result == rplan_audit::AuditResult::Fail { return Err("Improved plan failed native RPLAN audit.".into()); }
    rplan_audit::verify_audit_certificate(&certificate,Some(&document.plan),Some(&context)).map_err(|e|e.to_string())?;
    let certificate_json = serde_json::to_string_pretty(&certificate).map_err(|e|e.to_string())?;
    let result_label = match certificate.result {rplan_audit::AuditResult::Pass=>"pass",rplan_audit::AuditResult::Fail=>"fail",rplan_audit::AuditResult::PassWithWarnings=>"pass-with-warnings"};
    let manifest = bisect_report::PlanManifest {
        label:document.metadata.label.clone(),state_code:document.plan.units.state.clone().unwrap_or_else(||document.metadata.jurisdiction.clone()),
        year:document.plan.units.year.map(|y|y.to_string()).unwrap_or_default(),chamber:document.metadata.chamber.clone(),num_districts:document.plan.k,
        population_source:"rplan-context".into(),partition_mode:"local-search".into(),binary_version:env!("CARGO_PKG_VERSION").into(),created_at:settings.generated_at_utc,
        balance_tolerance_pct:settings.tolerance_percent,population_balance_valid:true,rplan_path:Some("improved.rplan".into()),rctx_path:Some("improved.rctx".into()),
        audit_certificate_path:Some("audit-certificate.json".into()),audit_certificate_sha256:Some(hash(&certificate_json)),audit_certificate_content_hash:Some(certificate.content_hash.clone()),
        audit_result:Some(result_label.into()),legal_profile_id:Some(profile.profile_id.clone()),context_hash:certificate.context_hash.clone(),n_units:document.plan.assignment.len(),unit_type:"rplan unit".into(),
        ..bisect_report::PlanManifest::default()
    };
    let files = std::collections::BTreeMap::from([
        ("improved.rplan",rplan_io::write_rplan_string(&document).map_err(|e|e.to_string())?),
        ("improved.rctx",rplan_io::write_rctx_string(&context).map_err(|e|e.to_string())?),
        ("local-search-summary.json",summary_json),("audit-certificate.json",certificate_json),
        ("manifest.json",serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?),
    ]);
    Ok(json!({"schema_version":"bisect-local-improvement-output-v1","files":files,
        "file_sha256":files.iter().map(|(name,text)|(*name,hash(text))).collect::<std::collections::BTreeMap<_,_>>(),
        "summary":result.summary,"moved_vertex":result.moved_vertex,"moved_from":result.moved_from,"moved_to":result.moved_to,
        "parent_plan_hash":parent_hash,"plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"context_hash":context.context_hash,
        "certificate_id":certificate.certificate_id,"audit_result":certificate.result,"total_population":measurements.total_population.map(|n|n.to_string()),
        "algorithm_lineage_json":serde_json::to_string_pretty(&lineage).map_err(|e|e.to_string())?,
        "profile_json":serde_json::to_string_pretty(&profile).map_err(|e|e.to_string())?,
        "scope":"Native best feasible single neighboring-unit move minimizing unweighted graph edge cut, including bridges. Tie breaking uses unit and district order. Population feasibility uses native f64 arithmetic. Package audit checks population and contiguity against the generated improvement profile; it does not establish global optimality or every legal requirement. Original plan metadata, extensions and geometry are retained; geometric boundary shapes are not recomputed from changed assignments. Browser runtime provenance is recorded."}))
}
