//! Execute state analyses and emit the native label index without filesystem access.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use bisect_report::label_indexes::{AnalysisIndex, StateAnalysisStatus, serialize_analysis_index};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub build_index_json: String,
    pub registry_json: Option<String>,
    pub created: String,
    pub states: BTreeMap<String, crate::analysis_package::Input>,
    pub types: Option<Vec<String>>,
    pub external_import: Option<ExternalImport>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExternalImport {
    pub input: crate::label_import::Input,
    pub district_bases: BTreeMap<String,String>,
}
#[derive(Deserialize)]
pub(crate) struct SourceIndex {
    pub label:String,
    pub year:String,
    pub states:BTreeMap<String,bisect_runner::build_index::StateIndexEntry>,
}
pub(crate) fn source_index(text:&str,external:bool)->Result<SourceIndex,String> {
    if external {serde_json::from_str(text).map_err(|e|e.to_string())}
    else {
        let raw:Value=serde_json::from_str(text).map_err(|e|e.to_string())?;
        if raw["algorithm"]["structure"]=="external" {return Err("External label analysis requires the retained original import source and numbering bases.".into());}
        let index:bisect_runner::build_index::BuildIndex=serde_json::from_value(raw).map_err(|e|e.to_string())?;
        Ok(SourceIndex{label:index.label,year:index.year,states:index.states})
    }
}
fn sha(text: &str) -> String { format!("{:x}", Sha256::digest(text.as_bytes())) }
pub(crate) fn run(input: Input) -> Result<Value, String> {
    run_internal(input,false)
}
pub(crate) fn run_for_years(input: Input) -> Result<Value, String> {
    run_internal(input,true)
}
fn run_internal(input: Input,allow_empty_selection:bool) -> Result<Value, String> {
    if input.build_index_json.len() > 1024*1024 || input.states.len() > 60 {
        return Err("Label analysis accepts a build index up to 1 MiB and at most 60 states.".into());
    }
    let external_bytes=input.external_import.as_ref().map(|proof|serde_json::to_string(&proof.input).map(|s|s.len())).transpose().map_err(|e|e.to_string())?.unwrap_or(0);
    if external_bytes>25*1024*1024 {return Err("Original external import sources exceed the 25 MiB batch limit.".into());}
    let imported=input.external_import.as_ref().map(|proof|crate::label_import::run(proof.input.clone())).transpose()?;
    if let Some(imported)=&imported {
        if imported["index_json"].as_str()!=Some(input.build_index_json.as_str()) {return Err("External batch index differs from the freshly reparsed native import.".into());}
        let proof=input.external_import.as_ref().unwrap();
        if proof.district_bases.len()>60 || input.states.keys().any(|name|!proof.district_bases.contains_key(name)) || proof.district_bases.iter().any(|(name,basis)| imported["states"].get(name).is_none()||!["one-based","zero-based"].contains(&basis.as_str())) {return Err("External batches need an explicit valid numbering basis for every selected imported state.".into());}
    }
    let build=source_index(&input.build_index_json,imported.is_some())?;
    let types = crate::analysis_package::selected_types(input.types)?;
    let mut index = AnalysisIndex {label:build.label.clone(), year:build.year.clone(), run:build.label.clone(), run_index_sha256:sha(&input.build_index_json), types:types.clone(), created:input.created, states:HashMap::new()};
    // Validate path metadata before any state execution.
    serialize_analysis_index(&index)?;
    let registry=crate::label_stage::promote(input.registry_json.as_deref(),&build.label,&build.year,false)?;
    if build.states.is_empty() || build.states.len()>60 || input.states.is_empty()&&!allow_empty_selection {
        return Err("Select at least one state source and a build index containing 1–60 states.".into());
    }
    for (name, state) in &build.states {
        index.states.insert(name.clone(),StateAnalysisStatus{status:"skipped".into(),error:None,type_statuses:HashMap::new()});
        if !matches!(state.status.as_str(),"ok"|"failed"|"missing") {return Err("Unknown native build state status.".into());}
    }
    serialize_analysis_index(&index)?;
    let policy:Value = serde_json::from_str(include_str!("../../../data/location_policy.json")).map_err(|e|e.to_string())?;
    let locations = bisect_analysis::location_registry::PolicyRegistry::new(&policy);
    let mut total_sources=input.build_index_json.len()+external_bytes;
    for (name, sources) in &input.states {
        let entry=build.states.get(name).ok_or("Selected state is absent from the build index.")?;
        if entry.status!="ok" {return Err("Selected state must have a successful build entry.".into());}
        let plan=crate::plan_audit_text::plan(&sources.document_json)?;
        let resolved=locations.state_name(plan.units.state.as_deref().ok_or("Plan state is required.")?).ok_or("Unknown plan state code.")?.to_lowercase().replace(' ',"_");
        if &resolved!=name || plan.units.year.ok_or("Plan year is required.")?.to_string()!=build.year || imported.is_none()&&entry.districts!=Some(plan.k as usize) {
            return Err("Selected plan state, year or district count does not match its build entry.".into());
        }
        if let Some(imported)=&imported {
            let prepared=crate::label_import::prepare_imported_plan(imported,name,sources.context_json.as_deref().ok_or("External label analysis requires each state's exact RCTX.")?,&sha(&input.build_index_json),&input.external_import.as_ref().unwrap().district_bases[name])?;
            if prepared["document_json"].as_str()!=Some(sources.document_json.as_str()) {return Err("External state plan differs from fresh native source/context alignment.".into());}
        }
        total_sources+=sources.document_json.len();
        for s in [&sources.context_json,&sources.geography_json,&sources.demographic_csv,&sources.urban_csv,&sources.election_csv].into_iter().flatten() {total_sources+=s.len();}
        if let Some(g)=&sources.compactness {total_sources+=g.unit_geometry_json.len()+g.crs_label.len()+g.population_centroids_csv.as_ref().map_or(0,String::len);}
        if total_sources>25*1024*1024 {return Err("Combined label analysis sources exceed 25 MiB; select a smaller batch.".into());}
    }
    let root=format!("analysis/{}/{}",build.label,build.year);
    let mut files=BTreeMap::new();let mut outcomes=Vec::new();let mut output_bytes=0usize;
    for (name, sources) in input.states {
        let status=index.states.get_mut(&name).expect("validated state");
        match crate::analysis_package::run_selected(sources,&types) {
            Err(error)=>{status.status="failed".into();status.error=Some(error.clone());outcomes.push(json!({"state_name":name,"status":"failed","error":error}));}
            Ok(package)=>{
                let mut errors=Vec::new();
                for step in package["steps"].as_array().ok_or("Missing analyzer steps.")? {
                    let kind=step["name"].as_str().ok_or("Missing analyzer name.")?;
                    let native=match step["status"].as_str(){Some("computed")=>"ok",Some("failed")=>{errors.push(step["error"].as_str().unwrap_or("Analyzer failed").to_string());"failed"},Some("unavailable")=>"unavailable",_=>return Err("Unknown analyzer status.".into())};
                    status.type_statuses.insert(kind.into(),native.into());
                }
                status.status=if !errors.is_empty(){"failed"}else if status.type_statuses.values().any(|s|s!="ok"){"unavailable"}else{"ok"}.into();
                if !errors.is_empty(){status.error=Some(errors.join("; ").chars().take(10000).collect());}
                for (filename,text) in package["files"].as_object().ok_or("Missing analyzer files.")? {
                    let text=text.as_str().ok_or("Analyzer file is not text.")?;
                    output_bytes+=text.len();if output_bytes>25*1024*1024{return Err("Label analysis outputs exceed 25 MiB; select a smaller batch.".into());}
                    files.insert(format!("{root}/{name}/{filename}"),text.to_string());
                }
                outcomes.push(json!({"state_name":name,"status":status.status,"report_ready":package["report_ready"],"steps":package["steps"],"plan_source_sha256":package["plan_source_sha256"],"source_sha256":package["source_sha256"]}));
            }
        }
    }
    let index_json=serialize_analysis_index(&index)?;let index_path=format!("{root}/index.json");
    if output_bytes+index_json.len()>25*1024*1024{return Err("Label analysis outputs exceed 25 MiB.".into());}
    files.insert(index_path.clone(),index_json.clone());
    if let Some(text)=&registry{files.insert(".bisect".into(),text.clone());}
    if files.values().map(String::len).sum::<usize>()>25*1024*1024{return Err("Analysis outputs and registry exceed 25 MiB.".into());}
    let hashes:BTreeMap<_,_>=files.iter().map(|(p,t)|(p.clone(),sha(t))).collect();
    let mut output=json!({"schema_version":"bisect-label-analysis-v1","label":build.label,"year":build.year,"index_path":index_path,"index_json":index_json,"index_sha256":sha(&index_json),"files":files,"file_sha256":hashes,"states":outcomes,"registry_promoted":registry.is_some(),"scope":"Runs shared native analyses on supplied state plans and emits the native flattened label analysis index. The chain hash binds the exact supplied build-index bytes. State, year and district count must match the recorded build entry; the build index itself does not authenticate plan assignments. Unselected states are skipped, missing analyzer sources are unavailable, and failures preserve successful files. Optional supplied registries record native command-completion stages even for partial outcomes; individual statuses remain authoritative. No disk registry is written. Save source projects separately to retain raw CSV and geometry inputs."});
    if let Some(text)=registry{output["registry_json"]=json!(text);output["registry_source_sha256"]=json!(sha(input.registry_json.as_deref().expect("supplied registry")));}
    if let Some(imported)=imported {
        output["external_import_verified"]=json!(true);
        output["external_source"]=imported["source"].clone();
        output["external_district_bases"]=json!(input.external_import.unwrap().district_bases);
        output["scope"]=json!("External native import index retained byte-for-byte. Original source/sidecars reparsed and every selected RPLAN reconstructed from its exact RCTX and explicit district numbering before any analysis executes. Shared native plan analyzers and native analysis index/registry completion semantics; unselected states skipped, missing evidence unavailable. This proves correspondence to supplied bytes, not source truth, engine generation or certification. Registry snapshot returned for review, never installed. Portable batch bound 25 MiB including original import payload.");
    }
    Ok(output)
}
