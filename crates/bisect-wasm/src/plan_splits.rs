//! Native subdivision split analysis of an exact plan and optional local lookups.
use std::collections::{HashMap,HashSet};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Default,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Geography {
 #[serde(default)] county_names:HashMap<String,String>,
 #[serde(default)] place_to_tracts:HashMap<String,Vec<String>>,
 #[serde(default)] place_names:HashMap<String,String>,
}
pub(crate) fn analyze(plan_text:&str,geography_text:Option<&str>,context_text:Option<&str>)->Result<Value,String>{
 if plan_text.len().saturating_add(geography_text.map(str::len).unwrap_or(0))>3*1024*1024{return Err("Plan and geography lookup must total at most 3 MiB.".into());}
 let plan=crate::plan_audit_text::plan(plan_text)?;
 rplan_audit::measurements::measure_plan(&plan,None)?;
 if plan.units.unit_universe_hash!=plan.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Split analysis plan unit-universe hash mismatch.".into());}
 let custom=matches!(plan.units.unit_kind,rplan_core::UnitKind::Precinct|rplan_core::UnitKind::Imported);
 let mut explicit_municipalities=None;
 let explicit_counties=if custom {
  let context=crate::plan_audit_text::context(context_text)?.ok_or("Custom-unit county splits require a matching RCTX with complete explicit county identities; missing is not zero splits.")?;
  rplan_audit::measurements::measure_plan(&plan,Some(&context))?;
  let counties=context.subdivisions.as_ref().and_then(|s|s.county_ids.as_ref()).ok_or("Custom-unit RCTX has no explicit county identities; splits were not evaluated.")?;
  explicit_municipalities=context.subdivisions.as_ref().and_then(|s|s.municipal_ids.as_deref())
   .map(|ids|bisect_analysis::splits::ExplicitMunicipalIdentities::from_ordered_units(&context.units.unit_ids,ids))
   .transpose().map_err(|e|e.to_string())?;
  Some(bisect_analysis::splits::explicit_counties_from_ordered_units(&plan.units.unit_ids,counties).map_err(|e|e.to_string())?)
 }else{None};
 if !custom&&plan.units.unit_ids.iter().any(|id|id.len()<5||!id.bytes().all(|b|b.is_ascii_digit())){return Err("County splits require numeric Census unit IDs with a five-digit county FIPS prefix.".into());}
 let geography:Geography=geography_text.map(crate::certified_exact::parse).transpose()?.unwrap_or_default();
 if geography.place_to_tracts.len()>10000||geography.county_names.len()>10000||geography.place_names.len()>10000{return Err("Geography lookup supports at most 10,000 entities per map.".into());}
 for (id,name) in geography.county_names.iter().chain(geography.place_names.iter()){if id.is_empty()||id.len()>200||name.is_empty()||name.len()>500{return Err("Geography names require bounded nonempty IDs and names.".into());}}
 let assignments:HashMap<_,_>=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect();
 let mut links=0usize;let mut unmatched=0usize;
 for (id,tracts) in &geography.place_to_tracts {
  if id.is_empty()||id.len()>200||tracts.is_empty(){return Err("Municipality IDs must be nonempty and at most 200 bytes, with at least one unit.".into());}
  let mut seen=HashSet::new();
  for tract in tracts {links+=1;if links>100000||tract.is_empty()||tract.len()>200||!seen.insert(tract){return Err("Municipal lookups require unique bounded unit IDs per place and at most 100,000 links.".into());}if !assignments.contains_key(tract){unmatched+=1;}}
 }
 let state=plan.units.state.as_deref().ok_or("Plan state is required.")?;let year=plan.units.year.ok_or("Plan year is required.")?.to_string();
 let mut report=if let Some(counties)=&explicit_counties {bisect_analysis::splits::assemble_explicit_splits_report(&assignments,counties,state,&year,optional_names(&geography.county_names),&geography.place_to_tracts,&geography.place_names)}else{bisect_analysis::splits::assemble_splits_report(&assignments,state,&year,optional_names(&geography.county_names),&geography.place_to_tracts,&geography.place_names)}.map_err(|e|e.to_string())?;
 if let Some(municipalities)=&explicit_municipalities {
  if !geography.place_to_tracts.is_empty(){return Err("Select a single municipal membership source: the matching RCTX already supplies municipal identities.".into());}
  municipalities.attach(&mut report,&assignments,&geography.place_names).map_err(|e|e.to_string())?;
  links=municipalities.known_units();
 }
 let report_json=serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?;
 let mut result=json!({"schema_version":"bisect-plan-splits-v1","report":report,"report_sha256":format!("{:x}",Sha256::digest(report_json.as_bytes())),"report_json":report_json,"plan_source_sha256":format!("{:x}",Sha256::digest(plan_text.as_bytes())),"geography_source_sha256":geography_text.map(|s|format!("{:x}",Sha256::digest(s.as_bytes()))),"coverage":{"municipal_links":links,"matched_links":links-unmatched,"unmatched_links":unmatched},"scope":"Shared native CLI county and municipal splits report. County identity uses the first five digits of supplied Census unit IDs. Municipality membership and names are supplied claims; unmatched links are excluded from district membership, but native totals still count every supplied municipality. Overlapping municipalities are evaluated separately. Missing municipality lookup remains unavailable. Bundled state policy descriptions are recorded native context, not a current legal determination. No source authentication, geographic intersection, population weighting, contiguity or legal certification is performed."});
 if custom {result["schema_version"]=json!("bisect-explicit-plan-splits-v1");result["context_source_sha256"]=json!(format!("{:x}",Sha256::digest(context_text.unwrap().as_bytes())));result["county_identity_source"]=json!("matching-rctx-explicit-counties");result["scope"]=json!("Shared native operational county and municipal split counts for custom units. County identities come only from complete, ordered, matching RCTX subdivisions; no Census prefix, FIPS name or legal assessment is inferred. Optional names/membership are supplied claims. Missing identities refuse analysis rather than becoming zero splits. No source authenticity, geographic intersection, legal preservation finding or certification is asserted.");}
 if explicit_municipalities.is_some(){
  result["schema_version"]=json!("bisect-explicit-plan-splits-v2");
  result["municipal_identity_source"]=json!("matching-rctx-explicit-municipalities");
  result["scope"]=json!("Shared native operational subdivision counts for custom units. County and municipal identities are supplied by matching RCTX. Null municipal identity is unknown; incomplete municipal coverage leaves municipal counts unavailable. Names are supplied claims. No source authentication or legal preservation finding is asserted.");
 }
 Ok(result)
}
fn optional_names(names:&HashMap<String,String>)->Option<&HashMap<String,String>>{if names.is_empty(){None}else{Some(names)}}
