fn main()->Result<(),String>{
    let args:Vec<_>=std::env::args().collect();if args.len()!=2{return Err("Expected request path".into());}
    let value:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[1]).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let manifest:bisect_cli::fetch::Manifest=serde_json::from_str(value["manifest_json"].as_str().ok_or("Missing manifest")?).map_err(|e|e.to_string())?;
    let states:Vec<String>=serde_json::from_value(value["states"].clone()).map_err(|e|e.to_string())?;
    let names:Vec<bisect_core::fetch_plan::DataType>=serde_json::from_value(value["data_types"].clone()).map_err(|e|e.to_string())?;
    use bisect_core::fetch_plan::DataType as P;use bisect_cli::args::DataType as N;
    let types:Vec<_>=names.into_iter().map(|v|match v{P::Tiger=>N::Tiger,P::Redistricting=>N::Redistricting,P::Adjacency=>N::Adjacency,P::Enacted=>N::Enacted,P::Geography=>N::Geography,P::Elections=>N::Elections,P::Lodes=>N::Lodes,P::SchoolDistricts=>N::SchoolDistricts,P::Eia861=>N::Eia861,P::LodesOd=>N::LodesOd,P::AcsHousing=>N::AcsHousing,P::All=>N::All}).collect();
    let selected_year=value["year"].as_str().ok_or("Missing year")?;
    let years:Vec<&str>=if selected_year=="all"{vec!["2020","2010","2000"]}else{vec![selected_year]};
    let mut items=vec![];for year in years{items.extend(bisect_cli::fetch::build_fetch_list(&manifest,&states,year,&types));}
    let rows:Vec<_>=items.into_iter().map(|v|serde_json::json!({"state_code":v.state_code,"year":v.year,"kind":v.kind,"url":v.url,"local_path":v.local_path.to_string_lossy().replace('\\',"/"),"done_marker":v.done_marker.to_string_lossy().replace('\\',"/"),"available_locally":v.available_locally,"done":v.is_done()})).collect();
    println!("{}",serde_json::to_string(&rows).map_err(|e|e.to_string())?);Ok(())
}
