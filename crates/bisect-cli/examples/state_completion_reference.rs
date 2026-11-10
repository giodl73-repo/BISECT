use serde::Deserialize;
use std::{collections::HashSet,path::Component};
#[derive(Deserialize)]struct Input{year:String,states:Vec<String>,paths:Vec<String>,reprocess:bool}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let input:Input=serde_json::from_str(&std::fs::read_to_string(std::env::args().nth(1).ok_or("Select oracle input")?)?)?;
 let root=tempfile::tempdir()?;
 for relative in &input.paths{
  let path=std::path::Path::new(relative);
  if !path.components().all(|p|matches!(p,Component::Normal(_))){return Err("Oracle paths must stay within its temporary output root".into());}
  let file=root.path().join(path);std::fs::create_dir_all(file.parent().ok_or("No parent")?)?;std::fs::write(file,b"")?;
 }
 let filters:HashSet<_>=input.states.iter().map(|s|s.to_uppercase()).collect();
 let configs:Vec<_>=bisect_cli::runner::load_all_states(&input.year)?.into_iter().filter(|r|filters.is_empty()||filters.contains(&r.0)).map(|(code,name,districts)|{let mut c=bisect_cli::runner::StateConfig::new_bulk(code,name,districts,input.year.clone(),"test".into(),root.path().to_path_buf(),0);c.reprocess=input.reprocess;c}).collect();
 let queued:Vec<_>=bisect_cli::runner::filter_incomplete(configs).into_iter().map(|c|(c.state_code,c.state_name,c.num_districts)).collect();
 print!("{}",serde_json::to_string(&queued)?);Ok(())
}
