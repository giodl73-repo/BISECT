//! Controlled local-provider reference for the real native Fetch download branch.
use clap::ValueEnum;
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Input { manifest_json:String, state:String, year:String, data_type:String, source_url:String, owned_root:String, force:bool }
fn collect(dir:&Path, files:&mut std::collections::BTreeMap<String,Vec<u8>>)->Result<(),String>{
    for entry in std::fs::read_dir(dir).map_err(|e|e.to_string())? {
        let path=entry.map_err(|e|e.to_string())?.path();
        if path.is_dir(){collect(&path,files)?;}else{
            files.insert(path.to_string_lossy().replace('\\',"/"),std::fs::read(&path).map_err(|e|e.to_string())?);
        }
    }
    Ok(())
}
fn main()->Result<(),String>{
    let input:Input=serde_json::from_str(&std::fs::read_to_string(std::env::args().nth(1).ok_or("Expected request path")?).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    if !input.source_url.starts_with("http://127.0.0.1:"){return Err("Reference accepts an owned loopback provider only".into());}
    let owned=PathBuf::from(&input.owned_root);
    if !owned.is_relative()||!owned.starts_with("target/fetch-archive-acquisition")||owned.components().any(|c|!matches!(c,std::path::Component::Normal(_))){return Err("Choose an owned target/fetch-archive-acquisition root".into());}
    let manifest:bisect_cli::fetch::Manifest=serde_json::from_str(&input.manifest_json).map_err(|e|e.to_string())?;
    let data_type=bisect_cli::args::DataType::from_str(&input.data_type,false)?;
    let mut items=bisect_cli::fetch::build_fetch_list(&manifest,&[input.state],&input.year,&[data_type]);
    if items.len()!=1{return Err("Reference requires one selected archive item".into());}
    let mut item=items.remove(0);
    for path in [&item.local_path,&item.done_marker] {
        if !path.starts_with(&owned)||path.components().any(|c|!matches!(c,std::path::Component::Normal(_))){return Err("Native target escaped owned root".into());}
    }
    let original_url=item.url.clone();item.url=Some(input.source_url);
    bisect_cli::fetch::download_items(&[item.clone()],input.force,false,&manifest,0)?;
    let mut files=std::collections::BTreeMap::new();collect(item.local_path.parent().ok_or("Missing target parent")?,&mut files)?;
    println!("{}",json!({"files_bytes":files,"target_present":item.local_path.exists(),"is_done":item.is_done(),"original_planned_url":original_url,"transport_override":"owned loopback provider supplies fixed retained bytes; native downloader/cache/extractor/marker are unmodified"}));Ok(())
}
