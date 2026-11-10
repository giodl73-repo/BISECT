//! Native political and proportionality analyzers on explicit complete election sources.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeSet,BTreeMap};
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {pub bootstrap_seed:String,pub bootstrap_replicates:usize}
impl Default for Settings{fn default()->Self{Self{bootstrap_seed:"42".into(),bootstrap_replicates:1000}}}
pub(crate) fn analyze(plan_text:&str,csv_text:&str,settings:Option<Settings>)->Result<Value,String>{
 let settings=settings.unwrap_or_default();let seed=settings.bootstrap_seed.parse::<u64>().map_err(|_|"Supply an unsigned 64-bit bootstrap seed.")?;
 if seed.to_string()!=settings.bootstrap_seed{return Err("Bootstrap seed must use canonical unsigned decimal notation.".into());}
 if !(1..=5000).contains(&settings.bootstrap_replicates){return Err("Bootstrap replicates must be 1–5,000.".into());}
 if plan_text.len().saturating_add(csv_text.len())>3*1024*1024{return Err("Plan and election CSV must total at most 3 MiB.".into());}
 let plan=crate::plan_audit_text::plan(plan_text)?;
 if plan.units.unit_universe_hash!=plan.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Election analysis plan unit-universe hash mismatch.".into());}
 rplan_audit::measurements::measure_plan(&plan,None)?;
 let assignments=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect::<std::collections::HashMap<_,_>>();
 let mut reader=csv::Reader::from_reader(csv_text.as_bytes());let headers=reader.headers().map_err(|e|e.to_string())?.clone();
 let mut columns=BTreeSet::new();for header in &headers{if !columns.insert(header){return Err("Election CSV has duplicate headers.".into());}}
 for header in ["geoid","dem_votes","rep_votes"]{if !columns.contains(header){return Err(format!("Election CSV requires {header}."));}}
 let id_column=headers.iter().position(|h|h=="geoid").unwrap();let mut seen=BTreeSet::new();let mut matched=0usize;
 for row in reader.records(){let row=row.map_err(|e|e.to_string())?;let id=row.get(id_column).unwrap_or("");if id.is_empty()||id.len()>200||!seen.insert(id.to_string()){return Err("Election CSV has an empty, oversized or duplicate unit ID.".into());}if seen.len()>100_000{return Err("Election analysis supports at most 100,000 CSV rows.".into());}if assignments.contains_key(id){matched+=1;}}
 if matched!=assignments.len(){return Err(format!("Election CSV covers {matched} of {} plan units. Supply every plan unit explicitly, including zero-vote rows.",assignments.len()));}
 let political=bisect_analysis::political::analyze_political_csv(csv_text.as_bytes(),&assignments,plan.k).map_err(|e|e.to_string())?;
 let proportionality=bisect_analysis::proportionality::analyze_proportionality_csv(csv_text.as_bytes(),&assignments,plan.k).map_err(|e|e.to_string())?;
 if plan.k.saturating_mul(settings.bootstrap_replicates)>1_000_000{return Err("Partisan bootstrap exceeds the browser work limit.".into());}
 let district_votes=political.districts.iter().map(|d|bisect_analysis::DistrictElection{district:d.district,dem_votes:d.dem_votes,rep_votes:d.rep_votes}).collect::<Vec<_>>();
 let partisan=bisect_analysis::try_compute_partisan_metrics(&district_votes,Some(seed),settings.bootstrap_replicates).map_err(|e|e.to_string())?;
 let commission=bisect_analysis::partisan_report::assemble_partisan_report(&district_votes,&partisan,"supplied_two_party_votes".into(),"supplied-election.csv".into(),Some("Supplied two-party votes are historical measurements, not election predictions. Election office, vintage and source authenticity are not established by this CSV. Native report district margins divide by max(total votes, 1); fractional totals below one use that native convention. Bootstrap intervals resample districts independently; they do not model election uncertainty or spatial correlation.".into()));
 let files=BTreeMap::from([("partisan.json",serde_json::to_string_pretty(&commission).map_err(|e|e.to_string())?),("partisan-metrics.json",serde_json::to_string_pretty(&partisan).map_err(|e|e.to_string())?),("political.json",serde_json::to_string_pretty(&political).map_err(|e|e.to_string())?),("proportionality.json",serde_json::to_string_pretty(&proportionality).map_err(|e|e.to_string())?)]);
 let hashes:BTreeMap<_,_>=files.iter().map(|(n,s)|(*n,format!("{:x}",Sha256::digest(s.as_bytes())))).collect();
 Ok(json!({"schema_version":"bisect-plan-election-analysis-v1","political":political,"proportionality":proportionality,"partisan":partisan,"files":files,"file_sha256":hashes,
 "plan_hash":plan.plan_hash().map_err(|e|e.to_string())?,"plan_source_sha256":format!("{:x}",Sha256::digest(plan_text.as_bytes())),"election_source_sha256":format!("{:x}",Sha256::digest(csv_text.as_bytes())),"coverage":{"matched_units":matched,"plan_units":assignments.len(),"source_rows":seen.len(),"unmatched_source_rows":seen.len()-matched},
 "scope":"Shared native political and proportionality analyzers. Election CSV geoid, dem_votes and rep_votes are joined to exact plan unit IDs. Every plan unit requires an explicit row, including zero votes; duplicate IDs/headers are rejected. Extra source units are counted and excluded from district/state totals. Votes use native finite nonnegative f64 values, including fractional allocations, not exact integer count arithmetic. Two-party totals exclude other parties. Native political/proportionality ties (including zero-total districts) count as Democratic-leaning; zero-party districts are flagged uncontested. These are supplied historical vote measurements, not election predictions, sampler convergence, source authentication, VRA findings or legal certification. Geographic connectivity and population balance are not evaluated. Native partisan metrics include efficiency gap, mean-median, partisan bias, declination and the 61-point uniform-swing seats-votes curve. Bootstrap intervals require at least ten districts and resample districts independently. The named ChaCha12/u64-index protocol changes legacy SmallRng intervals; it records an exact decimal seed and replicate count. Portable libm atan is used for declination. Partisan ties count as Republican wins, whereas political/proportionality ties count as Democratic-leaning; zero-vote districts have partisan share 0.5. Favoring directions at zero follow native conventions. The native commission partisan.json is assembled through the shared CLI report schema, alongside the unchanged full partisan-metrics.json. Report assembly retains supplied election labels and caveats; it does not certify partisan fairness or election provenance."}))
}
