//! Native plan audits with exact JSON text across the JavaScript boundary.
use rplan_audit::{AlgorithmLineage, AuditCertificate, AuditConstraint, LegalProfile};
use rplan_core::{DistrictPlan, RplanContext};
use serde_json::{json, Value};

/// Preserve legacy bytes unless whitespace alone exceeds the source boundary.
pub(crate) fn portable_context_json(context: &RplanContext) -> Result<String,String> {
    let text=context_json_for_export(context)?;
    if text.len()>8*1024*1024 {return Err("Context exceeds 8 MiB even without indentation.".into());}
    Ok(text)
}

/// Compact large exports without discarding an otherwise available context.
/// Each consuming tool still applies its own input boundary.
pub(crate) fn context_json_for_export(context: &RplanContext) -> Result<String,String> {
    let pretty=rplan_io::write_rctx_string(context).map_err(|e|e.to_string())?;
    if pretty.len()<=8*1024*1024 {return Ok(pretty);}
    let compact=rplan_io::write_rctx_compact_string(context).map_err(|e|e.to_string())?;
    rplan_io::read_rctx_str(&compact).map_err(|e|e.to_string())?;
    Ok(compact)
}

pub(crate) fn plan(text: &str) -> Result<DistrictPlan, String> {
    // Bound district allocation before invoking the native reader, including v0.1 conversion.
    if text.len()>8*1024*1024 {return Err("Plan exceeds 8 MiB.".into());}
    let raw: Value = serde_json::from_str(text).map_err(|e|format!("Invalid exact artifact: {e}"))?;
    let (k, n) = if raw["rplan_version"] == "0.1" {
        (raw["metadata"]["num_districts"].as_u64(), raw["assignments"].as_object().map(|a| a.len()))
    } else {
        (raw["plan"]["k"].as_u64(), raw["plan"]["units"]["unit_ids"].as_array().map(|a| a.len()))
    };
    if !k.is_some_and(|k| (1..=500).contains(&k)) || !n.is_some_and(|n| n <= 100_000) {
        return Err("Text audit supports at most 100,000 units and 1 through 500 districts.".into());
    }
    Ok(rplan_io::read_rplan_str(text).map_err(|e| e.to_string())?.plan)
}

pub(crate) fn context(text: Option<&str>) -> Result<Option<RplanContext>, String> {
    text.map(|text| {
        if text.len() > 8 * 1024 * 1024 { return Err("Context exceeds 8 MiB.".into()); }
        let ctx = rplan_io::read_rctx_str(text).map_err(|e| e.to_string())?;
        if ctx.units.unit_ids.len() > 100_000 || ctx.graph.as_ref().is_some_and(|g| g.adjacency.iter().map(Vec::len).sum::<usize>() > 2_000_000) {
            return Err("Text audit context exceeds the portable graph limit.".into());
        }
        Ok(ctx)
    }).transpose()
}

pub(crate) fn audit(document_json: &str, context_json: Option<&str>, profile_json: &str,
    constraints: Vec<AuditConstraint>, generated_at_utc: String, lineage_json: Option<&str>) -> Result<Value, String> {
    if constraints.len() > 6 || generated_at_utc.trim().is_empty() || generated_at_utc.len() > 200 {
        return Err("Supply at most six audit checks and a nonempty recorded timestamp of at most 200 bytes.".into());
    }
    let lineage:Option<AlgorithmLineage>=lineage_json.map(crate::certified_exact::parse).transpose()?;
    if let Some(lineage)=&lineage { validate_lineage(lineage)?; }
    let profile: LegalProfile = crate::certified_exact::parse(profile_json)?;
    let plan = plan(document_json)?;
    let context = context(context_json)?;
    // Shared measurements validate ordered identity and checked population sums before auditing.
    let measurements = rplan_audit::measurements::measure_plan(&plan, context.as_ref())?;
    let certificate = crate::toolkit::audit_native(&plan, context.as_ref(), &profile, &constraints, &generated_at_utc, lineage)?;
    let certificate_json = serde_json::to_string_pretty(&certificate).map_err(|e| e.to_string())?;
    Ok(json!({"schema_version":"bisect-text-plan-audit-v1","certificate_json":certificate_json,
        "certificate_id":certificate.certificate_id,"plan_hash":certificate.plan_hash,"context_hash":certificate.context_hash,
        "result":certificate.result,"total_population":measurements.total_population.map(|n|n.to_string()),
        "checks":certificate.checks.iter().map(|check|json!({"name":check.name,"status":check.status,"summary":check.summary})).collect::<Vec<_>>(),
        "scope":"Native audit of supplied plan/context against supplied profile and selected checks. Counts remain exact in the certificate JSON text. Pass refers to this profile, not every legal requirement. Source truth, recorded runtime provenance and solver optimality are not authenticated."}))
}

