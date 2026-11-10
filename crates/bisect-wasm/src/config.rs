//! Inspect, resolve, render and execute the shared native YAML schema in memory.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn uses_metis(split: &bisect_runner::config::ResolvedSplit) -> bool {
    use bisect_runner::config::ResolvedSplit as Split;
    !matches!(split, Split::MovingKnife {..} | Split::Spectral {..}
        | Split::Regionalization | Split::FlowConstruction)
}

#[derive(Clone,serde::Deserialize,serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfiguredRun {
    #[serde(default,skip_serializing_if="Option::is_none")]pub compact_geometry:Option<String>,
    #[serde(default)]pub electoral:Option<crate::electoral_input::Input>,
    #[serde(default)]pub coi:Option<crate::coi_input::Input>,
    #[serde(default)]pub capture_rounds:bool,
    #[serde(default)]pub vra_absent_file_fallback:bool,
    pub yaml_text: String,
    pub source_name: String,
    pub profile: String,
    pub graph: crate::engine::PreparedGraph,
    pub chamber: String,
    pub districts: usize,
    #[serde(default)]
    pub demographics: Option<crate::engine::DemographicInput>,
}

/// Execute the selected graph using the native YAML compositor's actual dispatch rules.
/// Build-wide fetching, retries and analyses are separate workflows.
pub(crate) fn prepare(input: ConfiguredRun) -> Result<(Value, crate::engine::Request), String> {
    use bisect_runner::config::{ResolvedEngine, ResolvedSearch as Search, ResolvedSplit as Split};
    let mut provenance = validate(&input.yaml_text, &input.source_name, &input.profile)?;
    let yaml = bisect_runner::config::AlgoYaml::from_text(&input.yaml_text)?;
    let resolved = yaml.resolve()?;
    if input.compact_geometry.as_ref().is_some_and(|profile|
        !matches!(resolved.split,Split::CompactBisect{..}) || !["prepared","native-state-circular"].contains(&profile.as_str())) {
        return Err("Configured geometry override requires CompactBisect and prepared or native-state-circular.".into());
    }
    if input.vra_absent_file_fallback && (!matches!(resolved.split,Split::VraSection{..}) || input.demographics.is_some()) {return Err("Configured native absent-file fallback requires VRASection and no demographic input.".into());}
    let method_requires_metis = uses_metis(&resolved.split);
    let single_district = input.districts == 1;
    let metis_used = method_requires_metis && !single_district;
    if metis_used && !matches!(resolved.metis.engine, ResolvedEngine::RedistMetis)
        && !(matches!(resolved.metis.engine,ResolvedEngine::CFfi)&&crate::metis_backend::c_available()) {
        return Err("The declared YAML METIS backend is unavailable in this WASM build; no engine is substituted.".into());
    }
    if input.chamber.is_empty()||input.chamber.len()>200||input.chamber.chars().any(char::is_control){return Err("Record a chamber name of 1-200 UTF-8 bytes without controls.".into());}
    let key = match input.chamber.as_str() {
        "congressional" => "balance_tolerance_congressional_pct",
        "house" | "lower" | "assembly" => "balance_tolerance_house_pct",
        "senate" | "upper" => "balance_tolerance_senate_pct",
        _ => "",
    };
    const POLICY: &str = include_str!("../../../data/location_policy.json");
    let policy: Value = serde_json::from_str(POLICY).map_err(|e| e.to_string())?;
    // Shared accessor also enforces the native percent range.
    let tolerance = match yaml.run_balance_tolerance()? {
        Some(fraction) => fraction * 100.0,
        None => policy[&input.graph.state][key].as_f64().filter(|v| *v > 0.0)
            .unwrap_or_else(||bisect_analysis::location_registry::chamber_balance_tolerance(&policy,&input.graph.state,&input.chamber)*100.0),
    };
    let weights = if resolved.weights.minority_weighting {"vra-aligned"}
        else if resolved.weights.alpha_county > 1e-10 {"county"}
        else if resolved.weights.geographic {"geographic"} else {"unweighted"};
    let mut options = json!({"structure":"standard-bisect","weights":weights,"search":"single",
        "districts":input.districts,"seed":resolved.metis.seed.map(|v|v.to_string()),
        "seeds":1,"steps":200,"percentile":0.0,"alpha_county":if weights=="county"{resolved.weights.alpha_county}else{0.0},
        "balance_tolerance":tolerance,"area_swing":1.10,"iterations":resolved.metis.niter});
    let count = match &resolved.seeds {
        Search::Single => 1,
        Search::Multi {seeds} => *seeds,
        Search::ConvergenceSweep {threshold} => *threshold as usize,
        Search::ShortBurst {n_bursts,..} | Search::ShortBurstForest {n_bursts,..}
            | Search::ShortBurstMergeSplit {n_bursts,..} => *n_bursts,
    };
    let interpretation = match &resolved.split {
        Split::Bisect => {
            match &resolved.seeds {
                Search::Single => {},
                Search::Multi {seeds} => {options["search"]=json!("multi"); options["seeds"]=json!(seeds);},
                Search::ConvergenceSweep {threshold} => {
                    options["search"]=json!("convergence"); options["seeds"]=json!(threshold);
                    options["convergence_unlimited"]=json!(true);
                },
                Search::ShortBurst {burst_length,n_bursts,p}
                | Search::ShortBurstForest {burst_length,n_bursts,p}
                | Search::ShortBurstMergeSplit {burst_length,n_bursts,p} => {
                    options["search"]=json!(match resolved.seeds {
                        Search::ShortBurst {..} => "short-burst",
                        Search::ShortBurstForest {..} => "short-burst-forest",
                        _ => "short-burst-merge-split",
                    });
                    options["burst_length"]=json!(burst_length); options["n_bursts"]=json!(n_bursts);
                    options["percentile"]=json!(p);
                },
            }
            "native-search-compositor"
        },
        Split::GeoSection | Split::AreaSection {..} | Split::VraSection {..} | Split::CompactBisect {..} => {
            options["structure"]=json!(match &resolved.split {
                Split::GeoSection => "ratio-optimal",
                Split::AreaSection {area_swing,..} => {
                    options["area_swing"]=json!(area_swing); options["area_init"]=json!("ratio-optimal"); "ratio-optimal-area"
                },
                Split::VraSection {w_vra} => {options["w_vra"]=json!(w_vra); "ratio-optimal-vra"},
                Split::CompactBisect {epsilon} => {options["compact_epsilon"]=json!(epsilon); "compact-polsby"},
                _ => unreachable!(),
            });
            options["seeds"]=json!(count);
            if count != 1 {options["search"]=json!("multi");}
            "native-seed-count-per-split"
        },
        split => {
            options["structure"]=json!(match split {
                Split::NWay => "nway", Split::ApportionRegions => "prime-factor",
                Split::MovingKnife {n_orientations,metric} => {
                    options["mka_orientations"]=json!(n_orientations);
                    // Preserve native runner behavior, including its exact token check.
                    options["mka_metric"]=json!(if metric=="polsby" {"polsby"} else {"reock"}); "moving-knife"
                },
                Split::Spectral {max_iters} => {options["steps"]=json!(max_iters); "spectral"},
                Split::Regionalization => "regionalization", Split::FlowConstruction => "flow-construction",
                _ => unreachable!(),
            });
            "native-structure-ignores-search"
        },
    };
    if resolved.weights.minority_weighting&&resolved.weights.alpha_county>1e-10{options["county_augmentation"]=json!(resolved.weights.alpha_county);}
    if input.capture_rounds {options["capture_rounds"]=json!(true);}
    if let Some(profile)=&input.compact_geometry {options["compact_geometry"]=json!(profile);provenance["compact_geometry"]=json!(profile);}
    if input.vra_absent_file_fallback {options["vra_absent_file_fallback"]=json!(true);}
    if metis_used&&matches!(resolved.metis.engine,ResolvedEngine::CFfi){options["metis_engine"]=json!("c-ffi");options["metis_runtime"]=json!("msvcrt-v1");}
    let request = crate::engine::Request {native_weighting:None,native_auxiliary:None,zone_membership:None,native_character_source:None,graph:input.graph,
        options:serde_json::from_value(options).map_err(|e|format!("Invalid resolved run options: {e}"))?,
        demographics:input.demographics, character:None,elections:None,partisan:None,coi:input.coi,electoral:input.electoral};
    if let Some(coi)=&request.coi{crate::coi_input::validate(coi)?;provenance["coi_weights_source_sha256"]=json!(crate::coi_input::hash(coi));}
    if let Some(settings)=&request.electoral {settings.resolve(request.options.districts)?;provenance["electoral_settings"]=serde_json::to_value(settings).map_err(|e|e.to_string())?;}
    provenance["search_interpretation"]=json!(interpretation);
    provenance["execution_chamber"]=json!(input.chamber);
    provenance["metis_engine_used"]=json!(metis_used);
    provenance["metis_engine_required_by_method"]=json!(method_requires_metis);
    let native_single_construction=single_district&&["capacity-clustering","regionalization","flow-construction"].contains(&request.options.structure.as_str());
    provenance["partitioning_shortcut"]=json!(single_district&&!native_single_construction);
    provenance["browser_engine_available"]=json!(true);
    provenance["browser_execution_backend"]=json!(if native_single_construction {
        "shared-native-rust-no-metis"
    } else if single_district {
        "single-district-no-partitioning"
    } else if metis_used {
        if matches!(resolved.metis.engine,ResolvedEngine::CFfi){"c-metis-msvcrt-v1"}else{"metis-core"}
    } else {
        "shared-native-rust-no-metis"
    });
    provenance["execution_tolerance_percent"]=json!(tolerance);
    provenance["embedded_policy_sha256"]=json!(format!("{:x}",Sha256::digest(POLICY.as_bytes())));
    provenance["execution_scope"]=json!("One selected prepared graph; native first-attempt compositor dispatch. Embedded location policy is used when YAML omits tolerance. Native environment policy overrides, build-wide years/workers/analyses, fetching and retry orchestration are not executed.");
    provenance["scope"]=json!("Shared native YAML compositor validation followed by execution of one selected prepared graph. A one-district request retains native declarations and assigns all validated units without partitioning. Other methods using METIS require the declared metis-core engine; moving-knife, spectral, regionalization and flow construction execute shared native Rust kernels without METIS. Original backend declarations remain unchanged when unused, and no backend is substituted. Graph/options/data validation, population and connectivity checks still apply. Exact raw source, effective options and native search interpretation are retained. Court profile filtering does not establish legal compliance and heuristic completion does not establish optimality.");
    if metis_used&&matches!(resolved.metis.engine,ResolvedEngine::CFfi){provenance["scope"]=json!("Shared native YAML compositor executed on one selected prepared graph with the explicit c-ffi backend and msvcrt-v1 browser runtime. The C module must be loaded for execution; declared native backend and exact raw YAML are retained. This runtime profile does not imply equivalence to every native libc or establish legal compliance, capacity or global optimality.");}
    Ok((provenance, request))
}

