use serde::Deserialize;
use serde_json::{json,Value};
use base64::Engine;
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(tag="kind",rename_all="kebab-case",deny_unknown_fields)]
pub(crate) enum Source {AcsHousing {input:crate::acs_housing::Input},Lodes {input:crate::lodes_source::Input},Unavailable404}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub selection:crate::fetch_plan::Input,pub item_index:usize,#[serde(default)]pub force:bool,pub source:Option<Source>}
pub(crate) fn prepare(input:Input)->Result<Value,String>{
    let plan=crate::fetch_plan::plan(input.selection)?;
    let item=plan["items"].as_array().and_then(|v|v.get(input.item_index)).ok_or("Select a Fetch item from the source plan")?;
    let kind=item["kind"].as_str().unwrap_or("");
    if !["acs-housing","lodes-wac","lodes-od"].contains(&kind){return Err("Derived Fetch packages currently require ACS housing or LODES items".into());}
    if !input.force&&item["done"]==true{return Ok(json!({"schema_version":"bisect-fetch-data-package-v1","status":"skipped","item":item,"files_base64":{},"preparation":null,"installation_performed":false,"source_availability":"not-read","scope":"Native incremental skip observes a selected target plus marker. Neither source bytes nor disk contents were validated or modified."}));}
    let (csv,mut preparation,availability)=match input.source.ok_or("Select source bytes or an explicit HTTP404 claim")? {
        Source::AcsHousing {input} if kind=="acs-housing"=>{let value=crate::acs_housing::prepare(input)?;if value["state"]!=item["state_code"]||value["year"]!=item["year"]{return Err("ACS source scope does not match the Fetch item".into());}(value["csv_text"].as_str().ok_or("Missing CSV")?.to_string(),Some(value),"supplied")},
        Source::Lodes {input} if kind.starts_with("lodes-")=>{let value=crate::lodes_source::prepare(input)?;if value["state"]!=item["state_code"]||value["year"]!=item["year"]||value["kind"]!=if kind=="lodes-wac"{"wac"}else{"od"}{return Err("LODES source scope does not match the Fetch item".into());}(value["csv_text"].as_str().ok_or("Missing CSV")?.to_string(),Some(value),"supplied")},
        Source::Unavailable404=>{let header=match kind{"acs-housing"=>bisect_core::acs_housing_import::HEADER,"lodes-wac"=>"geoid,c000,cns07,cns09,cns10,cns11,cns01,cns02,cns05,cns08\n",_=>"home_geoid,work_geoid,s000\n"};(header.to_string(),None,"http404-claim")},
        _=>return Err("Source type does not match the selected Fetch item".into()),
    };
    if let Some(object)=preparation.as_mut().and_then(Value::as_object_mut){object.remove("csv_text");object.remove("character_input");}
    let marker=bisect_core::fetch_plan::DONE_MARKER_BYTES;
    let target=item["local_path"].as_str().ok_or("Missing target")?;let done=item["done_marker"].as_str().ok_or("Missing marker")?;
    if target==done{return Err("Fetch target and marker collide".into());}
    let encoded=base64::engine::general_purpose::STANDARD;
    let files=std::collections::BTreeMap::from([(target,encoded.encode(csv.as_bytes())),(done,encoded.encode(marker))]);
    Ok(json!({"schema_version":"bisect-fetch-data-package-v1","status":"prepared","item":item,"files_base64":files,"preparation":preparation,"source_availability":availability,"target_sha256":format!("{:x}",Sha256::digest(csv.as_bytes())),"marker_sha256":format!("{:x}",Sha256::digest(marker)),"installation_performed":false,
        "scope":"Derived Fetch target CSV and native done marker bytes, rebuilt from supplied source through shared native parsers; force bypasses the selected incremental marker check. Explicit HTTP404 claims reproduce native header-only soft-skip output and marker. A marker records a completed Fetch step, not valid or available data. Download/extraction is a separate host action; existing targets must be checked before replacement. No files, FLETCH source cache/manifest or browser cache were installed, and no engine or data-admissibility check ran."}))
}
