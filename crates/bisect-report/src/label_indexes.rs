//! Shared native label analysis/report index schemas; no filesystem or stage mutation.
use std::collections::{HashMap,BTreeMap};
use serde::{Serialize,Serializer};

/// Serialised form of `analysis/{label}/{year}/index.json`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AnalysisIndex {
    pub label: String,
    pub year: String,
    /// The label name again (mirrors the build index `label` field for clarity).
    pub run: String,
    /// SHA-256 of `runs/{label}/{year}/index.json`.
    pub run_index_sha256: String,
    /// Analysis types that were requested (may be empty if types came from `all`).
    pub types: Vec<String>,
    pub created: String,
    /// Per-state status map.  Key = state_name (lowercase), value = status object.
    #[serde(serialize_with="sorted_map")]
    pub states: HashMap<String, StateAnalysisStatus>,
}

/// Per-state entry inside the analysis index.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StateAnalysisStatus {
    /// Overall status: `"ok"` or `"failed"` or `"skipped"`.
    pub status: String,
    /// Populated when status is `"failed"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// One entry per analysis type, e.g. `{"proportionality": "ok", ...}`.
    #[serde(flatten,serialize_with="sorted_map")]
    pub type_statuses: HashMap<String, String>,
}

// ── ReportIndex ───────────────────────────────────────────────────────────────

/// Serialised form of `reports/{label}/{year}/index.json`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReportIndex {
    pub label: String,
    pub year: String,
    /// SHA-256 of `analysis/{label}/{year}/index.json`.
    pub analysis_index_sha256: String,
    pub format: String,
    pub created: String,
    /// Names of files written to `reports/{label}/{year}/`.
    pub files: Vec<String>,
}


fn sorted_map<S:Serializer,T:Serialize>(map:&HashMap<String,T>,serializer:S)->Result<S::Ok,S::Error>{map.iter().collect::<BTreeMap<_,_>>().serialize(serializer)}
/// Prevent filenames and flattened status keys from escaping the native index schema.
pub fn validate_analysis_types(types:&[String])->Result<(),String>{
 if types.len()>32||types.iter().any(|t|t.is_empty()||t.len()>100||!t.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||["status","error","__proto__","constructor","prototype"].contains(&t.as_str())){return Err("Analysis types require bounded safe names without reserved status keys.".into());}Ok(())
}
fn header(label:&str,year:&str,created:&str,hash:&str)->Result<(),String>{
 crate::label_rename::validate_name(label)?;
 if year.len()!=4||!year.bytes().all(|b|b.is_ascii_digit())||created.trim().is_empty()||created.len()>100||(!hash.is_empty()&&(hash.len()!=64||!hash.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)))){return Err("Supply a four-digit year, bounded recorded timestamp and empty or lowercase SHA-256 chain hash.".into());}Ok(())
}
pub fn serialize_analysis_index(index:&AnalysisIndex)->Result<String,String>{
 header(&index.label,&index.year,&index.created,&index.run_index_sha256)?;validate_analysis_types(&index.types)?;
 if index.run!=index.label||index.states.len()>100{return Err("Analysis index requires matching run/label and at most 100 states.".into());}
 for (name,state) in &index.states {
  if name.is_empty()||name.len()>100||!name.bytes().all(|b|b.is_ascii_lowercase()||b==b'_')||["__proto__","constructor","prototype"].contains(&name.as_str())||!["ok","failed","skipped","unavailable"].contains(&state.status.as_str())||state.error.as_ref().is_some_and(|e|e.len()>10000){return Err("Invalid bounded state analysis status.".into());}
  validate_analysis_types(&state.type_statuses.keys().cloned().collect::<Vec<_>>())?;
  if state.type_statuses.values().any(|s|!["ok","failed","skipped","unavailable"].contains(&s.as_str())){return Err("Invalid per-analyzer status.".into());}
 }
 serde_json::to_string_pretty(index).map_err(|e|e.to_string())
}
pub fn serialize_report_index(index:&ReportIndex)->Result<String,String>{
 header(&index.label,&index.year,&index.created,&index.analysis_index_sha256)?;
 if !["html","json"].contains(&index.format.as_str())||index.files.len()>512||index.files.iter().any(|p|p.is_empty()||p.len()>240||p.split('/').any(|s|s.is_empty()||s=="."||s=="..")||!p.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-'||b==b'.'||b==b'/')){return Err("Report index requires a native format and bounded safe relative filenames.".into());}
 serde_json::to_string_pretty(index).map_err(|e|e.to_string())
}
#[cfg(test)]
mod tests {
 use super::*;
 fn index(reverse:bool)->AnalysisIndex{
  let mut states=HashMap::new();for name in if reverse{vec!["vermont","rhode_island"]}else{vec!["rhode_island","vermont"]}{let mut type_statuses=HashMap::new();for t in if reverse{vec!["summary","contiguity"]}else{vec!["contiguity","summary"]}{type_statuses.insert(t.into(),"ok".into());}states.insert(name.into(),StateAnalysisStatus{status:"ok".into(),error:None,type_statuses});}
  AnalysisIndex{label:"fixture".into(),year:"2020".into(),run:"fixture".into(),run_index_sha256:"a".repeat(64),types:vec!["summary".into(),"contiguity".into()],created:"2026-10-07T00:00:00Z".into(),states}
 }
 #[test] fn deterministic_flattened_schema(){let text=serialize_analysis_index(&index(false)).unwrap();assert_eq!(text,serialize_analysis_index(&index(true)).unwrap());let value:serde_json::Value=serde_json::from_str(&text).unwrap();assert_eq!(value["states"]["vermont"]["status"],"ok");assert_eq!(value["states"]["vermont"]["summary"],"ok");assert!(value["states"]["vermont"].get("error").is_none());let decoded:AnalysisIndex=serde_json::from_str(&text).unwrap();assert_eq!(serialize_analysis_index(&decoded).unwrap(),text);}
 #[test] fn failures_preserved(){let mut i=index(false);let state=i.states.get_mut("vermont").unwrap();state.status="failed".into();state.error=Some("Missing source".into());state.type_statuses.insert("summary".into(),"failed".into());let text=serialize_analysis_index(&i).unwrap();assert!(text.contains("Missing source"));assert!(text.contains("failed"));}
 #[test] fn reserved_or_escaping_types_rejected(){for name in ["status","error","../outside","a/b","__proto__"]{assert!(validate_analysis_types(&[name.into()]).is_err());}let mut i=index(false);i.states.get_mut("vermont").unwrap().type_statuses.insert("status".into(),"ok".into());assert!(serialize_analysis_index(&i).is_err());}
 #[test] fn unsafe_index_metadata_rejected(){let mut i=index(false);i.run="different".into();assert!(serialize_analysis_index(&i).is_err());i.run=i.label.clone();i.run_index_sha256="bad".into();assert!(serialize_analysis_index(&i).is_err());}
 #[test] fn report_roundtrip_and_paths(){let mut i=ReportIndex{label:"fixture".into(),year:"2020".into(),analysis_index_sha256:"b".repeat(64),format:"html".into(),created:"2026-10-07T00:00:00Z".into(),files:vec!["fixture_2020.html".into()]};let text=serialize_report_index(&i).unwrap();let parsed:ReportIndex=serde_json::from_str(&text).unwrap();assert_eq!(serialize_report_index(&parsed).unwrap(),text);i.files.push("../outside".into());assert!(serialize_report_index(&i).is_err());}
}
