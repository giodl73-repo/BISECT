//! Exercise the actual filesystem-backed native analyzer entry points for parity gates.
use bisect_analysis::{Analyzer,AnalyzerContext,PoliticalAnalyzer,ProportionalityAnalyzer};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let request=std::env::args().nth(1).ok_or("request path required")?;let root=std::env::args().nth(2).ok_or("fixture output root required")?;
 let value:serde_json::Value=serde_json::from_slice(&std::fs::read(request)?)?;
 let document=rplan_io::read_rplan_str(value["document_json"].as_str().ok_or("plan required")?)?;
 let csv=value["election_csv"].as_str().ok_or("CSV required")?;
 let root=std::path::PathBuf::from(root);std::fs::create_dir_all(root.join("2020/elections"))?;std::fs::write(root.join("2020/elections/presidential_by_tract.csv"),csv)?;
 let assignments=document.plan.units.unit_ids.iter().cloned().zip(document.plan.assignment.iter().map(|d|*d as usize+1)).collect();
 let context=AnalyzerContext{assignments:&assignments,state_name:"fixture",state_code:"ZZ",year:"2020",version:"v1",num_districts:document.plan.k,data_root:&root,output_root:&root,balance_tolerance:0.005};
 let political=PoliticalAnalyzer::run(&context)?;let proportionality=ProportionalityAnalyzer::run(&context)?;
 let seed=value["settings"]["bootstrap_seed"].as_str().unwrap_or("42").parse::<u64>()?;let replicates=value["settings"]["bootstrap_replicates"].as_u64().unwrap_or(1000) as usize;
 let districts=political.districts.iter().map(|d|bisect_analysis::DistrictElection{district:d.district,dem_votes:d.dem_votes,rep_votes:d.rep_votes}).collect::<Vec<_>>();
 let partisan=bisect_analysis::try_compute_partisan_metrics(&districts,Some(seed),replicates)?;std::fs::write(root.join("partisan-metrics.json"),serde_json::to_string_pretty(&partisan)?)?;
 let commission=bisect_analysis::partisan_report::assemble_partisan_report(&districts,&partisan,"supplied_two_party_votes".into(),"supplied-election.csv".into(),Some("Supplied two-party votes are historical measurements, not election predictions. Election office, vintage and source authenticity are not established by this CSV. Native report district margins divide by max(total votes, 1); fractional totals below one use that native convention. Bootstrap intervals resample districts independently; they do not model election uncertainty or spatial correlation.".into()));
 std::fs::write(root.join("partisan.json"),serde_json::to_string_pretty(&commission)?)?;
 std::fs::write(root.join("political.json"),serde_json::to_string_pretty(&political)?)?;std::fs::write(root.join("proportionality.json"),serde_json::to_string_pretty(&proportionality)?)?;
 println!("{}",serde_json::json!({"political":political,"proportionality":proportionality,"partisan":partisan}));Ok(())
}
