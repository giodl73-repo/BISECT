//! Explicit in-memory boundary for the native policy command.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use bisect_analysis::location_policy as native;
use base64::Engine;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag="status",rename_all="kebab-case",deny_unknown_fields)]
pub(crate) enum Candidate {
    File {name:String,base64:String},
    Unavailable {name:String},
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Fallback {pub primary:Option<Candidate>,pub alias:Option<Candidate>}

/// Reproduce source precedence without ambient environment/filesystem access.
/// Selection stops after JSON parse success, even if lookup/shape checks fail.
pub(crate) fn inspect_fallback(query:&str,input:Fallback)->Result<Value,String>{
    if query.trim().is_empty()||query.len()>200 {return Err("Supply a location code or name of 1–200 UTF-8 bytes.".into());}
    let mut attempts=Vec::new();let mut selected=None;
    for (role,candidate) in [("primary",input.primary),("alias",input.alias)] {
        let Some(candidate)=candidate else {attempts.push(json!({"role":role,"status":"not-configured"}));continue;};
        let name=match &candidate {Candidate::File{name,..}|Candidate::Unavailable{name}=>name};
        if name.is_empty()||name.len()>200||name.chars().any(|c|c.is_control()) {return Err("Invalid policy fallback source display name.".into());}
        let Candidate::File{name,base64}=candidate else {attempts.push(json!({"role":role,"name":name,"status":"unavailable","authority":"Input observation only; no live path access or authenticated read failure."}));continue;};
        if base64.len()>699_052 {return Err("Policy fallback source exceeds 512 KiB.".into());}
        let bytes=base64::engine::general_purpose::STANDARD.decode(&base64).map_err(|_|"Invalid policy fallback source base64.")?;
        if bytes.len()>512*1024 {return Err("Policy fallback source exceeds 512 KiB.".into());}
        let hash=format!("sha256:{:x}",Sha256::digest(&bytes));
        let text=match String::from_utf8(bytes) {
            Ok(text)=>text,
            Err(_)=>{attempts.push(json!({"role":role,"name":name,"status":"read-refused","reason":"invalid-utf8","sha256":hash}));continue;},
        };
        match serde_json::from_str::<Value>(&text) {
            Ok(_)=>{
                attempts.push(json!({"role":role,"name":name,"status":"selected","sha256":hash}));
                selected=Some((role,name,text,hash));break;
            },
            Err(error)=>attempts.push(json!({"role":role,"name":name,"status":"parse-refused","reason":error.to_string(),"sha256":hash})),
        }
    }
    let (role,name,text,hash)=selected.unwrap_or_else(||("embedded","location_policy.json".into(),native::EMBEDDED_POLICY.into(),format!("sha256:{:x}",Sha256::digest(native::EMBEDDED_POLICY.as_bytes()))));
    if role=="embedded" {attempts.push(json!({"role":role,"name":name,"status":"selected","sha256":hash}));}
    let selection=json!({"mode":"native-override-fallback","selected_role":role,"selected_name":name,"selected_sha256":hash,"attempts":attempts,"authority":"Explicit supplied bytes/unavailable observations. No environment or filesystem reads; imported observations are not live authority."});
    // Empty valid JSON selects a source natively; malformed empty text falls through.
    let lookup=if role=="embedded"{inspect(query,None,None)}else{inspect(query,Some(&name),Some(&text))};
    match lookup {
        Ok(mut result)=>{result["selection"]=selection;Ok(result)},
        Err(error)=>Ok(json!({"schema_version":"bisect-location-policy-fallback-v1","status":"lookup-refused","query":query,"selection":selection,"lookup_error":error,"scope":"Source selection completed; selected database lookup refused. No further fallback after a valid JSON parse. No plan execution or certification."})),
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;
    fn file(name:&str,text:&[u8])->Candidate{Candidate::File{name:name.into(),base64:base64::engine::general_purpose::STANDARD.encode(text)}}
    #[test]fn primary_success_does_not_decode_alias(){
        let out=inspect_fallback("XX",Fallback{primary:Some(file("primary.json",br#"{"XX":{"name":"Primary"}}"#)),alias:Some(Candidate::File{name:"alias.json".into(),base64:"not base64".into()})}).unwrap();
        assert_eq!(out["policy"]["name"],"Primary");assert_eq!(out["selection"]["attempts"].as_array().unwrap().len(),1);
    }
    #[test]fn parse_and_utf8_failures_continue_to_alias(){
        for raw in [b"{".as_slice(),b"",b"\xef\xbb\xbf{}",b"\xff"]{
            let out=inspect_fallback("XX",Fallback{primary:Some(file("primary.json",raw)),alias:Some(file("alias.json",br#"{"XX":{"name":"Alias"}}"#))}).unwrap();
            assert_eq!(out["selection"]["selected_role"],"alias");assert_eq!(out["policy"]["name"],"Alias");
        }
    }
    #[test]fn unavailable_sources_use_embedded(){
        let out=inspect_fallback("RI",Fallback{primary:Some(Candidate::Unavailable{name:"missing.json".into()}),alias:None}).unwrap();
        assert_eq!(out["selection"]["selected_role"],"embedded");assert_eq!(out["policy"]["name"],"Rhode Island");
    }
    #[test]fn valid_json_lookup_failure_never_uses_alias(){
        for text in ["{}","[]","42","null"]{
            let out=inspect_fallback("XX",Fallback{primary:Some(file("primary.json",text.as_bytes())),alias:Some(file("alias.json",br#"{"XX":{"name":"Alias"}}"#))}).unwrap();
            assert_eq!(out["status"],"lookup-refused");assert_eq!(out["selection"]["selected_role"],"primary");assert_eq!(out["selection"]["attempts"].as_array().unwrap().len(),1);
        }
    }
}
pub(crate) fn inspect(query:&str,source_name:Option<&str>,policy_json:Option<&str>)->Result<Value,String>{
    if query.trim().is_empty()||query.len()>200 {return Err("Supply a location code or name of 1–200 UTF-8 bytes.".into());}
    let (database,name,kind,source_hash)=database(source_name,policy_json)?;
    let object=database.as_object().unwrap();
    let (code,policy,fuzzy)=native::resolve(&database,query)?;
    let policy_json=serde_json::to_string_pretty(&policy).map_err(|e|e.to_string())?;
    let fields:Vec<_>=native::DISPLAY_FIELDS.iter().filter_map(|(label,key)|policy.get(*key).filter(|v|!v.is_null()).map(|v|json!({"label":label,"key":key,"display":native::display_value(key,v)}))).collect();
    let locations:Vec<_>=object.iter().filter(|(k,_)|!k.starts_with('_')).filter_map(|(k,v)|v.get("name").and_then(Value::as_str).map(|n|json!({"code":k,"name":n}))).collect();
    let mut display=policy.clone();exact_display(&mut display);
    Ok(json!({"schema_version":"bisect-location-policy-v1","query":query,"resolved_code":code,"fuzzy_match":fuzzy,
        "source_kind":kind,"source_name":name,"source_sha256":source_hash,
        "policy":display,"policy_json":policy_json,"table_text":native::render_table(&code,&policy),"fields":fields,"locations":locations,
        "scope":"Shared native location-policy lookup and CLI text/JSON rendering. Exact case-insensitive code lookup precedes unique case-insensitive name-substring lookup; ambiguous names fail. Embedded policy is the database compiled into this build; a local override uses only the explicitly supplied JSON and never reads an environment variable, filesystem path or network resource. Source hashes identify bytes, not source authority. Raw JSON retains native integer values; values outside the JavaScript safe numeric range are decimal strings in the browser view. Policy entries, citations, counts and tolerances are retained reference data; their current legal accuracy has not been verified. No policy is applied to a plan, no engine runs and no legal certification is produced."}))
}

fn validate_tree(value:&Value,depth:usize,count:&mut usize)->Result<(),String>{
 *count+=1;if depth>64||*count>50_000{return Err("Location policy exceeds the browser JSON complexity limit.".into());}
 match value {
  Value::Object(fields)=>for (key,val) in fields {if ["__proto__","constructor","prototype"].contains(&key.as_str()){return Err("Unsafe location policy property.".into());}validate_tree(val,depth+1,count)?;},
  Value::Array(values)=>for val in values{validate_tree(val,depth+1,count)?;},_=>{}
 }
 Ok(())
}
pub(crate) fn exact_display(value:&mut Value){
 match value {
  Value::Number(n) if n.as_f64().is_some_and(|x|x.abs()>9_007_199_254_740_991.0)=>*value=Value::String(n.to_string()),
  Value::Object(fields)=>fields.values_mut().for_each(exact_display),
  Value::Array(values)=>values.iter_mut().for_each(exact_display),_=>{}
 }
}

pub(crate) fn database(source_name:Option<&str>,policy_json:Option<&str>)->Result<(Value,String,String,String),String>{
    let (text,name,kind)=match (source_name,policy_json) {
        (None,None)=>(native::EMBEDDED_POLICY,"location_policy.json","embedded"),
        (Some(name),Some(text)) if !name.is_empty()&&name.len()<=200&&(1..=512*1024).contains(&text.len())=>(text,name,"local-override"),
        _=>return Err("Supply both override source name and JSON, at most 512 KiB, or neither.".into()),
    };
    let database:Value=serde_json::from_str(text).map_err(|e|format!("Invalid location policy JSON: {e}"))?;
    validate_tree(&database,0,&mut 0)?;
    let object=database.as_object().ok_or("Location policy database must be a JSON object.")?;
    if object.len()>1000 {return Err("Location policy database exceeds 1,000 entries.".into());}
    Ok((database,name.to_string(),kind.to_string(),format!("sha256:{:x}",Sha256::digest(text.as_bytes()))))
}
