//! Match explicitly selected bytes to recorded source hashes, never native paths.
use base64::{engine::general_purpose::STANDARD, Engine};
use bisect_runner::native_replay_recipe::{graph_digest, ReadOnlyRecipe as Recipe};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source { pub name:String, pub base64:String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub prechecks:crate::manifest_prechecks::Input,
    pub sources:Vec<Source>,
}
fn companion_text(bytes:Option<&[u8]>)->Result<Option<&str>,String>{
    bytes.map(std::str::from_utf8).transpose().map_err(|_|"Native graph companions require UTF-8".to_owned())
}

pub(crate) struct CheckedSources {pub recipe:Recipe,pub graph:bisect_runner::adjacency_loader::LoadedGraph,pub fine_graph:Option<bisect_runner::adjacency_loader::LoadedGraph>,pub areas:Vec<f64>,pub perimeters:Vec<f64>,pub selected_by_hash:BTreeMap<String,Vec<u8>>}
pub(crate) fn inspect(input:Input)->Result<Value,String>{inspect_checked(input).map(|(receipt,_)|receipt)}
pub(crate) fn inspect_checked(input:Input)->Result<(Value,Option<CheckedSources>),String>{
    if input.sources.len()>64{return Err("Select at most 64 recorded source files.".into());}
    let binding=crate::native_recipe_binding::inspect(input.prechecks)?;
    let mut sources=Value::Null;let mut checked_sources=None;
    let mut error=binding["error"].clone();
    let mut status="binding-failed";
    if binding["status"]=="binding-checked" {
        let checked=(||->Result<(Value,CheckedSources),String>{
            let recipe:Recipe=serde_json::from_str(binding["binding"]["recipe_json"].as_str().ok_or("Missing checked recipe text")?).map_err(|e|e.to_string())?;
            recipe.validate()?;
            let mut names=BTreeSet::new();let mut total=0usize;
            let mut selected=BTreeMap::<String,Vec<u8>>::new();let mut receipts=Vec::new();
            let expected:BTreeSet<_>=recipe.sources.iter().filter_map(|s|s.sha256.as_deref()).collect();
            for source in input.sources {
                if !crate::package_verification::safe(&source.name)||!names.insert(source.name.to_ascii_lowercase()){
                    return Err("Unsafe or case-colliding selected source name.".into());
                }
                if source.base64.len()>(64usize*1024*1024).div_ceil(3)*4{return Err("Selected source exceeds 64 MiB.".into());}
                let bytes=STANDARD.decode(&source.base64).map_err(|_|"Invalid selected source base64")?;
                total=total.checked_add(bytes.len()).ok_or("Selected source byte count overflow")?;
                if total>64*1024*1024||STANDARD.encode(&bytes)!=source.base64{return Err("Select canonical source base64 totaling at most 64 MiB.".into());}
                let digest=format!("sha256:{:x}",Sha256::digest(&bytes));
                if !expected.contains(digest.as_str()){return Err(format!("Selected source does not match a recorded source hash: {}",source.name));}
                let roles:Vec<_>=recipe.sources.iter().filter(|s|s.sha256.as_deref()==Some(digest.as_str())).map(|s|s.role.clone()).collect();
                receipts.push(json!({"name":source.name,"sha256":digest,"bytes_decimal":bytes.len().to_string(),"matched_roles":roles}));
                selected.insert(digest,bytes);
            }
            for source in &recipe.sources {
                if source.sha256.as_ref().is_some_and(|h|!selected.contains_key(h)){return Err(format!("Recorded source bytes not selected: {}",source.role));}
            }
            let role=|role:&str|->Result<Option<&[u8]>,String>{
                let source=recipe.sources.iter().find(|s|s.role==role).ok_or_else(||format!("Recipe does not record required source role: {role}"))?;
                Ok(source.sha256.as_ref().and_then(|h|selected.get(h)).map(Vec::as_slice))
            };
            let binary=role("adjacency-binary")?.ok_or("Recipe must record an existing native binary graph.")?;
            let graph=crate::native_adjacency::load_selected_graph(binary,companion_text(role("geoid-companion")?)?,companion_text(role("centroid-companion")?)?)?;
            let digest=graph_digest(&graph)?;
            if digest!=recipe.source_graph_sha256{return Err("Selected native source graph differs from the recorded graph.".into());}
            let fine_graph=if bisect_runner::native_replay_recipe::state_uses_multiscale(&recipe.state)&&matches!(recipe.state.multiscale_fine.as_str(),"bg"|"block_group"){
                // These labels never become browser IO paths. Every byte lookup
                // remains by a recorded hash in the explicitly selected set.
                let source=|name:&str|recipe.sources.iter().find(|r|r.role==name).ok_or_else(||format!("Missing recorded multiscale source identity: {name}"));
                let requested=source("multiscale-fine-request")?.path.replace('\\',"/");
                let (binary_path,stem)=if let Some(stem)=requested.strip_suffix(".adj.bin"){(requested.clone(),stem.to_owned())}else{
                    let slash=requested.rfind('/').map_or(0,|i|i+1);let cut=requested.rfind('.').filter(|&i|i>=slash).unwrap_or(requested.len());let stem=requested[..cut].to_owned();(format!("{stem}.adj.bin"),stem)
                };
                for(name,path,required)in[("multiscale-fine-binary",binary_path,true),("multiscale-fine-geoid-companion",format!("{stem}_geoids.json"),true),("multiscale-fine-centroid-companion",format!("{stem}_centroids.json"),false)]{
                    let item=source(name)?;if item.path.replace('\\',"/")!=path||required&&item.sha256.is_none(){return Err(format!("Multiscale source linkage or required bytes differ: {name}"));}
                }
                let binary=role("multiscale-fine-binary")?.ok_or("Select the recorded native fine binary graph")?;
                Some(crate::native_adjacency::load_selected_graph(binary,companion_text(role("multiscale-fine-geoid-companion")?)?,companion_text(role("multiscale-fine-centroid-companion")?)?)?)
            }else{None};
            let absent:Vec<_>=recipe.sources.iter().filter(|s|s.sha256.is_none()).map(|s|s.role.clone()).collect();
            let geometry=bisect_data::deserialize_adjacency(binary).map_err(|e|e.to_string())?;
            let algorithm_geometry=crate::native_tiger::prepare(&recipe,&graph,&selected)?;
            let mut receipt=json!({"selected_files":receipts,"source_graph_sha256":digest,"vertices_decimal":graph.n_vertices.to_string(),
                "edges_decimal":graph.n_edges.to_string(),"recorded_absent_roles":absent,
                "external_absence_verified":false,"upstream_authenticity_verified":false});
            if let Some(f)=&fine_graph{receipt["multiscale_fine_graph"]=json!({"vertices_decimal":f.n_vertices.to_string(),"edges_decimal":f.n_edges.to_string(),"source_graph_sha256":graph_digest(f)?});}
            let (areas,perimeters)=if let Some(g)=algorithm_geometry{receipt["algorithm_geometry"]=g.evidence;(g.areas,g.perimeters)}else{(geometry.vertex_areas,geometry.vertex_ext_perimeters)};
            Ok((receipt,CheckedSources{recipe,graph,fine_graph,areas,perimeters,selected_by_hash:selected}))
        })();
        match checked {Ok((value,checked))=>{checked_sources=Some(checked);status="sources-and-graph-checked";sources=value;error=Value::Null;},Err(message)=>{status="source-binding-failed";error=Value::String(message);}}
    } else if binding["status"]=="not-recorded" {status="not-recorded";}
    Ok((json!({"schema_version":"bisect-native-recipe-source-binding-v1","status":status,"recipe_binding":binding,"sources":sources,"error":error,
        "execution_performed":false,"recorded_paths_opened":false,
        "scope":"Explicit selected-byte inspection: checks recipe/manifest/context bindings first, matches every recorded existing source hash to selected bytes, and compares the shared native reader's full graph digest. Source names are display labels; recorded native paths grant no filesystem or network authority. Recorded absent files remain historical claims, not verified absence on disk. Local byte/graph consistency does not authenticate a source or producer, cover unrecorded implicit inputs, establish backend availability or execute a replay."}),checked_sources))
}
