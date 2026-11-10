//! Shared native discovery/refinement adapter with exact incumbent evidence and in-memory packages.
use std::collections::BTreeMap;
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {pub districts:usize,pub tolerance:f64,pub generated_at:String,pub seed:String,pub refinement:bisect_runner::certified_discovery::DiscoveryRefinementArg}
fn digest(text:&str)->String{format!("{:x}",Sha256::digest(text.as_bytes()))}
pub(crate) fn solve(text:&str,settings:Settings)->Result<Value,String>{inner(text,settings).map_err(|e|e.to_string())}
fn inner(text:&str,settings:Settings)->Result<Value,Box<dyn std::error::Error>>{
 if text.len()>8*1024*1024{return Err("Discovery context exceeds 8 MiB".into());}
 if !(2..=100000).contains(&settings.districts)||!settings.tolerance.is_finite()||!(0.0..=100.0).contains(&settings.tolerance)||settings.generated_at.is_empty()||settings.generated_at.len()>200||settings.generated_at.chars().any(char::is_control)||settings.seed.is_empty()||settings.seed.len()>20||!settings.seed.bytes().all(|c|c.is_ascii_digit()){return Err("Invalid discovery settings or exact decimal seed".into());}
 let seed=settings.seed.parse::<u64>()?;
 let context=rplan_io::read_rctx_str(text)?;
 if context.units.unit_ids.len()>100000{return Err("Discovery context exceeds 100000 units".into());}
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
 let adjacency=graph.adjacency.iter().map(|edges|edges.iter().map(|edge|edge.to as usize).collect::<Vec<_>>()).collect::<Vec<_>>();
 let land_adjacency=graph.adjacency.iter().map(|edges|edges.iter().filter(|edge|!matches!(edge.kind,rplan_core::EdgeKind::Bridge)).map(|edge|edge.to as usize).collect::<Vec<_>>()).collect::<Vec<_>>();
 let discovery=bisect_runner::certified_discovery::discover(&root,&adjacency,&land_adjacency,seed,settings.refinement)?;
 let mut files:BTreeMap<String,String>=BTreeMap::from([("certified-split-instance.json".into(),serde_json::to_string_pretty(&root)?),("certified-discovery.json".into(),serde_json::to_string_pretty(&discovery)?)]);
 if settings.districts==2 {
 let generated_at=settings.generated_at.clone();let context=&context;let assignment=&discovery.objective.canonical_assignment;
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "US".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: assignment.iter().map(|&label| u32::from(label)).collect(),
            k: settings.districts,
            display_labels: (1..=settings.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "certified-discovery-incumbent".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some(
                "Connected deterministic discovery incumbent; exact proof status is separate."
                    .to_string(),
            ),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                (
                    "method".to_string(),
                    serde_json::json!("certified-discovery"),
                ),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = exact_legal_profile(
        &jurisdiction,
        &document.metadata.chamber,
        document.plan.units.year.unwrap_or(2020),
        settings.tolerance,
    );
    let certificate = rplan_audit::audit_plan(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect-wasm".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: None,
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "METIS discovery".to_string(),
                version: None,
                mode: Some("unproved-incumbent".to_string()),
                time_limit_secs: None,
                optimality_gap: None,
            }),
        },
        &[rplan_audit::AuditConstraint::Contiguity],
        &generated_at,
    )?;
    if matches!(certificate.result, rplan_audit::AuditResult::Fail) {
        return Err(format!("certified discovery failed contiguity audit: {}",serde_json::to_string(&certificate.checks)?).into());
    }
 files.insert("discovery.rplan".into(),rplan_io::write_rplan_string(&document)?);
 files.insert("discovery.rctx".into(),rplan_io::write_rctx_string(context)?);
 files.insert("audit-certificate.json".into(),serde_json::to_string_pretty(&certificate)?);
 }
 let hashes=files.iter().map(|(name,text)|(name.clone(),digest(text))).collect::<BTreeMap<_,_>>();
 let manifest=json!({"schema_version":"certified-split-discovery-package-v1","method":"certified-discovery","status":"unproved-incumbent","instance_hash":root.hash()?,"discovery_id":discovery.discovery_id,"seed":seed,"files":hashes,"claim_boundary":"Deterministic connected METIS incumbent and exact objective record; not an optimality proof or certified split."});
 files.insert("certified-discovery-manifest.json".into(),serde_json::to_string_pretty(&manifest)?);
 let file_sha256=files.iter().map(|(name,text)|(name.clone(),digest(text))).collect::<BTreeMap<_,_>>();
 Ok(json!({"schema_version":"bisect-certified-discovery-context-v1","status":"unproved-incumbent","districts":settings.districts,"seed":settings.seed,"refinement":settings.refinement,"assignment":discovery.objective.canonical_assignment,"objective":{"max_population_deviation_scaled":discovery.objective.primary.max_population_deviation_scaled.to_string(),"total_population_deviation_scaled":discovery.objective.primary.total_population_deviation_scaled.to_string(),"weighted_boundary_cut":discovery.objective.primary.weighted_boundary_cut.to_string()},"discovery_id":discovery.discovery_id,"document_json":files.get("discovery.rplan"),"context_json":files.get("discovery.rctx"),"audit_certificate_json":files.get("audit-certificate.json"),"files":files,"file_sha256":file_sha256,"context_hash":context.context_hash,"context_source_sha256":digest(text),"total_population_text":populations.iter().map(|p|*p as i128).sum::<i128>().to_string(),"scope":"Shared native seeded discovery and all seven native refinement profiles. The connected incumbent has an exact integer objective record, not an optimality proof. For two districts, exports include the native RPLAN/RCTX and contiguity-only audit; population tolerance is recorded but population compliance is not audited. For larger district counts this is only the root split into two children with floor/ceil seat counts, with no complete district plan. Source truth and legal compliance remain unverified. Runtime records bisect-wasm; geographic geometry is separate."}))
}
fn exact_legal_profile(
    jurisdiction: &str,
    chamber: &str,
    year: u16,
    tolerance_percent: f64,
) -> rplan_audit::LegalProfile {
    let chamber = match chamber {
        "congressional" => rplan_audit::Chamber::Congressional,
        other => rplan_audit::Chamber::Custom(other.to_string()),
    };
    rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "BISECT_BRANCH_PRICE_EXACT_V1".to_string(),
        jurisdiction: jurisdiction.to_string(),
        chamber,
        year,
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: tolerance_percent,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::CountOnly,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    }
}

