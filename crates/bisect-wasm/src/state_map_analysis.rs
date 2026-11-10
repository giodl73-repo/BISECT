//! Native state CLI row semantics are deliberately different from national maps.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use base64::{engine::general_purpose::STANDARD,Engine};
use std::collections::BTreeMap;
fn statistics(value:&Value,map_type:&str,k:usize)->(Value,Vec<u64>,Value){
 statistics_for_labels(value,map_type,&(1..=k as u64).collect())
}
pub(crate) fn statistics_for_labels(value:&Value,map_type:&str,labels:&std::collections::BTreeSet<u64>)->(Value,Vec<u64>,Value){
 let (field,fallback)=match map_type{"political"=>("dem_pct",0.5),"demographic"=>("pct_minority",0.0),_=>("polsby_popper",0.3)};
 let mut stats=BTreeMap::<u64,(f64,bool)>::new();let rows=value.get("districts").and_then(Value::as_array);let mut numeric=0usize;
 if let Some(rows)=rows{for row in rows{
  let id=row.get("district").and_then(Value::as_u64).unwrap_or(0);let stat=row.get(field).and_then(Value::as_f64);if stat.is_some(){numeric+=1;}
  // Unlike native national filter_map, state_map_items inserts fallback for
  // unusable statistics, overwriting any prior valid value for the same ID.
  stats.insert(id,(stat.unwrap_or(fallback),stat.is_none()));
 }}
 let normalized=stats.iter().filter(|(id,_)|labels.contains(id)).map(|(id,(stat,_))|{let mut row=json!({"district":id});row[field]=json!(stat);row}).collect::<Vec<_>>();
 let defaults=labels.iter().copied().filter(|id|stats.get(id).is_none_or(|(_,default)|*default)).collect::<Vec<_>>();
 let ignored_metadata=["analyzer","state","year","plan_hash"].into_iter().filter(|key|value.get(*key).is_some()).collect::<Vec<_>>();
 (json!({"districts":normalized}),defaults,json!({"status":"parsed","metadata_validation":"native-cli-unchecked","ignored_metadata_fields":ignored_metadata,"rows_seen":rows.map_or(0,Vec::len),"numeric_rows":numeric,"statistic_field":field,"default_value":fallback,"duplicate_district_policy":"last row wins; unusable numeric statistics overwrite with default"}))
}
pub(crate) fn prepare(input:&crate::map_export::Input)->Result<(bisect_map::renderer::DistrictGeometries,usize,Value,Value),String>{
 if !["political","demographic","compactness"].contains(&input.map_type.as_str()){return Err("Native state analysis policy requires a political, demographic or compactness map".into());}
 if input.analysis_json.is_some()&&input.analysis_base64.is_some(){return Err("Select one state analysis text or byte source".into());}
 let raw=if let Some(encoded)=&input.analysis_base64{
  if encoded.len()>4*1024*1024{return Err("Native state analysis exceeds 3 MiB".into());}
  STANDARD.decode(encoded).map_err(|_|"Invalid native state analysis Base64")?
 }else{input.analysis_json.as_ref().ok_or("Select native analysis JSON for the chosen map type")?.as_bytes().to_vec()};
 if raw.len()>3*1024*1024{return Err("Native state analysis exceeds 3 MiB".into());}
 let text=std::str::from_utf8(&raw).map_err(|e|format!("Native state analysis is not UTF-8: {e}"))?;
 let value:Value=serde_json::from_str(text).map_err(|e|e.to_string())?;
 let mut geometry_input=input.clone();geometry_input.analysis_policy=crate::map_export::StateAnalysisPolicy::Validated;geometry_input.map_type="districts".into();geometry_input.analysis_json=None;geometry_input.analysis_base64=None;
 let (districts,k,_,mut metadata)=crate::map_export::prepare(&geometry_input)?;
 let (analysis,defaults,observation)=statistics(&value,&input.map_type,k);
 metadata["map_type"]=json!(input.map_type);metadata["fallback_districts"]=json!(defaults);metadata["analysis_source_sha256"]=json!(format!("{:x}",Sha256::digest(&raw)));metadata["analysis_policy"]=json!("native-state");metadata["analysis_observation"]=observation;
 metadata["scope"]=json!("Native state CLI analysis rows on a validated exact RPLAN and matching unit geometry. Original analysis bytes are hashed and retained by the project. Analysis must be present, readable UTF-8 and valid JSON; missing/invalid files remain fatal. Any JSON shape is accepted. Last district row wins, including a fallback overwrite for an unusable numeric statistic. Political/demographic/compactness defaults are 0.5/0.0/0.3. Declared analyzer/state/year/plan_hash metadata is ignored, not authenticated. Source/geometry/work/output limits and native renderer/projection/font remain. This renders supplied data, does not generate or certify a plan, and defaults are display values rather than measured statistics.");
 Ok((districts,k,analysis,metadata))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn invalid_later_row_resets_state_statistic(){let (rows,defaults,report)=statistics(&json!({"districts":[{"district":1,"dem_pct":0.7},{"district":1,"dem_pct":null},{"district":2,"dem_pct":0.2},{"district":4294967298u64,"dem_pct":0.9}]}),"political",3);assert_eq!(rows["districts"][0]["dem_pct"],0.5);assert_eq!(rows["districts"][1]["dem_pct"],0.2);assert_eq!(defaults,vec![1,3]);assert_eq!(report["rows_seen"],4);}
 #[test]fn arbitrary_json_shapes_use_state_defaults(){for value in [Value::Null,json!([]),json!({"districts":"bad"})]{let (_,defaults,_)=statistics(&value,"compactness",2);assert_eq!(defaults,vec![1,2]);}}
 #[test]fn noninteger_district_ids_do_not_claim_district_one(){let value:Value=serde_json::from_str(r#"{"districts":[{"district":1.0,"pct_minority":0.8},{"district":4294967297,"pct_minority":0.9}]}"#).unwrap();let (rows,defaults,_)=statistics(&value,"demographic",1);assert_eq!(rows["districts"],json!([]));assert_eq!(defaults,vec![1]);}
}
