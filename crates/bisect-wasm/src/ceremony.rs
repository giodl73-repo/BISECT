//! Staged ceremony records; no publication or external receipt authentication.
use crate::certified_exact::{bounded,parse,read_tree};
use bisect_ilp::{CeremonyConfig,CeremonyTranscript};
use serde_json::{json,Value};

fn receipts(values: &[String]) -> Result<(),String> {
    if values.len()>100 || values.iter().any(|v|v.len()>2000) {
        return Err("Ceremony receipts exceed 100 entries or 2000 UTF-8 bytes per entry.".into());
    }
    Ok(())
}
fn config(value: &CeremonyConfig) -> Result<(),String> {
    if !(2..=24).contains(&value.district_count) || value.rule_profile_hash.len()>200
        || value.root_instance_hash.len()>200 || value.minimum_witness_receipts>100 {
        return Err("Ceremony configuration exceeds the browser seat, identity, witness or review bounds.".into());
    }
    receipts(&value.genesis_witness_receipts)
}
fn transcript(text: &str) -> Result<CeremonyTranscript,String> {
    let value: CeremonyTranscript = parse(text)?;
    config(&value.config)?;
    if value.rounds.len()>24 || value.rounds.iter().map(|r|r.cuts.len()).sum::<usize>()>23
        || value.halt_reason.as_ref().is_some_and(|s|s.len()>2000) {
        return Err("Ceremony transcript exceeds the browser round, cut or reason bounds.".into());
    }
    for round in &value.rounds {
        receipts(&round.witness_receipts)?;
        for cut in &round.cuts { bounded(&cut.artifacts.instance)?; }
    }
    Ok(value)
}
fn output(value: &CeremonyTranscript) -> Result<Value,String> {
    Ok(json!({"schema_version":"bisect-ceremony-output-v1","status":value.status,
        "ceremony_id":value.ceremony_id,"transcript_id":value.transcript_id,"districts":value.config.district_count,
        "released_rounds":value.rounds.len(),"released_cuts":value.rounds.iter().map(|r|r.cuts.len()).sum::<usize>(),
        "last_recorded_unix_seconds":value.rounds.last().map(|r|r.published_unix_seconds.to_string()),
        "transcript_json":serde_json::to_string(value).map_err(|e|e.to_string())?,
        "source_boundary":"Local staged transcript: native checks validate split proofs, ordering, hash links and recorded review intervals. Claimed publication times, witness receipts and rule-profile identity are not externally authenticated; this operation publishes nothing."}))
}
pub fn start(tree_text: &str,config_text: &str) -> Result<Value,String> {
    let tree=read_tree(tree_text)?;
    let settings: CeremonyConfig=parse(config_text)?;
    config(&settings)?;
    let value=bisect_ilp::start_ceremony(&tree,settings).map_err(|e|e.to_string())?;
    output(&value)
}
pub fn announce(tree_text: &str,transcript_text: &str,time_text: &str,witness_receipts: Vec<String>) -> Result<Value,String> {
    let tree=read_tree(tree_text)?;
    let mut value=transcript(transcript_text)?;
    let time=time_text.parse::<i64>().map_err(|_|"Recorded timestamp must be a canonical decimal i64.".to_string())?;
    if time.to_string()!=time_text {return Err("Recorded timestamp must be a canonical decimal i64.".into());}
    receipts(&witness_receipts)?;
    bisect_ilp::announce_next_round(&mut value,&tree,time,witness_receipts).map_err(|e|e.to_string())?;
    output(&value)
}
pub fn halt(tree_text: &str,transcript_text: &str,reason: String) -> Result<Value,String> {
    if reason.len()>2000 {return Err("Ceremony halt reason exceeds 2000 UTF-8 bytes.".into());}
    let tree=read_tree(tree_text)?;
    let mut value=transcript(transcript_text)?;
    bisect_ilp::halt_ceremony(&mut value,&tree,reason).map_err(|e|e.to_string())?;
    output(&value)
}
pub fn verify(transcript_text: &str,tree_text: Option<&str>) -> Result<Value,String> {
    verify_scope(transcript_text,tree_text,false)
}
pub fn inspect_prefix(transcript_text: &str) -> Result<Value,String> {
    verify_scope(transcript_text,None,true)
}
fn verify_scope(transcript_text: &str,tree_text: Option<&str>,prefix_inspection:bool) -> Result<Value,String> {
    let value=transcript(transcript_text)?;
    let scope=if let Some(text)=tree_text {
        let tree=read_tree(text)?;
        bisect_ilp::verify_ceremony(&value,&tree).map_err(|e|e.to_string())?;
        "supplied-complete-tree"
    } else {
        bisect_ilp::verify_ceremony_prefix(&value).map_err(|e|e.to_string())?;
        if !prefix_inspection && value.status==bisect_ilp::CeremonyStatus::Completed {
            return Err("completed ceremony verification requires --tree for final binding".into());
        }
        "released-prefix-only"
    };
    Ok(json!({"schema_version":"bisect-ceremony-verification-v1","verified":true,
        "verification_scope":scope,"transcript_id":value.transcript_id,"status":value.status,
        "released_rounds":value.rounds.len(),
        "source_boundary":"Verified native proof/record consistency within the stated scope. External publication, timestamps, witness identities/receipts and legal compliance remain unverified."}))
}
