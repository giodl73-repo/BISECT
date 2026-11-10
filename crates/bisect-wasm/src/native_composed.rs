//! Selected-byte loading for the shared, ordered native State weight pipeline.
use crate::{engine::{Request,PreparedGraph,Options},native_auxiliary::{Input,Source},character_input::CharacterData};
use bisect_runner::{state_config::WeightSpec,native_weighting::{LoadedSignals,Route,route,compose}};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,HashMap,HashSet};

pub(crate) fn validate_request(r:&Request)->Result<(),String>{
 match &r.native_weighting{
  None=>if r.options.weights=="native-composed"||r.native_auxiliary.as_ref().is_some_and(|i|i.demographics.is_some()||i.economic.is_some()||i.housing.is_some()){return Err("Native composed weights require their resolved WeightSpec.".into());},
  Some(spec)=>{
   crate::engine::validate_profile(&r.options,spec.partisan_shares.is_some(),None)?;
   if r.options.weights!="native-composed"||r.options.county_augmentation.is_some()||r.options.character_alpha.is_some()||r.options.dem_threshold.is_some()||r.options.rep_threshold.is_some(){return Err("Native composed weights use one resolved WeightSpec, without duplicate scalar weighting controls.".into());}
   if r.demographics.is_some()||r.partisan.is_some()||r.character.is_some()||r.native_character_source.is_some()||r.zone_membership.is_some(){return Err("Native composed weighting requires selected original bytes, without conflicting typed observations.".into());}
  }
 }Ok(())
}
pub(crate) struct Computed {pub weights:HashMap<(usize,usize),f64>,pub demographic:Option<(Vec<f64>,String)>,pub metrics:BTreeMap<String,Value>}
fn bytes(source:&Source)->Result<Option<Vec<u8>>,String>{crate::native_auxiliary::decode(source,8*1024*1024)}
fn receipt(source:&Source,blob:Option<&[u8]>)->Value{json!({"source_name":source.source_name,"source_sha256":blob.map(|b|format!("{:x}",Sha256::digest(b))),"recorded_absence_claim":blob.is_none()})}
pub(crate) fn build(spec:&WeightSpec,input:Option<&Input>,g:&PreparedGraph,o:&Options)->Result<Computed,String>{
 let selected_route=route(spec,o.districts);let composing=selected_route==Route::Compose;
 let mut loaded=LoadedSignals::default();let mut metrics=BTreeMap::new();let ids=g.native_source_index();
 let physical:HashMap<_,_>=g.edges.iter().map(|&(u,v,w)|((u,v),w)).collect();
 let mut demographic=None;let demo=input.and_then(|i|i.demographics.as_ref());
 let needs_demo=(composing&&spec.minority_weighting)||(o.districts>1&&(o.structure=="ratio-optimal-vra"||o.search=="vra-recom"));
 if let Some(source)=demo{
  let blob=bytes(source)?;let mut report=receipt(source,blob.as_deref());
  if needs_demo{
   if let Some(blob)=blob{
    let text=std::str::from_utf8(&blob).map_err(|_|"Native demographic source is not UTF-8")?;
    let values=bisect_data::native_demographics::parse_native_demographics(text).map_err(|e|format!("demographics load failed: {e}"))?;
    let fractions:Vec<_>=(0..g.geoids.len()).map(|i|g.native_source_geoid(i).and_then(|id|values.get(id)).copied().unwrap_or(0.0)).collect();let matched=(0..g.geoids.len()).filter(|&i|g.native_source_geoid(i).is_some_and(|id|values.contains_key(id))).count();
    report["status"]=json!("native-reader-aligned");report["matched_units"]=json!(matched);report["defaulted_units"]=json!(g.geoids.len()-matched);report["ignored_units"]=json!(values.len()-matched);report["missing_fraction"]=json!(0.0);report["nonfinite_unit_ids"]=json!(g.geoids.iter().zip(&fractions).filter(|(_,f)|!f.is_finite()).map(|(id,_)|id).collect::<Vec<_>>());report["scope"]=json!("Native total-population CSV: exact uppercase header names, GEOID left padding, numeric zero defaults, last-row duplicates, clamped fractions and zero for missing graph IDs. Nonfinite values remain native floating-point observations, never imputed fractions or VAP counts.");
    let mut hash=Sha256::new();hash.update(b"BISECT_NATIVE_DEMOGRAPHIC_ALIGNMENT_V1\0");for(id,value)in g.geoids.iter().zip(&fractions){hash.update(id.as_bytes());hash.update([0]);hash.update(value.to_le_bytes());}
    if composing&&spec.minority_weighting{loaded.minority_fractions=Some(fractions.clone());}demographic=Some((fractions,format!("{:x}",hash.finalize())));
   }else if composing&&spec.minority_weighting{return Err("Required native demographic source is recorded absent; minority weighting cannot continue.".into())}
   else{report["status"]=json!("native-absent-method-fallback");}
  }else{report["status"]=json!("skipped-native-weighting-route");}
  metrics.insert("native_demographic_source_evidence".into(),report);
 }else if needs_demo{return Err("Select the recorded native demographic source or its bound absence claim.".into())}
 let partisan=input.and_then(|i|i.partisan.as_ref());
 if composing&&spec.partisan_shares.as_ref().is_some_and(|p|!p.as_os_str().is_empty()){
  if partisan.is_none(){return Err("Select the recorded native partisan source.".into());}
  let p=crate::native_auxiliary::partisan(input,g,true,o.districts)?;loaded.partisan_shares=p.data.map(|d|d.0);if let Some(e)=p.evidence{metrics.insert("native_partisan_source_evidence".into(),e);}
 }else if let Some(source)=partisan{
  let blob=bytes(source)?;let mut e=receipt(source,blob.as_deref());e["status"]=json!(if selected_route==Route::SingleDistrict{"skipped-single-district-native-weighting"}else{"skipped-native-weighting-route"});metrics.insert("native_partisan_source_evidence".into(),e);
 }
 let mut character_reports=vec![];
 for(kind,active)in [("economic",spec.economic_character),("housing",spec.housing_character)]{
  let source=input.and_then(|i|if kind=="economic"{i.economic.as_ref()}else{i.housing.as_ref()});
  let legacy=input.and_then(|i|i.character.as_ref()).filter(|c|c.kind==kind).map(|c|&c.source);
  if source.is_some()&&legacy.is_some(){return Err("Choose one recorded source per character kind.".into());}
  let source=source.or(legacy);
  if !active{if source.is_some(){return Err("Recorded character source has no configured native stage.".into());}continue}
  let Some(source)=source else{if composing{return Err(format!("Select the recorded native {kind} source or absence claim."));}continue};
  let blob=bytes(source)?;let mut report=receipt(source,blob.as_deref());report["kind"]=json!(kind);
  if !composing{report["status"]=json!(if selected_route==Route::SingleDistrict{"skipped-single-district-native-weighting"}else{"skipped-native-weighting-route"});}
  else if blob.is_none(){report["status"]=json!("native-warning-fallback");report["reason"]=json!("recorded-source-absent");report["computed_weights_preserved"]=json!(true);}
  else{
   let native=crate::native_character_source::Input{state:g.state.clone(),year:g.year.clone(),data_year:g.year.clone(),source_label:"Recorded native character discovery".into(),kind:kind.into(),source_name:source.source_name.clone(),source_base64:source.bytes_base64.clone().unwrap()};
   let(_,parsed)=crate::native_character_source::load(&native)?;
   match parsed{
    Err(error)=>{report["status"]=json!("native-warning-fallback");report["native_status"]=json!("error");report["error"]=json!(error);report["computed_weights_preserved"]=json!(true);},
    Ok(data)=>{
     let source_ids:Vec<_>=match &data{CharacterData::Economic{characters,..}=>characters.keys().cloned().collect(),CharacterData::Housing{characters}=>characters.keys().cloned().collect()};
     if source_ids.is_empty(){report["status"]=json!("native-warning-fallback");report["native_status"]=json!("empty");report["computed_weights_preserved"]=json!(true);}
     else{
      let supplied:HashSet<&str>=source_ids.iter().map(String::as_str).collect();let target:HashSet<&str>=(0..g.geoids.len()).filter_map(|i|g.native_source_geoid(i)).collect();let missing:Vec<_>=(0..g.geoids.len()).filter(|&i|g.native_source_geoid(i).is_none_or(|id|!supplied.contains(id))).map(|i|g.geoids[i].clone()).collect();let ignored:Vec<_>=source_ids.iter().filter(|id|!target.contains(id.as_str())).cloned().collect();report["status"]=json!("native-reader-aligned");report["native_status"]=json!("loaded");report["matched_units"]=json!(g.geoids.len()-missing.len());report["missing_unit_ids"]=json!(missing);report["ignored_source_ids"]=json!(ignored);
      match data{
       CharacterData::Economic{characters,..}=>loaded.economic=Some((0..g.geoids.len()).map(|i|g.native_source_geoid(i).and_then(|id|characters.get(id)).cloned().unwrap_or_else(bisect_core::economic_character::EconChar::zero)).collect()),
       CharacterData::Housing{characters}=>loaded.housing=Some((0..g.geoids.len()).map(|i|g.native_source_geoid(i).and_then(|id|characters.get(id)).copied().unwrap_or_else(bisect_core::housing_character::HousingChar::neutral)).collect()),
      }
     }
    }
   }
  }
  report["scope"]=json!("Shared native character reader and exact-ID alignment. A failed/empty stage preserves all prior composed weights; it does not reset the pipeline to geographic weights.");character_reports.push(report.clone());metrics.insert(format!("native_{kind}_source_evidence"),report);
 }
 if character_reports.len()==1{metrics.insert("native_character_source_evidence".into(),character_reports[0].clone());}
 let weights=match selected_route{Route::Unweighted=>HashMap::new(),Route::SingleDistrict=>physical.clone(),Route::Compose=>compose(spec,&g.adjacency,&physical,&ids,loaded)?};
 let mut rows:Vec<_>=weights.iter().map(|(&(u,v),&w)|json!({"u":u,"v":v,"weight":w,"class":crate::native_character_weights::class(w),"weight_bits_hex":format!("{:016x}",w.to_bits())})).collect();rows.sort_by_key(|v|(v["u"].as_u64(),v["v"].as_u64()));
 metrics.insert("native_weighting_evidence".into(),json!({"method":"native-composed-state-weighting-v1","route":format!("{selected_route:?}"),"resolved_weights":spec,"stage_order":["geographic-base","minority-override","partisan-override","county-augmentation","economic-character","housing-character"],"zone_stage":"native-spatial-join-not-implemented","effective_sparse_edges":rows,"scope":"Shared native ordered compositor. Sparse and empty weight maps are retained exactly; no stage fills missing edge entries or changes original configuration. These are computational observations, not source authentication or legal certification."}));
 Ok(Computed{weights,demographic,metrics})
}
