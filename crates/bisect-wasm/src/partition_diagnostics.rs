//! Observe supplied labeled partition traces using the shared native diagnostics.
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet,HashSet};
#[derive(Deserialize)]
#[serde(untagged)]
enum Label {Number(u64),Text(String)}
impl Label {fn value(&self)->Result<u64,String>{match self {Self::Number(v)=>Ok(*v),Self::Text(s)=>{if s.is_empty()||s.len()>20||!s.bytes().all(|b|b.is_ascii_digit())||(s.len()>1&&s.starts_with('0')){return Err("Labels require unsigned integers or canonical decimal u64 strings.".into());}s.parse().map_err(|_|"Label exceeds u64.".into())}}}}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Traces {schema_version:String,unit_ids:Vec<String>,chains:Vec<Vec<Vec<Label>>>}
pub(crate) fn execute(text:&str,max_lag:usize)->Result<Value,String>{
 if text.len()>3*1024*1024||max_lag>100000{return Err("Partition diagnostics support at most 3 MiB and 100,000 requested lags.".into());}
 let input:Traces=serde_json::from_str(text).map_err(|e|e.to_string())?;let n=input.unit_ids.len();
 if input.schema_version!="bisect-partition-traces-v1"||n==0||n>100000||input.chains.is_empty()||input.chains.len()>32{return Err("Supply 1–32 partition chains and 1–100,000 ordered units.".into());}
 let mut ids=HashSet::new();if input.unit_ids.iter().any(|id|id.is_empty()||id.len()>200||id.chars().any(char::is_control)||!ids.insert(id)){return Err("Unit IDs must be unique nonempty strings without controls, at most 200 UTF-8 bytes.".into());}
 let mut work=0usize;let mut values=Vec::new();let mut unique=BTreeSet::new();
 for chain in input.chains {
  if chain.is_empty()||chain.len()>5000{return Err("Supply 1–5,000 partitions per chain.".into());}
  let lag=max_lag.min(chain.len()-1);let comparisons=(0..=lag).map(|k|chain.len()-k).sum::<usize>();work=work.saturating_add(comparisons.saturating_mul(n));if work>200000000{return Err("Partition diagnostics exceed 200 million label comparisons.".into());}
  let mut rows=Vec::new();for row in chain {if row.len()!=n{return Err("Every partition must cover the exact ordered unit universe.".into());}let labels=row.iter().map(Label::value).collect::<Result<Vec<_>,_>>()?;unique.extend(labels.iter().copied());rows.push(labels);}values.push(rows);
 }
 // A global bijection preserves all equality comparisons and full u64 labels
 // while using the same usize native kernel on both 32- and 64-bit targets.
 let lookup:BTreeMap<_,_>=unique.into_iter().enumerate().map(|(i,v)|(v,i)).collect();
 let compact:Vec<Vec<Vec<usize>>>=values.iter().map(|chain|chain.iter().map(|row|row.iter().map(|v|lookup[v]).collect()).collect()).collect();
 let records=bisect_analysis::ensemble_diagnostics::hamming_records(&compact,max_lag).map_err(|e|e.to_string())?;
 let report_json=serde_json::to_string_pretty(&records).map_err(|e|e.to_string())?;
 Ok(json!({"schema_version":"bisect-partition-diagnostics-v1","source_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"unit_count":n,"chain_lengths":compact.iter().map(Vec::len).collect::<Vec<_>>(),"max_lag_requested":max_lag,"records":records,"report_json":report_json,"label_policy":"global-equality-preserving-u64-index","scope":"Native mean fraction of units whose district labels differ at each lag, and native integrated time computed from one minus that distance. District renumbering changes distances; unit order must be shared by all partitions. This does not certify valid partitions, independent sampling, convergence, optimality or legal correctness."}))
}