pub(crate) fn run(input: ConfiguredRun) -> Result<Value, String> {
    run_at(input,None)
}
pub(crate) fn prepare_at(input:ConfiguredRun,attempt:Option<u32>)->Result<(Value,crate::engine::Request),String>{
    let (mut configuration,mut request)=prepare(input)?;
    if let Some(attempt)=attempt {
        if attempt>bisect_runner::balance_retry::MAX_BALANCE_RETRIES {return Err("Configured replay attempt exceeds the native retry policy.".into());}
        request.options.seed=bisect_runner::balance_retry::balance_retry_seed(request.options.seed,attempt);
        configuration["execution_attempt"]=json!(attempt);
        configuration["execution_seed"]=json!(request.options.seed.map(|s|s.to_string()));
        configuration["execution_scope"]=json!("One selected prepared graph at the explicitly selected native retry attempt. Earlier attempts are not executed. Embedded location policy is used when YAML omits tolerance; build-wide acquisition, workers and analyses are not executed.");
    }
    Ok((configuration,request))
}
pub(crate) fn run_at(input:ConfiguredRun,attempt:Option<u32>)->Result<Value,String>{
    let (provenance, request) = prepare_at(input,attempt)?;
    let request_json = serde_json::to_value(&request).map_err(|e|e.to_string())?;
    let result = crate::engine::execute(request)?;
    Ok(json!({"schema_version":"bisect-configured-run-v1","configuration":provenance,
        "engine_request":request_json,"result":result}))
}

