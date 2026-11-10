//! Actual filesystem-backed annotation reader and native report writer reference.
use base64::Engine;
use bisect_analysis::{BlocVotingConfig,ProvenanceBlock,WriteContext};
use serde_json::{json,Value};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();let req:Value=serde_json::from_slice(&std::fs::read(args.get(1).ok_or("request path required")?)?)?;
 let root=std::path::Path::new(args.get(2).ok_or("fixture root required")?);std::fs::create_dir_all(root)?;
 let name=req["annotation_source_name"].as_str().ok_or("annotation name required")?;
 if name.contains(['/', '\\', ':']){return Err("unsafe annotation filename".into());}
 let csv=root.join(name);std::fs::write(&csv,req["annotation_csv"].as_str().ok_or("annotation CSV required")?)?;
 for (path,text)in req["attestation_files"].as_object().ok_or("documents required")?{
  if path.split('/').any(|p|p.is_empty()||p==".."||p==".")||path.contains(['\\', ':']){return Err("unsafe fixture path".into());}
  let target=root.join(path);std::fs::create_dir_all(target.parent().ok_or("invalid fixture path")?)?;
  std::fs::write(target,base64::engine::general_purpose::STANDARD.decode(text.as_str().ok_or("base64 required")?)?)?;
 }
 let source=root.join("precincts.tsv");std::fs::write(&source,req["precincts_tsv"].as_str().ok_or("precinct TSV required")?)?;
 let s=&req["settings"];let annotations=bisect_analysis::parse_race_of_candidate_csv(&csv)?;
 let tests=bisect_analysis::bloc_voting_input::read_precinct_family(std::fs::File::open(source)?,s["min_precincts"].as_u64().ok_or("threshold required")? as usize)?;
 let cfg=BlocVotingConfig{bootstrap_samples:s["bootstrap_samples"].as_u64().ok_or("replicates required")? as usize,bootstrap_seed:s["bootstrap_seed"].as_str().ok_or("seed required")?.parse()?,ci_level:s["ci_level"].as_f64().ok_or("confidence required")?,alpha:s["alpha"].as_f64().ok_or("alpha required")?,provenance:Some(annotations.provenance.clone()),..Default::default()};
 let family=bisect_analysis::run_bloc_voting_family(&tests,&cfg)?;
 let provenance=ProvenanceBlock{version:env!("CARGO_PKG_VERSION").into(),build_commit:"unknown (not embedded in this boundary)".into(),build_commit_short:None,rustc_version:"not embedded in this boundary".into(),args:Some(json!({"bootstrap_seed":s["bootstrap_seed"],"bootstrap_samples":s["bootstrap_samples"],"ci_level":s["ci_level"],"alpha":s["alpha"],"min_precincts":s["min_precincts"],"method":"wls"}))};
 let year=s["year"].to_string();let ctx=WriteContext{state:s["state"].as_str().ok_or("state required")?,year:&year,election:s["election"].as_str().ok_or("election required")?,party:s["party"].as_str().ok_or("party required")?,method:"wls",minority_group:s["minority_group"].as_str().ok_or("composition required")?,alpha:cfg.alpha,bootstrap_samples:cfg.bootstrap_samples,provenance:&provenance,race_provenance:&annotations.provenance};
 let report=bisect_analysis::build_bloc_voting_json(&family,&ctx);bisect_analysis::write_bloc_voting_outputs(&report,&root.join("reports"))?;Ok(())
}
