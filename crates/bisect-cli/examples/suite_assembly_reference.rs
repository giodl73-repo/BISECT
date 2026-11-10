use std::collections::HashMap;
fn main()->anyhow::Result<()>{
 let args:Vec<_>=std::env::args().skip(1).collect();if args.len()!=2{anyhow::bail!("Expected request JSON and output base, or --validate and suite directory.");}
 if args[0]=="--validate"{let result=bisect_cli::suite::run_suite_validate(std::path::Path::new(&args[1]),"portable")?;println!("{}",serde_json::to_string(&result)?);return Ok(());}
 let request:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[0])?)?;
 if let Some(policy)=request["policy_json"].as_str(){let path=std::path::Path::new(&args[1]).join("policy.json");std::fs::create_dir_all(&args[1])?;std::fs::write(&path,policy)?;std::env::set_var("BISECT_LOCATION_POLICY",path);}else{std::env::remove_var("BISECT_LOCATION_POLICY");}
 let read=|key:&str|->anyhow::Result<_>{Ok(rplan_io::read_rplan_str(request[key].as_str().ok_or_else(||anyhow::anyhow!("Missing {key}"))?)?.plan)};
 let house=read("house_json")?;let senate=read("senate_json")?;let congress=request["congressional_json"].as_str().map(rplan_io::read_rplan_str).transpose()?.map(|d|d.plan);
 let assignments=|p:&rplan_core::DistrictPlan|p.units.unit_ids.iter().cloned().zip(p.assignment.iter().map(|d|*d as usize+1)).collect::<HashMap<_,_>>();
 let h=assignments(&house);let s=assignments(&senate);let c=congress.as_ref().map(assignments);
 let name=request["suite_name"].as_str().unwrap();let year=house.units.year.ok_or_else(||anyhow::anyhow!("Missing year"))?.to_string();
 let draw=bisect_cli::suite::SuiteDrawArgs{state:house.units.state.clone().unwrap(),year:year.clone(),version:"v1".into(),name:name.into(),congressional_districts:None,house_districts:None,senate_districts:None,nest_mode:request["nest_mode"].as_str().unwrap().parse()?,nest_ratio:request["required_ratio"].as_u64().map(|n|n as usize),seed:None,output_base:args[1].clone(),force:false};
 bisect_cli::suite::run_suite_draw(&draw,&h,&s,c.as_ref())?;
 let path=std::path::Path::new(&args[1]).join("v1").join(year).join("suites").join(name).join("suite.json");print!("{}",std::fs::read_to_string(path)?);Ok(())
}
