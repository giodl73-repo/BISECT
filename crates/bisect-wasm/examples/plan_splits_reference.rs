//! Independent native split kernels and historical CLI report field assembly.
use std::collections::HashMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let req:serde_json::Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("request required")?)?)?;
 let plan=rplan_io::read_rplan_str(req["document_json"].as_str().ok_or("plan required")?)?.plan;
 let geography:serde_json::Value=req["geography_json"].as_str().map(serde_json::from_str).transpose()?.unwrap_or(serde_json::json!({}));
 let names:HashMap<String,String>=serde_json::from_value(geography.get("county_names").cloned().unwrap_or(serde_json::json!({})))?;
 let places:HashMap<String,Vec<String>>=serde_json::from_value(geography.get("place_to_tracts").cloned().unwrap_or(serde_json::json!({})))?;
 let place_names:HashMap<String,String>=serde_json::from_value(geography.get("place_names").cloned().unwrap_or(serde_json::json!({})))?;
 let assignments=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect();
 let state=plan.units.state.as_deref().ok_or("state required")?;let year=plan.units.year.ok_or("year required")?.to_string();
 let c=bisect_analysis::splits::try_analyze_county_splits_with_state(&assignments,if names.is_empty(){None}else{Some(&names)},Some(state))?;
 let m=bisect_analysis::splits::try_analyze_municipal_splits(&assignments,&places,&place_names)?;
 let standard=bisect_analysis::split_standards::get_split_standard(state);
 let (term,plural)=standard.map(|s|(s.subdivision_term,s.subdivision_term_plural)).unwrap_or_else(||("county".into(),"counties".into()));
 let report=serde_json::json!({"analyzer":"splits","state":state,"year":year,"counties":{"subdivision_term":term,"subdivision_term_plural":plural,"total":c.total,"split":c.split,"preservation_score":c.preservation_score,"split_list":c.split_list,"legal_standard":c.legal_standard,"compliance_assessment":c.compliance_assessment,"disclaimer":c.disclaimer},"municipalities":{"available":m.available,"total":m.total,"split":m.split,"preservation_score":m.preservation_score,"split_list":m.split_list}});
 print!("{}",serde_json::to_string_pretty(&report)?);Ok(())
}
