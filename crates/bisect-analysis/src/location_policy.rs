//! Pure lookup and display of the native location-policy database.
use serde_json::Value;
use std::fmt::Write;

pub const EMBEDDED_POLICY: &str = include_str!("../../../data/location_policy.json");
pub fn embedded_policy() -> Value {
    serde_json::from_str(EMBEDDED_POLICY).expect("embedded location_policy.json is valid JSON")
}
/// Native State chamber-count semantics, with host-owned error presentation.
/// Lookup is deliberately case sensitive, matching the CLI's policy lookup.
pub fn state_chamber_count(policy:&Value,state:&str,chamber:&str,fallback:u64)->Result<(u64,bool),String>{
    if chamber=="congressional" {return Ok((fallback,false));}
    let key=match chamber {"house"|"lower"|"assembly"=>"house_districts","senate"|"upper"=>"senate_districts",_=>return Ok((fallback,false))};
    if let Some(entry)=policy.get(state){
        if let Some(n)=entry.get(key).and_then(Value::as_u64){
            if n>0 {return Ok((n,true));}
            let notes=entry.get("notes").and_then(Value::as_str).unwrap_or("");
            let hint=if notes.to_lowercase().contains("unicameral"){format!(" {state} has a unicameral legislature — use --chamber house.")}else{format!(" {state} has no {chamber} chamber.")};
            return Err(format!("No {chamber} chamber for {state}.{hint}"));
        }
    }
    Ok((fallback,false))
}
/// Native code lookup first, then unique case-insensitive name substring.
/// Test entries are available by explicit code but excluded from fuzzy matches.
pub fn resolve(policy: &Value, query: &str) -> Result<(String, Value, bool), String> {
    let code=query.to_uppercase();
    if let Some(entry)=policy.get(&code) { return Ok((code,entry.clone(),false)); }
    let matches:Vec<_>=policy.as_object().into_iter().flat_map(|o|o.iter())
        .filter(|(k,_)|!k.starts_with('_'))
        .filter_map(|(k,v)|v.get("name").and_then(Value::as_str)
            .filter(|n|n.to_lowercase().contains(&code.to_lowercase())).map(|n|(k,n)))
        .collect();
    if matches.len()==1 { let (k,_)=matches[0];return Ok((k.clone(),policy[k].clone(),true)); }
    if matches.len()>1 {
        return Err(format!("Multiple matches for '{code}': {}. Use the 2-letter code.",
            matches.iter().map(|(k,n)|format!("{k} ({n})")).collect::<Vec<_>>().join(", ")));
    }
    Err(format!("Location '{code}' not found in policy database. Use a location code (e.g., IE for Ireland)."))
}

pub const DISPLAY_FIELDS: &[(&str,&str)] = &[
    ("House districts","house_districts"),("Senate districts","senate_districts"),
    ("Subdivision term","subdivision_term_plural"),("Municipal term","municipal_term_plural"),
    ("House balance tolerance","balance_tolerance_house_pct"),
    ("Senate balance tolerance","balance_tolerance_senate_pct"),
    ("Congressional balance tolerance","balance_tolerance_congressional_pct"),
    ("Population basis","population_basis"),("Prison gerrymandering reform","prison_gerrymandering"),
    ("Student counting","student_counting"),("Nesting requirement","nesting_requirement"),
    ("Nesting ratio","nesting_ratio"),("Preservation standard","preservation_standard"),
    ("Legal citation","preservation_citation"),("Commission type","commission_type"),
    ("VRA Section 2 applies","vra_section2_applies"),
];
pub fn display_value(key:&str,val:&Value)->String {
    match val {
        Value::String(s)=>s.clone(), Value::Number(n) if key.contains("pct")=>format!("{n}%"),
        Value::Bool(b)=>if *b {"Yes"}else{"No"}.into(), other=>other.to_string(),
    }
}
pub fn render_table(code:&str,state:&Value)->String {
    let name=state["name"].as_str().unwrap_or(code);
    let mut output=format!("=== {name} ({code}) Redistricting Policy ===\n\n");
    for (label,key) in DISPLAY_FIELDS {
        if let Some(val)=state.get(*key).filter(|v|!v.is_null()) {
            writeln!(output,"  {label:<38} {}",display_value(key,val)).unwrap();
        }
    }
    for (key,label) in [("notes","Notes"),("independent_city_note","Independent cities")] {
        if let Some(note)=state.get(key).and_then(Value::as_str) {writeln!(output,"\n{label}: {note}").unwrap();}
    }
    output
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn all_embedded_codes_and_case_are_exact(){
        let db=embedded_policy();for (code,value) in db.as_object().unwrap().iter().filter(|(_,v)|v.get("name").is_some()){
            let (resolved,entry,fuzzy)=resolve(&db,&code.to_lowercase()).unwrap();
            assert_eq!(&resolved,code);assert_eq!(&entry,value);assert!(!fuzzy);
        }
    }
    #[test] fn fuzzy_and_ambiguity_follow_native_lookup(){
        let db=embedded_policy();assert_eq!(resolve(&db,"ireland").unwrap().0,"IE");
        assert!(resolve(&db,"carolina").unwrap_err().contains("Multiple matches"));
        assert!(resolve(&db,"Eldoria").is_err());assert!(resolve(&db,"ZZ").is_err());
        assert!(resolve(&db,"_test_el").is_ok());
    }
    #[test] fn display_retains_native_formats_and_notes(){
        let text=render_table("XX",&serde_json::json!({"name":"Example","balance_tolerance_house_pct":1.5,"vra_section2_applies":true,"nesting_ratio":"2:1","notes":"Exact note","independent_city_note":"City note"}));
        assert!(text.contains("1.5%"));assert!(text.contains("Yes"));assert!(text.contains("2:1"));
        assert!(text.contains("\nNotes: Exact note\n"));assert!(text.ends_with("Independent cities: City note\n"));
    }
}
