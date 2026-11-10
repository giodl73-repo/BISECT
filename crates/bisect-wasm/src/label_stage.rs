//! Optional native registry updates on a supplied in-memory snapshot.
use bisect_report::label_registry::{LabelEntry,mark_analyzed,mark_reported};
use std::collections::BTreeMap;
pub(crate) fn promote(source:Option<&str>,label:&str,year:&str,report:bool)->Result<Option<String>,String>{
    let Some(text)=source else{return Ok(None)};
    if text.len()>1024*1024{return Err("Registry exceeds 1 MiB.".into());}
    let mut registry:BTreeMap<String,LabelEntry>=bisect_report::label_registry::parse(text)?;
    if registry.len()>500||registry.iter().any(|(n,e)|n.len()>200||["__proto__","constructor","prototype"].contains(&n.as_str())||e.built.len()>100||e.analyzed.len()>100||e.reported.len()>100){return Err("Registry exceeds browser label/year limits or contains unsafe names.".into());}
    let mut entry=registry.get(label).cloned().unwrap_or_default();
    if report{mark_reported(&mut entry,label,year)?;}else{mark_analyzed(&mut entry,label,year)?;}
    if entry.analyzed.len()>100||entry.reported.len()>100{return Err("Completed registry exceeds 100 years per stage.".into());}
    registry.insert(label.into(),entry);
    let output=serde_json::to_string_pretty(&registry).map_err(|e|e.to_string())?;
    if output.len()>1024*1024{return Err("Completed registry exceeds 1 MiB.".into());}Ok(Some(output))
}