pub(crate) fn build(input: ConfiguredRun) -> Result<Value, String> {build_typed(input).map_err(|error|error.message())}
pub(crate) fn build_typed(input: ConfiguredRun) -> Result<Value, crate::engine_build::BuildFailure> {
    let (mut provenance,mut request)=prepare(input)?;
    let mut result=crate::engine_build::run_detailed(request.clone())?;
    let report=result.as_object_mut().ok_or("Missing build result.")?.remove("build_report").ok_or("Missing build report.")?;
    request.options=serde_json::from_value(result["options"].clone()).map_err(|e|e.to_string())?;
    provenance["execution_attempt"]=report["accepted_attempt"].clone();
    provenance["execution_seed"]=json!(request.options.seed.map(|s|s.to_string()));
    provenance["execution_scope"]=json!("One selected graph with the shared native 50-retry seed policy and native final-plan population assertion. Build-wide fetching, workers, years, analyses and native environment overrides are not executed.");
    Ok(json!({"schema_version":"bisect-configured-run-v1","configuration":provenance,
        "engine_request":request,"result":result,"build_report":report}))
}

/// Resolve and export supplied assignments. No generation or replay takes place.
pub(crate) fn export(input: ConfiguredRun, assignments: std::collections::BTreeMap<String,u32>, label:String, created_at:String, attempt:Option<u32>) -> Result<Value,String> {
    use rplan_io::{RplanDocument,write_rplan_string,read_rplan_str,read_rctx_str};
    let chamber=input.chamber.clone();
    let (mut configuration,mut request)=prepare(input)?;
    if let Some(attempt)=attempt {
        if attempt>bisect_runner::balance_retry::MAX_BALANCE_RETRIES {return Err("Build export attempt exceeds the native retry policy.".into());}
        request.options.seed=bisect_runner::balance_retry::balance_retry_seed(request.options.seed,attempt);
        configuration["execution_attempt"]=json!(attempt);
        configuration["execution_seed"]=json!(request.options.seed.map(|s|s.to_string()));
    }
    configuration["scope"]=json!("Shared native YAML resolution for supplied assignments. Export does not execute or replay generation, authenticate inputs, certify legal compliance or establish optimality.");
    configuration["execution_scope"]=json!("Recorded selected-graph configuration; export only, with embedded chamber policy and native compositor resolution. Native build-wide orchestration and environment policy overrides are not executed.");
    let mut output=crate::plan_export::export(request,assignments,label,chamber,created_at)?;
    let mut context:rplan_core::RplanContext=serde_json::from_value(output["context"].clone()).map_err(|e|e.to_string())?;
    let mut document:RplanDocument=serde_json::from_value(output["document"].clone()).map_err(|e|e.to_string())?;
    context.source_hashes.entries.insert("bisect.algorithm-yaml".into(),configuration["source_sha256"].as_str().ok_or("Missing YAML identity.")?.into());
    if configuration["balance_tolerance_percent"].is_null() {
        context.source_hashes.entries.insert("bisect.embedded-location-policy".into(),configuration["embedded_policy_sha256"].as_str().ok_or("Missing policy identity.")?.into());
    }
    context.context_hash=context.compute_context_hash().map_err(|e|e.to_string())?;
    context.validate().map_err(|e|e.to_string())?;
    document.provenance.source_hashes=context.source_hashes.entries.clone();
    document.provenance.producer.insert("algorithm_configuration".into(),configuration.clone());
    let document_json=write_rplan_string(&document).map_err(|e|e.to_string())?;
    let context_json=crate::plan_audit_text::portable_context_json(&context)?;
    read_rplan_str(&document_json).map_err(|e|e.to_string())?;
    read_rctx_str(&context_json).map_err(|e|e.to_string())?;
    output["schema_version"]=json!("bisect-configured-plan-export-v1");
    output["configuration"]=configuration;
    output["document"]=serde_json::to_value(document).map_err(|e|e.to_string())?;
    output["context_hash"]=json!(context.context_hash);
    output["context"]=serde_json::to_value(context).map_err(|e|e.to_string())?;
    output["document_json"]=json!(document_json);
    output["context_json"]=json!(context_json);
    Ok(output)
}

