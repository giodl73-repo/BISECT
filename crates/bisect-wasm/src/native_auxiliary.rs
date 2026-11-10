//! Selected native bytes, native TSV/COI policies, and explicit fallback evidence.
use base64::{engine::general_purpose::STANDARD,Engine};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,HashMap};
use crate::engine::PreparedGraph;

#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {pub source_name:String,pub bytes_base64:Option<String>}
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub partisan:Option<Source>,pub coi:Option<Source>,#[serde(default,skip_serializing_if="Option::is_none")]pub character:Option<Character>,#[serde(default,skip_serializing_if="Option::is_none")]pub demographics:Option<Source>,#[serde(default,skip_serializing_if="Option::is_none")]pub economic:Option<Source>,#[serde(default,skip_serializing_if="Option::is_none")]pub housing:Option<Source>}
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Character {pub kind:String,pub source:Source}
pub(crate) fn decode(source:&Source,limit:usize)->Result<Option<Vec<u8>>,String>{
 if source.source_name.is_empty()||source.source_name.len()>4096||source.source_name.contains('\0'){return Err("Invalid recorded native auxiliary source name.".into());}
 source.bytes_base64.as_ref().map(|encoded|{
  if encoded.len()>limit.div_ceil(3)*4{return Err("Selected native auxiliary exceeds its byte bound.".into());}
  let bytes=STANDARD.decode(encoded).map_err(|_|"Invalid native auxiliary base64.")?;
  if bytes.len()>limit||STANDARD.encode(&bytes)!=*encoded{return Err("Native auxiliary requires bounded canonical base64.".into());}Ok(bytes)
 }).transpose()
}
impl Input{pub(crate) fn has_sources(&self)->bool{self.partisan.is_some()||self.coi.is_some()||self.character.is_some()||self.demographics.is_some()||self.economic.is_some()||self.housing.is_some()}}
pub(crate) fn validate(input:&Input)->Result<(),String>{
 if !input.has_sources(){return Err("Select at least one recorded native auxiliary source.".into());}
 if let Some(s)=&input.partisan{decode(s,8*1024*1024)?;}if let Some(s)=&input.coi{decode(s,3*1024*1024)?;}for source in [&input.demographics,&input.economic,&input.housing].into_iter().flatten(){decode(source,8*1024*1024)?;}if let Some(c)=&input.character{if !["economic","housing"].contains(&c.kind.as_str()){return Err("Unknown native character kind.".into());}decode(&c.source,8*1024*1024)?;}Ok(())
}
pub(crate) fn validate_selection(input:Option<&Input>,explicit_partisan:bool,explicit_coi:bool)->Result<(),String>{
 if let Some(input)=input{validate(input)?;if input.partisan.is_some()&&explicit_partisan||input.coi.is_some()&&explicit_coi{return Err("Select one explicit or native source per auxiliary kind.".into());}}Ok(())
}
pub(crate) struct Partisan {pub data:Option<(Vec<f64>,String)>,pub evidence:Option<Value>}
pub(crate) fn partisan(input:Option<&Input>,g:&PreparedGraph,active:bool,districts:usize)->Result<Partisan,String>{
 let Some(source)=input.and_then(|i|i.partisan.as_ref())else{return Ok(Partisan{data:None,evidence:None})};
 let bytes=decode(source,8*1024*1024)?;
 let identity=bytes.as_ref().map(|b|format!("{:x}",Sha256::digest(b)));
 if districts==1{return Ok(Partisan{data:None,evidence:Some(json!({"status":"skipped-single-district-native-weighting","source_name":source.source_name,"source_sha256":identity,"recorded_absence_claim":bytes.is_none(),"scope":"Native State skips partisan file loading for a single district. Selected bytes are retained; no shares are fabricated or parsed."}))})}
 if !active{return Err("Native partisan source has no effect without partisan weighting.".into());}
 let bytes=bytes.ok_or("Recorded partisan source is absent; native multi-district loading cannot continue.")?;
 let text=std::str::from_utf8(&bytes).map_err(|_|"Native partisan source is not UTF-8.")?;
 let shares=bisect_runner::partisan_shares::parse_native(text,&source.source_name)?;
 let ids=g.native_source_index();
 let aligned=bisect_runner::partisan_shares::align_native(&shares,&ids,g.geoids.len());
 let matched=(0..g.geoids.len()).filter(|&i|g.native_source_geoid(i).is_some_and(|id|shares.contains_key(id))).count();
 let mut h=Sha256::new();h.update(b"BISECT_NATIVE_PARTISAN_ALIGNMENT_V1\0");h.update(g.state.as_bytes());h.update([0]);h.update(g.year.as_bytes());h.update([0]);
 for(id,share)in g.geoids.iter().zip(&aligned){h.update(id.as_bytes());h.update([0]);h.update(share.to_le_bytes());}
 Ok(Partisan{data:Some((aligned,format!("{:x}",h.finalize()))),evidence:Some(json!({"status":"native-reader-aligned","source_name":source.source_name,"source_sha256":identity,"supplied_units":shares.len(),"matched_units":matched,"defaulted_units":g.geoids.len()-matched,"ignored_units":shares.len()-matched,"missing_share":0.5,"duplicate_policy":"last-row-wins","geoid_policy":"pad-left-to-eleven","scope":"Shared native TSV reader: first unparseable share row is a header, extra columns are ignored, duplicate IDs use the last row, and unmatched graph units use the native 0.5 default. This is distinct from the strict explicit-share importer. Original bytes remain selected; no source authenticity is established."}))})
}
pub(crate) fn coi(input:Option<&Input>,g:&PreparedGraph,base:HashMap<(usize,usize),f64>)->Result<(HashMap<(usize,usize),f64>,Option<Value>),String>{
 let Some(source)=input.and_then(|i|i.coi.as_ref())else{return Ok((base,None))};
 // Invalid transport is an API error, not a native file-content fallback.
 let bytes=decode(source,3*1024*1024)?;let identity=bytes.as_ref().map(|b|format!("{:x}",Sha256::digest(b)));
 let checked=(||->Result<_,String>{
  let text=std::str::from_utf8(bytes.as_deref().ok_or("Recorded COI source is absent.")?).map_err(|_|"Native COI file is not UTF-8.")?;
  let values=bisect_runner::coi_weights::parse(text)?;let ids=g.native_source_index();
  let adjusted=bisect_runner::coi_weights::apply(base.clone(),&values,&ids)?;
  let matched=(0..g.geoids.len()).filter(|&i|g.native_source_geoid(i).is_some_and(|id|values.contains_key(id))).count();let changed=base.iter().filter(|(key,value)|adjusted[key].to_bits()!=value.to_bits()).count();
  Ok((adjusted,json!({"status":"native-adjustment-applied","supplied_units":values.len(),"matched_units":matched,"defaulted_units":g.geoids.len()-matched,"ignored_units":values.len()-matched,"applied_edges":base.len(),"changed_edges":changed})))
 })();
 let (weights,mut evidence)=match checked{Ok(v)=>v,Err(error)=>(base,json!({"status":"native-warning-fallback","error":error,"computed_weights_preserved":true}))};
 evidence["source_name"]=json!(source.source_name);evidence["source_sha256"]=json!(identity);evidence["recorded_absence_claim"]=json!(bytes.is_none());
 evidence["scope"]=json!("Native COI post-weighter with native warning/fallback policy: read/parse/adjustment failure preserves the entire previously computed weight map. Missing units use one; unweighted maps stay empty. Original selected bytes or recorded absence claims remain explicit; no filesystem path is opened or source authority authenticated.");
 Ok((weights,Some(evidence)))
}
pub(crate) fn source_hashes(input:&Input)->Result<BTreeMap<String,String>,String>{
 validate(input)?;let mut hashes=BTreeMap::new();
 hashes.insert("bisect.native-auxiliary-claims".into(),rplan_core::canonical_sha256(&serde_json::to_value(input).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?);
 for(role,source,limit)in[("partisan",input.partisan.as_ref(),8*1024*1024),("coi",input.coi.as_ref(),3*1024*1024),("character",input.character.as_ref().map(|c|&c.source),8*1024*1024),("demographics",input.demographics.as_ref(),8*1024*1024),("economic",input.economic.as_ref(),8*1024*1024),("housing",input.housing.as_ref(),8*1024*1024)]{
  if let Some(source)=source{if let Some(bytes)=decode(source,limit)?{hashes.insert(format!("bisect.native-{role}-bytes"),format!("{:x}",Sha256::digest(bytes)));}}
 }Ok(hashes)
}

pub(crate) struct CharacterResult {pub computed:Option<crate::native_character_weights::Computed>,pub evidence:Option<Value>}
pub(crate) fn character(input:Option<&Input>,g:&PreparedGraph,o:&crate::engine::Options,explicit:bool)->Result<CharacterResult,String>{
 let Some(c)=input.and_then(|i|i.character.as_ref())else{return Ok(CharacterResult{computed:None,evidence:None})};
 if explicit{return Err("Native recorded character input and explicit/native CSV observations are mutually exclusive.".into());}
 if format!("{}-character",c.kind)!=o.weights{return Err("Native recorded character kind must match selected character weights.".into());}
 let bytes=decode(&c.source,8*1024*1024)?;let identity=bytes.as_ref().map(|b|format!("{:x}",Sha256::digest(b)));
 let mut receipt=json!({"kind":c.kind,"source_name":c.source.source_name,"source_sha256":identity,"recorded_absence_claim":bytes.is_none(),"scope":"Original native working-directory character discovery. Selected bytes or a recorded absence claim are retained. Single-district weighting skips the reader. Missing/empty/read/parse failures preserve the previously composed weights; source or historical execution authenticity is not established."});
 if o.districts==1{receipt["status"]=json!("skipped-single-district-native-weighting");return Ok(CharacterResult{computed:None,evidence:Some(receipt)})}
 let Some(_bytes)=bytes else{receipt["status"]=json!("native-warning-fallback");receipt["reason"]=json!("recorded-source-absent");receipt["computed_weights_preserved"]=json!(true);return Ok(CharacterResult{computed:None,evidence:Some(receipt)})};
 let source=crate::native_character_source::Input{state:g.state.clone(),year:g.year.clone(),data_year:g.year.clone(),source_label:"Recorded native character discovery".into(),kind:c.kind.clone(),source_name:c.source.source_name.clone(),source_base64:c.source.bytes_base64.clone().unwrap()};
 let computed=crate::native_character_weights::build(crate::native_character_weights::Input{source,target:crate::native_character_weights::Target{state:g.state.clone(),year:g.year.clone(),unit_kind:g.unit_kind,county_ids:g.county_ids.clone(),geoids:g.geoids.clone(),edges:g.edges.clone()},alpha:o.character_alpha.ok_or("Native character replay requires blend alpha")?,allow_absent_fallback:true})?;
 let fallback=computed.report["geographic_fallback"]==true;
 receipt["status"]=json!(if fallback{"native-warning-fallback"}else{"native-reader-aligned"});receipt["native_status"]=computed.report["native_status"].clone();receipt["error"]=computed.report["native_error"].clone();receipt["computed_weights_preserved"]=json!(fallback);receipt["matched_units"]=computed.report["matched_tracts"].clone();receipt["missing_unit_ids"]=computed.report["missing_tract_ids"].clone();receipt["ignored_source_ids"]=computed.report["ignored_source_ids"].clone();
 Ok(CharacterResult{computed:Some(computed),evidence:Some(receipt)})
}
