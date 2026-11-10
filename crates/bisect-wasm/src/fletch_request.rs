use serde::Deserialize;
use serde_json::{json,Value};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Observation {sha256:String,bytes:String,modified_at_ms:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {dataset_id:String,version:Option<String>,source_url:String,force:bool,offline:bool,object_exists:bool,observation:Option<Observation>,trusted_entries_json:String,expected_sha256:Option<String>,fetched_at_ms:Option<String>,freshness:Option<String>,max_age_days:Option<u32>,now_ms:Option<String>,object_modified_at_ms:Option<String>}
fn integer(text:&str)->Result<u64,String>{let n=text.parse::<u64>().map_err(|_|"Invalid exact cache-request u64")?;if n.to_string()!=text{return Err("Cache-request u64 must be canonical decimal".into());}Ok(n)}
/// Pinned fetch_to_cache no-header HTTP branch and cache_hit_outcome.
/// Observations are supplied by the host after hashing the actual selected body.
pub(crate) fn resolve(input:Input)->Result<Value,String>{
 if input.dataset_id.trim().is_empty()||input.dataset_id.len()>4096||input.source_url.trim().is_empty()||input.source_url.len()>16384||input.version.as_ref().is_some_and(|v|v.len()>1024)||input.trusted_entries_json.len()>8*1024*1024{return Err("Invalid bounded immutable cache request".into());}
 let key=crate::fletch_stage::http_cache_key(&input.dataset_id,input.version.as_deref(),&input.source_url);let relative_path=format!("objects/sha256/{}",&key[7..]);
 let requested_timestamp=input.fetched_at_ms.as_deref().map(integer).transpose()?;
 let policy=input.freshness.as_deref().unwrap_or("immutable");
 if !["immutable","always-check","max-age-days"].contains(&policy)||policy!="max-age-days"&&input.max_age_days.is_some(){return Err("Invalid cache freshness policy".into());}
 let fresh=match policy {
  "immutable"=>true,
  "always-check"=>false,
  _=>{let days=input.max_age_days.ok_or("Maximum-age policy requires u32 days")?;
   if input.object_exists&&!input.force {let now=integer(input.now_ms.as_deref().ok_or("Maximum-age policy requires exact current time")?)?;let modified=integer(input.object_modified_at_ms.as_deref().ok_or("Maximum-age policy requires object storage time")?)?;now.saturating_sub(modified)<=u64::from(days)*86_400_000}else{false}}
 };
 if input.force||!input.object_exists||!fresh {
  if input.offline {return Err(if input.object_exists{format!("[OFFLINE] cache entry {} exists at {relative_path} but is stale or bypassed and live fetches are disabled",input.dataset_id)}else{format!("[OFFLINE] cache entry {} is missing and live fetches are disabled",input.dataset_id)});}
  return Ok(json!({"schema_version":"bisect-fletch-cache-request-v1","decision":"fetch","cache_key":key,"relative_path":relative_path,"scope":"Selected no-header HTTP cache policy requires acquisition. No network operation was performed."}));
 }
 let observed=input.observation.ok_or("Cache hit requires freshly observed body identity")?;
 if observed.sha256.len()!=71||!observed.sha256.starts_with("sha256:")||!observed.sha256[7..].bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err("Invalid observed cache SHA256".into());}
 let bytes=integer(&observed.bytes)?;let modified_at_ms=integer(&observed.modified_at_ms)?;
 let trusted:Vec<bisect_core::fletch_cache::ManifestEntry>=serde_json::from_str(&input.trusted_entries_json).map_err(|e|e.to_string())?;if trusted.len()>2000{return Err("Too many trusted cache entries".into());}
 let trusted=trusted.iter().find(|e|e.cache_key==key&&e.dataset_id==input.dataset_id&&e.source_url==input.source_url);
 let mismatch=|expected:&str|format!("[VERIFY] checksum mismatch for {}: expected {}, got {}",input.dataset_id,expected,observed.sha256);
 if let Some(entry)=trusted {if entry.sha256!=observed.sha256||entry.bytes!=bytes{return Err(mismatch(&entry.sha256));}}
 if let Some(expected)=&input.expected_sha256 {if *expected!=observed.sha256{return Err(mismatch(expected));}}
 let timestamp=trusted.map(|e|e.fetched_at_ms).or(requested_timestamp).unwrap_or(modified_at_ms);
 let entry=bisect_core::fletch_cache::ManifestEntry{dataset_id:input.dataset_id,version:input.version,source_url:input.source_url,cache_key:key.clone(),relative_path:relative_path.clone(),sha256:observed.sha256.clone(),bytes,fetched_at_ms:timestamp,verified:trusted.is_some_and(|e|e.verified)||input.expected_sha256.is_some(),fetch_attempts:trusted.map(|e|e.fetch_attempts).unwrap_or_default(),retry_count:trusted.map(|e|e.retry_count).unwrap_or_default(),last_retryable_error:trusted.and_then(|e|e.last_retryable_error.clone())};
 Ok(json!({"schema_version":"bisect-fletch-cache-request-v1","decision":"cache-hit","cache_key":key,"relative_path":relative_path,"entries_json":serde_json::to_string_pretty(&vec![entry]).map_err(|e|e.to_string())?,"attempt_status":{"attempts":0,"retries":0,"last_retryable_error":null},"source_sha256":&observed.sha256[7..],"byte_count_text":bytes.to_string(),"fetched_at_ms_text":timestamp.to_string(),"scope":"Selected no-header HTTP cache policy permits reuse over freshly observed body SHA256/size. Maximum age uses supplied host object storage/current times at millisecond precision. Native trusted-entry and expected-digest rules determine verified metadata; historical retry fields and timestamp are preserved. This attempt makes zero provider requests; source authentication and freshness against providers are not established."}))
}
