//! Exercise actual filesystem-backed civic ingest and conflict CLI entry points.
use bisect_cli::civic::*;
fn main()->anyhow::Result<()>{
 let a:Vec<String>=std::env::args().collect();
 let cmd=if a[1]=="show"{CivicSubcommand::Show(CivicShowArgs{label:a[3].clone(),output_base:a[2].clone(),version:"v1".into()})}else if a[1]=="list"{CivicSubcommand::List(CivicListArgs{output_base:a[2].clone(),version:"v1".into()})}else if a[1]=="conflicts"{CivicSubcommand::Conflicts(CivicConflictsArgs{label:a[3..].to_vec(),output_base:a[2].clone(),version:"v1".into(),race_conflict_threshold:0.10})}else{CivicSubcommand::Ingest(CivicIngestArgs{csv:a[2].clone().into(),label:a[3].clone(),state:a[4].clone(),year:a[5].clone(),validate:a[6].clone(),submitter:a.get(8).cloned(),output_base:a[7].clone(),version:"v1".into(),snapshot_urls:a.get(9).is_some_and(|s|s=="--snapshot-urls")})};
 run_civic(&cmd)
}
