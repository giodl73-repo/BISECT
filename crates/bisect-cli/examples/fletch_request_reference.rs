fn main()->Result<(),Box<dyn std::error::Error>>{
 use std::path::PathBuf;
 let args=std::env::args().skip(1).collect::<Vec<_>>();let input:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[0])?)?;
 let mut plan=fletch_core::fetch_plan_with_kind(input["dataset_id"].as_str().ok_or("dataset")?,input["source_url"].as_str().ok_or("url")?,fletch_core::SourceKind::Http)?;
 plan.cache_policy.freshness=match input["freshness"].as_str().unwrap_or("immutable") {
 "immutable"=>fletch_core::FreshnessPolicy::Immutable,
 "always-check"=>fletch_core::FreshnessPolicy::AlwaysCheck,
 "max-age-days"=>fletch_core::FreshnessPolicy::MaxAgeDays(u32::try_from(input["max_age_days"].as_u64().ok_or("days")?)?),
 _=>return Err("invalid policy".into()),
 };
 plan.version=input["version"].as_str().map(str::to_owned);let key=fletch_core::cache_key(&plan);let destination=PathBuf::from(&args[1]).join("objects").join("sha256").join(&key[7..]);let mut modified_at_ms=None;
 if args[2]!="none"{std::fs::create_dir_all(destination.parent().ok_or("parent")?)?;std::fs::copy(&args[2],&destination)?;if let Some(time)=input["object_modified_at_ms"].as_str(){let modified=std::time::UNIX_EPOCH+std::time::Duration::from_millis(time.parse()?);std::fs::File::options().write(true).open(&destination)?.set_times(std::fs::FileTimes::new().set_modified(modified))?;}
 modified_at_ms=Some(destination.metadata()?.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_millis().to_string());}
 let mut options=fletch_core::FetchOptions::new(&args[1]).with_force(input["force"].as_bool().ok_or("force")?).with_offline(input["offline"].as_bool().ok_or("offline")?);
 options.trusted_entries=serde_json::from_str(input["trusted_entries_json"].as_str().ok_or("trusted")?)?;
 options.expected_sha256=input["expected_sha256"].as_str().map(str::to_owned);options.fetched_at_ms=input["fetched_at_ms"].as_str().map(str::parse::<u64>).transpose()?;
 let output=match fletch_core::fetch_to_cache(&plan,options){Ok(outcome)=>serde_json::json!({"entries_json":serde_json::to_string_pretty(&vec![outcome.entry])?,"attempt_status":outcome.attempt_status,"modified_at_ms":modified_at_ms}),Err(error)=>serde_json::json!({"error":error.to_string(),"modified_at_ms":modified_at_ms})};print!("{}",serde_json::to_string(&output)?);Ok(())
}
