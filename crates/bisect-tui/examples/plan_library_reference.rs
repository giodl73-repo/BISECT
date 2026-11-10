use bisect_tui::app::{App,SortColumn,SortDir};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().skip(1).collect();if args.len()!=4{return Err("base filter column direction".into());}
 let mut app=App::default();app.plans=bisect_tui::plans::discover_plans(&args[0],"v1","2020");app.filter=args[1].clone();app.sort=match args[2].as_str(){"label"=>SortColumn::Label,"splits"=>SortColumn::Splits,"pp"=>SortColumn::Pp,"deviation"=>SortColumn::Deviation,_=>return Err("invalid column".into())};app.sort_dir=match args[3].as_str(){"asc"=>SortDir::Asc,"desc"=>SortDir::Desc,_=>return Err("invalid direction".into())};
 let prefix=std::path::Path::new(&args[0]).join("v1").join("2020");
 let rows:Vec<_>=app.visible_plans().into_iter().map(|r|serde_json::json!({"label":r.label,"state_code":r.state_code,"state_name":r.state_name,"chamber":r.chamber,"year":r.year,"num_districts":r.num_districts.to_string(),"mean_pp":r.mean_pp,"max_deviation_pct":r.max_deviation_pct,"county_splits":r.county_splits.map(|n|n.to_string()),"all_contiguous":r.all_contiguous,"plan_dir":r.plan_dir.strip_prefix(&prefix).unwrap().to_string_lossy().replace('\\',"/")})).collect();
 println!("{}",serde_json::to_string(&rows)?);Ok(())
}
