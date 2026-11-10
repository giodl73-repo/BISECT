//! Native file loading/alignment and exact floating-point weighting, including
//! legacy values. Reports distinguish nonfinite values; they are never imputed.
use crate::{character_input::CharacterData,native_character_source};
use bisect_core::{economic_character::{EconChar,blend_edge as economic_edge},housing_character::{HousingChar,blend_edge as housing_edge}};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::HashSet;

#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target{pub state:String,pub year:String,
 #[serde(default,skip_serializing_if="Option::is_none")]pub unit_kind:Option<rplan_core::UnitKind>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub county_ids:Option<Vec<String>>,
 pub geoids:Vec<String>,pub edges:Vec<(usize,usize,f64)>}
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input{pub source:native_character_source::Input,pub target:Target,pub alpha:f64,pub allow_absent_fallback:bool}
pub fn class(value:f64)->&'static str{if value.is_nan(){"nan"}else if value==f64::INFINITY{"positive-infinity"}else if value==f64::NEG_INFINITY{"negative-infinity"}else if value==0.&&value.is_sign_negative(){"negative-zero"}else{"finite"}}
pub(crate) struct Computed{pub report:Value,pub weights:Vec<(usize,usize,f64)>}
pub fn evaluate(input:Input)->Result<Value,String>{Ok(build(input)?.report)}
pub(crate) fn build(input:Input)->Result<Computed,String>{
 let s=&input.source;let t=&input.target;let fips=crate::tract_input::state_fips(&s.state)?;
 if t.state!=s.state||t.year!=s.year||t.geoids.is_empty()||t.geoids.len()>100_000||t.edges.len()>1_000_000||!input.alpha.is_finite()||input.alpha.is_sign_negative()||input.alpha>1.{return Err("Invalid native character target scope, size or blend alpha.".into());}
 crate::engine::validate_unit_identity(&t.state,&t.year,t.unit_kind,t.county_ids.as_ref(),&t.geoids)?;
 // Preserve the original undeclared tract target contract. Declared namespaces
 // follow exactly the prepared-graph validator, including custom IDs/counties.
 if t.unit_kind.is_none(){crate::tract_input::import_scope(&t.state,&t.year,&s.source_label)?;if t.geoids.iter().any(|id|!id.starts_with(fips)){return Err("Native character weighting requires canonical selected-state target tracts.".into());}}
 let ids:HashSet<_>=t.geoids.iter().cloned().collect();
 let mut edges=HashSet::new();for &(u,v,w)in &t.edges{if u>=v||v>=t.geoids.len()||!w.is_finite()||w<=0.||!edges.insert((u,v)){return Err("Native character weighting requires unique canonical positive target boundaries.".into());}}
 let(bytes,loaded)=native_character_source::load(s)?;
 let mut native_error=None;let mut source_ids=Vec::new();let mut records=Vec::new();let mut weights=t.edges.clone();
 let status=match loaded{
  Err(e)=>{native_error=Some(e);"error"},
  Ok(data)=>{
   match &data{CharacterData::Economic{characters,..}=>source_ids.extend(characters.keys().cloned()),CharacterData::Housing{characters}=>source_ids.extend(characters.keys().cloned())};
   if source_ids.len()>100_000{return Err("Native character source exceeds100,000 unique records.".into());}
   if source_ids.is_empty(){"empty"}else{
    match data{
     CharacterData::Economic{characters,..}=>{
      let values:Vec<_>=t.geoids.iter().map(|id|characters.get(id).cloned().unwrap_or_else(EconChar::zero)).collect();
      for(id,c)in t.geoids.iter().zip(&values){let v=[c.commercial_intensity,c.industrial_fraction,c.jobs_per_resident];records.push(json!({"geoid":id,"supplied":characters.contains_key(id),"values":v,"classes":v.map(class)}));}
      for edge in &mut weights{edge.2=economic_edge(edge.2,&values[edge.0],&values[edge.1],input.alpha);}
     },
     CharacterData::Housing{characters}=>{
      let values:Vec<_>=t.geoids.iter().map(|id|characters.get(id).copied().unwrap_or_else(HousingChar::neutral)).collect();
      for(id,c)in t.geoids.iter().zip(&values){let v=[c.pct_single_family,c.pct_multifamily,c.pct_owner,c.housing_vintage];records.push(json!({"geoid":id,"supplied":characters.contains_key(id),"values":v,"classes":v.map(class)}));}
      for edge in &mut weights{edge.2=housing_edge(edge.2,&values[edge.0],&values[edge.1],input.alpha);}
     }
    };"loaded"
   }
  }
 };
 let fallback=status!="loaded";if fallback&&!input.allow_absent_fallback{return Err("Native source is empty or failed to load; explicit geographic fallback permission is required.".into());}
 source_ids.sort();let supplied:HashSet<_>=source_ids.iter().cloned().collect();let mut missing:Vec<_>=ids.difference(&supplied).cloned().collect();missing.sort();let ignored:Vec<_>=source_ids.into_iter().filter(|id|!ids.contains(id)).collect();
 let report=json!({"schema_version":"bisect-native-character-weights-v1","state":s.state,"year":s.year,"data_year":s.data_year,"kind":s.kind,"alpha":input.alpha,"source_name":s.source_name,"source_sha256":format!("{:x}",Sha256::digest(&bytes)),"source_byte_length":bytes.len(),"target_json":serde_json::to_string(t).map_err(|e|e.to_string())?,"target_sha256":format!("{:x}",Sha256::digest(serde_json::to_vec(t).map_err(|e|e.to_string())?)),"native_status":status,"native_error":native_error,"geographic_fallback":fallback,"matched_tracts":if fallback{0}else{t.geoids.len()-missing.len()},"missing_tract_ids":missing,"ignored_source_ids":ignored,"records":records,"edges":weights.iter().map(|&(u,v,w)|json!({"u":u,"v":v,"weight":w,"class":class(w)})).collect::<Vec<_>>(),"execution_performed":false,"scope":"Native file-reader, exact-ID alignment in the declared unit namespace and geographic/cosine edge-weighter evaluation only. No aggregation, parent-ID matching or source-resolution conversion. Legacy matched_tracts/missing_tract_ids names count selected target units. Missing economic IDs use zero vectors; missing housing IDs use neutral 0.5 vectors only for a nonempty loaded source. Empty/error sources use geographic edges only with explicit permission. Nonfinite values are null with explicit classes; signed zero is classified. No partition, source authentication or plan certification."});
 Ok(Computed{report,weights})
}

pub(crate) fn for_engine(g:&crate::engine::PreparedGraph,o:&crate::engine::Options,source:Option<&native_character_source::Input>,has_character:bool)->Result<Option<Computed>,String>{
 let Some(source)=source else{return Ok(None)};
 if has_character{return Err("Native file input and explicit character observations are mutually exclusive.".into());}
 if !["economic-character","housing-character"].contains(&o.weights.as_str())||format!("{}-character",source.kind)!=o.weights{return Err("Native character source kind must match selected character weights.".into());}
 let alpha=o.character_alpha.ok_or("Native character source requires blend alpha.")?;
 build(Input{source:source.clone(),target:Target{state:g.state.clone(),year:g.year.clone(),unit_kind:g.unit_kind,county_ids:g.county_ids.clone(),geoids:g.geoids.clone(),edges:g.edges.clone()},alpha,allow_absent_fallback:o.character_absent_file_fallback==Some(true)}).map(Some)
}
pub(crate) fn evidence(report:&Value,scope:&str)->Value{json!({"method":"native-character-file-v1","evaluation":report,"scope":scope})}
