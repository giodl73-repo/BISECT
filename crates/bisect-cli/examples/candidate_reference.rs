fn main()->anyhow::Result<()>{
 let a:Vec<String>=std::env::args().collect();
 bisect_cli::civic::run_civic(&bisect_cli::civic::CivicSubcommand::AddCandidateRace(bisect_cli::civic::CivicAddCandidateRaceArgs{csv:a[1].clone().into(),attestation_doc:a[2].clone().into(),label:Some(a[3].clone()),state:a[4].clone(),year:a[5].clone(),submitter:a[6].clone(),output_base:a[7].clone(),version:"v1".into()}))
}
