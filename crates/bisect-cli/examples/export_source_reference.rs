fn main()->Result<(),Box<dyn std::error::Error>>{
 let args=std::env::args().collect::<Vec<_>>();
 let plan=bisect_cli::export_cmd::load_rplan_from_plan_dir(std::path::Path::new(args.get(1).ok_or("directory required")?),args.get(2).ok_or("label required")?)?;
 println!("{}",serde_json::to_string(&plan)?);Ok(())
}
