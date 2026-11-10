use serde::Deserialize;
use std::{collections::BTreeMap,path::Component};
#[derive(Deserialize)]struct Folder{paths:Vec<String>,manifests:BTreeMap<String,String>}
#[derive(Deserialize)]struct Input{version:String,year:String,label:String,force:bool,folder:Folder}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let input:Input=serde_json::from_str(&std::fs::read_to_string(std::env::args().nth(1).ok_or("Select target oracle input")?)?)?;let root=tempfile::tempdir()?;
 for segment in [&input.version,&input.year,&input.label]{if !std::path::Path::new(segment).components().all(|c|matches!(c,Component::Normal(_)))||segment.contains('/')||segment.contains('\\'){return Err("Oracle scope must stay within the temporary root".into());}}
 for relative in &input.folder.paths{let p=std::path::Path::new(relative);if !p.components().all(|c|matches!(c,Component::Normal(_))){return Err("Oracle paths must stay within the temporary root".into());}let target=root.path().join(&input.version).join(p);std::fs::create_dir_all(target.parent().ok_or("Missing parent")?)?;std::fs::write(target,input.folder.manifests.get(relative).map(String::as_bytes).unwrap_or(b""))?;}

 let target=root.path().join(&input.version).join(&input.year).join("plans").join(&input.label);
 let result=bisect_report::check_incomplete_plan(&target,&input.label).and_then(|_|bisect_report::check_plan_collision(&target,input.force));
 print!("{}",serde_json::to_string(&serde_json::json!({"ready":result.is_ok(),"error":result.err().map(|e|e.to_string())}))?);Ok(())
}
