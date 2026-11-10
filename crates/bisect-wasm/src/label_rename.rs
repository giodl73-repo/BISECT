use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use sha2::{Digest, Sha256};

pub(crate) fn rename(registry_json:String, files_base64:BTreeMap<String,String>, directories:Vec<String>, src:String, dst:String, force:bool)->Result<Value,String>{
    if registry_json.len()>1024*1024 || files_base64.len()>512 || directories.len()>512 || src.len()>100 || dst.len()>100 || files_base64.values().map(String::len).sum::<usize>()>24*1024*1024 {
        return Err("Label rename supports a 1 MiB registry, 512 files/directories, 100-byte labels and 16 MiB file bytes.".into());
    }
    let safe=|p:&str| !p.is_empty() && p.len()<=240 && p.bytes().all(|b|b.is_ascii_alphanumeric()||b"_./ -".contains(&b)) && !p.split('/').any(|s|s.is_empty()||[".","..","__proto__","constructor","prototype"].contains(&s));
    if [src.as_str(),dst.as_str()].iter().any(|s|["__proto__","constructor","prototype"].contains(s)){return Err("Reserved browser label name.".into());}
    let registry=bisect_report::label_registry::parse(&registry_json)?;
    if registry.len()>500 || registry.iter().any(|(n,e)|n.len()>200||e.built.len()>100||e.analyzed.len()>100||e.reported.len()>100){return Err("Registry exceeds browser label/stage limits.".into());}
    let mut decoded=BTreeMap::new(); let mut folded=BTreeSet::new(); let mut total=0;
    for (path,text) in files_base64 {
        if !safe(&path)||!folded.insert(path.to_ascii_lowercase())||path==".bisect"||path==".bisect.tmp" {return Err("Unsafe, duplicate or reserved package path.".into());}
        let bytes=STANDARD.decode(text).map_err(|_|"Invalid file base64.")?;
        total+=bytes.len(); if total>16*1024*1024{return Err("Package exceeds 16 MiB.".into());}
        decoded.insert(path,bytes);
    }
    let mut dir_folded=BTreeSet::new();
    for path in &directories {
        let bare=path.strip_suffix('/').ok_or("Directory paths must end in '/'.")?;
        if !safe(bare)||!dir_folded.insert(bare.to_ascii_lowercase())||folded.contains(&bare.to_ascii_lowercase()){return Err("Unsafe, duplicate or file-colliding directory.".into());}
    }
    for path in decoded.keys().chain(directories.iter()) {
        let pieces:Vec<_>=path.trim_end_matches('/').split('/').collect();
        for i in 1..pieces.len(){if folded.contains(&pieces[..i].join("/").to_ascii_lowercase()){return Err("Package path is both a file and directory.".into());}}
    }
    // Case-insensitive platforms cannot safely distinguish source/destination labels.
    if src!=dst && src.eq_ignore_ascii_case(&dst){return Err("Case-only renames are not portable across filesystems.".into());}
    let (updated,mut files,dirs)=bisect_report::label_rename::rename_package(&registry,&decoded,&directories,&src,&dst,force)?;
    let registry_text=serde_json::to_string_pretty(&updated).map_err(|e|e.to_string())?;
    files.insert(".bisect".into(),registry_text.as_bytes().to_vec());
    let mut output_folded=BTreeSet::new();
    if files.values().map(Vec::len).sum::<usize>()>18*1024*1024{return Err("Renamed package exceeds the 18 MiB output limit.".into());}
    for path in files.keys(){if path.len()>240||!output_folded.insert(path.to_ascii_lowercase()){return Err("Renamed paths exceed limits or collide on case-insensitive filesystems.".into());}}
    for dir in &dirs {if dir.len()>241||output_folded.contains(&dir.trim_end_matches('/').to_ascii_lowercase()){return Err("Renamed directory collides with a file.".into());}}
    let hashes:Vec<_>=files.iter().map(|(p,b)|json!({"path":p,"sha256":format!("{:x}",Sha256::digest(b)),"bytes":b.len()})).collect();
    // Keep unchanged binary bytes only in retained inputs, avoiding double-sized projects.
    let output:Vec<_>=files.iter().map(|(p,b)|{
        let mut source_path=p.clone();
        for stage in ["runs","analysis","reports"] {
            let prefix=format!("{stage}/{dst}/");
            if let Some(rest)=p.strip_prefix(&prefix){let candidate=format!("{stage}/{src}/{rest}");if decoded.contains_key(&candidate){source_path=candidate;}}
        }
        if decoded.get(&source_path).is_some_and(|source|source==b){json!({"path":p,"source_path":source_path})}
        else {json!({"path":p,"base64":STANDARD.encode(b)})}
    }).collect();
    Ok(json!({"schema_version":"bisect-label-rename-v1","status":"packaged","source_label":src,"destination_label":dst,"force":force,"registry_json":registry_text,"files":output,"directories":dirs,"output_hashes":hashes,"scope":"Creates a new selected repository package with native label move and top-level index patch semantics. No local files are overwritten. Force replaces destination stages only where a source stage exists. Year indexes, config references and recorded cross-index hashes are not rewritten; run SHA verification explicitly afterward. The registry lists declarations. This does not execute, certify or authenticate a plan. Unselected files and empty directories cannot be discovered. Native partial filesystem failure and lock semantics are not reproduced by an immutable download."}))
}
