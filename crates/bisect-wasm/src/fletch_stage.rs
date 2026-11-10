use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    source_name: String,
    source_base64: String,
    dataset_id: String,
    version: Option<String>,
    source_url: String,
    fetched_at_ms: String,
    expected_source_sha256: Option<String>,
    fetch_history: Option<FetchHistory>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FetchHistory {
    attempts: u32,
    retries: u32,
    last_retryable_error: Option<String>,
}
// Pinned FLETCH HTTP FetchPlan key without headers, as used by BISECT Fetch.
pub(crate) fn http_cache_key(dataset_id: &str, version: Option<&str>, source_url: &str) -> String {
    let mut hash=Sha256::new();hash.update(dataset_id.as_bytes());hash.update([0]);
    if let Some(version)=version {hash.update(version.as_bytes());}
    hash.update([0]);hash.update(b"http");hash.update([0]);hash.update(source_url.as_bytes());
    format!("sha256:{:x}",hash.finalize())
}
pub(crate) fn stage(input: Input) -> Result<Value, String> {
    if input.source_name.is_empty() || input.source_name.len()>200
        || input.dataset_id.trim().is_empty() || input.dataset_id.len()>4096
        || input.source_url.trim().is_empty() || input.source_url.len()>16384
        || input.version.as_ref().is_some_and(|v|v.len()>1024) {
        return Err("Invalid bounded FLETCH source identity".into());
    }
    let timestamp=input.fetched_at_ms.parse::<u64>().map_err(|_|"Invalid FLETCH timestamp u64")?;
    if timestamp.to_string()!=input.fetched_at_ms {return Err("FLETCH timestamp requires canonical decimal u64".into());}
    const LIMIT:usize=64*1024*1024;
    if input.source_base64.len()>LIMIT.div_ceil(3)*4 {return Err("FLETCH source exceeds 64 MiB".into());}
    let bytes=STANDARD.decode(&input.source_base64).map_err(|_|"Invalid FLETCH source base64")?;
    if bytes.len()>LIMIT || STANDARD.encode(&bytes)!=input.source_base64 {return Err("Invalid or oversized canonical FLETCH source".into());}
    let digest=format!("{:x}",Sha256::digest(&bytes));
    if input.expected_source_sha256.as_ref().is_some_and(|h|h!=&digest) {return Err("Retained source no longer matches acquisition byte identity".into());}
    let history=input.fetch_history.unwrap_or(FetchHistory {attempts:1,retries:0,last_retryable_error:None});
    if history.last_retryable_error.as_ref().is_some_and(|v|v.len()>16384) {return Err("FLETCH history error exceeds 16 KiB".into());}
    let key=http_cache_key(&input.dataset_id,input.version.as_deref(),&input.source_url);
    let relative_path=format!("objects/sha256/{}",&key[7..]);
    let entry=bisect_core::fletch_cache::ManifestEntry {
        dataset_id:input.dataset_id,version:input.version,source_url:input.source_url,
        cache_key:key.clone(),relative_path:relative_path.clone(),sha256:format!("sha256:{digest}"),
        bytes:bytes.len() as u64,fetched_at_ms:timestamp,verified:true,
        fetch_attempts:history.attempts,retry_count:history.retries,last_retryable_error:history.last_retryable_error,
    };
    let entries_json=serde_json::to_string_pretty(&vec![entry.clone()]).map_err(|e|e.to_string())?;
    let manifest=bisect_core::fletch_cache::Manifest {schema_version:"fletch.cache-manifest.v1".into(),generated_by:"fletch-core/0.1.0".into(),cache_root:"cache".into(),entries:vec![entry]};
    let manifest_json=serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-fletch-cache-staging-v1","entries_json":entries_json,
        "manifest_sha256":format!("{:x}",Sha256::digest(manifest_json.as_bytes())),"manifest_json":manifest_json,
        "dataset_id":manifest.entries[0].dataset_id,"version":manifest.entries[0].version,"source_url":manifest.entries[0].source_url,
        "source_sha256":digest,"source_name":input.source_name,"byte_count_text":bytes.len().to_string(),
        "cache_key":key,"relative_path":relative_path,"fetched_at_ms_text":input.fetched_at_ms,
        "scope":"Supplied raw bytes hashed in WASM and represented as a native FLETCH HTTP cache entry with no headers. URL, version, timestamp and supplied fetch history are selected claims; no request is made. Verified records observed byte identity, not authenticated provenance or provider freshness. No cache object is persisted, installed or activated by this operation."}))
}
