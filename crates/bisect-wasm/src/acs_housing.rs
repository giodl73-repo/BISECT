use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub source_name:String,pub body_json:String,pub state:String,pub year:String,pub data_year:String,pub source_label:String}
pub(crate) fn prepare(input:Input)->Result<Value,String>{
    if input.source_name.is_empty()||input.source_name.len()>200||input.body_json.is_empty()||input.body_json.len()>16*1024*1024{return Err("Select a nonempty ACS response up to 16 MiB and a source name up to 200 bytes.".into());}
    crate::tract_input::import_scope(&input.state,&input.year,&input.source_label)?;
    if input.data_year.len()!=4||!input.data_year.bytes().all(|b|b.is_ascii_digit()){return Err("ACS observation year must contain four digits.".into());}
    let report=bisect_core::acs_housing_import::convert(&input.body_json)?;
    let character=crate::character_input::import_csv(report.csv_text.as_bytes(),&input.state,&input.year,&input.data_year,&input.source_label,"housing");
    let (character_status,character_input,character_error)=match character{Ok(v)=>("ready",Some(v),None),Err(e)=>("unavailable",None,Some(e))};
    Ok(json!({"schema_version":"bisect-acs-housing-preparation-v1","status":"prepared","state":input.state,"year":input.year,"data_year":input.data_year,
        "source_name":input.source_name,"source_sha256":format!("{:x}",Sha256::digest(input.body_json.as_bytes())),
        "csv_text":report.csv_text,"csv_sha256":format!("{:x}",Sha256::digest(report.csv_text.as_bytes())),
        "written_rows":report.written_rows,"skipped_short_rows":report.short_rows,"skipped_fips_rows":report.malformed_fips.len(),
        "warnings":report.malformed_fips.iter().take(100).collect::<Vec<_>>(),"empty_response":report.empty_response,
        "character_status":character_status,"character_input":character_input,"character_error":character_error,
        "execution_performed":false,"scope":"Shared native Fetch ACS housing response conversion, preserving input row order, native numeric/default/vintage formulas and six-decimal CSV bytes. Short rows and malformed FIPS-length rows are disclosed; warnings show the first 100. Required-column bounds prevent malformed reordered rows from panicking. Housing character input is offered only when the existing strict importer accepts every CSV row for the selected state/geometry vintage; duplicate or foreign units remain unavailable. Observation year and source label are selected claims. No network request, disk installation, cache/done-marker activation, graph coverage verification or engine execution occurs; full Fetch/FLETCH acquisition remains pending."}))
}
