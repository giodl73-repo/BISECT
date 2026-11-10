//! Bind native report counts to supplied, exactly matched plan/context artifacts.
use crate::state_report_sources::Observation;
use rplan_core::RplanContext;
use rplan_io::RplanDocument;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const SOURCE_KEY: &str = "bisect.native-report-source-snapshot";
const PRODUCER_KEY: &str = "native_report_source_binding";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub arguments: Vec<String>,
    pub observations: Vec<Observation>,
    pub observation_origin: String,
    pub document_json: String,
    pub context_json: String,
}

fn hash(text: &[u8]) -> String { format!("{:x}", Sha256::digest(text)) }

pub(crate) fn bind(input: Input) -> Result<Value, String> {
    let plan = crate::plan_audit_text::plan(&input.document_json)?;
    let mut document = rplan_io::read_rplan_str(&input.document_json).map_err(|e| e.to_string())?;
    let mut context = crate::plan_audit_text::context(Some(&input.context_json))?.ok_or("Report binding needs a matching context.")?;
    if plan.units != context.units || context.graph.is_none() || context.populations.is_none()
        || plan.units.unit_universe_hash != plan.units.compute_unit_universe_hash().map_err(|e| e.to_string())? {
        return Err("Report-source binding requires exactly matched ordered plan/context units, graph and populations.".into());
    }
    let arguments_sha256 = hash(&serde_json::to_vec(&input.arguments).map_err(|e| e.to_string())?);
    let args = bisect_runner::state_command::parse(input.arguments.clone())?;
    let mut preparation = crate::state_report_sources::inspect(input.arguments, Some(input.observations),
        Some(plan.units.unit_ids.clone()), Some(input.observation_origin))?;
    let identity = &preparation["identity"];
    if identity["state_code"].as_str() != plan.units.state.as_deref()
        || identity["effective_year"].as_str() != plan.units.year.map(|y| y.to_string()).as_deref()
        || document.metadata.jurisdiction != plan.units.state.as_deref().unwrap_or("")
        || identity["chamber"].as_str() != Some(document.metadata.chamber.as_str())
        || identity["effective_label"].as_str() != Some(document.metadata.label.as_str())
        || args.districts.is_some_and(|k| k != plan.k) {
        return Err("Original native State name/label/state/year/chamber/district scope differs from the supplied accepted export.".into());
    }
    // Do not echo source bytes or duplicate unit/count arrays in the binding receipt.
    preparation.as_object_mut().ok_or("Invalid source preparation")?.remove("source_bundle");
    preparation.as_object_mut().unwrap().remove("unit_ids");
    if preparation["status"] == "native-load-failure" {
        return Ok(json!({"schema_version":"bisect-native-state-report-binding-v1", "status":"native-load-failure",
            "preparation":preparation, "artifact_units_bound":true,
            "partition_execution_performed":false, "historical_execution_authenticated":false,
            "original_plan_source_sha256":hash(input.document_json.as_bytes()),
            "original_context_source_sha256":hash(input.context_json.as_bytes()),
            "scope":"Native report reader failed on the first existing supplied candidate. No bound export was produced; original artifacts and accepted engine assignments are unchanged."}));
    }
    if preparation["status"] != "prepared-report-counts" { return Err("Native report count preparation did not complete.".into()); }
    preparation["graph_binding_verified"] = json!(true);
    let demographic_text = preparation.as_object_mut().unwrap().remove("demographic_context_json").ok_or("Missing native report counts")?;
    let demographics = match demographic_text {
        Value::Null => None,
        Value::String(text) => Some(serde_json::from_str::<rplan_core::DemographicContext>(&text).map_err(|e| e.to_string())?),
        _ => return Err("Invalid native report count text.".into()),
    };
    let previous_context_hash = context.context_hash.clone();
    let previous_report_hash = context.source_hashes.entries.get(SOURCE_KEY).cloned();
    let previous_demographics_present = context.demographics.is_some();
    let bundle_hash = preparation["source_bundle_sha256"].as_str().ok_or("Missing report-source identity")?.to_owned();
    let source_hash = format!("sha256:{bundle_hash}");
    context.demographics = demographics;
    context.source_hashes.entries.insert(SOURCE_KEY.into(), source_hash.clone());
    context.validate().map_err(|e| e.to_string())?;
    context.context_hash = context.compute_context_hash().map_err(|e| e.to_string())?;
    let marker = json!({"schema_version":"bisect-native-report-provenance-v1", "state":plan.units.state,
        "year":plan.units.year, "chamber":document.metadata.chamber,
        "effective_state_name":preparation["identity"]["effective_state_name"],
        "requested_population_source":preparation["requested_population_source"],
        "source_bundle_sha256":bundle_hash, "context_hash":context.context_hash,
        "source_command_sha256":arguments_sha256, "observation_origin":preparation["observation_origin"],
        "selected_report_path":preparation["selected_report_path"], "source_metadata":preparation["source_metadata"],
        "balance_population_source":"unchanged-supplied-context-populations",
        "source_authentication":false, "partition_execution_performed":false});
    document.provenance.producer.insert(PRODUCER_KEY.into(), marker);
    document.provenance.source_hashes.insert(SOURCE_KEY.into(), source_hash);
    if document.provenance.conversion_lineage.len() >= 1024 { return Err("Report-binding lineage exceeds the portable 1024-step limit.".into()); }
    document.provenance.conversion_lineage.push(json!({"operation":"bind-native-state-report-sources",
        "source_bundle_sha256":bundle_hash, "previous_report_source_hash":previous_report_hash,
        "previous_context_hash":previous_context_hash, "context_hash":context.context_hash,
        "plan_assignment_unchanged":true, "graph_and_population_unchanged":true}));
    let document_json = rplan_io::write_rplan_string(&document).map_err(|e| e.to_string())?;
    if document_json.len() > 3 * 1024 * 1024 { return Err("Bound report plan exceeds 3 MiB.".into()); }
    let context_json = crate::plan_audit_text::portable_context_json(&context)?;
    rplan_io::read_rplan_str(&document_json).map_err(|e| e.to_string())?;
    rplan_io::read_rctx_str(&context_json).map_err(|e| e.to_string())?;
    Ok(json!({"schema_version":"bisect-native-state-report-binding-v1", "status":"bound",
        "document_json":document_json, "context_json":context_json, "document":document, "context":context,
        "plan_hash":plan.plan_hash().map_err(|e| e.to_string())?, "context_hash":context.context_hash,
        "previous_context_hash":previous_context_hash, "previous_demographics_present":previous_demographics_present,
        "original_plan_source_sha256":hash(input.document_json.as_bytes()),
        "original_context_source_sha256":hash(input.context_json.as_bytes()),
        "document_source_sha256":hash(document_json.as_bytes()), "context_source_sha256":hash(context_json.as_bytes()),
        "source_bundle_sha256":bundle_hash, "preparation":preparation,
        "artifact_units_bound":true, "graph_binding_verified":true, "graph_and_population_unchanged":true, "partition_execution_performed":false,
        "historical_execution_authenticated":false,
        "scope":"Native report counts replace only supplied matching RCTX demographics. Ordered units, graph, populations and plan assignments are preserved; native context hashes and explicit source/provenance associations are regenerated. Requested population-source metadata does not select balance weights. Complete source snapshots remain caller-retained data, not filesystem grants, source authentication or proof of historical execution."}))
}

