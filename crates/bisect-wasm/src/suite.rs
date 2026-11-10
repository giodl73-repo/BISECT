//! Native multi-chamber nesting validation on explicit bounded plan texts.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
pub(crate) fn assemble(house_text:&str,senate_text:&str,congressional_text:Option<&str>,name:&str,mode:&str,ratio:Option<usize>,policy_name:Option<&str>,policy_text:Option<&str>)->Result<Value,String>{
 use std::collections::{BTreeMap,BTreeSet,HashMap};
 use std::str::FromStr;
 if name.is_empty()||name.len()>100||matches!(name,"__proto__"|"constructor"|"prototype")||!name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||ratio.is_some_and(|r|!(1..=500).contains(&r)){return Err("Suite name must contain 1–100 ASCII letters, digits, underscores or hyphens and avoid reserved object names; optional nesting ratio must be 1–500.".into());}
 if house_text.len().saturating_add(senate_text.len()).saturating_add(congressional_text.map(str::len).unwrap_or(0))>3*1024*1024{return Err("All chamber plan sources must total at most 3 MiB.".into());}
 let nest_mode=bisect_analysis::suite_assembly::NestMode::from_str(mode).map_err(|e|e.to_string())?;
 let (policy,source_name,source_kind,source_hash)=crate::location_policy::database(policy_name,policy_text)?;
 let mut sources=vec![("house",house_text),("senate",senate_text)];if let Some(t)=congressional_text{sources.push(("congressional",t));}
 let mut maps=BTreeMap::new();let mut hashes=BTreeMap::new();let mut files=BTreeMap::new();let mut identity=None;
 for (chamber,text) in sources{
  let plan=crate::plan_audit_text::plan(text)?;
  if plan.units.unit_universe_hash!=plan.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Suite plan unit-universe hash mismatch.".into());}
  rplan_audit::measurements::measure_plan(&plan,None)?;
  let actual=(plan.units.state.clone(),plan.units.year,plan.units.unit_kind.clone(),plan.units.unit_ids.iter().cloned().collect::<BTreeSet<_>>());
  if let Some(ref first)=identity{if first!=&actual{return Err("Suite chamber plans must cover the same state, year, unit kind and unit IDs.".into());}}else{identity=Some(actual);}
  maps.insert(chamber,plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect::<HashMap<_,_>>());
  hashes.insert(chamber,format!("sha256:{:x}",Sha256::digest(text.as_bytes())));
  files.insert(format!("{name}_{chamber}.rplan"),text.to_owned());
 }
 let (state,year,_,_)=identity.ok_or("Missing chamber identity")?;
 let state=state.filter(|s|!s.is_empty()).ok_or("Chamber plans need an explicit unit state code")?;
 let year=year.ok_or("Chamber plans need an explicit unit census year")?;let mut messages=Vec::new();
 let suite=bisect_analysis::suite_assembly::prepare(&policy,name,&state,&year.to_string(),nest_mode,ratio,&maps["house"],&maps["senate"],congressional_text.is_some(),|message|messages.push(message)).map_err(|e|format!("{e}{}",if messages.is_empty(){String::new()}else{format!("\n{}",messages.join("\n"))}))?;
 let manifest=serde_json::to_string_pretty(&suite).map_err(|e|e.to_string())?;files.insert(format!("{name}/suite.json"),manifest.clone());
 let resolved_ratio=if mode=="senate-in-house"{Some(bisect_analysis::suite_assembly::resolve_nesting_ratio(&policy,&state,ratio,|_|{}).map_err(|e|e.to_string())?)}else{None};
 Ok(json!({"schema_version":"bisect-suite-assembly-v1","status":"assembled","suite":suite,"manifest_json":manifest,"files":files,"source_sha256":hashes,"messages":messages,"resolved_ratio":resolved_ratio,"policy_source_name":source_name,"policy_source_kind":source_kind,"policy_source_sha256":source_hash,
  "scope":"Native suite manifest assembly from supplied chamber plans, using the shared CLI nesting-mode and policy-ratio rules. House and Senate nesting is checked only in senate-in-house mode; congressional assignments are independent. Policies are supplied or embedded snapshots and their current legal accuracy is not established. Sources must have matching unit identity; complete source bytes are retained without conversion. ZIP layout puts suite.json in the suite-name folder and chamber RPLAN files in its parent, as the native suite validator expects. This operation does not draw plans or execute the engine, and it does not check population balance, geographic contiguity, source authenticity or full legal compliance. Imported sources/results are unverified until explicitly run."}))
}
pub(crate) fn stability(sources:&std::collections::BTreeMap<String,String>)->Result<Value,String>{
    if !(2..=32).contains(&sources.len()){return Err("Supply 2–32 uniquely named RPLAN files.".into());}
    if sources.iter().map(|(n,s)|n.len().saturating_add(s.len())).sum::<usize>()>3*1024*1024{return Err("Stability plan sources must total at most 3 MiB.".into());}
    let mut assignments=Vec::new();let mut first=None;let mut source_hashes=std::collections::BTreeMap::new();let mut plan_hashes=std::collections::BTreeMap::new();
    for (name,text) in sources {
        if name.is_empty()||name.len()>120||name.starts_with('.')||!name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-'||b==b'.'){return Err("Stability plans need unique plain basenames of at most 120 bytes.".into());}
        let plan=crate::plan_audit_text::plan(text)?;
        if plan.units.unit_universe_hash!=plan.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Stability plan unit-universe hash mismatch.".into());}
        rplan_audit::measurements::measure_plan(&plan,None)?;
        let ids=plan.units.unit_ids.iter().cloned().collect::<std::collections::BTreeSet<_>>();
        let identity=(plan.units.unit_kind.clone(),plan.units.state.clone(),plan.units.year,ids);
        if let Some(ref expected)=first {if expected!=&identity{return Err("Stability plans must cover the same unit IDs, unit kind, state and census year.".into());}}else{first=Some(identity);}
        let pairs=sources.len()*(sources.len()-1)/2;
        if pairs.saturating_mul(plan.units.unit_ids.len())>50_000_000{return Err("Stability comparison exceeds the browser work limit.".into());}
        let map=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect::<std::collections::HashMap<_,_>>();
        assignments.push((name.clone(),map));source_hashes.insert(name,format!("{:x}",Sha256::digest(text.as_bytes())));plan_hashes.insert(name,plan.plan_hash().map_err(|e|e.to_string())?);
    }
    let report=bisect_analysis::suite_stability::analyze_suite_stability(&assignments)?;
    let mut pairs=Vec::new();for i in 0..assignments.len(){for j in i+1..assignments.len(){let sim=bisect_analysis::suite_stability::compute_jaccard_similarity(&assignments[i].1,&assignments[j].1);pairs.push(json!({"plan_a":assignments[i].0,"plan_b":assignments[j].0,"similarity":sim,"passes_threshold":sim>=bisect_analysis::suite_stability::STABILITY_THRESHOLD}));}}
    let report_json=serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-suite-stability-output-v1","report":report,"report_json":report_json,"pairs":pairs,"source_sha256":source_hashes,"plan_hashes":plan_hashes,"threshold":bisect_analysis::suite_stability::STABILITY_THRESHOLD,
      "scope":"Shared native suite stability metric: matching exact district labels divided by maximum assignment-map size. The CLI calls this Jaccard; it is label-sensitive agreement, not label-independent partition similarity. Pure district renumbering changes the score. Browser inputs must cover identical unit IDs, unit kind, state and year; reordered units are joined by ID. Plans are compared in lexicographic filename order. Stable means every pair scores at least 0.95; it does not establish sampler convergence, valid plan populations/connectivity, source authenticity, an optimal cut or legal certification. No samples or multi-chamber plans were generated."}))
}
pub(crate) fn validate(house_text:&str,senate_text:&str,name:&str,required_ratio:Option<usize>)->Result<Value,String>{
    if house_text.len().saturating_add(senate_text.len())>3*1024*1024{return Err("Suite plan inputs must total at most 3 MiB.".into());}
    if name.trim().is_empty()||name.len()>200||required_ratio.is_some_and(|r|!(1..=500).contains(&r)){return Err("Supply a suite name of 1–200 bytes and optional ratio 1–500.".into());}
    let house=crate::plan_audit_text::plan(house_text)?;let senate=crate::plan_audit_text::plan(senate_text)?;
    for plan in [&house,&senate]{
        if plan.units.unit_universe_hash!=plan.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Suite plan unit-universe hash mismatch.".into());}
        rplan_audit::measurements::measure_plan(plan,None)?;
    }
    if house.units.unit_kind!=senate.units.unit_kind||house.units.state!=senate.units.state||house.units.year!=senate.units.year{return Err("Suite plans must use matching unit kind, state and census year.".into());}
    let map=|plan:&rplan_core::DistrictPlan|plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect::<std::collections::HashMap<_,_>>();
    let houses=map(&house);let senates=map(&senate);
    let ratio=required_ratio.unwrap_or_else(||houses.values().copied().max().unwrap_or(0)/senates.values().copied().max().unwrap_or(1).max(1));
    let report=bisect_analysis::nesting::validate_suite_assignments(name,&houses,&senates,required_ratio).map_err(|e|e.to_string())?;
    let report_json=serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-suite-nesting-output-v1","report":report,"report_json":report_json,"required_ratio":ratio,"ratio_explicit":required_ratio.is_some(),
        "house_plan_hash":house.plan_hash().map_err(|e|e.to_string())?,"senate_plan_hash":senate.plan_hash().map_err(|e|e.to_string())?,
        "house_source_sha256":format!("{:x}",Sha256::digest(house_text.as_bytes())),"senate_source_sha256":format!("{:x}",Sha256::digest(senate_text.as_bytes())),
        "scope":"Shared native multi-chamber suite nesting check. Each Senate district must contain the required number of whole House districts; House districts cannot span Senate districts. Plans must cover the same explicit unit IDs, state, census year and unit kind; differing unit orders are joined by ID. District IDs are converted to native one-based IDs. Without an explicit ratio, the native CLI infers it from maximum used district labels. Constitutional ratios, population balance, geographic connectivity, source authenticity and other legal requirements are not checked here. A nesting pass is not full suite certification. No plan drawing or stability analysis has run."}))
}
