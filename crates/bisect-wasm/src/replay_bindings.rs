//! Selected manifest/run/original correspondence, without engine execution.
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn normalized<T: serde::de::DeserializeOwned + serde::Serialize>(
    value: &Value,
) -> Result<Value, String> {
    let typed: T = serde_json::from_value(value.clone())
        .map_err(|e| format!("Invalid recorded replay provenance: {e}"))?;
    serde_json::to_value(typed).map_err(|e| e.to_string())
}

pub(crate) fn check(
    manifest_json: &str,
    manifest_path: &str,
    files: &BTreeMap<String, Vec<u8>>,
    run_json: &str,
    original_json: &str,
) -> Result<Value, String> {
    let manifest: bisect_report::PlanManifest =
        serde_json::from_str(manifest_json).map_err(|e| e.to_string())?;
    let inputs = crate::engine_replay::binding_inputs(run_json)?;
    let request = &inputs["request"];
    let graph = &request["graph"];
    let options = &request["options"];
    let seed = options["seed"]
        .as_u64()
        .or_else(|| options["seed"].as_str().and_then(|v| v.parse::<u64>().ok()));
    let recorded_seed = match manifest.engine_seed {
        Some(v) => Some(v),
        None => manifest
            .seed
            .map(|v| {
                u64::try_from(v).map_err(|_| {
                    "Manifest has a negative seed unsupported by the unsigned portable engine."
                        .to_string()
                })
            })
            .transpose()?,
    };
    if graph["state"] != manifest.state_code
        || graph["year"] != manifest.year
        || options["districts"].as_u64() != Some(manifest.num_districts as u64)
    {
        return Err(
            "Manifest state, year or district count differs from selected replay inputs.".into(),
        );
    }
    if seed != recorded_seed {
        return Err("Manifest seed differs from selected replay inputs. Select the recorded accepted attempt when a build changed its seed.".into());
    }
    if options["balance_tolerance"].as_f64() != Some(manifest.balance_tolerance_pct) {
        return Err("Manifest population tolerance differs from selected replay inputs.".into());
    }
    if let Some(chamber) = inputs["chamber"].as_str() {
        if chamber != manifest.chamber {
            return Err("Manifest chamber differs from selected configured replay input.".into());
        }
    }
    if !request["electoral"].is_null() {
        let settings:crate::electoral_input::Input=serde_json::from_value(request["electoral"].clone()).map_err(|e|e.to_string())?;
        let (seats,total,system)=settings.resolve(manifest.num_districts as usize)?;
        if manifest.seats_per_district!=seats||manifest.total_seats!=total||manifest.electoral_system!=system {return Err("Manifest electoral settings differ from the selected replay inputs.".into());}
    } else if manifest.seats_per_district!=1||![0,manifest.num_districts as usize].contains(&manifest.total_seats)||manifest.electoral_system!="single_member" {
        return Err("Select the recorded electoral settings for this manifest replay.".into());
    }
    let original = crate::assignment_verification::assignments(original_json)?;
    if original.is_empty() {
        return Ok(
            json!({"status":"unavailable","manifest_identity":"verified","original_unit_universe":"unavailable","original_plan":"not-run","engine_options":"not-run","prepared_graph":"not-run","scope":"No original assignments are available; engine replay aborts before execution."}),
        );
    }
    let ids = inputs["unit_ids"]
        .as_array()
        .ok_or("Replay unit IDs are unavailable.")?;
    if original.len() != ids.len()
        || ids
            .iter()
            .any(|id| id.as_str().is_none_or(|id| !original.contains_key(id)))
    {
        return Err(
            "Original assignments do not cover exactly the selected replay unit universe.".into(),
        );
    }
    if original
        .values()
        .any(|&d| d < 1 || d > manifest.num_districts)
    {
        return Err("Original assignment district is outside 1..=manifest district count.".into());
    }
    let read = |relative: &str| -> Result<&Vec<u8>, String> {
        if !crate::package_verification::safe(relative) {
            return Err("Unsafe manifest replay binding path.".into());
        }
        let path = manifest_path.rsplit_once('/').map_or_else(
            || relative.to_string(),
            |(root, _)| format!("{root}/{relative}"),
        );
        files
            .get(&path)
            .ok_or_else(|| format!("Manifest binding file not selected: {path}"))
    };
    let mut plan_status = "not-recorded";
    let mut options_status = "not-recorded";
    let mut graph_status = "not-recorded";
    let mut auxiliary_records = BTreeMap::from([
        ("demographics", "not-recorded"),
        ("character", "not-recorded"),
        ("elections", "not-recorded"),
        ("partisan", "not-recorded"),
    ]);
    // Preserve legacy receipts when COI is neither selected nor recorded.
    if !request["coi"].is_null() { auxiliary_records.insert("coi", "not-recorded"); }
    if !request["electoral"].is_null() {auxiliary_records.insert("electoral", "not-recorded");}
    if !inputs["configured_input"].is_null() {auxiliary_records.insert("configuration", "not-recorded");}
    let mut multiscale_status = if inputs["multiscale_input"].is_null() {
        "not-applicable"
    } else {
        "not-recorded"
    };
    let mut fine_boundaries_status = if inputs["multiscale_input"].is_null() {
        "not-applicable"
    } else {
        "not-recorded"
    };
    let mut plan_hash = None;
    let mut context_hash = None;
    if let Some(path) = manifest.rplan_path.as_deref() {
        let text = std::str::from_utf8(read(path)?).map_err(|e| e.to_string())?;
        let document =
            rplan_io::read_rplan_str(text).map_err(|e| format!("Read bound replay plan: {e}"))?;
        let plan = &document.plan;
        if plan.k != manifest.num_districts
            || plan.units.state.as_deref() != Some(manifest.state_code.as_str())
            || plan.units.year.map(|v| v.to_string()).as_deref() != Some(manifest.year.as_str())
            || document.metadata.chamber != manifest.chamber
        {
            return Err(
                "Recorded RPLAN metadata differs from the manifest replay identity.".into(),
            );
        }
        if plan.units.unit_ids.len() != original.len()
            || plan
                .units
                .unit_ids
                .iter()
                .zip(&plan.assignment)
                .any(|(id, &d)| original.get(id) != Some(&(d as usize + 1)))
        {
            return Err("Original assignments differ from the manifest-recorded RPLAN.".into());
        }
        plan_hash = Some(plan.plan_hash().map_err(|e| e.to_string())?);
        plan_status = "verified";
        if let Some(recorded) = document.provenance.producer.get("engine_options") {
            if &normalized::<crate::engine::Options>(recorded)? != options {
                return Err(
                    "Selected engine options differ from recorded RPLAN engine options.".into(),
                );
            }
            options_status = "verified";
        }
        if let Some(recorded)=document.provenance.producer.get("algorithm_configuration") {
            auxiliary_records.insert("configuration", "not-recorded");
            if !inputs["configured_input"].is_null() {
                let input:crate::config::ConfiguredRun=serde_json::from_value(inputs["configured_input"].clone()).map_err(|e|e.to_string())?;
                let (selected,_) = crate::config::prepare_at(input,inputs["configured_attempt"].as_u64().map(|n|n as u32))?;
                for key in ["yaml_text","source_name","source_sha256","profile","execution_chamber","execution_tolerance_percent","embedded_policy_sha256","electoral_settings","coi_weights_source_sha256"] {
                    let equal=if key=="execution_tolerance_percent" {selected[key].as_f64()==recorded[key].as_f64()} else {selected[key]==recorded[key]};
                    if !equal {return Err(format!("Selected configuration {key} differs from recorded RPLAN provenance."));}
                }
                auxiliary_records.insert("configuration", "recorded-input-only");
                if !inputs["configured_attempt"].is_null()&&["execution_attempt","execution_seed"].iter().any(|key|selected[*key]!=recorded[*key]) {return Err("Selected configured replay attempt differs from recorded execution provenance.".into());}
            }
        }
        for (key, producer_key) in [
            ("character", "character_input"),
            ("elections", "election_input"),
            ("partisan", "partisan_input"),
            ("coi", "coi_input"),
            ("electoral", "electoral_settings"),
        ] {
            if let Some(recorded) = document.provenance.producer.get(producer_key) {
                let value = match key {
                    "character" => normalized::<crate::character_input::CharacterInput>(recorded)?,
                    "electoral" => normalized::<crate::electoral_input::Input>(recorded)?,
                    "coi" => normalized::<crate::coi_input::Input>(recorded)?,
                    "elections" => normalized::<crate::election_input::ElectionInput>(recorded)?,
                    _ => normalized::<crate::engine::PartisanInput>(recorded)?,
                };
                if value != request[key] {
                    return Err(format!(
                        "Selected {key} input differs from recorded RPLAN auxiliary provenance."
                    ));
                }
                auxiliary_records.insert(key, "recorded-input-only");
            }
        }
        if let Some(recorded) = document.provenance.producer.get("multiscale_input") {
            if normalized::<crate::multiscale_input::MultiscaleRequest>(recorded)?
                != inputs["multiscale_input"]
            {
                return Err(
                    "Selected multiscale inputs differ from recorded RPLAN multiscale inputs."
                        .into(),
                );
            }
            options_status = "verified";
            multiscale_status = "verified";
            if !request["coi"].is_null() { auxiliary_records.insert("coi", "recorded-input-only"); }
            if !request["electoral"].is_null() {auxiliary_records.insert("electoral", "recorded-input-only");}
        }
    }
    if let Some(path) = manifest.rctx_path.as_deref() {
        let text = std::str::from_utf8(read(path)?).map_err(|e| e.to_string())?;
        let context =
            rplan_io::read_rctx_str(text).map_err(|e| format!("Read bound replay context: {e}"))?;
        if manifest
            .context_hash
            .as_ref()
            .is_some_and(|hash| hash != &context.context_hash)
        {
            return Err("Recorded RCTX hash differs from manifest context hash.".into());
        }
        if let Some(hash) = context.source_hashes.entries.get("bisect.prepared-graph") {
            let actual = rplan_core::canonical_sha256(graph).map_err(|e| e.to_string())?;
            if hash != &actual {
                return Err(
                    "Selected prepared graph differs from recorded RCTX prepared graph hash."
                        .into(),
                );
            }
            let request: crate::engine::Request =
                serde_json::from_value(request.clone()).map_err(|e| e.to_string())?;
            let assignments = original
                .iter()
                .map(|(id, &d)| (id.clone(), d as u32))
                .collect();
            let mut expected = if inputs["configured_input"].is_null() {
                crate::plan_export::export(request,assignments,manifest.label.clone(),manifest.chamber.clone(),manifest.created_at.clone())?
            } else {
                let input:crate::config::ConfiguredRun=serde_json::from_value(inputs["configured_input"].clone()).map_err(|e|e.to_string())?;
                crate::config::export(input,assignments,manifest.label.clone(),manifest.created_at.clone(),inputs["configured_attempt"].as_u64().map(|n|n as u32))?
            };
            // Bare accepted engine artifacts cannot establish selected YAML identity.
            // Keep those known source claims unverified, while reconstructing all
            // physical and supplied engine/auxiliary context contents exactly.
            if inputs["configured_input"].is_null() {
                let mut reconstructed:rplan_core::RplanContext=serde_json::from_value(expected["context"].clone()).map_err(|e|e.to_string())?;
                for key in ["bisect.algorithm-yaml","bisect.embedded-location-policy"] {
                    if let Some(value)=context.source_hashes.entries.get(key) {
                        reconstructed.source_hashes.entries.insert(key.into(),value.clone());
                        auxiliary_records.insert("configuration", "not-recorded");
                    }
                }
                expected["context_hash"]=json!(reconstructed.compute_context_hash().map_err(|e|e.to_string())?);
            }
            if expected["context_hash"] != context.context_hash {
                return Err(
                    "Recorded RCTX contents differ from the selected prepared engine inputs."
                        .into(),
                );
            }
            graph_status = "verified";
        }
        if let Some(hash) = context.source_hashes.entries.get("bisect.multiscale-input") {
            if inputs["multiscale_input"].is_null() {
                return Err(
                    "Recorded multiscale context requires selected multiscale inputs.".into(),
                );
            }
            let actual = rplan_core::canonical_sha256(&inputs["multiscale_input"])
                .map_err(|e| e.to_string())?;
            if hash != &actual {
                return Err(
                    "Selected multiscale inputs differ from recorded RCTX input hash.".into(),
                );
            }
            multiscale_status = "verified";
            let input: crate::multiscale_input::MultiscaleRequest =
                serde_json::from_value(inputs["multiscale_input"].clone())
                    .map_err(|e| e.to_string())?;
            if input.block_groups.is_some() && inputs["fine_edges"].is_null() {
                fine_boundaries_status = "not-selected";
                graph_status = "input-hash-only";
            } else {
                let fine_edges: Option<Vec<(usize, usize, f64)>> =
                    serde_json::from_value(inputs["fine_edges"].clone())
                        .map_err(|e| e.to_string())?;
                let assignments = original
                    .iter()
                    .map(|(id, &d)| (id.clone(), d as u32))
                    .collect();
                let expected = crate::plan_export::export_multiscale(
                    input,
                    fine_edges,
                    assignments,
                    manifest.label.clone(),
                    manifest.chamber.clone(),
                    manifest.created_at.clone(),
                )?;
                if expected["context_hash"] != context.context_hash {
                    return Err("Recorded fine RCTX contents or boundary evidence differ from selected multiscale inputs.".into());
                }
                fine_boundaries_status = "verified";
                graph_status = "verified";
            }
        }
        context_hash = Some(context.context_hash);
    }
    let mut auxiliary_keys = vec!["demographics", "character", "elections", "partisan"];
    if auxiliary_records.contains_key("coi") { auxiliary_keys.push("coi"); }
    if auxiliary_records.contains_key("electoral") {auxiliary_keys.push("electoral");}
    for key in auxiliary_keys {
        if request[key].is_null() {
            auxiliary_records.insert(key, "not-applicable");
        } else if graph_status == "verified"
            && (key == "demographics" || auxiliary_records[key] == "recorded-input-only")
        {
            auxiliary_records.insert(key, "verified");
        }
    }
    if !inputs["configured_input"].is_null()&&graph_status=="verified"&&auxiliary_records.get("configuration")==Some(&"recorded-input-only") {
        auxiliary_records.insert("configuration", "verified");
    }
    let auxiliary = if auxiliary_records.values().all(|&s| s == "not-applicable") {
        "not-applicable"
    } else if auxiliary_records
        .values()
        .all(|&s| ["verified", "not-applicable"].contains(&s))
    {
        "verified"
    } else {
        "partial"
    };
    Ok(
        json!({"multiscale_inputs":multiscale_status,"fine_boundaries":fine_boundaries_status,"auxiliary_inputs":auxiliary,"auxiliary_bindings":auxiliary_records,"status":if ["verified","not-applicable"].contains(&auxiliary)&&plan_status=="verified"&&options_status=="verified"&&graph_status=="verified"{"verified"}else{"partial"},"manifest_identity":"verified","original_unit_universe":"verified","original_plan":plan_status,"engine_options":options_status,"prepared_graph":graph_status,"recorded_plan_hash":plan_hash,"recorded_context_hash":context_hash,"seed":seed.map(|v|v.to_string()),"scope":"Checks selected state/year/district count, exact optional seed and tolerance against typed manifest; configured chamber when available; original one-based assignment universe/range; recorded RPLAN assignments and available engine/multiscale provenance; recorded prepared-graph hash and exact native-exported RCTX contents when that prepared-graph provenance is available. Multiscale inputs bind through normalized recorded provenance and native RCTX input hash; exact fine contexts and supplied physical fine edges are reconstructed with the native multiscale export. Missing block-group boundary input yields partial correspondence. Missing recorded evidence remains not-recorded and yields partial correspondence. Source and provenance records remain unauthenticated. Supplied demographics are bound through the exact native context export including full input hash and retained counts. Character, partisan and election inputs require both matching recorded RPLAN auxiliary provenance and the exact native context export/source hashes. Missing evidence remains partial. Build artifacts are compared before retry; accepted-attempt replay is required when its recorded seed differs."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rehashed_context_with_copied_graph_claim_is_rejected() {
        let graph = json!({"schema_version":1,"state":"RI","year":"2020","geoids":["44001000001","44001000002","44001000003","44001000004"],"adjacency":[[1,2],[0,3],[0,3],[1,2]],"population":[100,100,100,100],"edges":[[0,1,1.0],[0,2,1.0],[1,3,1.0],[2,3,1.0]],"areas":[1.0,1.0,1.0,1.0],"exterior_perimeters":[1.0,1.0,1.0,1.0],"centroids":[[0.0,0.0],[1.0,0.0],[0.0,1.0],[1.0,1.0]]});
        let options = json!({"structure":"standard-bisect","weights":"geographic","search":"single","districts":2,"seed":42,"seeds":1,"steps":1,"percentile":0.0,"alpha_county":0.0,"balance_tolerance":50.0,"area_swing":1.1,"iterations":10});
        let request: crate::engine::Request =
            serde_json::from_value(json!({"graph":graph,"options":options})).unwrap();
        let original = json!({"44001000001":1,"44001000002":1,"44001000003":2,"44001000004":2});
        let exported = crate::plan_export::export(
            request.clone(),
            serde_json::from_value(original.clone()).unwrap(),
            "binding-test".into(),
            "congressional".into(),
            "2026-10-08T00:00:00Z".into(),
        )
        .unwrap();
        let doc = exported["document_json"].as_str().unwrap();
        let ctx = exported["context_json"].as_str().unwrap();
        let reported = crate::plan_audit_text::report_manifest(doc, ctx).unwrap();
        let manifest_text = reported["manifest_json"].as_str().unwrap();
        let run = serde_json::to_string(&json!({"kind":"engine","request":request})).unwrap();
        let mut files = BTreeMap::from([
            ("plan.rplan".into(), doc.as_bytes().to_vec()),
            ("context.rctx".into(), ctx.as_bytes().to_vec()),
        ]);
        assert_eq!(
            check(
                manifest_text,
                "manifest.json",
                &files,
                &run,
                &original.to_string()
            )
            .unwrap()["status"],
            "verified"
        );
        let mut context = rplan_io::read_rctx_str(ctx).unwrap();
        context.populations.as_mut().unwrap()[0] += 1;
        let text = rplan_io::write_rctx_string(&context).unwrap();
        let context = rplan_io::read_rctx_str(&text).unwrap();
        let mut manifest: bisect_report::PlanManifest =
            serde_json::from_str(manifest_text).unwrap();
        manifest.context_hash = Some(context.context_hash);
        files.insert("context.rctx".into(), text.into_bytes());
        assert!(check(
            &serde_json::to_string(&manifest).unwrap(),
            "manifest.json",
            &files,
            &run,
            &original.to_string()
        )
        .unwrap_err()
        .contains("RCTX contents differ"));
    }
}
