//! Complete supplied-family WLS/HC3/cluster-bootstrap reports and annotation-byte binding.
use base64::Engine;
use bisect_analysis::{BlocVotingConfig,ProvenanceBlock,WriteContext};
use bisect_analysis::race_of_candidate::{parse_race_of_candidate_source,RaceParseError};
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub(crate) struct Settings {
 pub state:String,pub year:u16,pub election:String,pub party:String,pub minority_group:String,
 pub bootstrap_seed:String,pub bootstrap_samples:usize,pub ci_level:f64,pub alpha:f64,pub min_precincts:usize,
}
pub(crate) fn analyze(precincts:&str,annotations:&str,annotation_name:&str,encoded_documents:BTreeMap<String,String>,s:Settings)->Result<Value,String>{
 if s.state.len()!=2||!s.state.bytes().all(|c|c.is_ascii_uppercase())||!(1900..=2100).contains(&s.year){return Err("Supply a two-letter uppercase state and year 1900–2100.".into());}
 for value in [&s.election,&s.party,&s.minority_group]{if value.trim().is_empty()||value.len()>200||value.chars().any(char::is_control){return Err("Run labels must be nonempty and at most 200 bytes with no control characters.".into());}}
 if annotation_name.is_empty()||annotation_name.len()>200||annotation_name.contains(['/', '\\', ':'])||annotation_name.chars().any(char::is_control){return Err("Annotation source name must be a plain filename of 1–200 bytes.".into());}
 let seed=s.bootstrap_seed.parse::<u64>().map_err(|_|"Supply a canonical unsigned 64-bit bootstrap seed.")?;
 if seed.to_string()!=s.bootstrap_seed{return Err("Bootstrap seed must be a canonical decimal string.".into());}
 if !(1..=10_000).contains(&s.bootstrap_samples)||!(4..=100_000).contains(&s.min_precincts)||!s.ci_level.is_finite()||!(0.0<s.ci_level&&s.ci_level<1.0)||!s.alpha.is_finite()||!(0.0<s.alpha&&s.alpha<1.0){return Err("Use 1–10,000 replicates, at least four precincts, and confidence/alpha strictly between zero and one.".into());}
 if precincts.len().saturating_add(annotations.len())>3*1024*1024{return Err("Precinct TSV and annotation CSV must total at most 3 MiB.".into());}
 if encoded_documents.len()>256 || encoded_documents.values().map(String::len).sum::<usize>()>12*1024*1024 {return Err("Encoded attestation sources exceed the browser limit.".into());}
 let documents:BTreeMap<String,Vec<u8>>=encoded_documents.into_iter().map(|(name,text)|base64::engine::general_purpose::STANDARD.decode(text).map(|bytes|(name,bytes)).map_err(|_|"Invalid attestation base64 encoding.".to_string())).collect::<Result<_,_>>()?;
 if documents.contains_key(annotation_name){return Err("Annotation CSV filename collides with an attestation path.".into());}
 if documents.is_empty()||documents.len()>256||documents.values().map(Vec::len).sum::<usize>()>8*1024*1024{return Err("Supply 1–256 attestation documents totalling at most 8 MiB.".into());}
 for path in documents.keys(){if path.len()>240||!path.bytes().all(|b|b.is_ascii_alphanumeric()||b"_./ -".contains(&b))||path.split('/').any(|p|p.is_empty()||[".","..","__proto__","constructor","prototype"].contains(&p)){return Err("Attestation package paths must be safe relative slash-separated paths.".into());}}
 let annotation_set=parse_race_of_candidate_source(annotations.as_bytes(),annotation_name,|path|{
    let key=path.to_string_lossy().replace('\\',"/");
    documents.get(&key).map(|bytes|format!("{:x}",Sha256::digest(bytes))).ok_or_else(||RaceParseError::AttestationDocMissing{row:0,path:key})
 }).map_err(|e|e.to_string())?;
 if annotation_set.annotations.is_empty()||annotation_set.annotations.len()>2000{return Err("Supply 1–2,000 candidate annotations.".into());}
 let tests=bisect_analysis::bloc_voting_input::read_precinct_family(precincts.as_bytes(),s.min_precincts).map_err(|e|e.to_string())?;
 bisect_analysis::bloc_voting_input::validate_annotation_coverage(&tests,&annotation_set,&s.party).map_err(|e|e.to_string())?;
 if tests.len()>128||tests.iter().map(|t|t.precincts.len()).sum::<usize>()>100_000{return Err("Bloc analysis supports at most 128 candidate/variant tests and 100,000 precinct rows.".into());}
 let mut rows_work=0usize;
 for t in &tests {
    let mut counts=BTreeMap::new();for (i,p) in t.precincts.iter().enumerate(){
        if p.total_votes>1e12{return Err("Browser vote weights must not exceed 1e12 per precinct.".into());}
        *counts.entry(if p.county_fips.is_empty(){format!("singleton-{i}")}else{p.county_fips.clone()}).or_insert(0usize)+=1;
    }
    let expanded=counts.len().checked_mul(*counts.values().max().unwrap_or(&0)).ok_or("Bootstrap work overflow.")?;
    rows_work=rows_work.checked_add((expanded+t.precincts.len()).checked_mul(s.bootstrap_samples).ok_or("Bootstrap work overflow.")?).ok_or("Bootstrap work overflow.")?;
 }
 if rows_work>20_000_000{return Err("Bloc bootstrap exceeds the 20-million resampled-row work budget.".into());}
 let cfg=BlocVotingConfig{bootstrap_seed:seed,bootstrap_samples:s.bootstrap_samples,ci_level:s.ci_level,alpha:s.alpha,provenance:Some(annotation_set.provenance.clone()),..Default::default()};
 let family=bisect_analysis::run_bloc_voting_family(&tests,&cfg).map_err(|e|e.to_string())?;
 let provenance=ProvenanceBlock{version:env!("CARGO_PKG_VERSION").into(),build_commit:"unknown (not embedded in this boundary)".into(),build_commit_short:None,rustc_version:"not embedded in this boundary".into(),args:Some(json!({"bootstrap_seed":s.bootstrap_seed,"bootstrap_samples":s.bootstrap_samples,"ci_level":s.ci_level,"alpha":s.alpha,"min_precincts":s.min_precincts,"method":"wls"}))};
 let year=s.year.to_string();let ctx=WriteContext{state:&s.state,year:&year,election:&s.election,party:&s.party,method:"wls",minority_group:&s.minority_group,alpha:s.alpha,bootstrap_samples:s.bootstrap_samples,provenance:&provenance,race_provenance:&annotation_set.provenance};
 let report=bisect_analysis::build_bloc_voting_json(&family,&ctx);
 let files=BTreeMap::from([("bloc_voting.json",serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?),("bloc_voting_summary.md",bisect_analysis::render_summary_md(&report))]);
 let hashes:BTreeMap<_,_>=files.iter().map(|(n,text)|(*n,format!("{:x}",Sha256::digest(text.as_bytes())))).collect();
 let document_hashes:BTreeMap<_,_>=documents.iter().map(|(n,bytes)|(n.clone(),format!("{:x}",Sha256::digest(bytes)))).collect();
 let used:BTreeSet<_>=annotation_set.annotations.iter().map(|a|a.attestation_doc_path.to_string_lossy().replace('\\',"/")).collect();
 let mut disputes:Vec<_>=annotation_set.disputes.iter().map(|((candidate,party),curators)|json!({"candidate":candidate,"party":party,"curators":curators})).collect();disputes.sort_by_key(|d|d.to_string());
 Ok(json!({"schema_version":"bisect-bloc-voting-browser-v1","report":report,"holm_family_size":family.holm_family_size,"files":files,"file_sha256":hashes,
 "precincts_sha256":format!("{:x}",Sha256::digest(precincts.as_bytes())),"annotations_sha256":format!("{:x}",Sha256::digest(annotations.as_bytes())),"attestation_sha256":document_hashes,"unused_attestation_paths":documents.keys().filter(|n|!used.contains(*n)).collect::<Vec<_>>(),"annotation_disputes":disputes,"resampled_row_work_bound":rows_work,
 "scope":"Shared native supplied-family WLS with vote-count weights, HC3 standard errors, VIF, county-cluster and naive percentile bootstrap, joint Holm correction and native draft JSON/Markdown writers. Seed, replicates, configured confidence and the portable ChaCha12/u64-index protocol are recorded; this changes legacy SmallRng intervals. Regression p-values use the native normal-CDF approximation, not an exact finite-sample Student t calculation. Native ci_95_cluster field holds the configured confidence interval even when confidence differs from 0.95. Optional TSV variant rows materialize robustness/leave-one-out analyses; none are generated automatically. Annotation independent-verification flags are supplied declarations, not authentication. Selected attestation bytes are hashed without rendering or executing them; filename extensions are checked, not file signatures. Classification disputes are disclosed but do not cause automatic alternative regressions. Minority group is a recorded label; pct_minority must already contain the corresponding supplied composition. These are per-precinct associations, not individual-voter conclusions, election predictions or legal certification. Full-detail large-family work beyond the bounded browser budget remains unsupported."}))
}
