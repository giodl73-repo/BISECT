use serde::Deserialize;
#[derive(Deserialize)]struct Input{state:String,chamber:String}
fn main(){let file=std::env::args().nth(1).expect("request path");let input:Input=serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();let policy=serde_json::from_str(include_str!("../../../data/location_policy.json")).unwrap();let fraction=bisect_analysis::location_registry::chamber_balance_tolerance(&policy,&input.state,&input.chamber);println!("{}",serde_json::json!({"fraction":fraction,"percent":fraction*100.0}));}
