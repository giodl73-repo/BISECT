//! Native runner assignment file format: decimal unit indices and one-based districts.
pub fn serialize_native_assignments<I:IntoIterator<Item=(usize,usize)>>(rows:I)->Result<String,serde_json::Error>{
 let canonical:std::collections::BTreeMap<String,usize>=rows.into_iter().map(|(unit,district)|(unit.to_string(),district)).collect();
 serde_json::to_string_pretty(&canonical)
}