pub(crate) fn verify(certificate_json: &str, document_json: Option<&str>, context_json: Option<&str>) -> Result<Value, String> {
    let certificate: AuditCertificate = crate::certified_exact::parse(certificate_json)?;
    if let Some(lineage)=&certificate.algorithm_lineage { validate_lineage(lineage)?; }
    let plan = document_json.map(plan).transpose()?;
    let context = context(context_json)?;
    if let Some(plan) = &plan {
        rplan_audit::measurements::measure_plan(plan, context.as_ref())?;
    }
    let verified = rplan_audit::verify_audit_certificate(&certificate, plan.as_ref(), context.as_ref()).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-text-plan-audit-verification-v1","verified":true,
        "certificate_id":verified.certificate_id,"content_hash":verified.content_hash,
        "plan_hash":verified.plan_hash,"context_hash":verified.context_hash,"result":verified.result,
        "plan_supplied":plan.is_some(),"context_supplied":context.is_some(),
        "scope":"Native certificate content/hash and supplied plan/context binding verification. Recorded check outcomes are not rerun; generating a fresh audit recomputes selected checks. No source authentication or solver optimality proof."}))
}

fn validate_lineage(lineage:&AlgorithmLineage)->Result<(),String>{
    let computed=rplan_core::canonical_sha256(&lineage.extra).map_err(|e|e.to_string())?;
    if lineage.parameters_hash!=computed {return Err("Algorithm lineage parameters hash mismatch.".into());}Ok(())
}

pub(crate) fn validate(text:&str)->Result<Value,String>{
 let validated=plan(text)?;
 let document=rplan_io::read_rplan_str(text).map_err(|e|e.to_string())?;
 Ok(json!({"schema_version":"bisect-text-plan-validation-v1","document_json":rplan_io::write_rplan_string(&document).map_err(|e|e.to_string())?,
  "plan_hash":validated.plan_hash().map_err(|e|e.to_string())?,"units":validated.assignment.len(),"districts":validated.k,
  "scope":"Native RPLAN shape and ordered-unit identity validation. Geometry is a separate supplied display source; legal compliance and geometry authenticity are not certified."}))
}

/// Native CLI contiguity report on exactly matched in-memory RPLAN/RCTX inputs.
pub(crate) fn contiguity(document_json:&str,context_json:&str)->Result<Value,String>{
 use sha2::{Digest,Sha256};
 let plan=plan(document_json)?;
 let ctx=context(Some(context_json))?.ok_or("A matching RCTX graph is required.")?;
 rplan_audit::measurements::measure_plan(&plan,Some(&ctx))?;
 let graph=ctx.graph.as_ref().ok_or("RCTX graph is unavailable; contiguity was not evaluated.")?;
 let adjacency=graph.adjacency.iter().map(|row|row.iter().map(|e|e.to as usize).collect::<Vec<_>>()).collect::<Vec<_>>();
 let assignments=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect();
 let report=bisect_analysis::contiguity::try_check_contiguity(&assignments,&adjacency,&plan.units.unit_ids,plan.k).map_err(|e|e.to_string())?;
 let report_json=serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?;
 Ok(json!({"schema_version":"bisect-plan-contiguity-v1","report":report,"report_json":report_json,
 "report_sha256":format!("{:x}",Sha256::digest(report_json.as_bytes())),
 "plan_source_sha256":format!("{:x}",Sha256::digest(document_json.as_bytes())),
 "context_source_sha256":format!("{:x}",Sha256::digest(context_json.as_bytes())),
 "scope":"Native CLI BFS contiguity analysis on the supplied matching RCTX graph. Every recorded edge is included, including custom and bridge edges, irrespective of weights. Native empty districts are contiguous with zero components. Disconnected tracts identify non-primary components under native largest-component/tie ordering. This is graph connectivity, not geographic contiguity or source authentication. Native county names are display context. No legal profile, certificate or optimality claim is implied."}))
}

