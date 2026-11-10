//! Portable native registry entry schema. Stage records are claims, not verification.
use serde::{Deserialize,Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LabelEntry {pub built:Vec<String>,pub analyzed:Vec<String>,pub reported:Vec<String>}

/// Native stage mutations preserve insertion order and existing duplicate records.
/// These record command completion; they do not assert successful analysis of every state.
pub fn mark_analyzed(entry:&mut LabelEntry,label:&str,year:&str)->Result<(),String>{
    if !entry.built.iter().any(|y|y==year){return Err(format!("[CONFIG] analyze: '{label}' has not been built for year {year}.\nRun: bisect build {label} --year {year}"));}
    if !entry.analyzed.iter().any(|y|y==year){entry.analyzed.push(year.into());}Ok(())
}
pub fn mark_reported(entry:&mut LabelEntry,label:&str,year:&str)->Result<(),String>{
    if !entry.analyzed.iter().any(|y|y==year){return Err(format!("[CONFIG] report: '{label}' has not been analyzed for year {year}.\nRun: bisect analyze {label} --year {year}"));}
    if !entry.reported.iter().any(|y|y==year){entry.reported.push(year.into());}Ok(())
}

/// Reapply only the verified native stage transition onto a newer registry.
/// Callers must separately bind the generated index to the current upstream artifact.
pub fn reconcile_stage(
    source:&std::collections::BTreeMap<String,LabelEntry>,
    current:&std::collections::BTreeMap<String,LabelEntry>,
    proposed:&std::collections::BTreeMap<String,LabelEntry>,
    label:&str,year:&str,reported:bool,
)->Result<std::collections::BTreeMap<String,LabelEntry>,String>{
    let mark=|entry:&mut LabelEntry|if reported{mark_reported(entry,label,year)}else{mark_analyzed(entry,label,year)};
    let mut expected=source.clone();mark(expected.entry(label.into()).or_default())?;
    if &expected!=proposed{return Err("Proposed registry includes changes beyond the selected native stage transition.".into());}
    let mut merged=current.clone();mark(merged.entry(label.into()).or_default())?;Ok(merged)
}

#[cfg(test)]
mod stage_tests {
    use super::*;
    #[test] fn prerequisites_and_native_errors(){let mut e=LabelEntry::default();assert_eq!(mark_analyzed(&mut e,"fixture","2020").unwrap_err(),"[CONFIG] analyze: 'fixture' has not been built for year 2020.\nRun: bisect build fixture --year 2020");assert!(e.analyzed.is_empty());assert!(mark_reported(&mut e,"fixture","2020").is_err());assert!(e.reported.is_empty());}
    #[test] fn order_duplicates_and_idempotence(){let mut e=LabelEntry{built:vec!["2010".into(),"2020".into(),"2020".into()],analyzed:vec!["2010".into()],reported:vec![]};mark_analyzed(&mut e,"fixture","2020").unwrap();mark_analyzed(&mut e,"fixture","2020").unwrap();mark_reported(&mut e,"fixture","2020").unwrap();mark_reported(&mut e,"fixture","2020").unwrap();assert_eq!(e.built,vec!["2010","2020","2020"]);assert_eq!(e.analyzed,vec!["2010","2020"]);assert_eq!(e.reported,vec!["2020"]);}
    #[test] fn matches_local_native_prerequisite(){let mut e=LabelEntry{built:vec![],analyzed:vec!["2020".into()],reported:vec![]};mark_reported(&mut e,"fixture","2020").unwrap();assert_eq!(e.reported,vec!["2020"]);}
    #[test] fn reconcile_preserves_concurrent_changes(){
        let source=std::collections::BTreeMap::from([("fixture".into(),LabelEntry{built:vec!["2020".into()],analyzed:vec![],reported:vec![]})]);
        let mut proposed=source.clone();mark_analyzed(proposed.get_mut("fixture").unwrap(),"fixture","2020").unwrap();
        let mut current=source.clone();current.get_mut("fixture").unwrap().built.push("2010".into());current.insert("new_label".into(),LabelEntry::default());
        let merged=reconcile_stage(&source,&current,&proposed,"fixture","2020",false).unwrap();assert_eq!(merged["fixture"].built,vec!["2020","2010"]);assert!(merged.contains_key("new_label"));assert_eq!(merged["fixture"].analyzed,vec!["2020"]);assert_eq!(reconcile_stage(&source,&merged,&proposed,"fixture","2020",false).unwrap(),merged);
    }
    #[test] fn reconcile_rejects_nonstage_edits_and_missing_current_prerequisite(){
        let source=std::collections::BTreeMap::from([("fixture".into(),LabelEntry{built:vec!["2020".into()],analyzed:vec![],reported:vec![]})]);let mut proposed=source.clone();mark_analyzed(proposed.get_mut("fixture").unwrap(),"fixture","2020").unwrap();
        let mut current=source.clone();current.get_mut("fixture").unwrap().built.clear();assert!(reconcile_stage(&source,&current,&proposed,"fixture","2020",false).is_err());
        proposed.get_mut("fixture").unwrap().built.push("2010".into());assert!(reconcile_stage(&source,&source,&proposed,"fixture","2020",false).is_err());
    }
}