pub(crate) fn inspect(text: &str, source_name: &str) -> Result<Value, String> {
    if text.is_empty() || text.len() > 256 * 1024 {
        return Err("Configuration YAML must be nonempty and at most 256 KiB.".into());
    }
    if source_name.is_empty()
        || source_name.len() > 120
        || !source_name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    {
        return Err("Configuration requires a plain source basename.".into());
    }
    let yaml = bisect_runner::config::AlgoYaml::from_text(text)?;
    let a = &yaml.algorithm;
    for number in [
        a.alpha_county,
        a.percentile,
        a.balance_tolerance,
        a.area_swing,
        a.w_vra,
    ]
    .into_iter()
    .flatten()
    {
        if !number.is_finite() {
            return Err("Configuration numbers must be finite.".into());
        }
    }
    let mut document = serde_json::to_value(&yaml).map_err(|e| e.to_string())?;
    // The raw source and decimal strings retain integers that JS cannot represent exactly.
    for key in [
        "seed",
        "seeds",
        "burst_length",
        "n_bursts",
        "mka_orientations",
    ] {
        if let Some(n) = document["algorithm"][key].as_u64() {
            document["algorithm"][key] = Value::String(n.to_string());
        }
    }
    if let Some(n) = document["workers"].as_u64() {
        document["workers"] = Value::String(n.to_string());
    }
    Ok(json!({
        "schema_version":"bisect-algorithm-config-inspection-v1",
        "source_name":source_name,
        "source_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),
        "yaml_text":text,
        "document":document,
        "resolved_workers":yaml.resolved_workers().to_string(),
        "resolved_years":yaml.resolved_years(),
        "resolved_analysis_types":yaml.resolved_analysis_types(),
        "seed":a.seed.map(|n|n.to_string()),
        "declared_engine":a.engine.as_deref().unwrap_or("c-ffi"),
        "scope":"Parses the shared native closed YAML schema and resolves workers, years and analysis defaults. This inspection does not validate algorithm names, conditional options, court profile, data availability or run feasibility. No analysis or engine execution occurs. An omitted engine retains the native c-ffi declaration; no browser engine substitution is performed. Source hashes establish byte integrity, not authorship."
    }))
}

