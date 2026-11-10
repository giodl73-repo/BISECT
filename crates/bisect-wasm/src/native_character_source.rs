use crate::character_input::{CharacterInput,CharacterData};
use bisect_core::{economic_character::derive_economic_character,native_character_csv::{parse_lodes_wac_raw,parse_housing_tract}};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input{pub state:String,pub year:String,pub data_year:String,pub source_label:String,pub kind:String,pub source_name:String,pub source_base64:String}
pub(crate) fn decode(input:&Input)->Result<Vec<u8>,String>{
 use base64::Engine;
 crate::tract_input::state_fips(&input.state)?;
 if input.year.len()!=4||!input.year.bytes().all(|b|b.is_ascii_digit())||input.source_label.trim().is_empty()||input.source_label.len()>200{return Err("Record a four-digit unit year and source label of 1-200 UTF-8 bytes.".into());}
 if !["economic","housing"].contains(&input.kind.as_str())||input.data_year.len()!=4||!input.data_year.bytes().all(|x|x.is_ascii_digit())||input.source_name.is_empty()||input.source_name.len()>200||input.source_base64.len()>8*1024*1024/3*4+4{return Err("Invalid native character CSV scope, kind, source name or size.".into());}
 let bytes=base64::engine::general_purpose::STANDARD.decode(&input.source_base64).map_err(|_|"Invalid native character source encoding.")?;
 if bytes.len()>8*1024*1024{return Err("Native character CSV exceeds8MiB.".into());}
 Ok(bytes)
}
pub(crate) fn load(input:&Input)->Result<(Vec<u8>,Result<CharacterData,String>),String>{
 let bytes=decode(input)?;
 let data=std::str::from_utf8(&bytes).map_err(|_|"Native character CSV is not valid UTF-8.".to_string()).and_then(|text|{
  if input.kind=="economic"{parse_lodes_wac_raw(text,&input.source_name).map(|map|{let raw_counts:BTreeMap<_,_>=map.into_iter().collect();let characters=raw_counts.iter().map(|(id,raw)|(id.clone(),derive_economic_character(*raw))).collect();CharacterData::Economic{raw_counts,characters}})}else{parse_housing_tract(text,&input.source_name).map(|map|CharacterData::Housing{characters:map.into_iter().collect()})}
 });
 Ok((bytes,data))
}
pub fn prepare(input:Input)->Result<Value,String>{
 let(bytes,data)=load(&input)?;
 let mut rows=Vec::new();let mut character_input=None;let mut character_error=None;
 let (status,error)=match data{
  Err(e)=>("error",Some(e)),
  Ok(data)=>{
   let count=match &data{CharacterData::Economic{characters,..}=>characters.len(),CharacterData::Housing{characters}=>characters.len()};
   if count>100_000{return Err("Native character source exceeds100,000 unique records.".into());}
   match &data{
    CharacterData::Economic{characters,..}=>for(id,c)in characters{let values=[c.commercial_intensity,c.industrial_fraction,c.jobs_per_resident];rows.push(json!({"geoid":id,"values":values,"finite":values.iter().all(|v|v.is_finite())}));},
    CharacterData::Housing{characters}=>for(id,c)in characters{let values=[c.pct_single_family,c.pct_multifamily,c.pct_owner,c.housing_vintage];rows.push(json!({"geoid":id,"values":values,"finite":values.iter().all(|v|v.is_finite())}));}
   }
   let ids=rows.iter().map(|r|r["geoid"].as_str().unwrap().to_string()).collect::<Vec<_>>();
   let character=CharacterInput{schema_version:"bisect-character-v1".into(),state:input.state.clone(),year:input.year.clone(),data_year:input.data_year.clone(),source_label:input.source_label.clone(),data,alignment:None};
   match crate::character_input::build_weights(&character,&ids,&[],0.5){Ok(_)=>character_input=Some(character),Err(e)=>character_error=Some(e)};
   (if rows.is_empty(){"empty"}else{"loaded"},None)
  }
 };
 Ok(json!({"schema_version":"bisect-native-character-source-v1","state":input.state,"year":input.year,"data_year":input.data_year,"kind":input.kind,"source_name":input.source_name,"source_sha256":format!("{:x}",Sha256::digest(&bytes)),"source_byte_length":bytes.len(),"native_status":status,"native_error":error,"records":rows,"character_status":if character_input.is_some(){"ready"}else{"unavailable"},"character_error":character_error,"character_input":character_input,"execution_performed":false,"scope":"Shared native tract CSV reader semantics: first matching headers, last duplicate row, housing clamp/alias precedence, simple comma splitting and native parse errors. Nonfinite numeric results are null with finite=false. Strict character input is offered only for canonical selected-state observations and bounded finite values/raw counts. Original source bytes retained by the host; no disk loading, alignment, plan execution or certification."}))
}
