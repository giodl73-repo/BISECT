use std::collections::BTreeMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();let request:serde_json::Value=serde_json::from_slice(&std::fs::read(&args[1])?)?;
 let texts:BTreeMap<String,String>=serde_json::from_value(request["reports_json"].clone())?;
 let reports:Vec<_>=texts.values().map(|t|serde_json::from_str::<serde_json::Value>(t)).collect::<Result<_,_>>()?;
 let refs:Vec<_>=reports.iter().map(|r|(r["state"].as_str().unwrap(),r)).collect();let state_level=reports[0]["analyzer"]=="proportionality";
 let year=reports[0]["year"].as_str().map(str::to_owned).unwrap_or_else(||reports[0]["year"].to_string());
 use bisect_analysis::national::*;
 let merged=if state_level{merge_state_level_outputs(&year,&refs)}else{merge_analyzer_outputs(&refs)};
 std::fs::create_dir_all(&args[2])?;
 std::fs::write(format!("{}/national.json",args[2]),serde_json::to_string_pretty(&merged)?)?;
 std::fs::write(format!("{}/national.csv",args[2]),if state_level{state_records_to_csv(&merged)}else{districts_to_csv(&merged)})?;
 std::fs::write(format!("{}/records.json",args[2]),serde_json::to_string_pretty(&if state_level{state_records_to_json_array(&merged)}else{districts_to_json_array(&merged)})?)?;
 Ok(())
}