/// Only this versioned, matching association changes generic manifest metadata.
pub(crate) fn population_source(document: &RplanDocument, context: &RplanContext) -> Result<Option<String>, String> {
    let Some(marker) = document.provenance.producer.get(PRODUCER_KEY) else { return Ok(None); };
    let hash = marker["source_bundle_sha256"].as_str().ok_or("Missing report-source provenance hash")?;
    let source_hash = format!("sha256:{hash}");
    if marker["schema_version"] != "bisect-native-report-provenance-v1" || marker["source_authentication"] != false
        || marker["partition_execution_performed"] != false || hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || marker["context_hash"].as_str() != Some(context.context_hash.as_str())
        || marker["state"].as_str() != document.plan.units.state.as_deref()
        || marker["year"].as_u64() != document.plan.units.year.map(u64::from)
        || marker["chamber"].as_str() != Some(document.metadata.chamber.as_str())
        || context.source_hashes.entries.get(SOURCE_KEY) != Some(&source_hash)
        || document.provenance.source_hashes.get(SOURCE_KEY) != Some(&source_hash) {
        return Err("Report-source provenance differs from the supplied plan/context binding.".into());
    }
    let requested = marker["requested_population_source"].as_str().ok_or("Missing requested population source")?;
    if requested.len() > 4096 { return Err("Recorded population-source request exceeds 4096 bytes.".into()); }
    Ok(Some(requested.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rplan_core::*;
    use std::collections::BTreeMap;

    fn fixture() -> Input {
        let arguments = ["--state","RI","--districts","1","--population-source","cvap","--seed","18446744073709551615"].map(str::to_owned).to_vec();
        let paths = crate::state_report_sources::inspect(arguments.clone(), None, None, None).unwrap();
        let observations = paths["required_observation_paths"].as_array().unwrap().iter().map(|p| {
            let name = p.as_str().unwrap();
            let file = name == paths["availability_path"].as_str().unwrap();
            Observation {relative_path:name.into(),kind:if file {crate::state_report_sources::Kind::File} else {crate::state_report_sources::Kind::Missing},
                source_base64:if file {use base64::Engine;Some(base64::engine::general_purpose::STANDARD.encode("GEOID,total_vap,minority_vap\n44001000100,4,2\n44001000200,2,1\n"))}else{None}}
        }).collect();
        let mut units = PlanUnitIndex {unit_kind:UnitKind::Tract,state:Some("RI".into()),year:Some(2020),canonical_order:CanonicalOrder::ExplicitUnitIds,
            unit_ids:vec!["44001000100".into(),"44001000200".into()],unit_universe_hash:String::new(),source_id:Some("report-binding-test".into())};
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        let mut context = RplanContext {rctx_version:RCTX_VERSION.into(),context_hash:String::new(),units:units.clone(),
            graph:Some(UnitGraph {edge_semantics:EdgeSemantics::Undirected,adjacency:vec![vec![UnitEdge {to:1,kind:EdgeKind::Custom,weight:Some(1.0)}],vec![UnitEdge {to:0,kind:EdgeKind::Custom,weight:Some(1.0)}]]}),
            populations:Some(vec![9_007_199_254_740_999,1]),subdivisions:None,demographics:None,geometry:None,
            source_hashes:SourceHashes {entries:BTreeMap::from([("original.graph".into(),format!("sha256:{}","ab".repeat(32)))])}};
        context.context_hash = context.compute_context_hash().unwrap();
        let mut provenance = rplan_io::RplanProvenance {source_hashes:context.source_hashes.entries.clone(),..Default::default()};
        provenance.producer.insert("engine_options".into(),json!({"seed":u64::MAX,"structure":"nway","balance_tolerance":0.5}));
        let document = RplanDocument {rplan_version:rplan_io::RPLAN_V02.into(),plan:DistrictPlan {schema_version:DISTRICT_PLAN_SCHEMA_VERSION.into(),units,assignment:vec![0,0],k:1,display_labels:vec!["1".into()],allow_empty_districts:false},
            metadata:rplan_io::RplanMetadataV02 {label:"rhode_island_congressional_2020".into(),jurisdiction:"RI".into(),chamber:"congressional".into(),created_at:"2026-10-27T00:00:00Z".into(),description:None},provenance,geometry:None,extensions:BTreeMap::new()};
        Input {arguments,observations,observation_origin:"explicit-file-bundle".into(),document_json:rplan_io::write_rplan_string(&document).unwrap(),context_json:rplan_io::write_rctx_string(&context).unwrap()}
    }

    #[test]
    fn typed_binding_preserves_large_integers_graph_and_assignments() {
        let input = fixture();
        let original = rplan_io::read_rctx_str(&input.context_json).unwrap();
        let original_plan = rplan_io::read_rplan_str(&input.document_json).unwrap();
        let result = bind(input).unwrap();
        let context_text = result["context_json"].as_str().unwrap();
        let document_text = result["document_json"].as_str().unwrap();
        let context = rplan_io::read_rctx_str(context_text).unwrap();
        assert_eq!(context.graph,original.graph);assert_eq!(context.populations,original.populations);assert_eq!(context.units,original.units);
        assert_eq!(rplan_io::read_rplan_str(document_text).unwrap().plan,original_plan.plan);
        assert_ne!(context.context_hash,original.context_hash);
        assert!(context_text.contains("9007199254740999"));
        let report = crate::plan_audit_text::report_manifest(document_text,context_text).unwrap();
        let text = report["manifest_json"].as_str().unwrap();
        let manifest:bisect_report::PlanManifest = serde_json::from_str(text).unwrap();
        assert_eq!(manifest.population_source,"cvap");assert_eq!(manifest.engine_seed,Some(u64::MAX));
        assert!(text.contains("\"engine_seed\": 18446744073709551615"));
        assert_eq!(manifest.context_hash,Some(context.context_hash));
    }

    #[test]
    fn native_source_failure_produces_no_completed_export() {
        let mut input = fixture();
        for observation in &mut input.observations {if observation.source_base64.is_some(){observation.source_base64=Some(String::new());}}
        let result = bind(input).unwrap();
        assert_eq!(result["status"],"native-load-failure");assert!(result.get("document_json").is_none());
        assert_eq!(result["partition_execution_performed"],false);
    }

    #[test]
    fn scope_and_provenance_mismatches_refuse() {
        let mut wrong = fixture();wrong.arguments.extend(["--label".into(),"different".into()]);
        assert!(bind(wrong).is_err());
        let result = bind(fixture()).unwrap();
        let mut document = rplan_io::read_rplan_str(result["document_json"].as_str().unwrap()).unwrap();
        document.provenance.producer.get_mut(PRODUCER_KEY).unwrap()["context_hash"] = json!("stale");
        assert!(crate::plan_audit_text::report_manifest(&rplan_io::write_rplan_string(&document).unwrap(),result["context_json"].as_str().unwrap()).is_err());
    }
}