/// Native manifest schema with metadata derived from supplied validated portable artifacts.
pub(crate) fn report_manifest(document_json:&str,context_json:&str)->Result<Value,String>{
 use sha2::{Digest,Sha256};
 let plan=plan(document_json)?;let ctx=context(Some(context_json))?.ok_or("Matching context required.")?;
 let measured=rplan_audit::measurements::measure_plan(&plan,Some(&ctx))?;
 let document=rplan_io::read_rplan_str(document_json).map_err(|e|e.to_string())?;
 let metadata=&document.metadata;
 for field in [&metadata.label,&metadata.chamber,&metadata.created_at] {if field.trim().is_empty()||field.len()>200{return Err("Manifest metadata must be recorded and at most 200 bytes.".into());}}
 let state=plan.units.state.clone().ok_or("Plan state is required.")?;let year=plan.units.year.ok_or("Plan year is required.")?;
 let options=document.provenance.producer.get("engine_options").or_else(||document.provenance.producer.get("multiscale_input").and_then(|input|input.get("request")).and_then(|request|request.get("options")));
 let seed=options.and_then(|o|o.get("seed")).filter(|s|!s.is_null()).map(|s|{
  if let Some(n)=s.as_u64(){Ok(n)}else{s.as_str().ok_or("Invalid recorded engine seed.")?.parse::<u64>().map_err(|_|"Invalid recorded engine seed.")}
 }).transpose()?;
 let tolerance=options.and_then(|o|o.get("balance_tolerance")).and_then(Value::as_f64);
 if tolerance.is_some_and(|t|!t.is_finite()||!(0.0..=100.0).contains(&t)){return Err("Invalid recorded population tolerance.".into());}
 let population_balance_evaluated=measured.total_population.is_some()&&tolerance.is_some();
 let balanced=population_balance_evaluated&&measured.districts.iter().all(|d|d.population.as_ref().is_some_and(|p|p.percent_deviation<=tolerance.unwrap()));
 let settings=document.provenance.producer.get("electoral_settings").or_else(||document.provenance.producer.get("multiscale_input").and_then(|i|i.get("request")).and_then(|r|r.get("electoral")));
 let electoral:Option<crate::electoral_input::Input>=settings.filter(|s|!s.is_null()).map(|s|serde_json::from_value(s.clone()).map_err(|e|e.to_string())).transpose()?;
 if let Some(hash)=ctx.source_hashes.entries.get("bisect.electoral-settings") {
  let settings=electoral.as_ref().ok_or("Recorded electoral source hash requires retained settings provenance.")?;
  if settings.hash()?!=*hash {return Err("Recorded electoral settings differ from the RCTX source hash.".into());}
 }
 let (seats,total,system)=electoral.as_ref().map(|s|s.resolve(plan.k as usize)).transpose()?.unwrap_or((1,plan.k as usize,"single_member"));
 let manifest=bisect_report::PlanManifest{
 label:metadata.label.clone(),state_code:state,year:year.to_string(),chamber:metadata.chamber.clone(),num_districts:plan.k,
 seed:seed.and_then(|s|i64::try_from(s).ok()),engine_seed:seed.filter(|s|*s>i64::MAX as u64),
 population_source:crate::state_report_binding::population_source(&document,&ctx)?.unwrap_or_else(||if ctx.populations.is_some(){"supplied-rctx-populations".into()}else{String::new()}),
 partition_mode:options.and_then(|o|o.get("structure")).and_then(Value::as_str).unwrap_or("unrecorded").into(),
 created_at:metadata.created_at.clone(),balance_tolerance_pct:tolerance.unwrap_or(0.0),population_balance_valid:balanced,
 seats_per_district:seats,total_seats:total,electoral_system:system.into(),submission_type:"research".into(),
 source_tool:Some("bisect-wasm".into()),source_tool_version:Some(env!("CARGO_PKG_VERSION").into()),
 rplan_path:Some("plan.rplan".into()),rctx_path:Some("context.rctx".into()),context_hash:Some(ctx.context_hash.clone()),
 n_units:plan.assignment.len(),..Default::default()};
 let manifest_json=serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?;
 Ok(json!({"schema_version":"bisect-plan-report-manifest-v1","manifest_json":manifest_json,
 "manifest_sha256":format!("{:x}",Sha256::digest(manifest_json.as_bytes())),
 "plan_source_sha256":format!("{:x}",Sha256::digest(document_json.as_bytes())),"context_source_sha256":format!("{:x}",Sha256::digest(context_json.as_bytes())),
 "population_balance_evaluated":population_balance_evaluated,
 "scope":"Native manifest schema from supplied validated matching RPLAN/RCTX, recorded metadata and engine options. Recorded seed remains exact, including unsigned seeds beyond i64. Population balance is recomputed only when context populations and recorded tolerance exist. No Census provenance, executable binary hash, solver certificate or native replay command is fabricated. Source metadata is unauthenticated; classification is research. Retain exact browser project and configuration for replay."}))
}
fn bind_named_manifest(mut manifest:bisect_report::PlanManifest,identity:&Value,source_name:&str,source_hash:&str)->Result<bisect_report::PlanManifest,String>{
 if source_name.is_empty()||source_name.len()>200||source_name.chars().any(char::is_control)||source_hash.len()!=64||!source_hash.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err("Record the selected adjacency name and exact SHA-256, without opening a path.".into());}
 let name=identity["effective_state_name"].as_str().ok_or("Native State name required")?;
 if !crate::run_target::portable_state_component(name)||identity["native_year_valid"]!=true||identity["state_code"]!=manifest.state_code||identity["effective_year"]!=manifest.year||identity["chamber"]!=manifest.chamber||identity["effective_label"]!=manifest.label{return Err("Native name effects differ from the accepted plan label/state/year/chamber or use an unsafe source component.".into());}
 manifest.adjacency_file=identity["native_source_name_intents"]["adjacency_manifest"].as_str().ok_or("Native adjacency basename required")?.to_owned();
 manifest.adjacency_sha256=source_hash.to_owned();Ok(manifest)
}

