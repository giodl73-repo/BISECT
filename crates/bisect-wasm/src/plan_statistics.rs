//! Shared native demographic, urban and summary analyzers on explicit bounded sources.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet,HashMap};
fn coverage(text:&str,id_header:&str,assignments:&HashMap<String,usize>,complete:bool)->Result<Value,String>{
 let mut reader=csv::Reader::from_reader(text.as_bytes());let headers=reader.headers().map_err(|e|e.to_string())?.clone();
 let mut columns=BTreeSet::new();for h in &headers{if !columns.insert(h){return Err("Analyzer CSV has duplicate headers.".into());}}
 let index=headers.iter().position(|h|h==id_header).ok_or_else(||format!("Analyzer CSV requires {id_header}."))?;
 let mut ids=BTreeSet::new();let mut matched=BTreeSet::new();let mut rows=0usize;let mut unmatched=0usize;
 for row in reader.records(){let row=row.map_err(|e|e.to_string())?;rows+=1;if rows>100_000{return Err("Analyzer CSV supports at most 100,000 rows.".into());}
  let id=row.get(index).unwrap_or("");if id.is_empty()||id.len()>200{return Err("Analyzer CSV has an empty or oversized unit ID.".into());}
  if !ids.insert(id.to_string())&&complete{return Err("Demographic/election CSV has duplicate unit IDs.".into());}
  if assignments.contains_key(id){matched.insert(id.to_string());}else{unmatched+=1;}
 }
 if complete&&matched.len()!=assignments.len(){return Err(format!("Analyzer CSV covers {} of {} plan units. Supply every unit explicitly.",matched.len(),assignments.len()));}
 Ok(json!({"source_rows":rows,"matched_units":matched.len(),"plan_units":assignments.len(),"unmatched_source_rows":unmatched,"complete_unit_coverage":matched.len()==assignments.len()}))
}
fn exact_counts(mut view:Value,fields:&[&str])->Value{if let Some(rows)=view["districts"].as_array_mut(){for row in rows{for field in fields{if row[*field].is_number(){row[*field]=Value::String(row[*field].to_string());}}}}view}
pub(crate) fn analyze(plan_text:&str,demo_text:Option<&str>,urban_text:Option<&str>,election_text:Option<&str>,tolerance_percent:f64)->Result<Value,String>{
 analyze_with_context(plan_text,demo_text,urban_text,election_text,tolerance_percent,None)
}
pub(crate) fn analyze_with_context(plan_text:&str,demo_text:Option<&str>,urban_text:Option<&str>,election_text:Option<&str>,tolerance_percent:f64,context_text:Option<&str>)->Result<Value,String>{
 if demo_text.is_none()&&urban_text.is_none()&&election_text.is_none()&&context_text.is_none(){return Err("Select at least one analyzer CSV source or a matching population context.".into());}
 if !tolerance_percent.is_finite()||!(0.0..=100.0).contains(&tolerance_percent){return Err("Population tolerance percent must be between 0 and 100.".into());}
 if [Some(plan_text),demo_text,urban_text,election_text].into_iter().flatten().map(str::len).sum::<usize>()>3*1024*1024{return Err("Plan and analyzer CSV sources must total at most 3 MiB.".into());}
 let plan=crate::plan_audit_text::plan(plan_text)?;if plan.units.unit_universe_hash!=plan.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Statistics plan unit-universe hash mismatch.".into());}rplan_audit::measurements::measure_plan(&plan,None)?;
 let measured_populations=if demo_text.is_none(){
  let ctx=crate::plan_audit_text::context(context_text)?;
  ctx.as_ref().map(|ctx|rplan_audit::measurements::measure_plan(&plan,Some(ctx))).transpose()?.and_then(|m|m.districts.into_iter().map(|d|d.population.map(|p|p.population)).collect::<Option<Vec<_>>>())
 }else{None};
 if demo_text.is_none()&&urban_text.is_none()&&election_text.is_none()&&measured_populations.is_none(){return Err("The matching context has no population counts and no analyzer CSV was supplied.".into());}
 let assignments=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect::<HashMap<_,_>>();
 let mut coverage_results=BTreeMap::new();let mut hashes=BTreeMap::new();
 for (name,text,header,complete) in [("demographic",demo_text,"GEOID",true),("urban",urban_text,"GEOID",false),("political",election_text,"geoid",true)]{if let Some(text)=text{coverage_results.insert(name,coverage(text,header,&assignments,complete)?);hashes.insert(name,format!("{:x}",Sha256::digest(text.as_bytes())));}}
 let demographic=demo_text.map(|s|bisect_analysis::demographic::analyze_demographic_csv(s.as_bytes(),&assignments,plan.k)).transpose().map_err(|e|e.to_string())?;
 let urban=urban_text.map(|s|bisect_analysis::urban::analyze_urban_csv(s.as_bytes(),&assignments,plan.k)).transpose().map_err(|e|e.to_string())?;
 let political=election_text.map(|s|bisect_analysis::political::analyze_political_csv(s.as_bytes(),&assignments,plan.k)).transpose().map_err(|e|e.to_string())?;
 let mut summary=bisect_analysis::summary::summarize_results(demographic.as_ref(),political.as_ref(),urban.as_ref(),plan.k,tolerance_percent/100.0).map_err(|e|e.to_string())?;
 if let Some(populations)=&measured_populations{summary=bisect_analysis::summary::with_population_totals(summary,populations,tolerance_percent/100.0).map_err(|e|e.to_string())?;hashes.insert("context",format!("{:x}",Sha256::digest(context_text.ok_or("Missing population context.")?.as_bytes())));}
 let mut files=BTreeMap::new();files.insert("summary.json",serde_json::to_string_pretty(&summary).map_err(|e|e.to_string())?);
 for (name,value)in [("demographic.json",demographic.as_ref().map(serde_json::to_string_pretty)),("urban.json",urban.as_ref().map(serde_json::to_string_pretty)),("political.json",political.as_ref().map(serde_json::to_string_pretty))]{if let Some(value)=value{files.insert(name,value.map_err(|e|e.to_string())?);}}
 let file_hashes:BTreeMap<_,_>=files.iter().map(|(n,s)|(*n,format!("{:x}",Sha256::digest(s.as_bytes())))).collect();
 Ok(json!({"schema_version":"bisect-plan-statistics-v1","demographic":demographic.map(|r|serde_json::to_value(r).map(|v|exact_counts(v,&["total_pop"]))).transpose().map_err(|e|e.to_string())?,
 "urban":urban.map(|r|serde_json::to_value(r).map(|v|exact_counts(v,&["largest_city_pop"]))).transpose().map_err(|e|e.to_string())?,"political":political,"summary":exact_counts(serde_json::to_value(summary).map_err(|e|e.to_string())?,&["total_pop","ideal_pop"]),
 "population_balance_evaluated":demo_text.is_some()||measured_populations.is_some(),"population_source":if demo_text.is_some(){Some("demographic-csv")}else if measured_populations.is_some(){Some("matching-rctx")}else{None},"tolerance_percent":tolerance_percent,"coverage":coverage_results,"source_sha256":hashes,"files":files,"file_sha256":file_hashes,"plan_hash":plan.plan_hash().map_err(|e|e.to_string())?,"plan_source_sha256":format!("{:x}",Sha256::digest(plan_text.as_bytes())),
 "scope":"Shared native demographic, urban, political and merged summary analyzers on supplied unit CSVs. Native integer counts are checked in Rust and displayed as decimal strings; exact native JSON files retain their integer representation. Demographic and election sources require every plan unit; urban rows may be sparse or repeat a unit for multiple supplied place rows. Extras are counted and excluded. Urban num_places counts matched rows, and largest_city selects the largest individual row (last tied row wins), not aggregated city population or dissolved geometry. Demographic percentages use total population, not VAP/CVAP; minority is one minus White share with an inclusive 0.50 majority threshold. Native zero-population districts report minority share 1 and a majority flag; these must not be interpreted as opportunity districts. Summary ideal population uses native integer division of statewide demographic totals, with absolute fractional deviation compared to the supplied tolerance. Without demographic CSV, a supplied matching RCTX population vector may provide measured district totals and the same native integer ideal/deviation; race/minority fields stay unknown. If neither population source exists, population measurements stay unknown; the native summary balance boolean alone is not evidence of an evaluated check. These measurements do not authenticate sources, establish VRA/legal compliance, check connectivity or certify optimal cuts."}))
}
