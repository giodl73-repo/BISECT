use serde::Deserialize;
use serde_json::{json,Value};
use std::collections::{BTreeMap,HashSet};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Input{pub year:String,pub types:Vec<String>,pub format:String,pub csv:bool,pub reports:BTreeMap<String,BTreeMap<String,String>>,#[serde(default)]pub completion:Option<Completion>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub(crate) struct Completion{pub year:String,pub existing_outputs:Vec<String>,pub force:bool}
fn typ(s:&str)->Result<bisect_analysis::AnalyzerType,String>{use bisect_analysis::AnalyzerType::*;Ok(match s{"all"=>All,"compactness"=>Compactness,"demographic"=>Demographic,"political"=>Political,"partisan"=>Partisan,"proportionality"=>Proportionality,"urban"=>Urban,"summary"=>Summary,"contiguity"=>Contiguity,"splits"=>Splits,"bloc-voting"=>BlocVoting,_=>return Err("Unknown native analyzer type.".into())})}
pub(crate) fn run(input:Input)->Result<Value,String>{
 if !["2000","2010","2020"].contains(&input.year.as_str())||!["csv","json","none","parquet"].contains(&input.format.as_str())||input.types.is_empty()||input.types.len()>11||input.reports.is_empty()||input.reports.len()>60{return Err("Choose native Census year, analyzer types, format and 1-60 state folders.".into());}
 let selected=input.types.iter().map(|s|typ(s)).collect::<Result<Vec<_>,_>>()?;
 let types=if selected.contains(&bisect_analysis::AnalyzerType::All){bisect_analysis::AnalyzerType::all_concrete()}else{selected};
 let mut existing=HashSet::new();
 if let Some(c)=&input.completion{
  if c.year!=input.year||c.existing_outputs.len()>10{return Err("Aggregate output observation must match the selected year and contain at most ten native JSON paths.".into());}
  for path in &c.existing_outputs{
   let name=path.strip_prefix("us_").and_then(|p|p.strip_suffix(".json")).ok_or("Invalid aggregate output observation path")?;
   if name=="all"||typ(name).is_err()||!existing.insert(path.clone()){return Err("Aggregate observations require distinct concrete native output JSON names.".into());}
  }
 }
 let skip_type=|name:&str|input.completion.as_ref().is_some_and(|c|!c.force)&&existing.contains(&format!("us_{name}.json"));
 let mut bytes=0usize;let mut seen=HashSet::new();let mut parsed=BTreeMap::new();let mut hashes=BTreeMap::new();let mut skipped=Vec::new();
 for (state,reports) in &input.reports{
  if state.is_empty()||state.len()>60||!state.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'_'|b'-'|b' '))||!seen.insert(state.to_ascii_uppercase())||reports.len()>10{return Err("State folder labels must be distinct bounded portable names.".into());}
  for (name,text) in reports{
   if name=="all"{return Err("Select concrete analysis files.".into());}typ(name)?;
   if skip_type(name){continue;}
   bytes=bytes.checked_add(state.len()+name.len()+text.len()).ok_or("Aggregate input size overflow")?;
   if text.len()>3*1024*1024||bytes>8*1024*1024{return Err("Native aggregate sources exceed 3 MiB per file or 8 MiB combined.".into());}
   let path=format!("{state}/analysis/{name}.json");hashes.insert(path.clone(),format!("{:x}",Sha256::digest(text.as_bytes())));
   if !types.iter().any(|t|t.name()==name){continue;}
   match crate::national::parse(text){Ok(v)=>{if let Some(rows)=v.get("districts").and_then(Value::as_array){if rows.len()>500||rows.iter().any(|d|!d.is_object()){return Err("District rows must be bounded objects before native merge.".into());}}parsed.insert((state.clone(),name.clone()),v);},Err(error)=>skipped.push(json!({"source":path,"error":error}))}
  }
 }
 let mut files=BTreeMap::new();let mut summaries=Vec::new();let mut format_guidance=Vec::new();
 for t in types{let name=t.name();let refs:Vec<_>=parsed.iter().filter(|((_,kind),_)|kind==name).map(|((s,_),v)|(s.as_str(),v)).collect();
  if skip_type(name)||(!input.completion.as_ref().is_some_and(|c|c.force)&&files.contains_key(&format!("us_{name}.json"))){summaries.push(json!({"analyzer":name,"status":"skipped-existing","states":0}));continue;}
  if refs.is_empty(){summaries.push(json!({"analyzer":name,"status":"no-data","states":0}));continue;}
  use bisect_analysis::national::*;let state_level=t==bisect_analysis::AnalyzerType::Proportionality;
  let merged=if state_level{merge_state_level_outputs(&input.year,&refs)}else{merge_analyzer_outputs(&refs)};
  files.insert(format!("us_{name}.json"),serde_json::to_string_pretty(&merged).map_err(|e|e.to_string())?);
  if input.csv||input.format=="csv"{files.insert(format!("us_{name}.csv"),if state_level{state_records_to_csv(&merged)}else{districts_to_csv(&merged)});}
  if input.format=="json"{files.insert(format!("us_{name}_{}.json",if state_level{"states"}else{"districts"}),serde_json::to_string_pretty(&if state_level{state_records_to_json_array(&merged)}else{districts_to_json_array(&merged)}).map_err(|e|e.to_string())?);}
  summaries.push(json!({"analyzer":name,"status":"aggregated","states":merged["state_count"],"districts":merged["district_count"]}));
  if input.format=="parquet"{format_guidance.push(json!({"analyzer":name,"message":"Parquet format not yet supported. Use --format csv and load with pandas.read_csv() or polars."}));}
 }
 let output_hashes:BTreeMap<_,_>=files.iter().map(|(name,text)|(name.clone(),format!("{:x}",Sha256::digest(text.as_bytes())))).collect();
 let mut output=json!({"schema_version":"bisect-native-aggregate-v1","year":input.year,"summaries":summaries,"skipped":skipped,"source_sha256":hashes,"files":files,"file_sha256":output_hashes,"scope":"Native aggregate merge/export semantics over explicitly selected raw state analysis files. State labels and year come from the selected folder scope; source declarations are retained, not recomputed. Invalid JSON is recorded and skipped; unsafe district shapes and resource bounds refuse before native merge. All uses the native default type list, excluding its opt-in analyzers. No referenced paths are opened, no analysis is executed, and outputs do not authenticate observations."});
 if input.format=="parquet"{output["requested_format"]=json!("parquet");output["format_guidance"]=json!(format_guidance);}
 if let Some(c)=input.completion{output["completion"]=json!({"year":c.year,"existing_outputs":c.existing_outputs,"force":c.force,"policy":"Observed existing native JSON skips its analyzer before source parsing, unless force is selected. This metadata snapshot neither authenticates prior output nor restores filesystem authority; no existing files are replaced."});}
 Ok(output)
}
