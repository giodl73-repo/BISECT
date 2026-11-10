//! Native Doctor interpretation of supplied analysis artifacts. No filesystem access.
use std::collections::BTreeMap;
use serde_json::{json,Value};
pub const REQUIRED: [&str;3]=["summary.json","compactness.json","contiguity.json"];
pub const OPTIONAL: [&str;6]=["splits.json","demographic.json","partisan.json","vra_analysis.json","comparison.json","nesting.json"];
pub fn analysis_summary(name:&str,content:&str)->String {
 let Ok(v):Result<Value,_>=serde_json::from_str(content) else{return String::new();};
    match name {
        "summary.json" => {
            let valid = v["population_balance_valid"].as_bool().unwrap_or(false);
            let n = v["districts"].as_array().map(|a| a.len()).unwrap_or(0);
            let balance_context = if valid && n > 1 {
                " (within tolerance ✓)"
            } else if !valid {
                " (⚠ EXCEEDS TOLERANCE)"
            } else {
                ""
            };
            format!(
                "({} districts, balance {}{})",
                n,
                if valid { "valid" } else { "FAILED" },
                balance_context
            )
        }
        "compactness.json" => {
            let districts = v["districts"].as_array();
            if let Some(d) = districts {
                let pp_vals: Vec<f64> = d
                    .iter()
                    .filter_map(|x| x["polsby_popper"].as_f64())
                    .collect();
                if !pp_vals.is_empty() {
                    let mean = pp_vals.iter().sum::<f64>() / pp_vals.len() as f64;
                    let context = if mean < 0.15 {
                        " (low — may face legal challenge)"
                    } else if mean < 0.25 {
                        " (moderate)"
                    } else if mean < 0.35 {
                        " (good)"
                    } else {
                        " (well-compact)"
                    };
                    return format!("(mean PP: {:.3}{})", mean, context);
                }
            }
            String::new()
        }
        "contiguity.json" => {
            let all = v["all_contiguous"].as_bool().unwrap_or(false);
            format!(
                "({})",
                if all {
                    "all contiguous"
                } else {
                    "NON-CONTIGUOUS districts found"
                }
            )
        }
        _ => String::new(),
    }
}
/// Native --all table cells. Presence and parseability are deliberately separate.
pub fn scan_metrics(files:&BTreeMap<String,Option<String>>)->Value {
 let parsed=|name:&str|files.get(name).and_then(|t|t.as_deref()).and_then(|s|serde_json::from_str::<Value>(s).ok());
 let balance=if files.contains_key("summary.json"){parsed("summary.json").and_then(|v|v["population_balance_valid"].as_bool()).map(|b|if b{"OK"}else{"FAIL"}).unwrap_or("?")}else{"-"};
 let pp=parsed("compactness.json").and_then(|v|{let d=v["districts"].as_array()?;let vals:Vec<f64>=d.iter().filter_map(|x|x["polsby_popper"].as_f64()).collect();if vals.is_empty(){None}else{Some(format!("{:.3}",vals.iter().sum::<f64>()/vals.len() as f64))}}).unwrap_or_else(||"-".into());
 let splits=parsed("splits.json").and_then(|v|v["counties"]["split"].as_u64()).map(|n|n.to_string()).unwrap_or_else(||"-".into());
 let missing:Vec<_>=REQUIRED.iter().filter(|n|!files.contains_key(**n)).copied().collect();
 json!({"balance":balance,"mean_pp":pp,"splits":splits,"missing_required":missing,"missing_cell":if missing.is_empty(){"none".into()}else{format!("{} required",missing.len())}})
}
pub fn diagnose<C: std::fmt::Display>(label:&str,manifest:&crate::manifest_schema::PlanManifestWithCounts<C>,files:&BTreeMap<String,Option<String>>,present:&[String])->Value {
 let mut rows=Vec::new();let mut missing_optional=Vec::new();
 for name in REQUIRED.iter().chain(OPTIONAL.iter()) {
  let supplied=files.get(*name);let content=supplied.and_then(|s|s.as_deref());
  let parse_error=content.and_then(|s|serde_json::from_str::<Value>(s).err()).map(|e|e.to_string());
  let info=content.map(|s|analysis_summary(name,s)).unwrap_or_default();
  let native_status=if supplied.is_none(){"MISS"}else if *name=="compactness.json"&&(info.contains("low")||info.contains("moderate")){"WARN"}else{"PASS"};
  if OPTIONAL.contains(name)&&supplied.is_none(){missing_optional.push(*name);}
  rows.push(json!({"name":name,"required":REQUIRED.contains(name),"native_status":native_status,"native_summary":info,"readable":content.is_some(),"parse_error":parse_error}));
 }
 let scan=scan_metrics(files);
 let maps:Vec<_>=["maps/districts.png","maps/compactness.png","maps/political.png"].into_iter().filter(|p|present.iter().any(|x|x==p)).collect();
 json!({"label":label,"manifest_label":manifest.label,"state":manifest.state_code,"year":manifest.year,"chamber":manifest.chamber,"districts":manifest.num_districts.to_string(),"seed":manifest.seed.map(|s|s.to_string()),"analysis":rows,"scan":scan,"required_files_present":scan["missing_required"].as_array().is_some_and(|a|a.is_empty()),"missing_optional":missing_optional,"maps":maps,"audit_present":present.iter().any(|p|p=="audit.json"),"rplan_present":present.iter().any(|p|p==&format!("{label}.rplan"))})
}
/// The native --all table body; caller supplies directory discovery and the header.
/// Character truncation also handles non-ASCII labels without splitting UTF-8.
pub fn all_plans_table(rows:&[(String,Option<(String,Value)>)])->String {
 use std::fmt::Write;
 let mut out=String::new();
 writeln!(out,"{:<30} {:>9} {:>8} {:>6} {:>7} {:>8}","Label","Districts","Balance","PP","Splits","Missing").unwrap();
 writeln!(out,"{}","-".repeat(75)).unwrap();
 for (label,detail) in rows {
  match detail {
   None=>writeln!(out,"{:<30} [no manifest]",label).unwrap(),
   Some((n,scan))=>writeln!(out,"{:<30} {:>9} {:>8} {:>6} {:>7} {:>8}",label.chars().take(29).collect::<String>(),n,scan["balance"].as_str().unwrap_or("?"),scan["mean_pp"].as_str().unwrap_or("-"),scan["splits"].as_str().unwrap_or("-"),scan["missing_cell"].as_str().unwrap_or("-")).unwrap(),
  }
 }
 writeln!(out,"\nRun 'bisect doctor --label <label>' for detailed diagnosis.").unwrap();
 out
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn presence_does_not_prove_validity(){let files=BTreeMap::from([("summary.json".into(),Some("{".into())),("compactness.json".into(),None),("contiguity.json".into(),Some("{}".into()))]);let r=diagnose("fixture",&crate::PlanManifest::default(),&files,&[]);assert_eq!(r["required_files_present"],true);assert_eq!(r["scan"]["balance"],"?");assert_eq!(r["analysis"][0]["native_status"],"PASS");assert!(r["analysis"][0]["parse_error"].is_string());assert_eq!(r["analysis"][1]["readable"],false);assert!(r["analysis"][2]["native_summary"].as_str().unwrap().contains("NON-CONTIGUOUS"));}
 #[test] fn thresholds_and_exact_counts(){for (pp,word) in [(0.149,"low"),(0.15,"moderate"),(0.25,"good"),(0.35,"well-compact")]{assert!(analysis_summary("compactness.json",&json!({"districts":[{"polsby_popper":pp}]}).to_string()).contains(word));}let f=BTreeMap::from([("splits.json".into(),Some("{\"counties\":{\"split\":9007199254740993}}".into()))]);assert_eq!(scan_metrics(&f)["splits"],"9007199254740993");assert_eq!(scan_metrics(&f)["missing_cell"],"3 required");}
}
