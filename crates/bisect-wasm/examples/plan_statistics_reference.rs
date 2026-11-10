//! Verify exact reports from the native filesystem-backed analysis entry points.
use bisect_analysis::{Analyzer,AnalyzerContext,DemographicAnalyzer,UrbanAnalyzer,PoliticalAnalyzer,SummaryAnalyzer};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let req=std::env::args().nth(1).ok_or("request path required")?;let root=std::path::PathBuf::from(std::env::args().nth(2).ok_or("fixture output root required")?);
 let value:serde_json::Value=serde_json::from_slice(&std::fs::read(req)?)?;let doc=rplan_io::read_rplan_str(value["document_json"].as_str().ok_or("plan required")?)?;
 for (field,file)in [("demographic_csv","2020/demographics/fixture_demographics_2020.csv"),("urban_csv","data/2020/places/fixture_places_2020.csv"),("election_csv","2020/elections/presidential_by_tract.csv")]{let path=root.join(file);if let Some(text)=value[field].as_str(){std::fs::create_dir_all(path.parent().unwrap())?;std::fs::write(path,text)?;}else if path.exists(){std::fs::remove_file(path)?;}}
 let assignments=doc.plan.units.unit_ids.iter().cloned().zip(doc.plan.assignment.iter().map(|d|*d as usize+1)).collect();
 let context=AnalyzerContext{assignments:&assignments,state_code:"ZZ",state_name:"fixture",year:"2020",version:"v1",num_districts:doc.plan.k,data_root:&root,output_root:&root,balance_tolerance:value["tolerance_percent"].as_f64().ok_or("tolerance required")?/100.0};
 std::fs::create_dir_all(&root)?;
 if value["demographic_csv"].is_string(){std::fs::write(root.join("demographic.json"),serde_json::to_string_pretty(&DemographicAnalyzer::run(&context)?)?)?;}
 if value["urban_csv"].is_string(){std::fs::write(root.join("urban.json"),serde_json::to_string_pretty(&UrbanAnalyzer::run(&context)?)?)?;}
 if value["election_csv"].is_string(){std::fs::write(root.join("political.json"),serde_json::to_string_pretty(&PoliticalAnalyzer::run(&context)?)?)?;}
 std::fs::write(root.join("summary.json"),serde_json::to_string_pretty(&SummaryAnalyzer::run(&context)?)?)?;Ok(())
}
