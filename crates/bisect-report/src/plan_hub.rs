//! Native `plan` dispatch arguments and desktop-unavailable fallback text.
use crate::label_registry::LabelEntry;
use std::collections::BTreeMap;
use std::ffi::{OsStr,OsString};
use std::fmt::Write;
pub fn launch_arguments(label:Option<&str>,configure:bool,config:Option<&OsStr>)->Vec<OsString> {
    let mut args=Vec::new();
    if let Some(label)=label {args.extend([OsString::from("--label"),OsString::from(label)]);}
    if configure {args.push("--configure".into());}
    if let Some(config)=config {args.extend([OsString::from("--config"),config.to_owned()]);}
    args
}
pub const MISSING_NOTICE:&str="[INFO] bisect-tui not found. Install with: cargo install --path crates/bisect-tui\n";
pub fn label_index<'a>(labels:impl IntoIterator<Item=&'a str>,target:Option<&str>)->Option<usize>{
    target.and_then(|target|labels.into_iter().position(|label|label==target))
}
pub fn fallback(label:Option<&str>,configure:bool,registry:&Result<BTreeMap<String,LabelEntry>,String>)->String {
    let mut text=String::new();
    if configure {
        text.push_str("[INFO] bisect-tui not available. To edit algorithm config:\n");
        if let Some(label)=label {writeln!(text,"       bisect config validate configs/{label}.yml").unwrap();writeln!(text,"       bisect config new --name {label} --dry-run").unwrap();}
        else {text.push_str("       bisect config new --name <LABEL> --dry-run\n       bisect config validate configs/<LABEL>.yml\n");}
    } else if let Some(label)=label {
        match registry {
            Err(error)=>{writeln!(text,"[INFO] Could not read registry: {error}").unwrap();},
            Ok(registry)=>match registry.get(label) {
                None=>{writeln!(text,"[INFO] Label '{label}' not found in .bisect registry.").unwrap();writeln!(text,"       Run: bisect build {label} --year 2020").unwrap();},
                Some(entry)=>{writeln!(text,"[INFO] bisect-tui not available. Showing label: {label}").unwrap();for (name,years) in [("Built:   ",&entry.built),("Analyzed:",&entry.analyzed),("Reported:",&entry.reported)]{writeln!(text,"  {name} {}",if years.is_empty(){"(none)".into()}else{years.join(", ")}).unwrap();}}
            }
        }
    } else {
        let labels=registry.as_ref().ok().filter(|v|!v.is_empty());
        if let Some(labels)=labels {
            text.push_str("[INFO] bisect-tui not available. Label registry:\n");
            writeln!(text,"  {:<30} {:<20} {:<20} {:<20}","LABEL","BUILT","ANALYZED","REPORTED").unwrap();
            writeln!(text,"  {}","-".repeat(92)).unwrap();
            for (name,entry) in labels {writeln!(text,"  {:<30} {:<20} {:<20} {:<20}",name,entry.built.join(", "),entry.analyzed.join(", "),entry.reported.join(", ")).unwrap();}
        } else {text.push_str("[INFO] No labels found in .bisect registry.\n       Run: bisect build <LABEL> --year 2020\n");}
    }
    text
}
