//! Native canonical-exhaustive context solver and native package serialization.
use std::collections::BTreeMap;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use crate::certified_context::Settings;
fn digest(text:&str)->String{format!("{:x}",Sha256::digest(text.as_bytes()))}
pub(crate) fn solve(text:&str,settings:Settings)->Result<Value,String>{inner(text,settings).map_err(|e|e.to_string())}
fn inner(text:&str,settings:Settings)->Result<Value,Box<dyn std::error::Error>>{
 if text.len()>8*1024*1024{return Err("Canonical context exceeds 8 MiB".into());}
 if settings.districts!=2||!(2..=24).contains(&settings.exact_fixture_limit)||!settings.tolerance.is_finite()||!(0.0..=100.0).contains(&settings.tolerance)||settings.generated_at.is_empty()||settings.generated_at.len()>200||settings.generated_at.chars().any(char::is_control){return Err("Canonical context requires exactly two districts, a 2-24 unit limit, bounded tolerance and timestamp".into());}
 let context=rplan_io::read_rctx_str(text)?;
 if context.units.unit_ids.len()>settings.exact_fixture_limit{return Err("Canonical context exceeds the selected exact unit limit".into());}
 let populations=context.populations.as_ref().ok_or("RCTX populations are required for canonical exact solving")?;
 let graph=context.graph.as_ref().ok_or("RCTX graph is required for canonical exact solving")?;
 let mut edges=BTreeMap::new();
 for (left,neighbors) in graph.adjacency.iter().enumerate(){for edge in neighbors{
  let right=usize::try_from(edge.to)?;if left>=right{continue;}
  let weight=match edge.weight{None=>1,Some(value) if value.is_finite()&&value>0.0&&(value-value.round()).abs()<1e-9&&value<=u64::MAX as f64=>value as u64,Some(_)=>return Err("Canonical exact edges require positive integer weights".into())};
  edges.insert((left,right),bisect_ilp::ExactEdge{left,right,weight});
 }}
 let instance=bisect_ilp::ExactCanonicalInstance{schema_version:bisect_ilp::EXACT_INSTANCE_SCHEMA_VERSION.into(),model_id:bisect_ilp::EXACT_MODEL_ID.into(),unit_ids:context.units.unit_ids.clone(),populations:populations.clone(),edges:edges.into_values().collect(),k:2};
 instance.validate()?;
 let artifacts=bisect_ilp::solve_exact_canonical_artifacts(&instance)?;
 bisect_ilp::verify_exact_canonical_artifacts(&instance,&artifacts.certificate,&artifacts.proof)?;
 let certificate=&artifacts.certificate;
 let mut files:BTreeMap<String,String>=BTreeMap::from([("exact-canonical-instance.json".into(),serde_json::to_string_pretty(&instance)?),("exact-canonical-certificate.json".into(),serde_json::to_string_pretty(certificate)?),("exact-canonical-proof.json".into(),serde_json::to_string_pretty(&artifacts.proof)?)]);
 let (status,assignment,objective)=match &certificate.result{bisect_ilp::ExactCertificateResult::Optimal{assignment,objective}=>("optimal",Some(assignment),Some(json!({"max_population_deviation_scaled":objective.primary.max_population_deviation_scaled.to_string(),"total_population_deviation_scaled":objective.primary.total_population_deviation_scaled.to_string(),"weighted_boundary_cut":objective.primary.weighted_boundary_cut.to_string()}))),bisect_ilp::ExactCertificateResult::Infeasible=>("infeasible",None,None)};
 if let Some(assignment)=assignment{
  let context=&context;let exact_certificate=certificate;let generated_at=settings.generated_at.clone();
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
            assignment: assignment.iter().map(|&district| district as u32).collect(),
            k: settings.districts,
            display_labels: (1..=settings.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "exact-canonical".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some("E0 bounded exact canonical benchmark solution".to_string()),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                ("crate".to_string(), serde_json::json!("bisect-ilp")),
                (
                    "method".to_string(),
                    serde_json::json!("canonical-exhaustive"),
                ),
                (
                    "exact_certificate_id".to_string(),
                    serde_json::json!(exact_certificate.certificate_id),
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
        profile_id: "EXACT_CANONICAL_E0_MODEL_V1".to_string(),
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
        "canonical-exhaustive",
        Vec::new(),
        serde_json::json!({
            "exact_certificate_id": exact_certificate.certificate_id,
            "instance_hash": exact_certificate.instance_hash,
            "model_id": exact_certificate.model_id,
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
                name: "bounded-exhaustive-enumeration".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some("exact-canonical-k2".to_string()),
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
        return Err("exact canonical solution failed RPLAN audit".into());
    }

 files.insert("exact.rplan".into(),rplan_io::write_rplan_string(&document)?);
 files.insert("exact.rctx".into(),rplan_io::write_rctx_string(context)?);
 files.insert("audit-certificate.json".into(),serde_json::to_string_pretty(&audit)?);
 }
 let hashes=files.iter().map(|(name,text)|(name.clone(),digest(text))).collect::<BTreeMap<_,_>>();
 let manifest=json!({"schema_version":"exact-canonical-package-manifest-v1","method":"canonical-exhaustive","result":status,"instance_hash":certificate.instance_hash,"exact_certificate_id":certificate.certificate_id,"exact_proof_id":certificate.proof.transcript_id,"declared_population_tolerance_percent":settings.tolerance,"files":hashes,"claim_boundary":format!("Bounded E0 k=2 exhaustive certificate with at most {} units; not national exact readiness or legal certification.",settings.exact_fixture_limit)});
 files.insert("exact-package-manifest.json".into(),serde_json::to_string_pretty(&manifest)?);
 let file_sha256=files.iter().map(|(name,text)|(name.clone(),digest(text))).collect::<BTreeMap<_,_>>();
 let artifacts_json=serde_json::to_string(&json!({"instance":instance,"certificate":certificate,"proof":artifacts.proof}))?;
 Ok(json!({"schema_version":"bisect-certified-canonical-context-v1","status":status,"districts":2,"assignment":assignment,"objective":objective,"certificate_id":certificate.certificate_id,"artifacts_json":artifacts_json,"document_json":files.get("exact.rplan"),"context_json":files.get("exact.rctx"),"audit_certificate_json":files.get("audit-certificate.json"),"files":files,"file_sha256":file_sha256,"context_hash":context.context_hash,"context_source_sha256":digest(text),"total_population_text":populations.iter().map(|p|*p as i128).sum::<i128>().to_string(),"scope":"Native canonical-exhaustive two-district context model, complete proof verification and native package files. Optimal outputs pass native shape/population/contiguity audit at the supplied tolerance. Infeasible outputs retain model/proof files and contain no RPLAN or successful plan audit. The exact lexicographic optimum applies to this supplied connected two-district model, not unrestricted maps, source truth or legal compliance. Audit runtime is bisect-wasm with no native executable/git identity fabricated. Geographic geometry is separate."}))
}
