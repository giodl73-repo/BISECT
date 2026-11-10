//! Native Sweep v1 command planning; no engine execution or ranking.
use serde::{Deserialize, Serialize};

pub const VALID_METRICS: &[&str] = &["compactness", "splits", "deviation"];
mod seed_text {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &u64, s: S) -> Result<S::Ok,S::Error> { s.serialize_str(&v.to_string()) }
    pub fn deserialize<'de,D: Deserializer<'de>>(d:D)->Result<u64,D::Error>{
        #[derive(Deserialize)] #[serde(untagged)] enum Value { Number(u64), Text(String) }
        match Value::deserialize(d)? {
            Value::Number(n)=>Ok(n),
            Value::Text(s) if !s.is_empty() && s.bytes().all(|b|b.is_ascii_digit()) && (s=="0"||!s.starts_with('0')) => s.parse().map_err(serde::de::Error::custom),
            _=>Err(serde::de::Error::custom("Seed must be a canonical unsigned u64.")),
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub state:String, pub year:String, pub chamber:String, pub seeds:usize,
    #[serde(with="seed_text")] pub seed_start:u64,
    pub keep_top:usize, pub optimize_by:String, pub version:String,
    pub output_base:String, pub run:bool,
}
#[derive(Serialize)]
pub struct Row {
    pub label:String, #[serde(with="seed_text")] pub seed:u64,
    pub state_command:String, pub analyze_command:String,
}
#[derive(Serialize)]
pub struct Plan { pub rows:Vec<Row>, pub comparison_command:String, pub commands_stdout:String }
pub fn validate(settings:&Settings)->Result<(),String>{
    if settings.seeds<1{return Err("--seeds must be at least 1".into());}
    if settings.keep_top>settings.seeds{return Err(format!("--keep-top ({}) cannot exceed --seeds ({})",settings.keep_top,settings.seeds));}
    if !VALID_METRICS.contains(&settings.optimize_by.as_str()){return Err(format!("invalid --optimize-by '{}'; valid values: {}",settings.optimize_by,VALID_METRICS.join(", ")));}
    if settings.seed_start.checked_add((settings.seeds-1) as u64).is_none(){return Err("Seed range exceeds unsigned u64; reduce --seeds or --seed-start.".into());}
    Ok(())
}
/// Caller must validate the range first, as native run_sweep does.
pub fn labels(settings:&Settings)->Vec<String>{
    (0..settings.seeds).map(|i|format!("{}_{}_seed_{}",settings.state.to_lowercase(),settings.chamber,settings.seed_start+i as u64)).collect()
}
pub fn plan(settings:&Settings)->Result<Plan,String>{
    validate(settings)?;
    let rows:Vec<_>=labels(settings).into_iter().enumerate().map(|(i,label)|{
        let seed=settings.seed_start+i as u64;
        let state_command=format!("bisect state --state {} --year {} --chamber {} --label {} --seed {} --version {} --output-base {}",settings.state,settings.year,settings.chamber,label,seed,settings.version,settings.output_base);
        let analyze_command=format!("bisect analyze --state {} --year {} --label {} --version {} --types compactness splits summary",settings.state,settings.year,label,settings.version);
        Row{label,seed,state_command,analyze_command}
    }).collect();
    let comparison_command=format!("bisect compare --labels {} --optimize-by {}",rows.iter().take(settings.keep_top).map(|r|r.label.clone()).collect::<Vec<_>>().join(","),settings.optimize_by);
    let commands_stdout=rows.iter().map(|r|format!("{}\n{}\n",r.state_command,r.analyze_command)).collect();
    Ok(Plan{rows,comparison_command,commands_stdout})
}
#[cfg(test)] mod tests {
    use super::*;
    fn settings()->Settings{Settings{state:"WA".into(),year:"2020".into(),chamber:"house".into(),seeds:2,seed_start:42,keep_top:1,optimize_by:"splits".into(),version:"sweep".into(),output_base:"outputs".into(),run:false}}
    #[test] fn command_order_and_first_labels_are_native(){let p=plan(&settings()).unwrap();assert_eq!(p.rows[0].label,"wa_house_seed_42");assert_eq!(p.rows[1].seed,43);assert_eq!(p.comparison_command,"bisect compare --labels wa_house_seed_42 --optimize-by splits");assert!(p.commands_stdout.starts_with("bisect state"));assert_eq!(p.commands_stdout.lines().count(),4);}
    #[test] fn maximum_seed_and_range_overflow(){let mut s=settings();s.seed_start=u64::MAX;s.seeds=1;assert_eq!(plan(&s).unwrap().rows[0].seed,u64::MAX);s.seeds=2;assert!(validate(&s).unwrap_err().contains("u64"));}
    #[test] fn zero_keep_and_reserved_run_do_not_rank_or_execute(){let mut s=settings();s.keep_top=0;s.run=true;assert_eq!(plan(&s).unwrap().comparison_command,"bisect compare --labels  --optimize-by splits");}
}
