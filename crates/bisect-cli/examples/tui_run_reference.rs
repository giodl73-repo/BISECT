fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().skip(1).collect();let input:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[0])?)?;
 let form:bisect_report::tui_run::Form=serde_json::from_value(input["form"].clone())?;
 let seed=if form.seed.is_empty(){None}else{Some(form.seed.parse::<u64>()?)};
 let tolerance=if form.balance_tol.is_empty(){None}else{Some(form.balance_tol.parse::<f64>()?)};
 let yaml=bisect_runner::config::run_form_yaml(input["yaml_text"].as_str().ok_or("yaml_text")?,seed,tolerance)?;
 println!("{}",serde_json::json!({"arguments":bisect_report::tui_run::state_arguments(&form),"effective_yaml":yaml}));Ok(())
}