pub fn parse(text:&str)->Result<std::collections::BTreeMap<String,LabelEntry>,String>{
    if text.trim().is_empty(){return Ok(Default::default());}
    serde_json::from_str(text).map_err(|e|format!("failed to parse .bisect: {e}"))
}

pub fn render_list(labels:&[(String,LabelEntry)],json:bool)->Result<String,String>{
    use std::fmt::Write;
    if json {
        let map:serde_json::Map<String,serde_json::Value>=labels.iter().map(|(name,entry)|(name.clone(),serde_json::json!({"built":entry.built,"analyzed":entry.analyzed,"reported":entry.reported}))).collect();
        return serde_json::to_string_pretty(&serde_json::Value::Object(map)).map(|s|s+"\n").map_err(|e|format!("failed to serialize registry: {e}"));
    }
    if labels.is_empty(){return Ok("(no labels in registry)\n".into());}
    let col=labels.iter().map(|(n,_)|n.len()).max().unwrap_or(0).max(8);
    let mut text=String::new();
    writeln!(text,"{:<col$}   {:<22}  {:<16}  {}","LABEL","built","analyzed","reported").unwrap();
    writeln!(text,"{}","-".repeat(col+3+22+2+16+2+10)).unwrap();
    for(name,e)in labels{let stage=|v:&Vec<String>|if v.is_empty(){"--".into()}else{v.join(" ")};writeln!(text,"{:<col$}   {:<22}  {:<16}  {}",name,stage(&e.built),stage(&e.analyzed),stage(&e.reported)).unwrap();}
    Ok(text)
}

pub fn render_show(label:&str,entry:&LabelEntry,json:bool)->Result<String,String>{
    use std::{fmt::Write,path::PathBuf};
    let runs=PathBuf::from("runs").join(label);let analysis=PathBuf::from("analysis").join(label);let reports=PathBuf::from("reports").join(label);
    let mut years:Vec<&str>=entry.built.iter().chain(entry.analyzed.iter()).chain(entry.reported.iter()).map(String::as_str).collect();years.sort_unstable();years.dedup();
    if json {
        let status:std::collections::BTreeMap<_,_>=years.iter().map(|y|(*y,serde_json::json!({"built":entry.built.iter().any(|s|s==y),"analyzed":entry.analyzed.iter().any(|s|s==y),"reported":entry.reported.iter().any(|s|s==y)}))).collect();
        let value=serde_json::json!({"label":label,"years":status,"paths":{"runs":runs.to_string_lossy(),"analysis":analysis.to_string_lossy(),"reports":reports.to_string_lossy()}});
        return serde_json::to_string_pretty(&value).map(|s|s+"\n").map_err(|e|format!("failed to serialize show output: {e}"));
    }
    let mut text=format!("Label:    {label}\n\nPaths:\n  runs:     {}\n  analysis: {}\n  reports:  {}\n\n",runs.display(),analysis.display(),reports.display());
    if years.is_empty(){text.push_str("Years:    (none)\n");}else{text.push_str("Years:\n");for y in years{let b=if entry.built.iter().any(|s|s==y){"[built]"}else{"      "};let a=if entry.analyzed.iter().any(|s|s==y){"[analyzed]"}else{"          "};let r=if entry.reported.iter().any(|s|s==y){"[reported]"}else{"          "};writeln!(text,"  {y}  {b}  {a}  {r}").unwrap();}}
    Ok(text)
}

pub fn mark_built(entry:&mut LabelEntry,year:&str){if !entry.built.iter().any(|y|y==year){entry.built.push(year.into());}}