pub(crate) fn named_state_manifest(document_json:&str,context_json:&str,arguments:Vec<String>,source_name:&str,source_hash:&str)->Result<Value,String>{
 use sha2::{Digest,Sha256};
 let identity=crate::state_identity::inspect(arguments)?;
 let mut output=report_manifest(document_json,context_json)?;
 let manifest:bisect_report::PlanManifest=serde_json::from_str(output["manifest_json"].as_str().ok_or("Native manifest text missing")?).map_err(|e|e.to_string())?;
 let manifest=bind_named_manifest(manifest,&identity,source_name,source_hash)?;
 let text=serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?;
 output["manifest_sha256"]=json!(format!("{:x}",Sha256::digest(text.as_bytes())));output["manifest_json"]=json!(text);
 output["native_name_identity"]=identity;output["original_adjacency_source_name"]=json!(source_name);
 output["source_metadata_authenticated"]=json!(false);
 output["scope"]=json!("Native report manifest with matching original State name/year/chamber/default-label effects and supplied selected adjacency basename/hash. Native integer serialization preserves all u64 fields. The caller records the supplied source hash; this does not independently authenticate its relation to the plan, Census origins, a native executable or legal validity. No path is opened and no file authority is granted.");Ok(output)
}

#[cfg(test)]
mod named_manifest_tests{
 use super::*;
 #[test]
 fn exact_u64_seed_and_raw_source_basename(){
  let identity=crate::state_identity::inspect(["--state","RI","--year","2020","--adjacency","fixture.adj.bin","--state-name","Research Area"].iter().map(|s|s.to_string()).collect()).unwrap();
  let manifest=bisect_report::PlanManifest{label:"research_area_congressional_2020".into(),state_code:"RI".into(),year:"2020".into(),chamber:"congressional".into(),engine_seed:Some(u64::MAX),..Default::default()};
  let bound=bind_named_manifest(manifest.clone(),&identity,"fixture.adj.bin",&"ab".repeat(32)).unwrap();
  assert_eq!(bound.engine_seed,Some(u64::MAX));assert_eq!(bound.adjacency_file,"Research Area_adjacency_2020.adj.bin");
  assert!(serde_json::to_string(&bound).unwrap().contains("\"engine_seed\":18446744073709551615"));
  let mut wrong=manifest;wrong.label="different".into();assert!(bind_named_manifest(wrong,&identity,"fixture.adj.bin",&"ab".repeat(32)).is_err());
 }
}
