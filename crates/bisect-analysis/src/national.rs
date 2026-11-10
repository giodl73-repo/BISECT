//! Shared native national analysis merge and export semantics.
/// Merge per-state analyzer JSONs into one national JSON.
/// Adds "state" field to each district record.
pub fn merge_analyzer_outputs(states: &[(&str, &serde_json::Value)]) -> serde_json::Value {
    let analyzer = states
        .first()
        .and_then(|(_, v)| v.get("analyzer"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut all_districts: Vec<serde_json::Value> = vec![];
    for (state_name, state_json) in states {
        if let Some(districts) = state_json.get("districts").and_then(|d| d.as_array()) {
            for d in districts {
                let mut record = d.clone();
                record["state"] = serde_json::Value::String(state_name.to_string());
                all_districts.push(record);
            }
        }
    }

    serde_json::json!({
        "analyzer": analyzer,
        "scope": "national",
        "state_count": states.len(),
        "district_count": all_districts.len(),
        "districts": all_districts,
    })
}

/// Merge per-state STATE-LEVEL analyzer JSONs (e.g. proportionality) into one
/// national JSON. Each state's whole JSON becomes one record in the "states"
/// array, with "state" and "year" fields injected.
pub fn merge_state_level_outputs(
    year: &str,
    states: &[(&str, &serde_json::Value)],
) -> serde_json::Value {
    let analyzer = states
        .first()
        .and_then(|(_, v)| v.get("analyzer"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut records: Vec<serde_json::Value> = vec![];
    for (state_name, state_json) in states {
        let mut record = match state_json.as_object() {
            Some(obj) => serde_json::Value::Object(obj.clone()),
            None => continue,
        };
        record["state"] = serde_json::Value::String(state_name.to_string());
        record["year"] = serde_json::Value::String(year.to_string());
        records.push(record);
    }

    serde_json::json!({
        "analyzer": analyzer,
        "scope": "national",
        "year": year,
        "state_count": records.len(),
        "states": records,
    })
}

/// Convert state-level national JSON to CSV. Each state record becomes one row.
/// Column order: state, year, then all other fields from the first record
/// (excluding "analyzer" and "state"/"year" which are placed first).
pub fn state_records_to_csv(merged: &serde_json::Value) -> String {
    let records = match merged.get("states").and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return String::new(),
    };
    if records.is_empty() {
        return String::new();
    }

    // Build header: state + year first, then remaining fields in stable order
    let first = match records[0].as_object() {
        Some(o) => o,
        None => return String::new(),
    };
    let mut extra_headers: Vec<String> = first
        .keys()
        .filter(|k| {
            *k != "state"
                && *k != "year"
                && *k != "analyzer"
                && *k != "per_district_dem_share_sorted"
        })
        .cloned()
        .collect();
    extra_headers.sort(); // stable ordering
    let mut headers = vec!["state".to_string(), "year".to_string()];
    headers.extend(extra_headers);

    write_csv(records, &headers)
}

/// Convert state-level national JSON to a JSON array.
pub fn state_records_to_json_array(merged: &serde_json::Value) -> serde_json::Value {
    let records = merged
        .get("states")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    serde_json::Value::Array(records)
}

/// Convert national JSON to CSV string. Uses first district's keys as headers.
pub fn districts_to_csv(merged: &serde_json::Value) -> String {
    let districts = match merged.get("districts").and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return String::new(),
    };
    if districts.is_empty() {
        return String::new();
    }

    // Collect all keys from first record
    let headers: Vec<String> = districts[0]
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();

    write_csv(districts, &headers)
}

/// Convert national JSON districts array to a JSON array of objects.
/// Each district record becomes a JSON object preserving all fields.
pub fn districts_to_json_array(merged: &serde_json::Value) -> serde_json::Value {
    let districts = merged
        .get("districts")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    serde_json::Value::Array(districts)
}


fn write_csv(records:&[serde_json::Value],headers:&[String])->String{
 let mut writer=csv::WriterBuilder::new().terminator(csv::Terminator::Any(b'\n')).from_writer(Vec::new());
 writer.write_record(headers).expect("memory CSV write");
 for record in records{writer.write_record(headers.iter().map(|h|match record.get(h){None|Some(serde_json::Value::Null)=>String::new(),Some(serde_json::Value::String(s))=>s.clone(),Some(v)=>v.to_string()})).expect("memory CSV write");}
 String::from_utf8(writer.into_inner().expect("memory CSV flush")).expect("JSON strings are UTF-8")
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn csv_roundtrips_quotes_newlines_and_nested_values(){
  let merged=serde_json::json!({"districts":[{"state":"RI","note":"a, \"quote\"\nnext","nested":{"x":[1,2]},"count":"18446744073709551615"}]});
  let text=districts_to_csv(&merged);let mut reader=csv::Reader::from_reader(text.as_bytes());let headers=reader.headers().unwrap().clone();let row=reader.records().next().unwrap().unwrap();
  for(key,value)in merged["districts"][0].as_object().unwrap(){let index=headers.iter().position(|h|h==key).unwrap();assert_eq!(row.get(index).unwrap(),value.as_str().map(str::to_owned).unwrap_or_else(||value.to_string()));}
 }
}
