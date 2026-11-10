//! Actual native fallback against the current directory's .bisect snapshot.
use bisect_cli::args::PlanArgs;
fn main()->Result<(),String>{
    let args:Vec<_>=std::env::args().collect();
    let request:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[1]).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let input=&request["input"];
    let flags=PlanArgs{label:input["label"].as_str().map(str::to_owned),configure:input["configure"].as_bool().ok_or("Missing configure flag")?,config:input["config_path"].as_str().map(std::path::PathBuf::from)};
    if args.get(2).is_some_and(|mode|mode=="dispatch") {bisect_cli::plan_cmd::run_plan(flags)?;}
    else {print!("{}",bisect_cli::plan_cmd::render_fallback(&flags));}
    Ok(())
}
