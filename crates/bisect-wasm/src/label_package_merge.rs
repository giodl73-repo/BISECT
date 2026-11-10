//! Prepare a repository snapshot with explicit per-artifact compare-and-swap checks.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub reconciliation:crate::label_reconcile::Input,
    pub current_files:BTreeMap<String,String>,
    pub generated_files:BTreeMap<String,String>,
    pub generated_file_sha256:BTreeMap<String,String>,
    /// null requires absence; a digest requires that exact current file.
    pub expected_file_sha256:BTreeMap<String,Option<String>>,
}
fn sha(text:&str)->String {format!("{:x}",Sha256::digest(text.as_bytes()))}
fn path_safe(path:&str)->bool {
    !path.is_empty()&&path.len()<=512&&path.split('/').all(|part|
        !part.is_empty()&&part.len()<=200&&part!="."&&part!=".."&&!part.ends_with('.')&&
        part.bytes().all(|b|b.is_ascii_alphanumeric()||b"._-".contains(&b))&&
        !["__proto__","constructor","prototype","con","prn","aux","nul","com1","com2","com3","com4","com5","com6","com7","com8","com9","lpt1","lpt2","lpt3","lpt4","lpt5","lpt6","lpt7","lpt8","lpt9"].contains(&part.split('.').next().unwrap_or("").to_ascii_lowercase().as_str()))
}
fn paths(files:&BTreeMap<String,String>)->Result<(),String>{
    if files.len()>2000||files.values().map(String::len).sum::<usize>()>25*1024*1024{return Err("Repository snapshot exceeds 2,000 files or 25 MiB.".into());}
    let mut seen=BTreeSet::new();
    for path in files.keys(){if !path_safe(path)||!seen.insert(path.to_ascii_lowercase()){return Err("Unsafe or case-colliding repository path.".into());}}
    for path in &seen {for (offset,_) in path.match_indices('/') {if seen.contains(&path[..offset]){return Err("Repository file/directory path collision.".into());}}}
    Ok(())
}
pub(crate) fn merge(input:Input)->Result<Value,String>{
    use bisect_report::label_indexes::{AnalysisIndex,ReportIndex};
    paths(&input.current_files)?;paths(&input.generated_files)?;
    if input.generated_files.get(".bisect")!=Some(&input.reconciliation.proposed_registry_json)||input.current_files.get(".bisect")!=Some(&input.reconciliation.current_registry_json){return Err("Package registries differ from reconciliation snapshots.".into());}
    let generated_index=input.reconciliation.generated_index_json.clone();
    let upstream_index=input.reconciliation.upstream_index_json.clone();
    let receipt=crate::label_reconcile::reconcile(input.reconciliation)?;
    let upstream=receipt["upstream_index_path"].as_str().ok_or("Missing upstream receipt path.")?;
    let index_path=receipt["generated_index_path"].as_str().ok_or("Missing generated receipt path.")?;
    if input.current_files.get(upstream)!=Some(&upstream_index){return Err("Current package upstream index differs from selected snapshot.".into());}
    if input.generated_files.get(index_path)!=Some(&generated_index){return Err("Generated package index differs from the reconciled stage.".into());}
    if input.generated_file_sha256.keys().ne(input.generated_files.keys())||input.expected_file_sha256.len()+1!=input.generated_files.len()||input.generated_files.keys().filter(|p|p.as_str()!=".bisect").ne(input.expected_file_sha256.keys()){return Err("Supply one generated digest per file and one current expectation per non-registry output.".into());}
    let root=index_path.strip_suffix("index.json").ok_or("Invalid stage index path.")?;
    if receipt["stage"]=="reported" {
        let index:ReportIndex=serde_json::from_str(&generated_index).map_err(|e|e.to_string())?;
        let expected:BTreeSet<String>=index.files.iter().map(|p|format!("{root}{p}")).chain([index_path.to_string(),".bisect".into()]).collect();
        if input.generated_files.keys().cloned().collect::<BTreeSet<_>>()!=expected{return Err("Generated report files differ from the native index inventory.".into());}
    }else{
        let index:AnalysisIndex=serde_json::from_str(&generated_index).map_err(|e|e.to_string())?;
        let allowed=["plan.rplan","context.rctx","manifest.json","analysis-package.json","summary.json","demographic.json","urban.json","political.json","proportionality.json","partisan.json","partisan-metrics.json","contiguity.json","compactness.json","splits.json"];
        for path in input.generated_files.keys().filter(|p|p.as_str()!=".bisect"&&p.as_str()!=index_path){let parts:Vec<_>=path.strip_prefix(root).ok_or("Generated artifact escapes its stage root.")?.split('/').collect();if parts.len()!=2||!index.states.contains_key(parts[0])||!allowed.contains(&parts[1]){return Err("Unknown generated native analysis path.".into());}}
    }
    let mut merged=input.current_files.clone();let mut changes=Vec::new();
    for (path,text) in &input.generated_files {
        if input.generated_file_sha256.get(path)!=Some(&sha(text)){return Err(format!("Generated artifact digest mismatch: {path}"));}
        if path==".bisect"{continue;}
        let current=input.current_files.get(path);
        match input.expected_file_sha256.get(path).ok_or("Missing current artifact expectation.")? {
            None if current.is_some()=>return Err(format!("Artifact appeared since the stage snapshot: {path}")),
            Some(expected) if current.map(|text|sha(text)).as_ref()!=Some(expected)=>return Err(format!("Artifact changed or disappeared since the stage snapshot: {path}")),
            _=>{},
        }
        let action=if current==Some(text){"unchanged"}else if current.is_some(){"replace"}else{"add"};
        changes.push(json!({"path":path,"action":action,"previous_sha256":current.map(|t|sha(t)),"sha256":sha(text)}));
        merged.insert(path.clone(),text.clone());
    }
    let registry=receipt["registry_json"].as_str().ok_or("Missing reconciled registry.")?;
    changes.push(json!({"path":".bisect","action":if input.current_files.get(".bisect").map(String::as_str)==Some(registry){"unchanged"}else{"reconcile"},"previous_sha256":receipt["current_registry_sha256"],"sha256":receipt["registry_sha256"]}));
    merged.insert(".bisect".into(),registry.into());paths(&merged)?;
    let hashes:BTreeMap<_,_>=merged.iter().map(|(p,t)|(p.clone(),sha(t))).collect();
    let current_hashes:BTreeMap<_,_>=input.current_files.iter().map(|(p,t)|(p.clone(),sha(t))).collect();
    Ok(json!({"schema_version":"bisect-label-package-merge-v1","status":"prepared","label":receipt["label"],"year":receipt["year"],"stage":receipt["stage"],"reconciliation":receipt,"changes":changes,"files":merged,"file_sha256":hashes,"current_file_sha256":current_hashes,"scope":"Prepares an in-memory repository snapshot. Each generated artifact requires an explicit prior digest or absence expectation; conflicting current bytes reject the entire preparation. Native stage/index checks reconcile the registry while preserving concurrent entries and unrelated files. This does not authenticate analysis claims or write local disk. Recheck every current snapshot file and absence expectation before coordinated application; disk locking, atomic installation and binary/large archives remain unfinished."}))
}
