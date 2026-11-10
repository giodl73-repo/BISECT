//! Fresh State execution from selected RADJ bytes, never an imported RCTX.
use serde::{Deserialize, Serialize};
use bisect_runner::native_replay_recipe::RecursiveBackend;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub arguments: Vec<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub native_recursive_backend: Option<RecursiveBackend>,
    pub source: crate::native_adjacency::FreshPrepareInput,
    #[serde(default)] pub auxiliary_json: Option<String>,
    #[serde(default)] pub auxiliary_source_name: Option<String>,
    pub label: String,
    pub chamber: String,
    pub created_at: String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub multiscale: Option<Multiscale>,
}

#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Multiscale {
    pub sampler: bisect_runner::native_replay_recipe::NativeMultiscaleSampler,
    #[serde(default)] pub fine_source: Option<crate::native_adjacency::FreshPrepareInput>,
}

struct Compiled {
    prepared: Value,
    source: crate::native_adjacency::FreshGraph,
    resolution: String,
    input_sha256: String,
    fine: Option<crate::native_adjacency::FreshGraph>,
}

fn digest(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

fn compile(input: &Input) -> Result<Compiled, String> {
    let review = bisect_runner::state_command::inspect(input.arguments.clone())?;
    if review["status"] != "parsed" || review["profile_filter"]["allowed"] != true {
        return Err("Review valid native State flags/profile before fresh selected-source preparation.".into());
    }
    crate::native_state_run::validate_orchestration_flags(&review, true)?;
    let args = bisect_runner::state_command::parse(input.arguments.clone())?;
    let path = args.adjacency.as_deref().ok_or("Fresh selected State requires explicit --adjacency; the path is retained metadata only.")?;
    if path.rsplit(['/', '\\']).next() != Some(input.source.adjacency_name()) {
        return Err("Select the exact adjacency basename named in the State flags; no path is opened.".into());
    }
    let (state, year) = input.source.scope();
    if args.state.to_uppercase() != state || args.year != year || args.chamber != input.chamber {
        return Err("Fresh State flags must match selected source state/year and export chamber.".into());
    }
    let resolution = match args.resolution {
        bisect_runner::state_args::Resolution::Tract => "tract",
        bisect_runner::state_args::Resolution::BlockGroup => "block_group",
        bisect_runner::state_args::Resolution::Block => "block",
    }.to_owned();
    let mut source = crate::native_adjacency::prepare_fresh_state(input.source.clone(), &resolution)?;
    let algorithm = bisect_runner::state_config::AlgorithmConfig::from_state_args(&args);
    let recursive_backend=input.native_recursive_backend.unwrap_or(RecursiveBackend::MetisCore);
    let mut auxiliary: serde_json::Map<String, Value> = match &input.auxiliary_json {
        Some(text) => {
            if text.len() > 25 * 1024 * 1024 { return Err("Fresh native auxiliary JSON exceeds25MiB.".into()); }
            serde_json::from_str(text).map_err(|e| format!("Fresh native auxiliary JSON: {e}"))?
        }
        None => serde_json::Map::new(),
    };
    if auxiliary.keys().any(|k| k != "native_auxiliary" && k != "native_weighting") {
        return Err("Fresh native State uses original native_auxiliary source bytes/explicit absence claims. Derived typed observations use the existing prepared-graph workflow.".into());
    }
    let weights = serde_json::to_value(&algorithm.weights).map_err(|e| e.to_string())?;
    if auxiliary.get("native_weighting").is_some_and(|value| *value != weights) {
        return Err("Fresh native WeightSpec must match the resolved State flags.".into());
    }
    auxiliary.insert("native_weighting".into(), weights);
    let mut fine=None;
    let prepared = if let Some(multiscale)=&input.multiscale {
        let fine_needed=matches!(args.multiscale_fine.as_str(),"bg"|"block_group");
        if fine_needed!=multiscale.fine_source.is_some(){return Err("Select a complete raw fine source exactly when native multiscale flags request block groups.".into());}
        if let Some(selected)=&multiscale.fine_source {
            if selected.scope()!=input.source.scope(){return Err("Fine and primary source state/year claims must match.".into());}
            fine=Some(crate::native_adjacency::prepare_fresh_state(selected.clone(),"bg")?);
        }
        let mut request=json!({"graph":source.graph,"options":{"structure":"standard-bisect","search":"single","weights":"geographic","districts":1,"seed":null,"seeds":1,"steps":200,"percentile":0.0,"alpha_county":0.0,"balance_tolerance":0.5,"area_swing":1.10,"iterations":100}});
        for(key,value)in auxiliary{request[key]=value;}
        let block_groups=fine.as_ref().map(|selected|crate::multiscale_input::BlockGroupGraph{schema_version:"bisect-block-group-graph-v1".into(),state:source.graph.state.clone(),year:source.graph.year.clone(),geoids:selected.graph.geoids.clone(),adjacency:selected.graph.adjacency.clone(),population:selected.graph.population.clone()});
        let template=json!({"operation":"run-multiscale-build","input":{"request":request,"fine_level":if block_groups.is_some(){"bg"}else{"tract"},"coarse_level":args.multiscale_coarse,"total_steps":0,"alpha":0.3,"percentile":0.0,"block_groups":block_groups,"adaptive":null,"native_sampler":multiscale.sampler},"fine_edges":fine.as_ref().map(|selected|&selected.graph.edges)});
        crate::native_state_run::prepare_fresh_template(crate::native_state_run::Input{arguments:input.arguments.clone(),prepared:crate::prepared_engine::Input{request_json:serde_json::to_string(&template).map_err(|e|e.to_string())?,inspect_only:true,context_source:None,label:input.label.clone(),chamber:input.chamber.clone(),created_at:input.created_at.clone()}},recursive_backend)?
    } else {crate::native_state_run::prepare_fresh_graph(crate::native_state_run::GraphInput {
        arguments: input.arguments.clone(),
        graph_json: serde_json::to_string(&source.graph).map_err(|e| e.to_string())?,
        graph_source_name: "fresh-selected-native-adjacency.json".into(),
        auxiliary_json: Some(serde_json::to_string(&auxiliary).map_err(|e| e.to_string())?),
        auxiliary_source_name: input.auxiliary_source_name.clone().or(Some("fresh-native-source-claims.json".into())),
        context_source: None,
        label: input.label.clone(), chamber: input.chamber.clone(), created_at: input.created_at.clone(),
    },recursive_backend)?};
    let expected=if input.multiscale.is_some(){"multiscale"}else{"ordinary"};
    if prepared["prepared_review"]["run_kind"] != expected {
        return Err("Fresh selected sources must match the native ordinary or multiscale dispatch.".into());
    }
    if let Some(selected)=&fine{source.sources["fine"]=selected.sources.clone();}
    Ok(Compiled { prepared, source, fine, resolution, input_sha256: digest(&serde_json::to_vec(input).map_err(|e| e.to_string())?) })
}

pub(crate) fn prepare(input: Input) -> Result<Value, String> {
    let compiled = compile(&input)?;
    let native_backend=compiled.prepared["native_backend"].clone();
    Ok(json!({"schema_version":"bisect-fresh-native-state-preparation-v1","prepared":compiled.prepared,"native_backend":native_backend,"input_sha256":compiled.input_sha256,"selected_sources":compiled.source.sources,"execution_performed":false,"recorded_paths_opened":false,"scope":"Fresh selected RADJ/companion byte parsing and native State request preparation only. No original context, plan, imported receipt or filesystem path grants execution or source authenticity. Optional report/geometry observations and native replay recipe are not invented."}))
}

pub(crate) fn run(input: Input) -> Result<Value, String> {
    for field in [&input.label, &input.chamber, &input.created_at] {
        if field.trim().is_empty() || field.len() > 200 { return Err("Record nonempty fresh export label/chamber/time of at most200 bytes before execution.".into()); }
    }
    let compiled = compile(&input)?;
    let native_backend=compiled.prepared["native_backend"].clone();
    let template: Value = serde_json::from_str(compiled.prepared["prepared_request_json"].as_str().ok_or("Missing fresh prepared request")?).map_err(|e| e.to_string())?;
    let (mut request,result,mut multiscale)=if template["operation"]=="run-engine-build" {
        let request:crate::engine::Request=serde_json::from_value(template["request"].clone()).map_err(|e|e.to_string())?;
        let result=crate::engine_build::run(request.clone())?;(request,result,None)
    }else if template["operation"]=="run-multiscale-build" {
        let selected:crate::multiscale_input::MultiscaleRequest=serde_json::from_value(template["input"].clone()).map_err(|e|e.to_string())?;
        let result=crate::engine_build::run_multiscale(selected.clone())?;(selected.request.clone(),result,Some(selected))
    }else{return Err("Fresh State execution requires native ordinary/multiscale build/retry semantics.".into());};
    if let Some(attempt) = result.get("build_report").and_then(|r| r["attempts"].as_array()).and_then(|a| a.last()) {
        request.options.seed = match &attempt["seed"] { Value::Null => None, Value::String(s) => Some(s.parse().map_err(|_| "Invalid accepted native build seed")?), _ => return Err("Invalid accepted native build seed".into()) };
    }
    let executed = if let Some(selected)=&mut multiscale{selected.request.options.seed=request.options.seed;serde_json::to_value(selected)}else{serde_json::to_value(&request)}.map_err(|e|e.to_string())?;
    let exported = export(&input, &compiled, &request, &result, &executed);
    let (export, export_error) = match exported { Ok(v) => (Some(v), None), Err(e) => (None, Some(e)) };
    Ok(json!({"schema_version":"bisect-fresh-native-state-run-v1","prepared":compiled.prepared,"native_backend":native_backend,"input_sha256":compiled.input_sha256,"selected_sources":compiled.source.sources,"execution_performed":true,"recorded_paths_opened":false,"executed_request":executed,"result":result,"export":export,"export_error":export_error,"scope":"Explicit fresh native State execution from actual selected bytes, with native retries and shared native unit/boundary/context construction. Completed assignments survive export failure. New browser source/request hashes are explicit; no original native context, native replay recipe, publisher authenticity, legal certification or optimality is claimed."}))
}

fn export(input: &Input, compiled: &Compiled, request: &crate::engine::Request, result: &Value, executed: &Value) -> Result<Value, String> {
    for field in [&input.label, &input.chamber, &input.created_at] {
        if field.trim().is_empty() || field.len() > 200 { return Err("Record nonempty fresh export label/chamber/time of at most200 bytes.".into()); }
    }
    let physical=compiled.fine.as_ref().unwrap_or(&compiled.source);
    let resolution=if compiled.fine.is_some(){"bg"}else{&compiled.resolution};
    let units = bisect_runner::native_plan_context::unit_index(&request.graph.state, &request.graph.year, resolution, &physical.loaded)?;
    let expected_ids=if compiled.fine.is_some(){&physical.graph.geoids}else{&request.graph.geoids};
    if &units.unit_ids != expected_ids { return Err("Fresh executed units differ from the selected native source.".into()); }
    let assignments: HashMap<String, u32> = serde_json::from_value(result["assignments"].clone()).map_err(|e| e.to_string())?;
    if assignments.len() != units.unit_ids.len() { return Err("Fresh native assignments must cover the exact selected unit universe.".into()); }
    let indexed: HashMap<usize, usize> = units.unit_ids.iter().enumerate().map(|(i, id)| assignments.get(id).map(|&d| (i, d as usize)).ok_or("Missing fresh native assignment")).collect::<Result<_, _>>()?;
    let plan = rplan_core::DistrictPlan {
        schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.into(), units: units.clone(),
        assignment: bisect_runner::native_plan_context::assignment(&indexed, units.unit_ids.len(), request.options.districts)?,
        k: request.options.districts, display_labels: (1..=request.options.districts).map(|d| d.to_string()).collect(), allow_empty_districts: false,
    };
    plan.validate().map_err(|e| e.to_string())?;
    let mut hashes = BTreeMap::new();
    for (field, role) in [("adjacency","adjacency"),("geoids","browser.native-geoid-companion-v1"),("centroids","browser.native-centroid-companion-v1")] {
        if let Some(hash) = physical.sources[field]["sha256"].as_str() { hashes.insert(role.into(), format!("sha256:{hash}")); }
    }
    if compiled.fine.is_some(){for(field,role)in[("adjacency","browser.native-primary-adjacency-v1"),("geoids","browser.native-primary-geoid-companion-v1"),("centroids","browser.native-primary-centroid-companion-v1")]{if let Some(hash)=compiled.source.sources[field]["sha256"].as_str(){hashes.insert(role.into(),format!("sha256:{hash}"));}}}
    if let Some(fine)=&compiled.fine{for(field,role)in[("adjacency","browser.native-fine-adjacency-v1"),("geoids","browser.native-fine-geoid-companion-v1"),("centroids","browser.native-fine-centroid-companion-v1")]{if let Some(hash)=fine.sources[field]["sha256"].as_str(){hashes.insert(role.into(),format!("sha256:{hash}"));}}}
    hashes.insert("browser.native-state-request-v1".into(), format!("sha256:{}", compiled.prepared["prepared_request_sha256"].as_str().ok_or("Missing prepared request identity")?));
    hashes.insert("browser.native-state-executed-v1".into(), format!("sha256:{}", digest(&serde_json::to_vec(executed).map_err(|e| e.to_string())?)));
    if let Some(auxiliary) = &request.native_auxiliary { hashes.extend(crate::native_auxiliary::source_hashes(auxiliary)?); }
    let context = bisect_runner::native_plan_context::context(units, &physical.loaded, rplan_core::SourceHashes { entries: hashes }, None, None)?;
    context.validate().map_err(|e| e.to_string())?;
    let mut binding = json!({"schema_version":"bisect-fresh-native-state-context-binding-v1","input_sha256":compiled.input_sha256,"arguments":input.arguments,"native_backend":compiled.prepared["native_backend"],"selected_sources":compiled.source.sources,"native_plan_resolution":resolution,"original_context_loaded":false,"native_replay_recipe":"not-supplied-or-invented","optional_report_geometry_observations":"not-supplied; no absence/authenticity inference"});
    if let Some(multiscale)=&input.multiscale{binding["native_multiscale_sampler"]=serde_json::to_value(multiscale.sampler).map_err(|e|e.to_string())?;binding["primary_resolution"]=json!(compiled.resolution);}
    let mut producer = BTreeMap::from([
        ("engine_options".into(), serde_json::to_value(&request.options).map_err(|e| e.to_string())?),
        ("fresh_native_state_binding".into(), binding.clone()),
    ]);
    if let Some(weights) = &request.native_weighting { producer.insert("native_weighting_spec".into(), serde_json::to_value(weights).map_err(|e| e.to_string())?); }
    if let Some(auxiliary) = &request.native_auxiliary { producer.insert("native_auxiliary_input".into(), serde_json::to_value(auxiliary).map_err(|e| e.to_string())?); }
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.into(), plan,
        metadata: rplan_io::RplanMetadataV02 { label: input.label.clone(), jurisdiction: request.graph.state.clone(), chamber: input.chamber.clone(), created_at: input.created_at.clone(), description: Some("Fresh selected-byte native State output; source names/years remain owner claims, native recipe/report/geometry observations are not invented.".into()) },
        provenance: rplan_io::RplanProvenance { producer, source_hashes: context.source_hashes.entries.clone(), conversion_lineage: vec![json!({"operation":"fresh-native-state-context-v1","execution":"native-prepared-engine-build","output":"shared-native-plan-context","original_context_loaded":false})] },
        geometry: None, extensions: BTreeMap::new(),
    };
    let document_json = rplan_io::write_rplan_string(&document).map_err(|e| e.to_string())?;
    let context_json = rplan_io::write_rctx_string(&context).map_err(|e| e.to_string())?;
    rplan_io::read_rplan_str(&document_json).map_err(|e| e.to_string())?;
    rplan_io::read_rctx_str(&context_json).map_err(|e| e.to_string())?;
    Ok(json!({"schema_version":"bisect-fresh-native-state-export-v1","document_json":document_json,"context_json":context_json,"plan_hash":document.plan.plan_hash().map_err(|e| e.to_string())?,"context_hash":context.context_hash,"document":document,"context":context,"fresh_native_state_binding":binding,"source_boundary":"Actual selected RADJ/companion bytes and executed request, with shared native units, physical boundaries and county reports. Browser-v1 source/request hash roles differ explicitly from a native original recipe digest. No imported context, source authentication, inferred optional report/geometry data or native recipe is used."}))
}
