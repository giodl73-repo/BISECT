//! Independent native composer oracle. No WASM county/evidence helpers are used.
use bisect_cli::edge_weights::{ComposedWeighter,GeographicWeighter,PartisanOverrideWeighter,MinorityOverrideWeighter,SubdivisionWeighter,EconomicCharacterWeighter,HousingCharacterWeighter,ZoneMembershipWeighter};
use std::collections::HashMap;
use serde_json::{Value,json};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let r:Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(1).ok_or("Request path required")?)?)?;
 let g=&r["graph"];let o=&r["options"];let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;let edges:Vec<(usize,usize,f64)>=serde_json::from_value(g["edges"].clone())?;
 let base=edges.iter().map(|&(u,v,w)|((u,v),w)).collect();let pairs=edges.iter().map(|&(u,v,_)|(u,v)).collect();let index=ids.iter().cloned().enumerate().collect::<HashMap<_,_>>();
 let kind=o["weights"].as_str().ok_or("Weight kind required")?;let alpha=o["county_augmentation"].as_f64().ok_or("County alpha required")?;
 let mut c=ComposedWeighter::new();if kind!="unweighted"{c=c.push(GeographicWeighter::from_map(base));}
 if kind=="partisan" {let shares=ids.iter().map(|id|r["partisan"]["dem_shares"][id].as_f64().ok_or("Share required")).collect::<Result<Vec<_>,_>>()?;c=c.push(PartisanOverrideWeighter::new(pairs,shares,o["dem_threshold"].as_f64().ok_or("D threshold required")?,o["rep_threshold"].as_f64().ok_or("R threshold required")?));}
 else if kind=="vra-aligned"{let fractions=ids.iter().map(|id|r["demographics"]["minority_fractions"][id].as_f64().ok_or("Fraction required")).collect::<Result<Vec<_>,_>>()?;c=c.push(MinorityOverrideWeighter::new(pairs,fractions,0.4));}
 let county:Vec<Option<String>>=if g["county_ids"].is_null(){ids.iter().map(|id|id.get(..5).map(str::to_owned)).collect()}else{serde_json::from_value::<Vec<String>>(g["county_ids"].clone())?.into_iter().map(Some).collect()};let none=vec![None;ids.len()];
 c=c.push(SubdivisionWeighter::full(county,none.clone(),none.clone(),none,alpha,0.,0.,0.));
 if kind=="economic-character"||kind=="housing-character"{
  let blend=o["character_alpha"].as_f64().ok_or("Character alpha required")?;
  if !r["character"].is_null(){
   let data=&r["character"]["data"]["characters"];
   if kind=="economic-character"{let chars=ids.iter().map(|id|{let x=&data[id];Ok(bisect_core::economic_character::EconChar{commercial_intensity:x["commercial_intensity"].as_f64().ok_or("Commercial intensity")?,industrial_fraction:x["industrial_fraction"].as_f64().ok_or("Industrial fraction")?,jobs_per_resident:x["jobs_per_resident"].as_f64().ok_or("Jobs per resident")?})}).collect::<Result<Vec<_>,&str>>()?;c=c.push(EconomicCharacterWeighter::new(chars,blend));}
   else{let chars=ids.iter().map(|id|{let x=&data[id];Ok(bisect_core::housing_character::HousingChar{pct_single_family:x["pct_single_family"].as_f64().ok_or("Single family")?,pct_multifamily:x["pct_multifamily"].as_f64().ok_or("Multifamily")?,pct_owner:x["pct_owner"].as_f64().ok_or("Ownership")?,housing_vintage:x["housing_vintage"].as_f64().ok_or("Vintage")?})}).collect::<Result<Vec<_>,&str>>()?;c=c.push(HousingCharacterWeighter::new(chars,blend));}
  }else if !r["native_character_source"].is_null(){
   if kind=="economic-character"{if let Ok(chars)=bisect_cli::lodes::load_lodes_wac_tract("fixture","2020"){if !chars.is_empty(){c=c.push(EconomicCharacterWeighter::new(bisect_cli::lodes::align_lodes_to_adjacency(&chars,&index,ids.len()),blend));}}}
   else if let Ok(chars)=bisect_cli::housing::load_acs_housing_tract("fixture","2020"){if !chars.is_empty(){c=c.push(HousingCharacterWeighter::new(bisect_cli::housing::align_housing_to_adjacency(&chars,&index,ids.len()),blend));}}
  }
 }
 if kind=="zone-membership"&&!r["zone_membership"].is_null(){let d:Value=serde_json::from_str(r["zone_membership"]["assignments_json"].as_str().ok_or("Zone JSON required")?)?;let assignments:Vec<Vec<Option<String>>>=serde_json::from_value(d["assignments"].clone())?;c=c.push(ZoneMembershipWeighter::from_zone_assignments(&assignments,&edges.iter().map(|&(u,v,_)|(u,v)).collect::<Vec<_>>(),r["zone_membership"]["alpha"].as_f64().ok_or("Zone alpha")?));}
 let mut weights=c.apply();if let Some(text)=r["coi"]["weights_json"].as_str(){weights=bisect_runner::coi_weights::apply(weights,&serde_json::from_str(text)?,&index)?;}
 let mut rows=weights.into_iter().map(|((u,v),w)|(u,v,w)).collect::<Vec<_>>();rows.sort_by_key(|&(u,v,_)|(u,v));println!("{}",json!({"weighted_edges":rows,"native_composer":true}));Ok(())
}