fn exact_display(value: &mut Value) {
    match value {
        Value::Number(n) if n.as_u64().is_some_and(|x| x > 9_007_199_254_740_991) => {
            *value = Value::String(n.to_string())
        }
        Value::Object(fields) => fields.values_mut().for_each(exact_display),
        Value::Array(values) => values.iter_mut().for_each(exact_display),
        _ => {}
    }
}

pub(crate) fn validate(text: &str, source_name: &str, profile: &str) -> Result<Value, String> {
    if !matches!(profile, "open" | "court") {
        return Err("Configuration profile must be open or court.".into());
    }
    let mut output = inspect(text, source_name)?;
    let yaml = bisect_runner::config::AlgoYaml::from_text(text)?;
    if profile == "court" {
        yaml.validate_court(source_name)?;
    }
    let resolved = yaml.resolve()?;
    let metis_used = uses_metis(&resolved.split);
    let engine_available = !metis_used || (matches!(resolved.metis.engine,bisect_runner::config::ResolvedEngine::CFfi)&&crate::metis_backend::c_available()) || matches!(
        resolved.metis.engine,
        bisect_runner::config::ResolvedEngine::RedistMetis
    );
    let raw = serde_json::to_string_pretty(&resolved).map_err(|e| e.to_string())?;
    let mut display = serde_json::to_value(&resolved).map_err(|e| e.to_string())?;
    exact_display(&mut display);
    output["schema_version"] = json!("bisect-algorithm-config-validation-v1");
    output["resolution"] = display;
    output["resolution_json"] = json!(raw);
    output["native_summary"] =
        json!(yaml.validation_summary(source_name, output["source_sha256"].as_str().unwrap())?);
    output["profile"] = json!(profile);
    output["compositor_valid"] = json!(true);
    output["browser_engine_available"] = json!(engine_available);
    output["metis_engine_used"] = json!(metis_used);
    output["metis_engine_required_by_method"] = json!(metis_used);
    output["browser_execution_backend"] = json!(if !metis_used {
        "shared-native-rust-no-metis"
    } else if engine_available {
        if matches!(resolved.metis.engine,bisect_runner::config::ResolvedEngine::CFfi){"c-metis-msvcrt-v1"}else{"metis-core"}
    } else {
        "unavailable-declared-metis-engine"
    });
    output["balance_tolerance_percent"] = json!(yaml.algorithm.balance_tolerance);
    output["balance_tolerance_source"] = json!(if yaml.algorithm.balance_tolerance.is_some() {
        "explicit-yaml-percent"
    } else {
        "state-chamber-policy-at-execution"
    });
    output["scope"]=json!("Uses the shared native config.Validate compositor mapping, defaults and optional court-profile filter. Browser input additionally requires finite numeric fields and bounded UTF-8 source. Native YAML validation accepts some values that a run rejects; this does not validate data, resource limits, geometry, legal compliance or every execution parameter. The court profile is a configuration filter, not a legal certificate. An omitted engine retains native c-ffi. Methods requiring METIS can execute only a declared metis-core backend; moving-knife, spectral, regionalization and flow construction do not call METIS and retain unused backend declarations. No engine is substituted and no plan run occurs. Exact raw YAML and resolution JSON retain integer values; the display uses decimal strings outside the JavaScript safe range. Native proportional weights currently resolve to geographic weights, preserving native behavior.");
    if metis_used&&matches!(resolved.metis.engine,bisect_runner::config::ResolvedEngine::CFfi)&&engine_available{output["scope"]=json!("Shared native configuration inspection only; the optional C bridge is compiled in this engine. A C run requires the msvcrt-v1 sidecar runtime, loaded only by an explicit runtime/Run action. No C module, source data or plan executes during this inspection. Exact YAML/native resolution remain retained; profile filtering is not legal certification and runtime availability is not capacity or full-parity evidence.");}
    Ok(output)
}

pub(crate) fn create(settings: &bisect_runner::config::TemplateConfig) -> Result<Value, String> {
    let text = bisect_runner::config::render_template_config(settings)?;
    let filename = format!("{}.yml", settings.name);
    let mut result = validate(&text, &filename, "open")?;
    let mut source_settings = serde_json::to_value(settings).map_err(|e| e.to_string())?;
    exact_display(&mut source_settings);
    result["schema_version"] = json!("bisect-algorithm-config-template-v1");
    result["generated_filename"] = json!(filename);
    result["template_settings"] = source_settings;
    result["scope"]=json!("Creates YAML through the shared native config.New renderer and validates its compositor with the open profile. Explicit optional engine and exact decimal seed fields extend the native template; omission retains native c-ffi and no fixed seed. Plain label names, four-digit years and finite template numbers prevent malformed or injected YAML. Defaults and native template bytes are preserved for ordinary CLI inputs. Creation does not execute a plan, fetch inputs or write to disk. Use the generated YAML to inspect or validate another profile. Source hash establishes byte integrity, not authorship; compositor validation and browser engine availability do not establish run feasibility or legal compliance.");
    Ok(result)
}
