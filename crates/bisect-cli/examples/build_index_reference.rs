use bisect_cli::{build_cmd::{BuildArgs,build_build_index,run_build},runner::{StateConfig,StateResult}};
use bisect_runner::build_index::{ConfigRow,ResultRow};
#[derive(serde::Deserialize)]
struct Input{label:String,year:String,yaml_text:String,config_path:String,command:String,output_dir:String,created:String,configs:Vec<ConfigRow>,results:Vec<ResultRow>}
fn main()->Result<(),String>{
 let args:Vec<String>=std::env::args().collect();
 if args[1]=="dry-force"{return run_build(BuildArgs{label:"a".into(),config:args[2].clone().into(),year:Some("2020".into()),states:vec![],workers:None,dry_run:true,force:true,no_interactive:true});}
 let input:Input=serde_json::from_slice(&std::fs::read(&args[1]).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
 let yaml=bisect_runner::config::AlgoYaml::from_text(&input.yaml_text)?;
 let configs:Vec<_>=input.configs.into_iter().map(|c|StateConfig::new_bulk(c.state_code,c.state_name,c.num_districts,input.year.clone(),input.label.clone(),input.output_dir.clone().into(),2)).collect();
 let results:Vec<_>=input.results.into_iter().map(|r|StateResult{state_code:r.state_code,success:r.success,error:r.error,elapsed_ms:0}).collect();
 let hash=bisect_runner::config::AlgoYaml::file_sha256(std::path::Path::new(&args[2]))?;
 let mut index=build_build_index(&input.label,&input.year,std::path::Path::new(&input.config_path),&hash,&input.command,std::path::Path::new(&input.output_dir),&yaml,&configs,&results)?;
 index.created=input.created;
 print!("{}",serde_json::to_string_pretty(&index).map_err(|e|e.to_string())?);
 Ok(())
}
